use jv_desktop_process::{native_turn_gate, validate_workspace, ErrorCategory, Permissions};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Mutex};
use tauri::{ipc::Channel, Manager};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct DesktopState {
    selected: Mutex<Option<PathBuf>>,
}

fn app_directory() -> Result<PathBuf, ErrorCategory> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .ok_or(ErrorCategory::Internal)
}
fn home_directory() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

#[tauri::command]
async fn choose_workspace(app: tauri::AppHandle) -> Result<Option<String>, ErrorCategory> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(chosen) = app
            .dialog()
            .file()
            .set_title("Open a project folder")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let path = chosen
            .into_path()
            .map_err(|_| ErrorCategory::InvalidWorkspace)?;
        let canonical = validate_workspace(&path, home_directory().as_deref(), &app_directory()?)?;
        let display = canonical
            .to_str()
            .ok_or(ErrorCategory::InvalidWorkspace)?
            .to_owned();
        *app.state::<DesktopState>()
            .selected
            .lock()
            .map_err(|_| ErrorCategory::Internal)? = Some(canonical);
        Ok(Some(display))
    })
    .await
    .map_err(|_| ErrorCategory::Internal)?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendStatus {
    availability: &'static str,
    can_run: bool,
    detail: &'static str,
}

#[tauri::command]
async fn detect_backend() -> BackendStatus {
    #[cfg(not(target_os = "linux"))]
    {
        BackendStatus {
            availability: "unsupported",
            can_run: false,
            detail: "Native JV CLI is currently Linux-only. Use the offline demo on this platform.",
        }
    }
    #[cfg(target_os = "linux")]
    {
        let result = tauri::async_runtime::spawn_blocking(|| {
            let paths = std::env::var_os("PATH").ok_or(ErrorCategory::Unavailable)?;
            let program = jv_desktop_process::resolve_executable("jvcli", &paths)
                .ok_or(ErrorCategory::Unavailable)?;
            jv_desktop_process::process::detect(program, app_directory()?)
        })
        .await;
        match result {
            Ok(Ok(())) => BackendStatus { availability: "detected", can_run: false,
                detail: "JV CLI detected. Native runs await secure sign-in and prompt transport. Detection does not verify an account or service health." },
            Ok(Err(ErrorCategory::Timeout)) => BackendStatus { availability: "unavailable", can_run: false,
                detail: "The local JV CLI version check timed out. No task was submitted." },
            _ => BackendStatus { availability: "unavailable", can_run: false,
                detail: "JV CLI is unavailable. Install the official Linux CLI, then check again." },
        }
    }
}

// No Debug: user text must not enter diagnostics.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TurnRequest {
    run_id: String,
    workspace: String,
    prompt: String,
    permissions: Permissions,
}

#[tauri::command]
async fn run_turn(
    request: TurnRequest,
    on_event: Channel<serde_json::Value>,
    state: tauri::State<'_, DesktopState>,
) -> Result<(), ErrorCategory> {
    let _ = on_event; // Reserved transport for the future reviewed native adapter.
    if request.run_id.is_empty()
        || request.run_id.len() > 64
        || !request
            .run_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(ErrorCategory::Internal);
    }
    let selected = state
        .selected
        .lock()
        .map_err(|_| ErrorCategory::Internal)?
        .clone()
        .ok_or(ErrorCategory::InvalidWorkspace)?;
    let canonical = validate_workspace(
        std::path::Path::new(&request.workspace),
        home_directory().as_deref(),
        &app_directory()?,
    )?;
    if canonical != selected {
        return Err(ErrorCategory::InvalidWorkspace);
    }
    // Unconditional fail-closed gate. No real exec/resume invocation exists in
    // the command path, even if a compromised frontend bypasses its disabled UI.
    native_turn_gate(&request.prompt, request.permissions)
}

#[tauri::command]
fn cancel_turn(run_id: String) -> Result<(), ErrorCategory> {
    let _ = run_id;
    // Idempotent: native submission is disabled, so there is no native child.
    // The process-core fixture tests exercise actual Linux group cancellation.
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .manage(DesktopState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            choose_workspace,
            detect_backend,
            run_turn,
            cancel_turn
        ])
        .run(tauri::generate_context!())
        .expect("JV Desktop could not initialize");
}
