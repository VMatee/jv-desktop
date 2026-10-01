use jv_desktop_process::{validate_workspace, ErrorCategory, Permissions};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{ipc::Channel, Manager};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct DesktopState {
    lifecycle: Mutex<()>,
    selected: Mutex<Option<PathBuf>>,
    bridge: Mutex<Option<Arc<jv_desktop_process::bridge::BridgeSupervisor>>>,
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
        let state = app.state::<DesktopState>();
        let _lifecycle = state
            .lifecycle
            .lock()
            .map_err(|_| ErrorCategory::Internal)?;
        disconnect(&state)?;
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
    detail: String,
}

#[tauri::command]
async fn detect_backend() -> BackendStatus {
    #[cfg(not(target_os = "linux"))]
    {
        BackendStatus {
            availability: "unsupported",
            can_run: false,
            detail: "Native JV CLI is currently Linux-only. Use the offline demo on this platform."
                .into(),
        }
    }
    #[cfg(target_os = "linux")]
    {
        let result = tauri::async_runtime::spawn_blocking(|| {
            let paths = std::env::var_os("PATH").ok_or(ErrorCategory::Unavailable)?;
            let program = jv_desktop_process::resolve_executable("jvcli", &paths)
                .ok_or(ErrorCategory::Unavailable)?;
            jv_desktop_process::bridge::version(program, app_directory()?)
        })
        .await;
        match result {
            Ok(Ok(version)) => BackendStatus { availability: "detected", can_run: true,
                detail: format!("{version} · Bridge protocol 1. Sign in after selecting a workspace. Detection does not verify service health.") },
            Ok(Err(ErrorCategory::UpgradeRequired)) => BackendStatus {availability:"upgrade-required",can_run:false,detail:"Upgrade JV CLI to 0.4.9 or newer. No legacy execution fallback is available.".into()},
            Ok(Err(ErrorCategory::Timeout)) => BackendStatus {availability:"unavailable",can_run:false,detail:"JV CLI version check timed out.".into()},
            _ => BackendStatus {availability:"unavailable",can_run:false,detail:"Install JV CLI 0.4.9 or newer for Linux, then check again.".into()},
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
    if request.run_id.is_empty()
        || request.run_id.len() > 64
        || !request
            .run_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
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
    jv_desktop_process::validate_prompt(&request.prompt)?;
    jv_desktop_process::bridge::policy(request.permissions)?;
    let bridge = state
        .bridge
        .lock()
        .map_err(|_| ErrorCategory::Internal)?
        .clone()
        .ok_or(ErrorCategory::AuthenticationRequired)?;
    if bridge.workspace() != selected {
        return Err(ErrorCategory::InvalidWorkspace);
    }
    tauri::async_runtime::spawn_blocking(move || {
        let result = bridge.run_turn(request.prompt, request.permissions, |event| {
            on_event
                .send(serde_json::json!({"runId":request.run_id,"event":event}))
                .map_err(|_| ErrorCategory::ProcessFailed)
        });
        if result.is_err() {
            bridge.shutdown();
        }
        result
    })
    .await
    .map_err(|_| ErrorCategory::Internal)?
}

#[tauri::command]
fn cancel_turn(run_id: String, state: tauri::State<'_, DesktopState>) -> Result<(), ErrorCategory> {
    let _ = run_id;
    if let Some(bridge) = state
        .bridge
        .lock()
        .map_err(|_| ErrorCategory::Internal)?
        .as_ref()
    {
        bridge.cancel();
    }
    Ok(())
}
fn disconnect(state: &DesktopState) -> Result<(), ErrorCategory> {
    let old = state
        .bridge
        .lock()
        .map_err(|_| ErrorCategory::Internal)?
        .take();
    if let Some(bridge) = old {
        bridge.shutdown();
    }
    Ok(())
}
#[tauri::command]
async fn close_bridge(app: tauri::AppHandle) -> Result<(), ErrorCategory> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DesktopState>();
        let _lifecycle = state
            .lifecycle
            .lock()
            .map_err(|_| ErrorCategory::Internal)?;
        disconnect(&state)
    })
    .await
    .map_err(|_| ErrorCategory::Internal)?
}
#[tauri::command]
async fn open_bridge(app: tauri::AppHandle) -> Result<(), ErrorCategory> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DesktopState>();
        let _lifecycle = state
            .lifecycle
            .lock()
            .map_err(|_| ErrorCategory::Internal)?;
        disconnect(&state)?;
        let selected = state
            .selected
            .lock()
            .map_err(|_| ErrorCategory::Internal)?
            .clone()
            .ok_or(ErrorCategory::InvalidWorkspace)?;
        let paths = std::env::var_os("PATH").ok_or(ErrorCategory::Unavailable)?;
        let program = jv_desktop_process::resolve_executable("jvcli", &paths)
            .ok_or(ErrorCategory::Unavailable)?;
        let data = app
            .path()
            .app_local_data_dir()
            .map_err(|_| ErrorCategory::Internal)?
            .join("jvcli-state");
        let bridge = jv_desktop_process::bridge::BridgeSupervisor::start(program, selected, data)?;
        *state.bridge.lock().map_err(|_| ErrorCategory::Internal)? = Some(Arc::new(bridge));
        Ok(())
    })
    .await
    .map_err(|_| ErrorCategory::Internal)?
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LoginRequest {
    username: String,
    base_url: String,
    password: String,
    permissions: Permissions,
}
#[tauri::command]
async fn authenticate(
    request: LoginRequest,
    state: tauri::State<'_, DesktopState>,
) -> Result<(), ErrorCategory> {
    let bridge = state
        .bridge
        .lock()
        .map_err(|_| ErrorCategory::Internal)?
        .clone()
        .ok_or(ErrorCategory::AuthenticationRequired)?;
    tauri::async_runtime::spawn_blocking(move || {
        let result = bridge.authenticate(
            request.username,
            request.base_url,
            request.password,
            request.permissions,
        );
        if result.is_err() {
            bridge.shutdown();
        }
        result
    })
    .await
    .map_err(|_| ErrorCategory::Internal)?
}
pub fn run() {
    tauri::Builder::default()
        .manage(DesktopState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            choose_workspace,
            detect_backend,
            run_turn,
            cancel_turn,
            open_bridge,
            close_bridge,
            authenticate
        ])
        .build(tauri::generate_context!())
        .expect("JV Desktop could not initialize")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                let _ = disconnect(&app.state::<DesktopState>());
            }
        });
}
#[cfg(all(test, target_os = "linux"))]
mod tests;
