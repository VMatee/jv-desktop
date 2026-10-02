//! OS process-tree ownership and pipe semantics live behind this boundary.
#[cfg(target_os = "windows")]
pub(crate) mod windows;
#[cfg(target_os = "linux")]
mod linux {
    use std::{
        io,
        os::unix::process::CommandExt,
        process::{Child, Command},
    };

    pub fn configure(command: &mut Command) {
        command.process_group(0);
    }

    pub fn nonblocking(pipe: &impl std::os::fd::AsRawFd) -> io::Result<()> {
        let fd = pipe.as_raw_fd();
        // SAFETY: fd is borrowed from an owned pipe and remains open here.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: preserves all existing flags, adding O_NONBLOCK only.
        if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    // WNOWAIT keeps the leader's PID reserved until its group is cleaned up.
    // This avoids sending a signal to a reused process-group ID after reaping.
    pub fn exited(child: &mut Child) -> io::Result<bool> {
        let mut info = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
        // SAFETY: valid output buffer; PID belongs to our unreaped direct child.
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                child.id(),
                info.as_mut_ptr(),
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: waitid initialized the buffer (zeroed for no state change).
        Ok(unsafe { info.assume_init().si_pid() } != 0)
    }

    pub fn signal_group(child: &Child, force: bool) -> io::Result<()> {
        let pid = i32::try_from(child.id()).map_err(|_| io::Error::other("invalid child id"))?;
        if pid <= 1 {
            return Err(io::Error::other("invalid child id"));
        }
        // SAFETY: our child is an unreaped group leader, so its PID cannot be
        // reused. No caller can supply a PID; only this owned Child is accepted.
        let result = unsafe { libc::kill(-pid, if force { libc::SIGKILL } else { libc::SIGTERM }) };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(error)
        }
    }
}
#[cfg(target_os = "linux")]
pub use linux::*;
