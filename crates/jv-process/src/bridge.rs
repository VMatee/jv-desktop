//! Versioned stdio bridge. Executable selection is Rust-only; no shell or secret environment.
use crate::{Access, ErrorCategory as E, Permissions};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};

pub fn compatible(version: &str) -> bool {
    let Some(v) = version.strip_prefix("JV CLI ") else {
        return false;
    };
    let parts: Vec<_> = v.trim().split('.').map(str::parse::<u64>).collect();
    matches!(parts.as_slice(), [Ok(a), Ok(b), Ok(c)] if (*a,*b,*c) >= (0,4,9))
}
pub fn policy(p: Permissions) -> Result<Value, E> {
    if p.access == Access::ReadOnly && p.network {
        return Err(E::PermissionConflict);
    }
    Ok(json!({"read_only":p.access == Access::ReadOnly,"allow_network":p.network}))
}
pub fn version(program: PathBuf, cwd: PathBuf) -> Result<String, E> {
    let mut result = None;
    crate::process::run_process(
        crate::process::ProcessSpec {
            program,
            cwd,
            args: vec!["--version".into()],
            stdin: vec![],
        },
        std::sync::Arc::new(AtomicBool::new(false)),
        Duration::from_secs(3),
        |frame| {
            if let crate::process::Frame::Line(line) = frame {
                if line.starts_with("JV CLI ") && line.len() < 128 {
                    result = Some(line)
                }
            }
        },
    )?;
    let value = result.ok_or(E::Unavailable)?;
    if !compatible(&value) {
        return Err(E::UpgradeRequired);
    }
    Ok(value)
}

