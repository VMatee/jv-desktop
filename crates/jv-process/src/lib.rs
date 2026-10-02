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
    #[cfg(target_os = "windows")]
    platform::windows::no_reparse(path).map_err(|_| ErrorCategory::InvalidWorkspace)?;
    let canonical = canonical_path(path).map_err(|_| ErrorCategory::InvalidWorkspace)?;
    let same_as = |other: &Path| canonical_path(other).is_ok_and(|p| p == canonical);
    if !canonical.is_dir()
        || canonical.parent().is_none()
        || home.is_some_and(same_as)
        || same_as(app_dir)
    {
        return Err(ErrorCategory::InvalidWorkspace);
    }
    Ok(canonical)
}

pub fn canonical_path(path: &Path) -> std::io::Result<PathBuf> {
    let path = path.canonicalize()?;
    #[cfg(target_os = "windows")]
    {
        platform::windows::local_path(&path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(path)
    }
}

/// Rust-owned fixed executable resolution. Windows prefers the per-user install;
/// no current-directory, project-tree or relative PATH lookup is permitted.
pub fn resolve_native_cli(workspace: Option<&Path>) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH").unwrap_or_default();
    #[cfg(target_os = "windows")]
    {
        let default = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|p| p.join("Programs/JV CLI/jvcli.exe"));
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|p| canonical_path(&p).ok());
        resolve_windows_cli(default, &paths, cwd.as_deref(), workspace)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = workspace;
        resolve_executable("jvcli", &paths)
    }
}

#[cfg(target_os = "windows")]
fn resolve_windows_cli(
    default: Option<PathBuf>,
    paths: &std::ffi::OsStr,
    cwd: Option<&Path>,
    workspace: Option<&Path>,
) -> Option<PathBuf> {
    default
        .into_iter()
        .chain(
            std::env::split_paths(paths)
                .filter(|p| p.is_absolute())
                .map(|p| p.join("jvcli.exe")),
        )
        .find_map(|p| {
            platform::windows::no_reparse(&p).ok()?;
            if !p.is_file() {
                return None;
            }
            let p = canonical_path(&p).ok()?;
            // An absolute install under a drive/home root is not cwd lookup.
            // Reject the actual cwd entry and every selected-project candidate.
            if cwd.is_some_and(|w| p.parent() == Some(w))
                || workspace.is_some_and(|w| p.starts_with(w))
            {
                return None;
            }
            Some(p)
        })
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
    #[cfg(target_os = "windows")]
    #[test]
    fn windows_resolver_prefers_install_and_excludes_cwd_relative_and_project_paths() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.cache/tmp/resolver")
            .join(format!("{}", std::process::id()));
        let installed = root.join("Programs/JV CLI");
        let fallback = root.join("fallback");
        let project = root.join("project");
        for dir in [&installed, &fallback, &project] {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::copy(std::env::current_exe().unwrap(), dir.join("jvcli.exe")).unwrap();
        }
        let root = canonical_path(&root).unwrap();
        let installed = canonical_path(&installed).unwrap().join("jvcli.exe");
        let fallback = canonical_path(&fallback).unwrap();
        let project = canonical_path(&project).unwrap();
        let paths = std::env::join_paths([Path::new("."), &project, &fallback]).unwrap();
        let drive = root.ancestors().last().unwrap();
        assert_eq!(
            resolve_windows_cli(Some(installed.clone()), &paths, Some(drive), Some(&project)),
            Some(installed)
        );
        assert_eq!(
            resolve_windows_cli(None, &paths, Some(&project), Some(&project)),
            Some(fallback.join("jvcli.exe"))
        );
        let unsafe_paths = std::env::join_paths([Path::new("."), &project, &fallback]).unwrap();
        assert!(
            resolve_windows_cli(None, &unsafe_paths, Some(&fallback), Some(&project)).is_none()
        );
    }
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
