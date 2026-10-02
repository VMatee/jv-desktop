//! Native ownership and pipes. No shell, PID discovery, or machine policy changes.
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read, Write},
    os::windows::{
        ffi::OsStrExt,
        fs::MetadataExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Component, Path, PathBuf, Prefix},
    ptr::{null, null_mut},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        SetHandleInformation, ERROR_BROKEN_PIPE, HANDLE_FLAG_INHERIT, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    Security::SECURITY_ATTRIBUTES,
    System::{
        JobObjects::{
            CreateJobObjectW, IsProcessInJob, JobObjectExtendedLimitInformation,
            SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
        Pipes::{CreatePipe, PeekNamedPipe},
        Threading::{
            CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
            InitializeProcThreadAttributeList, UpdateProcThreadAttribute, WaitForSingleObject,
            CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT,
            PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROC_THREAD_ATTRIBUTE_JOB_LIST,
            STARTF_USESTDHANDLES, STARTUPINFOEXW,
        },
        IO::CancelSynchronousIo,
    },
};

fn wide(s: &OsStr) -> io::Result<Vec<u16>> {
    let mut w: Vec<_> = s.encode_wide().collect();
    if w.contains(&0) {
        return Err(io::Error::other("invalid native string"));
    }
    w.push(0);
    Ok(w)
}

/// Windows C runtime argv quoting; never a shell command string.
fn argument(s: &OsStr) -> io::Result<Vec<u16>> {
    let w = wide(s)?;
    let mut out = vec![b'"' as u16];
    let mut slashes = 0;
    for &c in &w[..w.len() - 1] {
        if c == b'\\' as u16 {
            slashes += 1;
            continue;
        }
        out.extend(std::iter::repeat_n(
            b'\\' as u16,
            slashes * if c == b'"' as u16 { 2 } else { 1 },
        ));
        if c == b'"' as u16 {
            out.push(b'\\' as u16);
        }
        slashes = 0;
        out.push(c);
    }
    out.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
    out.push(b'"' as u16);
    Ok(out)
}

