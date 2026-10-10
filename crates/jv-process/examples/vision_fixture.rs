//! Opt-in Linux acceptance against the CLI repository's synthetic loopback fixture.
//! Fixed synthetic credentials; never use a production service or installed state.
#[cfg(target_os = "linux")]
fn main() {
    use jv_desktop_process::{bridge::BridgeSupervisor, Access, Permissions};
    use std::path::PathBuf;
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 4, "CLI WORKSPACE ISOLATED_STATE LOOPBACK_URL");
    let port = args[3].strip_prefix("http://127.0.0.1:").unwrap();
    assert!(port.parse::<u16>().is_ok_and(|p| p != 0));
    let state = PathBuf::from(&args[2]);
    assert!(
        !state.exists(),
        "Acceptance needs a fresh isolated state path"
    );
    let bridge =
        BridgeSupervisor::start(PathBuf::from(&args[0]), PathBuf::from(&args[1]), state).unwrap();
    let permissions = Permissions {
        access: Access::WorkspaceWrite,
        network: true,
    };
    bridge
        .authenticate_with_transport(
            "synthetic".into(),
            args[3].clone(),
            "SYNTHETIC_VISION_PASSWORD".into(),
            permissions,
            true,
        )
        .unwrap();
    for _ in 0..2 {
        let mut completed = false;
        bridge
            .run_turn(
                "Run the synthetic screenshot coding acceptance.".into(),
                permissions,
                |event| {
                    let text = event.to_string();
                    for forbidden in [
                        "SYNTHETIC_VISION_PASSWORD",
                        "SYNTHETIC_VISION_TOKEN",
                        "base64,",
                        "image_url",
                    ] {
                        assert!(!text.contains(forbidden));
                    }
                    assert_ne!(
                        event["type"], "turn.failed",
                        "Synthetic turn failed: {event}"
                    );
                    completed |= event["type"] == "turn.completed";
                    Ok(())
                },
            )
            .unwrap();
        assert!(completed);
    }
    bridge.shutdown();
    println!("Desktop Linux supervisor: two structured coding turns passed; bounded UI events contain no image payloads or synthetic credentials.");
}

#[cfg(not(target_os = "linux"))]
fn main() {
    panic!("Linux acceptance only");
}
