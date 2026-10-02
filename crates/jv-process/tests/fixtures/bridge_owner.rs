//! Disposable supervisor parent; optional loopback-only synthetic rejected login.
use jv_desktop_process::bridge::BridgeSupervisor;
use std::{
    io::{self, BufRead},
    path::PathBuf,
};
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert!((3..=4).contains(&args.len()));
    let workspace = jv_desktop_process::canonical_path(&PathBuf::from(&args[1])).unwrap();
    let bridge = BridgeSupervisor::start(
        PathBuf::from(&args[0]),
        workspace.clone(),
        PathBuf::from(&args[2]),
    )
    .unwrap();
    if args.len() == 4 {
        let url = args[3].to_str().unwrap();
        assert!(url.starts_with("http://127.0.0.1:"));
        // Public protocol against an HTTP-401 loopback fixture, never a real account.
        assert_eq!(
            bridge.authenticate(
                "synthetic".into(),
                url.into(),
                "TEST_PASSWORD_DO_NOT_LEAK".into(),
                jv_desktop_process::Permissions {
                    access: jv_desktop_process::Access::WorkspaceWrite,
                    network: true,
                }
            ),
            Err(jv_desktop_process::ErrorCategory::AuthenticationRequired)
        );
    }
    std::fs::write(
        workspace.join("owner.json"),
        serde_json::json!({"bridge":bridge.bridge_pid()}).to_string(),
    )
    .unwrap();
    println!("bridge owned and ready");
    let _ = io::stdin().lock().lines().next();
    bridge.shutdown();
}