/// Only local drive paths are supported. Canonical Windows paths lose the
/// verbatim prefix for compatibility with the CLI's canonical workspace frame.
pub fn local_path(path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute() || path.to_str().is_none() {
        return Err(io::Error::other("absolute local path required"));
    }
    match path.components().next() {
        Some(Component::Prefix(p))
            if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)) => {}
        _ => return Err(io::Error::other("unsupported path namespace")),
    }
    let value = path.to_str().unwrap();
    Ok(PathBuf::from(value.strip_prefix(r"\\?\").unwrap_or(value)))
}

pub fn no_reparse(path: &Path) -> io::Result<()> {
    local_path(path)?;
    for ancestor in path.ancestors() {
        match ancestor.symlink_metadata() {
            Ok(m) if m.file_attributes() & 0x400 != 0 => {
                return Err(io::Error::other("reparse path rejected"))
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// The packaged CLI does not need Python/Node lookup. Tools receive only an
/// absolute PATH; credentials, proxies, loader/startup and Python overrides are
/// never copied. Native runtime/system locations are the reviewed allowlist.
pub fn environment(state: Option<&Path>) -> io::Result<BTreeMap<OsString, OsString>> {
    let mut env = BTreeMap::new();
    for key in [
        "SYSTEMROOT",
        "WINDIR",
        "USERPROFILE",
        "LOCALAPPDATA",
        "TEMP",
        "TMP",
    ] {
        if let Some(value) = std::env::var_os(key) {
            let path = Path::new(&value);
            if local_path(path).is_ok() {
                env.insert(key.into(), value);
            }
        }
    }
    if !env.contains_key(OsStr::new("SYSTEMROOT")) {
        return Err(io::Error::other("system directory unavailable"));
    }
    if let Some(path) = std::env::var_os("PATH") {
        let paths = std::env::split_paths(&path)
            .filter(|p| local_path(p).is_ok())
            .collect::<Vec<_>>();
        let path =
            std::env::join_paths(paths).map_err(|_| io::Error::other("invalid search path"))?;
        env.insert("PATH".into(), path);
    }
    if let Some(state) = state {
        no_reparse(state)?;
        if state
            .components()
            .any(|c| matches!(c, Component::ParentDir))
            || state.parent().is_none()
        {
            return Err(io::Error::other("invalid state path"));
        }
        // JV CLI creates/protects this child itself. Never apply chmod or
        // pre-create a directory with an inherited Windows ACL here.
        env.insert("JVCLI_HOME".into(), local_path(state)?.into_os_string());
    }
    Ok(env)
}

struct Attributes {
    storage: Vec<usize>,
    initialized: bool,
}
impl Attributes {
    fn pointer(&mut self) -> *mut std::ffi::c_void {
        self.storage.as_mut_ptr().cast()
    }
    fn new() -> io::Result<Self> {
        let mut size = 0;
        // SAFETY: the sizing call writes only to size.
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 2, 0, &mut size);
        }
        let mut a = Self {
            storage: vec![0; size.div_ceil(std::mem::size_of::<usize>())],
            initialized: false,
        };
        // SAFETY: sufficiently sized, aligned buffer lives through CreateProcessW.
        if unsafe { InitializeProcThreadAttributeList(a.pointer(), 2, 0, &mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        a.initialized = true;
        Ok(a)
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: initialized list, freed once before its backing buffer.
        if self.initialized {
            unsafe {
                DeleteProcThreadAttributeList(self.pointer());
            }
        }
    }
}

fn pipe(parent_reads: bool) -> io::Result<(File, OwnedHandle)> {
    let sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    let (mut read, mut write) = (null_mut(), null_mut());
    // SAFETY: valid outputs and security attributes.
    if unsafe { CreatePipe(&mut read, &mut write, &sa, 4096) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: newly allocated handles transferred to RAII exactly once.
    let (read, write) = unsafe {
        (
            OwnedHandle::from_raw_handle(read),
            OwnedHandle::from_raw_handle(write),
        )
    };
    let (parent, child) = if parent_reads {
        (read, write)
    } else {
        (write, read)
    };
    // SAFETY: parent handle remains owned; prevent inheritance by any child.
    if unsafe { SetHandleInformation(parent.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((File::from(parent), child))
}

pub struct OwnedProcess {
    process: OwnedHandle,
    job: OwnedHandle,
    pub input: Option<File>,
    pub output: File,
    pub error: File,
}
impl OwnedProcess {
    pub fn spawn(
        program: &Path,
        args: &[OsString],
        cwd: &Path,
        state: Option<&Path>,
    ) -> io::Result<Self> {
        no_reparse(program)?;
        no_reparse(cwd)?;
        if !program.is_file() {
            return Err(io::ErrorKind::NotFound.into());
        }
        if !cwd.is_dir() {
            return Err(io::Error::other("invalid executable or workspace"));
        }
        let env = environment(state)?;
        let mut block = Vec::new();
        for (key, value) in env {
            let entry = wide(&OsString::from(format!(
                "{}={}",
                key.to_str()
                    .ok_or_else(|| io::Error::other("invalid environment key"))?,
                value
                    .to_str()
                    .ok_or_else(|| io::Error::other("invalid environment path"))?
            )))?;
            block.extend(entry);
        }
        block.push(0);
        let executable = wide(program.as_os_str())?;
        let directory = wide(local_path(cwd)?.as_os_str())?;
        let mut command = argument(program.as_os_str())?;
        for arg in args {
            command.push(b' ' as u16);
            command.extend(argument(arg)?);
        }
        command.push(0);
        if command.len() > 32767 {
            return Err(io::Error::other("native argv too large"));
        }
        // SAFETY: anonymous, non-inheritable job with lifetime owned by Desktop.
        let handle = unsafe { CreateJobObjectW(null(), null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: newly allocated owned handle.
        let job = unsafe { OwnedHandle::from_raw_handle(handle) };
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: fully initialized limits; breakaway flags are intentionally absent.
        if unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let (input, child_in) = pipe(false)?;
        let (output, child_out) = pipe(true)?;
        let (error, child_err) = pipe(true)?;
        let handles = [
            child_in.as_raw_handle(),
            child_out.as_raw_handle(),
            child_err.as_raw_handle(),
        ];
        let jobs = [handle];
        let mut attributes = Attributes::new()?;
        for (key, value, size) in [
            (
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                handles.as_ptr().cast(),
                std::mem::size_of_val(&handles),
            ),
            (
                PROC_THREAD_ATTRIBUTE_JOB_LIST,
                jobs.as_ptr().cast(),
                std::mem::size_of_val(&jobs),
            ),
        ] {
            // SAFETY: borrowed arrays live until CreateProcessW returns.
            if unsafe {
                UpdateProcThreadAttribute(
                    attributes.pointer(),
                    0,
                    key as usize,
                    value,
                    size,
                    null_mut(),
                    null_mut(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
        startup.StartupInfo.cb = std::mem::size_of_val(&startup) as u32;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput = handles[0];
        startup.StartupInfo.hStdOutput = handles[1];
        startup.StartupInfo.hStdError = handles[2];
        startup.lpAttributeList = attributes.pointer();
        let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: UTF-16 argv/environment, cwd, attributes and restricted handle
        // list remain alive. JOB_LIST establishes ownership before user code runs.
        // Failure never retries without job ownership.
        if unsafe {
            CreateProcessW(
                executable.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                1,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW,
                block.as_ptr().cast(),
                directory.as_ptr(),
                &startup.StartupInfo,
                &mut info,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful CreateProcessW transfers both new handles.
        let process = unsafe { OwnedHandle::from_raw_handle(info.hProcess) };
        drop(unsafe { OwnedHandle::from_raw_handle(info.hThread) });
        let result = Self {
            process,
            job,
            input: Some(input),
            output,
            error,
        };
        let mut member = 0;
        // SAFETY: root and job handles are alive and owned by this instance.
        if unsafe {
            IsProcessInJob(
                result.process.as_raw_handle(),
                result.job.as_raw_handle(),
                &mut member,
            )
        } == 0
            || member == 0
        {
            return Err(io::Error::other("native ownership verification failed"));
        }
        Ok(result)
    }
    #[cfg(feature = "test-fixture")]
    pub fn id(&self) -> u32 {
        // SAFETY: live borrowed root process handle.
        unsafe { windows_sys::Win32::System::Threading::GetProcessId(self.process.as_raw_handle()) }
    }
    pub fn exited(&self) -> io::Result<bool> {
        // SAFETY: live borrowed process handle; never a caller-selected PID.
        match unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } {
            WAIT_OBJECT_0 => Ok(true),
            WAIT_TIMEOUT => Ok(false),
            _ => Err(io::Error::last_os_error()),
        }
    }
    pub fn exit_code(&self) -> io::Result<i32> {
        let mut code = 0;
        // SAFETY: live process handle and initialized output pointer.
        if unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(code as i32)
    }
    pub fn terminate(&self) {
        // SAFETY: only this owner's non-inherited job is terminated.
        unsafe {
            TerminateJobObject(self.job.as_raw_handle(), 1);
        }
    }
    pub fn write(&mut self, bytes: Vec<u8>, timeout: Duration) -> io::Result<()> {
        let mut pipe = self
            .input
            .take()
            .ok_or_else(|| io::Error::other("closed stdin"))?;
        let (tx, rx) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let mut bytes = bytes;
            let result = pipe.write_all(&bytes);
            bytes.fill(0);
            let _ = tx.send((pipe, result));
        });
        let result = match rx.recv_timeout(timeout) {
            Ok((pipe, result)) => {
                self.input = Some(pipe);
                result
            }
            Err(_) => {
                // Closing all owned readers by killing the job releases any
                // blocked synchronous write. No writer/thread is detached.
                self.terminate();
                // The thread handle remains owned by JoinHandle. Retrying also
                // closes the race where cancellation precedes WriteFile entry.
                // Native pipe cancellation does not depend on remote readers.
                while !worker.is_finished() {
                    // SAFETY: borrowed live thread handle for our sole writer.
                    unsafe {
                        CancelSynchronousIo(worker.as_raw_handle());
                    }
                    thread::sleep(Duration::from_millis(1));
                }
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "pipe write deadline",
                ))
            }
        };
        drop(rx);
        let _ = worker.join();
        result
    }
}

#[cfg(all(test, feature = "test-fixture"))]
mod tests {
    use super::*;
    #[test]
    fn blocked_writer_cancels_with_a_reader_still_open_in_the_owner() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let exe = std::env::current_exe().unwrap();
        // Test-runner --help does not authenticate or execute any tool.
        let mut p = OwnedProcess::spawn(&exe, &["--help".into()], &root, None).unwrap();
        let (writer, reader) = pipe(false).unwrap();
        p.input = Some(writer);
        let start = Instant::now();
        assert_eq!(
            p.write(vec![b'x'; 32768], Duration::from_millis(50))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        drop(reader);
    }
}
impl Drop for OwnedProcess {
    fn drop(&mut self) {
        self.terminate();
        // SAFETY: terminate covers the owned job and wait uses its root handle.
        unsafe {
            WaitForSingleObject(self.process.as_raw_handle(), 5000);
        }
    }
}

/// One owner reads each synchronous pipe. Peek first, then read no more than the
/// available bytes, so the worker's polling loop remains cancellation responsive.
pub fn read(pipe: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    let mut available = 0;
    // SAFETY: owned live pipe; no data is copied during the availability query.
    if unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            &mut available,
            null_mut(),
        )
    } == 0
    {
        let error = io::Error::last_os_error();
        return if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
            Ok(0)
        } else {
            Err(error)
        };
    }
    if available == 0 {
        return Err(io::ErrorKind::WouldBlock.into());
    }
    let n = buffer.len().min(available as usize);
    pipe.read(&mut buffer[..n])
}

pub fn wait(process: &OwnedProcess, timeout: Duration) -> bool {
    let start = Instant::now();
    while !process.exited().unwrap_or(true) {
        if start.elapsed() >= timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(5));
    }
    true
}
