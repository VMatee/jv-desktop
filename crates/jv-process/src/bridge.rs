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

/// Windows support starts with the official 0.5.0 release; Linux keeps 0.4.9.
pub const MINIMUM_CLI_VERSION: &str = if cfg!(target_os = "windows") {
    "0.5.0"
} else {
    "0.4.9"
};

pub fn compatible(version: &str) -> bool {
    compatible_since(version, MINIMUM_CLI_VERSION)
}
fn compatible_since(version: &str, minimum: &str) -> bool {
    let Some(v) = version.strip_prefix("JV CLI ") else {
        return false;
    };
    parse_version(v.trim())
        .zip(parse_version(minimum))
        .is_some_and(|(found, floor)| found >= floor)
}
fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let value = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(value)
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[test]
    fn windows_release_floor_preserves_the_linux_baseline() {
        for version in ["JV CLI 0.4.9", "JV CLI 0.4.10"] {
            assert!(compatible_since(version, "0.4.9"));
            assert!(!compatible_since(version, "0.5.0"));
        }
        for minimum in ["0.4.9", "0.5.0"] {
            assert!(!compatible_since("JV CLI 0.4.8", minimum));
            assert!(compatible_since("JV CLI 0.5.0", minimum));
            assert!(compatible_since("JV CLI 1.0.0\r\n", minimum));
        }
    }

    #[test]
    fn neither_platform_accepts_malformed_or_prerelease_versions() {
        for version in [
            "JV CLI 0.5.0-beta",
            "JV CLI 0.5.0evil",
            "JV CLI 0.5.0.1",
            "JV CLI 0.5",
            "JV CLI 18446744073709551616.0.0",
            "Codex 1.0.0",
        ] {
            for minimum in ["0.4.9", "0.5.0"] {
                assert!(!compatible_since(version, minimum));
            }
        }
    }
}

pub fn policy(p: Permissions) -> Result<Value, E> {
    if (p.access == Access::ReadOnly && p.network)
        || (cfg!(target_os = "windows") && (p.access != Access::WorkspaceWrite || !p.network))
    {
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
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = (program, workspace, state);
            Err(E::UnsupportedPlatform)
        }
        #[cfg(any(target_os = "linux", target_os = "windows"))]
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
        let result = t.exchange(
            &mut request,
            "auth.succeeded",
            Duration::from_secs(40),
            &self.stopping,
        );
        request = Value::Null;
        drop(request); // best effort reference removal, not secure erasure
        result?;
        let mut request = policy.clone();
        request["type"] = json!("session.start");
        let response = t.exchange(
            &mut request,
            "session.started",
            Duration::from_secs(15),
            &self.stopping,
        )?;
        if self.stopping.load(Ordering::Acquire) {
            return Err(E::ProcessFailed);
        }
        if response["session_id"].as_str().is_none_or(|s| !valid_id(s))
            || response["network"] != policy["allow_network"]
            || response["sandbox"]
                != if cfg!(target_os = "windows") {
                    "danger-full-access"
                } else if permissions.access == Access::ReadOnly {
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
        let requested_policy = policy(permissions)?;
        let mut t = self.inner.try_lock().map_err(|_| E::ProcessFailed)?;
        if !t.authenticated || self.stopping.load(Ordering::Acquire) {
            return Err(E::AuthenticationRequired);
        }
        if t.policy.as_ref() != Some(&requested_policy) {
            return Err(E::PermissionConflict);
        }
        self.cancel.store(false, Ordering::Release);
        let request = t.send(json!({"type":"turn.start","text":prompt}))?;
        let mut turn = None;
        let mut cancel_sent = false;
        let mut cancel_request = None;
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
                    cancel_request = Some(t.send(json!({"type":"turn.cancel","turn_id":id}))?);
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
                if !cancel_sent
                    || frame["request_id"].as_str() != cancel_request.as_deref()
                    || frame["turn_id"].as_str() != turn.as_deref()
                {
                    return Err(E::ProtocolError);
                }
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
    /// Test-only ownership evidence; never exposed by frontend IPC.
    #[cfg(all(
        feature = "test-fixture",
        any(target_os = "linux", target_os = "windows")
    ))]
    pub fn bridge_pid(&self) -> u32 {
        self.inner.lock().unwrap().pid()
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

#[cfg(any(target_os = "linux", target_os = "windows"))]
use transport::Transport;
#[cfg(any(target_os = "linux", target_os = "windows"))]
#[path = "bridge_transport.rs"]
mod transport;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
struct Transport {
    authenticated: bool,
    policy: Option<Value>,
    session: Option<String>,
}
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
impl Transport {
    fn poll(&mut self) -> Result<Option<Value>, E> {
        Err(E::UnsupportedPlatform)
    }
    fn send(&mut self, _: Value) -> Result<String, E> {
        Err(E::UnsupportedPlatform)
    }
    fn exchange(
        &mut self,
        _: &mut Value,
        _: &str,
        _: Duration,
        _: &AtomicBool,
    ) -> Result<Value, E> {
        Err(E::UnsupportedPlatform)
    }
    fn shutdown(&mut self) {}
}
