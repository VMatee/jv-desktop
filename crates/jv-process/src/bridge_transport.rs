//! Shared framing/correlation with cfg-selected native I/O and ownership.
use super::{json, Duration, Value, E};
#[cfg(target_os = "linux")]
use linux::Io;
use std::{io, path::PathBuf, time::Instant};
#[cfg(target_os = "windows")]
use windows::Io;

pub(super) struct Transport {
    io: Io,
    pending: Vec<u8>,
    stderr_bytes: usize,
    sequence: u64,
    closed: bool,
    eof: bool,
    pub authenticated: bool,
    pub policy: Option<Value>,
    pub session: Option<String>,
}
impl Transport {
    pub fn spawn(program: PathBuf, workspace: PathBuf, state: PathBuf) -> Result<Self, E> {
        Ok(Self {
            io: Io::spawn(program, workspace, state)?,
            pending: Vec::new(),
            stderr_bytes: 0,
            sequence: 0,
            closed: false,
            eof: false,
            authenticated: false,
            policy: None,
            session: None,
        })
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
            bytes.fill(0);
            return Err(E::StreamLimit);
        }
        self.io.write(bytes, Duration::from_secs(3))?;
        Ok(id)
    }
    pub fn poll(&mut self) -> Result<Option<Value>, E> {
        if self.closed {
            return Err(E::ProcessFailed);
        }
        let mut buf = [0u8; 4096];
        match self.io.read_error(&mut buf) {
            Ok(n) => {
                self.stderr_bytes += n;
                if self.stderr_bytes > 64 * 1024 {
                    return Err(E::StreamLimit);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => return Err(E::ProcessFailed),
        }
        if let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
            if end + 1 > 128 * 1024 {
                return Err(E::StreamLimit);
            }
            let line: Vec<_> = self.pending.drain(..=end).collect();
            return parse(&line).map(Some);
        }
        if self.eof {
            if self.pending.is_empty() {
                return Err(E::ProcessFailed);
            }
            let line = std::mem::take(&mut self.pending);
            return parse(&line).map(Some);
        }
        match self.io.read_output(&mut buf) {
            Ok(0) => self.eof = true,
            Ok(n) => {
                self.pending.extend_from_slice(&buf[..n]);
                if self.pending.len() > 128 * 1024 + 4096 {
                    return Err(E::StreamLimit);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => return Err(E::ProcessFailed),
        }
        // At most one frame per poll; the caller enforces turn count/byte bounds.
        if self.pending.len() > 128 * 1024 && !self.pending.contains(&b'\n') {
            return Err(E::StreamLimit);
        }
        Ok(None)
    }
    pub fn next(&mut self, timeout: Duration) -> Result<Value, E> {
        let start = Instant::now();
        loop {
            if let Some(v) = self.poll()? {
                return Ok(v);
            }
            if start.elapsed() >= timeout {
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
        stopping: &std::sync::atomic::AtomicBool,
    ) -> Result<Value, E> {
        let id = self.send(value.take())?;
        let start = Instant::now();
        loop {
            if stopping.load(std::sync::atomic::Ordering::Acquire) {
                return Err(E::ProcessFailed);
            }
            if start.elapsed() >= timeout {
                return Err(E::Timeout);
            }
            let Some(v) = self.poll()? else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
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
            }
            return Err(E::ProtocolError);
        }
    }
    #[cfg(feature = "test-fixture")]
    pub fn pid(&self) -> u32 {
        self.io.pid()
    }
    pub fn shutdown(&mut self) {
        if self.closed {
            return;
        }
        let _ = self.send(json!({"type":"bridge.shutdown"}));
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(8) && !self.io.exited() {
            let _ = self.poll();
            std::thread::sleep(Duration::from_millis(10));
        }
        self.io.finish();
        self.closed = true;
        self.authenticated = false;
    }
}
fn parse(line: &[u8]) -> Result<Value, E> {
    if line.len() > 128 * 1024 {
        return Err(E::StreamLimit);
    }
    let v: Value = serde_json::from_slice(line).map_err(|_| E::ProtocolError)?;
    if !v.is_object() || !v["type"].is_string() {
        return Err(E::ProtocolError);
    }
    Ok(v)
}
impl Drop for Transport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use crate::platform::windows::{read, OwnedProcess};
    pub(super) struct Io {
        process: OwnedProcess,
    }
    impl Io {
        #[cfg(feature = "test-fixture")]
        pub fn pid(&self) -> u32 {
            self.process.id()
        }
        pub fn spawn(program: PathBuf, workspace: PathBuf, state: PathBuf) -> Result<Self, E> {
            let args = ["bridge", "--stdio", "--execution-mode", "trusted-host"].map(Into::into);
            let process = OwnedProcess::spawn(&program, &args, &workspace, Some(&state))
                .map_err(|_| E::Unavailable)?;
            Ok(Self { process })
        }
        pub fn write(&mut self, bytes: Vec<u8>, timeout: Duration) -> Result<(), E> {
            self.process.write(bytes, timeout).map_err(|e| {
                if e.kind() == io::ErrorKind::TimedOut {
                    E::Timeout
                } else {
                    E::ProcessFailed
                }
            })
        }
        pub fn read_output(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            read(&mut self.process.output, buf)
        }
        pub fn read_error(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            read(&mut self.process.error, buf)
        }
        pub fn exited(&mut self) -> bool {
            self.process.exited().unwrap_or(true)
        }
        pub fn finish(&mut self) {
            self.process.terminate();
            crate::platform::windows::wait(&self.process, Duration::from_secs(5));
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::platform;
    use std::{
        io::{Read, Write},
        os::unix::fs::PermissionsExt,
        process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
    };
    pub(super) struct Io {
        child: Child,
        input: ChildStdin,
        output: ChildStdout,
        error: ChildStderr,
        reaped: bool,
    }
    impl Io {
        #[cfg(feature = "test-fixture")]
        pub fn pid(&self) -> u32 {
            self.child.id()
        }
        pub fn spawn(program: PathBuf, workspace: PathBuf, state: PathBuf) -> Result<Self, E> {
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
                reaped: false,
            };
            platform::nonblocking(&t.input).map_err(|_| E::ProcessFailed)?;
            platform::nonblocking(&t.output).map_err(|_| E::ProcessFailed)?;
            platform::nonblocking(&t.error).map_err(|_| E::ProcessFailed)?;
            Ok(t)
        }
        pub fn write(&mut self, mut bytes: Vec<u8>, timeout: Duration) -> Result<(), E> {
            let start = Instant::now();
            let mut offset = 0;
            let result = (|| {
                while offset < bytes.len() {
                    match self.input.write(&bytes[offset..]) {
                        Ok(0) => return Err(E::ProcessFailed),
                        Ok(n) => offset += n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            if start.elapsed() > timeout {
                                return Err(E::Timeout);
                            }
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => return Err(E::ProcessFailed),
                    }
                }
                Ok(())
            })();
            bytes.fill(0);
            result
        }
        pub fn read_output(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.output.read(buf)
        }
        pub fn read_error(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.error.read(buf)
        }
        pub fn exited(&mut self) -> bool {
            platform::exited(&mut self.child).unwrap_or(true)
        }
        pub fn finish(&mut self) {
            if self.reaped {
                return;
            }
            if !self.exited() {
                let _ = platform::signal_group(&self.child, false);
                let start = Instant::now();
                while start.elapsed() < Duration::from_secs(20) && !self.exited() {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            // The leader remains unreaped, so the group identity cannot be reused.
            let _ = platform::signal_group(&self.child, true);
            let _ = self.child.wait();
            self.reaped = true;
        }
    }
    impl Drop for Io {
        fn drop(&mut self) {
            self.finish();
        }
    }
}