pub struct BridgeSupervisor {
    workspace: PathBuf,
    inner: Mutex<Transport>,
    cancel: AtomicBool,
    stopping: AtomicBool,
}
impl BridgeSupervisor {
    pub fn start(program: PathBuf, workspace: PathBuf, state: PathBuf) -> Result<Self, E> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (program, workspace, state);
            Err(E::UnsupportedPlatform)
        }
        #[cfg(target_os = "linux")]
        {
            version(program.clone(), workspace.clone())?;
            let mut transport = Transport::spawn(program, workspace.clone(), state)?;
            let ready = transport.next(Duration::from_secs(5))?;
            if ready["type"] != "bridge.ready"
                || ready["protocol"].as_u64() != Some(1)
                || !ready["cli_version"]
                    .as_str()
                    .is_some_and(|v| compatible(&format!("JV CLI {v}")))
                || ready["workspace"].as_str() != workspace.to_str()
            {
                return Err(E::ProtocolError);
            }
            Ok(Self {
                workspace,
                inner: Mutex::new(transport),
                cancel: AtomicBool::new(false),
                stopping: AtomicBool::new(false),
            })
        }
    }
    pub fn workspace(&self) -> &std::path::Path {
        &self.workspace
    }
    pub fn authenticate(
        &self,
        username: String,
        base_url: String,
        password: String,
        permissions: Permissions,
    ) -> Result<(), E> {
        if username.trim().is_empty()
            || username.len() > 256
            || base_url.len() > 2048
            || password.is_empty()
            || password.len() > 16384
        {
            return Err(E::AuthenticationRequired);
        }
        let policy = policy(permissions)?;
        let mut t = self.inner.lock().map_err(|_| E::Internal)?;
        if self.stopping.load(Ordering::Acquire) || t.authenticated {
            return Err(E::AuthenticationRequired);
        }
        let mut request = json!({"type":"auth.login","username":username,"base_url":base_url,"password":password});
        let result = t.exchange(&mut request, "auth.succeeded", Duration::from_secs(40));
        request = Value::Null;
        drop(request); // best effort reference removal, not secure erasure
        result?;
        let mut request = policy.clone();
        request["type"] = json!("session.start");
        let response = t.exchange(&mut request, "session.started", Duration::from_secs(15))?;
        if response["session_id"].as_str().is_none()
            || response["network"] != policy["allow_network"]
            || response["sandbox"]
                != if permissions.access == Access::ReadOnly {
                    "read-only"
                } else {
                    "workspace-write"
                }
        {
            return Err(E::ProtocolError);
        }
        t.authenticated = true;
        t.policy = Some(policy);
        t.session = response["session_id"].as_str().map(str::to_owned);
        Ok(())
    }
    pub fn run_turn(
        &self,
        prompt: String,
        permissions: Permissions,
        mut emit: impl FnMut(Value) -> Result<(), E>,
    ) -> Result<(), E> {
        crate::validate_prompt(&prompt)?;
        let mut t = self.inner.try_lock().map_err(|_| E::ProcessFailed)?;
        if !t.authenticated || self.stopping.load(Ordering::Acquire) {
            return Err(E::AuthenticationRequired);
        }
        if t.policy.as_ref() != Some(&policy(permissions)?) {
            return Err(E::PermissionConflict);
        }
        self.cancel.store(false, Ordering::Release);
        let request = t.send(json!({"type":"turn.start","text":prompt}))?;
        let mut turn = None;
        let mut cancel_sent = false;
        let mut count = 0;
        let mut bytes = 0usize;
        let start = std::time::Instant::now();
        let mut cancelled_at = None;
        loop {
            if self.stopping.load(Ordering::Acquire) {
                return Err(E::ProcessFailed);
            }
            if start.elapsed() > Duration::from_secs(3600) {
                return Err(E::Timeout);
            }
            if self.cancel.load(Ordering::Acquire) && !cancel_sent {
                if let Some(id) = &turn {
                    t.send(json!({"type":"turn.cancel","turn_id":id}))?;
                    cancel_sent = true;
                    cancelled_at = Some(std::time::Instant::now());
                }
            }
            if cancelled_at.is_some_and(|t| t.elapsed() > Duration::from_secs(20)) {
                return Err(E::Timeout);
            }
            let Some(frame) = t.poll()? else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
            count += 1;
            bytes += frame.to_string().len();
            if bytes > 4 * 1024 * 1024 {
                return Err(E::StreamLimit);
            }
            if count > 4000 {
                return Err(E::StreamLimit);
            }
            let typ = frame["type"].as_str().ok_or(E::ProtocolError)?;
            if typ == "request.error" {
                return Err(E::ProcessFailed);
            }
            if typ == "turn.started" {
                if turn.is_some() || frame["request_id"] != request {
                    return Err(E::ProtocolError);
                }
                let id = frame["turn_id"]
                    .as_str()
                    .filter(|s| valid_id(s))
                    .ok_or(E::ProtocolError)?;
                turn = Some(id.to_owned());
                emit(json!({"type":"turn.started"}))?;
                continue;
            }
            if typ == "turn.cancel.accepted" {
                continue;
            }
            if !matches!(
                typ,
                "command.started"
                    | "command.completed"
                    | "files.changed"
                    | "tool.started"
                    | "assistant.message"
                    | "turn.completed"
                    | "turn.failed"
                    | "turn.cancelled"
            ) {
                continue;
            }
            if turn.is_none() || frame["turn_id"].as_str() != turn.as_deref() {
                return Err(E::ProtocolError);
            }
            let normalized = normalize(&frame)?;
            if matches!(typ, "turn.completed" | "turn.failed" | "turn.cancelled") {
                if frame["request_id"] != request {
                    return Err(E::ProtocolError);
                }
                emit(normalized)?;
                return Ok(());
            }
            emit(normalized)?;
        }
    }
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
    }
    pub fn shutdown(&self) {
        self.stopping.store(true, Ordering::Release);
        self.cancel();
        if let Ok(mut t) = self.inner.lock() {
            t.shutdown();
        }
    }
}
fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn text(v: &Value, key: &str, max: usize) -> Result<String, E> {
    let value = v[key]
        .as_str()
        .filter(|s| s.len() <= max)
        .ok_or(E::ProtocolError)?;
    Ok(value
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect())
}
fn normalize(v: &Value) -> Result<Value, E> {
    let typ = v["type"].as_str().ok_or(E::ProtocolError)?;
    Ok(match typ {
        "command.started" => json!({"type":typ,"command":text(v,"command",8192)?}),
        "command.completed" => {
            if !v["exit_code"].is_null() && v["exit_code"].as_i64().is_none() {
                return Err(E::ProtocolError);
            }
            json!({"type":typ,"command":text(v,"command",8192)?,"output":text(v,"output",16384)?,"exit_code":v["exit_code"]})
        }
        "assistant.message" => json!({"type":typ,"text":text(v,"text",32768)?}),
        "tool.started" => json!({"type":typ,"name":text(v,"name",256)?}),
        "files.changed" => {
            let changes = v["changes"]
                .as_array()
                .filter(|a| a.len() <= 50)
                .ok_or(E::ProtocolError)?;
            let mut files = Vec::new();
            for c in changes {
                files.push(json!({"path":text(c,"path",1024)?,"kind":text(c,"kind",32)?}));
            }
            json!({"type":typ,"changes":files})
        }
        "turn.cancelled" => {
            if v["local"] != true || v["remote_state"] != "unknown" {
                return Err(E::ProtocolError);
            }
            json!({"type":typ,"local":true,"remote_state":"unknown"})
        }
        "turn.failed" | "turn.completed" => json!({"type":typ}),
        _ => return Err(E::ProtocolError),
    })
}

