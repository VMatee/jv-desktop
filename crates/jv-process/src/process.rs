use crate::ErrorCategory;
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

pub const MAX_LINE_BYTES: usize = 64 * 1024;
pub const MAX_STREAM_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_EVENTS: usize = 4_000;

/// Intentionally has no Debug implementation: stdin may contain user text.
pub struct ProcessSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    pub stdin: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Line(String),
    Diagnostic(&'static str),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub cancelled: bool,
}

/// Blocking runner intended for a worker thread. Bounded pipes, no shell, no
/// inherited secret environment. Only Linux process-tree semantics are enabled.
pub fn run_process(
    spec: ProcessSpec,
    cancel: Arc<AtomicBool>,
    timeout: Duration,
    on_frame: impl FnMut(Frame),
) -> Result<ProcessResult, ErrorCategory> {
    #[cfg(target_os = "linux")]
    {
        linux::run(spec, cancel, timeout, on_frame)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (spec, cancel, timeout, on_frame);
        Err(ErrorCategory::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::platform;
    use std::{
        io::{Read, Write},
        process::{Command, Stdio},
        sync::{
            atomic::Ordering,
            mpsc::{self, SyncSender},
        },
        thread,
        time::Instant,
    };

    enum Message {
        Frame(Frame),
        Fault(ErrorCategory),
    }
    struct OwnedChild {
        child: std::process::Child,
        stop_io: Arc<AtomicBool>,
        reaped: bool,
    }
    impl std::ops::Deref for OwnedChild {
        type Target = std::process::Child;
        fn deref(&self) -> &Self::Target {
            &self.child
        }
    }
    impl std::ops::DerefMut for OwnedChild {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.child
        }
    }
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            self.stop_io.store(true, Ordering::Release);
            if !self.reaped {
                let _ = platform::signal_group(&self.child, true);
                let _ = self.child.wait();
            }
        }
    }
    fn read_pipe(
        mut pipe: impl Read,
        tx: SyncSender<Message>,
        stdout: bool,
        stop: Arc<AtomicBool>,
    ) {
        let mut buffer = [0u8; 4096];
        let mut line = Vec::with_capacity(MAX_LINE_BYTES);
        let (mut dropping, mut total, mut count) = (false, 0usize, 0usize);
        loop {
            let n = match pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                Err(_) => {
                    let _ = tx.send(Message::Fault(ErrorCategory::ProcessFailed));
                    return;
                }
            };
            total += n;
            if total > MAX_STREAM_BYTES {
                let _ = tx.send(Message::Fault(ErrorCategory::StreamLimit));
                return;
            }
            if !stdout {
                continue;
            } // Raw stderr is intentionally never forwarded or logged.
            for byte in &buffer[..n] {
                if *byte == b'\n' {
                    count += 1;
                    if count > MAX_EVENTS {
                        let _ = tx.send(Message::Fault(ErrorCategory::StreamLimit));
                        return;
                    }
                    if !dropping && !send_line(&tx, &line) {
                        return;
                    }
                    line.clear();
                    dropping = false;
                } else if !dropping {
                    if line.len() == MAX_LINE_BYTES {
                        line.clear();
                        dropping = true;
                        if tx
                            .send(Message::Frame(Frame::Diagnostic("oversized-event")))
                            .is_err()
                        {
                            return;
                        }
                    } else {
                        line.push(*byte);
                    }
                }
            }
        }
        if stdout && !dropping && !line.is_empty() {
            if count == MAX_EVENTS {
                let _ = tx.send(Message::Fault(ErrorCategory::StreamLimit));
            } else {
                let _ = send_line(&tx, &line);
            }
        }
    }
    fn send_line(tx: &SyncSender<Message>, line: &[u8]) -> bool {
        let frame = match std::str::from_utf8(line) {
            Ok(s) => Frame::Line(s.trim_end_matches('\r').to_owned()),
            Err(_) => Frame::Diagnostic("invalid-utf8"),
        };
        tx.send(Message::Frame(frame)).is_ok()
    }

    pub fn run(
        spec: ProcessSpec,
        cancel: Arc<AtomicBool>,
        timeout: Duration,
        mut on_frame: impl FnMut(Frame),
    ) -> Result<ProcessResult, ErrorCategory> {
        if !spec.program.is_absolute() || !spec.cwd.is_absolute() || !spec.cwd.is_dir() {
            return Err(ErrorCategory::InvalidWorkspace);
        }
        if spec.stdin.len() > crate::MAX_PROMPT_BYTES {
            return Err(ErrorCategory::InvalidPrompt);
        }
        if cancel.load(Ordering::Acquire) {
            return Ok(ProcessResult {
                exit_code: None,
                cancelled: true,
            });
        }
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .env_clear()
            .env("LANG", "C.UTF-8")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .env("PYTHONNOUSERSITE", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Only a filtered executable search path is inherited, never arbitrary
        // JV_*, loader, proxy, shell startup, or authentication environment.
        if let Some(path) = std::env::var_os("PATH") {
            let safe: Vec<_> = std::env::split_paths(&path)
                .filter(|p| p.is_absolute())
                .collect();
            if let Ok(path) = std::env::join_paths(safe) {
                command.env("PATH", path);
            }
        }
        platform::configure(&mut command);
        let child = command.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ErrorCategory::Unavailable
            } else {
                ErrorCategory::ProcessFailed
            }
        })?;
        let stop_io = Arc::new(AtomicBool::new(false));
        let mut child = OwnedChild {
            child,
            stop_io: stop_io.clone(),
            reaped: false,
        };
        let (tx, rx) = mpsc::sync_channel(32);
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let mut stdin = child.stdin.take().expect("piped stdin");
        platform::nonblocking(&stdout).map_err(|_| ErrorCategory::ProcessFailed)?;
        platform::nonblocking(&stderr).map_err(|_| ErrorCategory::ProcessFailed)?;
        platform::nonblocking(&stdin).map_err(|_| ErrorCategory::ProcessFailed)?;
        let out_tx = tx.clone();
        let err_tx = tx.clone();
        let out_stop = stop_io.clone();
        let err_stop = stop_io.clone();
        let in_stop = stop_io.clone();
        let out = thread::spawn(move || read_pipe(stdout, out_tx, true, out_stop));
        let err = thread::spawn(move || read_pipe(stderr, err_tx, false, err_stop));
        let writer = thread::spawn(move || {
            let mut remaining = spec.stdin.as_slice();
            while !remaining.is_empty() && !in_stop.load(Ordering::Acquire) {
                match stdin.write(remaining) {
                    Ok(0) => {
                        let _ = tx.send(Message::Fault(ErrorCategory::ProcessFailed));
                        break;
                    }
                    Ok(n) => remaining = &remaining[n..],
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        let _ = tx.send(Message::Fault(ErrorCategory::ProcessFailed));
                        break;
                    }
                }
            }
            // Drop closes stdin even for an empty prompt (--version).
        });
        let start = Instant::now();
        let (mut fault, mut cancelled, mut stopping) = (None, false, None);
        let mut exited = false;
        loop {
            // Limit each drain pass so output cannot starve cancellation.
            for _ in 0..32 {
                match rx.try_recv() {
                    Ok(Message::Frame(frame)) => on_frame(frame),
                    Ok(Message::Fault(error)) => {
                        fault.get_or_insert(error);
                    }
                    Err(_) => break,
                }
            }
            cancelled |= cancel.load(Ordering::Acquire);
            if stopping.is_none() && start.elapsed() >= timeout && !cancelled {
                fault.get_or_insert(ErrorCategory::Timeout);
            }
            match platform::exited(&mut child) {
                Ok(done) => exited |= done,
                Err(_) => {
                    fault.get_or_insert(ErrorCategory::ProcessFailed);
                }
            }
            if stopping.is_none() && (cancelled || fault.is_some() || exited) {
                if platform::signal_group(&child, false).is_err() {
                    fault.get_or_insert(ErrorCategory::ProcessFailed);
                }
                stopping = Some(Instant::now());
            }
            if stopping.is_some_and(|time| time.elapsed() >= Duration::from_millis(200)) {
                if platform::signal_group(&child, true).is_err() {
                    fault.get_or_insert(ErrorCategory::ProcessFailed);
                }
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        let status = child.wait().map_err(|_| ErrorCategory::ProcessFailed);
        child.reaped = status.is_ok();
        stop_io.store(true, Ordering::Release);
        // Nonblocking readers stop even if a detached descendant retains a pipe.
        // Drain queued frames before joining; dropping a full bounded receiver first would
        // otherwise lose final messages, while joining first could deadlock.
        while let Ok(message) = rx.recv() {
            match message {
                Message::Frame(frame) => on_frame(frame),
                Message::Fault(e) => {
                    fault.get_or_insert(e);
                }
            }
        }
        for handle in [out, err, writer] {
            if handle.join().is_err() {
                fault.get_or_insert(ErrorCategory::Internal);
            }
        }
        if cancelled {
            return Ok(ProcessResult {
                exit_code: status.ok().and_then(|s| s.code()),
                cancelled: true,
            });
        }
        if let Some(error) = fault {
            return Err(error);
        }
        let status = status?;
        if !status.success() {
            return Err(ErrorCategory::ProcessFailed);
        }
        Ok(ProcessResult {
            exit_code: status.code(),
            cancelled: false,
        })
    }
}

/// Detection is not authentication or service health. Only --version is used.
pub fn detect(program: PathBuf, cwd: PathBuf) -> Result<(), ErrorCategory> {
    let mut valid = false;
    run_process(
        ProcessSpec {
            program,
            args: vec!["--version".into()],
            cwd,
            stdin: vec![],
        },
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(3),
        |frame| {
            if let Frame::Line(line) = frame {
                valid |= line.starts_with("JV CLI ") && line.len() <= 128;
            }
        },
    )?;
    if valid {
        Ok(())
    } else {
        Err(ErrorCategory::Unavailable)
    }
}
