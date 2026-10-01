//! Opt-in, no-authentication compatibility probe. Never submits a JV request.
use jv_desktop_process::{bridge::BridgeSupervisor, resolve_executable, validate_workspace};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(
        args.len(),
        2,
        "usage: bridge_probe WORKSPACE ISOLATED_STATE"
    );
    let program = resolve_executable("jvcli", &std::env::var_os("PATH").unwrap()).unwrap();
    let executable = std::env::current_exe().unwrap();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let workspace = validate_workspace(
        &PathBuf::from(&args[0]),
        home.as_deref(),
        executable.parent().unwrap(),
    )
    .unwrap();
    let bridge = BridgeSupervisor::start(program, workspace, PathBuf::from(&args[1])).unwrap();
    bridge.shutdown();
    println!("Rust supervisor: compatible bridge ready/shutdown; no authentication or turn sent");
}