#[cfg(target_os = "linux")]
use linux::Transport;
#[cfg(not(target_os = "linux"))]
struct Transport {
    authenticated: bool,
    policy: Option<Value>,
    session: Option<String>,
}
#[cfg(not(target_os = "linux"))]
impl Transport {
    fn poll(&mut self) -> Result<Option<Value>, E> {
        Err(E::UnsupportedPlatform)
    }
    fn send(&mut self, _v: Value) -> Result<String, E> {
        Err(E::UnsupportedPlatform)
    }
    fn exchange(&mut self, _v: &mut Value, _s: &str, _d: Duration) -> Result<Value, E> {
        Err(E::UnsupportedPlatform)
    }
    fn shutdown(&mut self) {}
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::platform;
    use std::{
        io::{Read, Write},
        process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
        time::Instant,
    };
    pub(super) struct Transport {
        child: Child,
        input: ChildStdin,
        output: ChildStdout,
        error: ChildStderr,
        pending: Vec<u8>,
        stderr_bytes: usize,
        sequence: u64,
        closed: bool,
        pub authenticated: bool,
        pub policy: Option<Value>,
        pub session: Option<String>,
    }
    impl Transport {
        pub fn spawn(program: PathBuf, workspace: PathBuf, state: PathBuf) -> Result<Self, E> {
            use std::os::unix::fs::PermissionsExt;
            if !program.is_absolute() || !workspace.is_absolute() || !state.is_absolute() {
                return Err(E::InvalidWorkspace);
            }
            for p in state.ancestors() {
                if p.symlink_metadata()
                    .is_ok_and(|m| m.file_type().is_symlink())
                {
                    return Err(E::InvalidWorkspace);
                }
            }
            std::fs::create_dir_all(&state).map_err(|_| E::Internal)?;
            std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o700))
                .map_err(|_| E::Internal)?;
            let mut c = Command::new(program);
            c.args(["bridge", "--stdio"])
                .current_dir(workspace)
                .env_clear()
                .env("LANG", "C.UTF-8")
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .env("PYTHONNOUSERSITE", "1")
                .env("JVCLI_HOME", &state)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            if let Some(home) = std::env::var_os("HOME") {
                c.env("HOME", home);
            }
            if let Some(path) = std::env::var_os("PATH") {
                let paths: Vec<_> = std::env::split_paths(&path)
                    .filter(|p| p.is_absolute())
                    .collect();
                if let Ok(p) = std::env::join_paths(paths) {
                    c.env("PATH", p);
                }
            }
            platform::configure(&mut c);
            let mut child = c.spawn().map_err(|_| E::Unavailable)?;
            let input = child.stdin.take().unwrap();
            let output = child.stdout.take().unwrap();
            let error = child.stderr.take().unwrap();
            let t = Self {
                child,
                input,
                output,
                error,
                pending: Vec::new(),
                stderr_bytes: 0,
                sequence: 0,
                closed: false,
                authenticated: false,
                policy: None,
                session: None,
            };
            platform::nonblocking(&t.input).map_err(|_| E::ProcessFailed)?;
            platform::nonblocking(&t.output).map_err(|_| E::ProcessFailed)?;
            platform::nonblocking(&t.error).map_err(|_| E::ProcessFailed)?;
            Ok(t)
        }
        pub fn send(&mut self, mut value: Value) -> Result<String, E> {
            if self.closed {
                return Err(E::ProcessFailed);
            }
            self.sequence += 1;
            let id = format!("desktop_{}", self.sequence);
            value["request_id"] = json!(id);
            value["protocol"] = json!(1);
            let mut bytes = serde_json::to_vec(&value).map_err(|_| E::ProtocolError)?;
            drop(value);
            bytes.push(b'\n');
            if bytes.len() > 256 * 1024 {
                return Err(E::StreamLimit);
            }
            let started = Instant::now();
            let mut offset = 0;
            let result = (|| {
                while offset < bytes.len() {
                    match self.input.write(&bytes[offset..]) {
                        Ok(0) => return Err(E::ProcessFailed),
                        Ok(n) => offset += n,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            if started.elapsed() > Duration::from_secs(3) {
                                return Err(E::Timeout);
                            }
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(_) => return Err(E::ProcessFailed),
                    }
                }
                Ok(id)
            })();
            bytes.fill(0);
            result
        }
        pub fn poll(&mut self) -> Result<Option<Value>, E> {
            if self.closed {
                return Err(E::ProcessFailed);
            }
            let mut buf = [0u8; 4096];
            match self.error.read(&mut buf) {
                Ok(n) => {
                    self.stderr_bytes += n;
                    if self.stderr_bytes > 64 * 1024 {
                        return Err(E::StreamLimit);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => return Err(E::ProcessFailed),
            }
            if let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
                if end + 1 > 128 * 1024 {
                    return Err(E::StreamLimit);
                }
                let line: Vec<_> = self.pending.drain(..=end).collect();
                let value: Value = serde_json::from_slice(&line).map_err(|_| E::ProtocolError)?;
                if !value.is_object() || !value["type"].is_string() {
                    return Err(E::ProtocolError);
                }
                return Ok(Some(value));
            }
            match self.output.read(&mut buf) {
                Ok(0) => return Err(E::ProcessFailed),
                Ok(n) => {
                    self.pending.extend_from_slice(&buf[..n]);
                    if self.pending.len() > 128 * 1024 + 4096 {
                        return Err(E::StreamLimit);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => return Err(E::ProcessFailed),
            };
            Ok(None)
        }
        pub fn next(&mut self, timeout: Duration) -> Result<Value, E> {
            let start = Instant::now();
            loop {
                if let Some(v) = self.poll()? {
                    return Ok(v);
                }
                if start.elapsed() > timeout {
                    return Err(E::Timeout);
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        pub fn exchange(
            &mut self,
            value: &mut Value,
            expected: &str,
            timeout: Duration,
        ) -> Result<Value, E> {
            let id = self.send(value.take())?;
            let start = Instant::now();
            for _ in 0..64 {
                let v = self.next(timeout.saturating_sub(start.elapsed()))?;
                if v["request_id"] == id {
                    if v["type"] == expected {
                        return Ok(v);
                    }
                    if v["type"] == "request.error" {
                        return Err(if expected == "auth.succeeded" {
                            E::AuthenticationRequired
                        } else {
                            E::ProcessFailed
                        });
                    }
                    return Err(E::ProtocolError);
                }
            }
            Err(E::StreamLimit)
        }
        pub fn shutdown(&mut self) {
            if self.closed {
                return;
            }
            let _ = self.send(json!({"type":"bridge.shutdown"}));
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(8) {
                if platform::exited(&mut self.child).unwrap_or(true) {
                    break;
                }
                let _ = self.poll();
                std::thread::sleep(Duration::from_millis(10));
            }
            if !platform::exited(&mut self.child).unwrap_or(true) {
                let _ = platform::signal_group(&self.child, false);
                let start = Instant::now();
                while start.elapsed() < Duration::from_secs(20) {
                    if platform::exited(&mut self.child).unwrap_or(true) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            if !platform::exited(&mut self.child).unwrap_or(true) {
                let _ = platform::signal_group(&self.child, true);
            }
            let _ = self.child.wait();
            self.closed = true;
            self.authenticated = false;
        }
    }
    impl Drop for Transport {
        fn drop(&mut self) {
            self.shutdown();
        }
    }
}
