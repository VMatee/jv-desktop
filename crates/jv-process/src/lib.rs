//! Desktop's process boundary. No JV source, credentials, HTTP client, or shell.
pub mod bridge;
mod platform;
pub mod process;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MAX_PROMPT_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCategory {
    Unavailable,
    UpgradeRequired,
    ProtocolError,
    InvalidWorkspace,
    InvalidPrompt,
    PermissionConflict,
    AuthenticationRequired,
    UnsupportedPlatform,
    ProcessFailed,
    Timeout,
    StreamLimit,
    Internal,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    ReadOnly,
    WorkspaceWrite,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
    pub access: Access,
    pub network: bool,
}

/// Flags reflect the installed 0.4.7 public parser, not engine-private options.
pub fn exec_args(permissions: Permissions) -> Result<Vec<String>, ErrorCategory> {
    let policy = match (permissions.access, permissions.network) {
        (Access::ReadOnly, true) => return Err(ErrorCategory::PermissionConflict),
        (Access::ReadOnly, false) => "--read-only",
        (Access::WorkspaceWrite, true) => "--allow-network",
        (Access::WorkspaceWrite, false) => "--no-network",
    };
    Ok(vec!["exec".into(), "--json".into(), policy.into()])
}

pub fn validate_prompt(prompt: &str) -> Result<(), ErrorCategory> {
    if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES || prompt.contains('\0') {
        return Err(ErrorCategory::InvalidPrompt);
    }
    Ok(())
}

/// Canonicalize an explicit selection without creating or editing any files.
pub fn validate_workspace(
    path: &Path,
    home: Option<&Path>,
    app_dir: &Path,
) -> Result<PathBuf, ErrorCategory> {
    if !path.is_absolute() {
        return Err(ErrorCategory::InvalidWorkspace);
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| ErrorCategory::InvalidWorkspace)?;
    let same_as = |other: &Path| other.canonicalize().is_ok_and(|p| p == canonical);
    if !canonical.is_dir()
        || canonical.parent().is_none()
        || home.is_some_and(same_as)
        || same_as(app_dir)
    {
        return Err(ErrorCategory::InvalidWorkspace);
    }
    Ok(canonical)
}

/// Explicit native gate: public exec requires argv prompts and noninteractive
/// authentication requires a secret environment value. Neither is enabled here.
/// No caller-supplied environment, executable, or shell reaches Tauri commands.
pub fn native_turn_gate(prompt: &str, permissions: Permissions) -> Result<(), ErrorCategory> {
    validate_prompt(prompt)?;
    exec_args(permissions)?;
    if !cfg!(target_os = "linux") {
        return Err(ErrorCategory::UnsupportedPlatform);
    }
    Err(ErrorCategory::AuthenticationRequired)
}

/// Ignore relative PATH entries (including an empty entry meaning cwd).
pub fn resolve_executable(name: &str, paths: &std::ffi::OsStr) -> Option<PathBuf> {
    if name != "jvcli" {
        return None;
    }
    std::env::split_paths(paths)
        .filter(|p| p.is_absolute())
        .find_map(|p| {
            let path = p.join(name);
            let metadata = path.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o111 == 0 {
                    return None;
                }
            }
            path.canonicalize().ok()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn permission_matrix_and_gate() {
        for (access, network, flag) in [
            (Access::ReadOnly, false, "--read-only"),
            (Access::WorkspaceWrite, false, "--no-network"),
            (Access::WorkspaceWrite, true, "--allow-network"),
        ] {
            let p = Permissions { access, network };
            assert_eq!(exec_args(p).unwrap(), vec!["exec", "--json", flag]);
            assert_eq!(
                native_turn_gate("สวัสดี", p),
                Err(if cfg!(target_os = "linux") {
                    ErrorCategory::AuthenticationRequired
                } else {
                    ErrorCategory::UnsupportedPlatform
                })
            );
        }
        assert_eq!(
            exec_args(Permissions {
                access: Access::ReadOnly,
                network: true
            }),
            Err(ErrorCategory::PermissionConflict)
        );
    }
    #[test]
    fn validates_prompt_by_bytes() {
        assert!(validate_prompt("สวัสดี\n$(not a shell)").is_ok());
        for p in [
            "".to_string(),
            " \n".to_string(),
            "x\0x".to_string(),
            "ก".repeat(MAX_PROMPT_BYTES),
        ] {
            assert_eq!(validate_prompt(&p), Err(ErrorCategory::InvalidPrompt));
        }
    }
    #[test]
    fn refuses_implicit_workspaces() {
        let app = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(validate_workspace(Path::new("."), None, app).is_err());
        assert!(validate_workspace(app, None, app).is_err());
        let parent = app.parent().unwrap();
        assert!(validate_workspace(parent, Some(parent), app).is_err());
        assert!(validate_workspace(parent, None, app).is_ok());
    }
    #[test]
    fn no_relative_executable_search() {
        assert_eq!(resolve_executable("jvcli", std::ffi::OsStr::new(".")), None);
        assert_eq!(
            resolve_executable("sh", std::ffi::OsStr::new("/usr/bin")),
            None
        );
    }
}
