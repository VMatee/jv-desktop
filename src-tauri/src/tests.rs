//! Exercise the registered IPC boundary using Tauri's mock runtime, never a CLI.
//! This does not replace a real WebView/folder-picker acceptance test.
use super::*;
use serde_json::{json, Value};
use tauri::{ipc::InvokeBody, test, webview::InvokeRequest, WebviewWindow};

fn app() -> (
    tauri::App<test::MockRuntime>,
    WebviewWindow<test::MockRuntime>,
    PathBuf,
) {
    let workspace =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.cache/tmp/native-ipc-workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let workspace = workspace.canonicalize().unwrap();
    let app = test::mock_builder()
        .manage(DesktopState {
            selected: Mutex::new(Some(workspace.clone())),
        })
        .invoke_handler(tauri::generate_handler![run_turn, cancel_turn])
        .build(tauri::generate_context!())
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .data_directory(workspace.parent().unwrap().join("native-ipc-webview"))
        .build()
        .unwrap();
    (app, window, workspace)
}

fn invoke(
    window: &WebviewWindow<test::MockRuntime>,
    command: &str,
    body: Value,
) -> Result<tauri::ipc::InvokeResponseBody, Value> {
    test::get_ipc_response(
        window,
        InvokeRequest {
            cmd: command.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: test::INVOKE_KEY.into(),
        },
    )
}

fn request(workspace: &std::path::Path) -> Value {
    json!({
        "request": {
            "runId": "offline-boundary-test",
            "workspace": workspace,
            "prompt": "ข้อความทดสอบ — offline only",
            "permissions": { "access": "read-only", "network": false }
        },
        "onEvent": "__CHANNEL__:1"
    })
}

#[test]
fn registered_native_turn_stays_gated_for_every_valid_policy() {
    let (_app, window, workspace) = app();
    for (access, network) in [
        ("read-only", false),
        ("workspace-write", false),
        ("workspace-write", true),
    ] {
        let mut body = request(&workspace);
        body["request"]["permissions"] = json!({"access": access, "network": network});
        assert_eq!(
            invoke(&window, "run_turn", body).err(),
            Some(json!("authentication-required"))
        );
    }
    assert!(invoke(
        &window,
        "cancel_turn",
        json!({"runId": "offline-boundary-test"})
    )
    .is_ok());
}

#[test]
fn ipc_rejects_invalid_workspace_and_executable_injection() {
    let (_app, window, workspace) = app();
    assert_eq!(
        invoke(&window, "run_turn", request(std::path::Path::new("/"))).err(),
        Some(json!("invalid-workspace"))
    );
    let mut body = request(&workspace);
    body["request"]["executable"] = json!("untrusted-executable");
    let error = invoke(&window, "run_turn", body).err().unwrap();
    assert!(error
        .as_str()
        .unwrap()
        .contains("unknown field `executable`"));
}

#[test]
fn unused_core_commands_are_not_authorized() {
    let (_app, window, _) = app();
    let error = invoke(&window, "plugin:app|version", json!({}))
        .err()
        .unwrap();
    assert!(error.as_str().unwrap().contains("not allowed"));
}
