#[cfg(all(
    any(target_os = "linux", target_os = "windows"),
    feature = "test-fixture"
))]
mod native {
    use jv_desktop_process::{
        bridge::{compatible, BridgeSupervisor},
        Access, ErrorCategory, Permissions,
    };
    use std::{
        path::PathBuf,
        sync::{mpsc, Arc},
        time::Duration,
    };
    fn workspace(name: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.cache/tmp/bridge-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
            .join(name);
        std::fs::create_dir_all(&root).unwrap();
        jv_desktop_process::canonical_path(&root).unwrap()
    }
    fn start(name: &str) -> Result<BridgeSupervisor, ErrorCategory> {
        let w = workspace(name);
        BridgeSupervisor::start(
            PathBuf::from(env!("CARGO_BIN_EXE_fake-bridge")),
            w.clone(),
            w.parent().unwrap().join("state"),
        )
    }
    fn permissions() -> Permissions {
        Permissions {
            access: if cfg!(target_os = "windows") {
                Access::WorkspaceWrite
            } else {
                Access::ReadOnly
            },
            network: cfg!(target_os = "windows"),
        }
    }
    fn login(b: &BridgeSupervisor) {
        b.authenticate(
            "synthetic".into(),
            "http://127.0.0.1:9".into(),
            "TEST_PASSWORD_DO_NOT_LEAK".into(),
            permissions(),
        )
        .unwrap()
    }
    #[test]
    fn version_and_ready() {
        assert!(!compatible("JV CLI 0.4.8"));
        assert!(compatible("JV CLI 0.4.9"));
        assert!(compatible("JV CLI 1.0.0"));
        assert!(!compatible("JV CLI 0.4.9evil"));
        assert!(matches!(start("old"), Err(ErrorCategory::UpgradeRequired)));
        assert!(matches!(
            start("mismatch"),
            Err(ErrorCategory::ProtocolError)
        ));
        let b = start("success").unwrap();
        b.shutdown();
        b.shutdown();
    }
    #[test]
    fn auth_session_and_thai_turn() {
        let b = start("success").unwrap();
        assert!(b
            .run_turn("test".into(), permissions(), |_| Ok(()))
            .is_err());
        assert!(b
            .authenticate(
                "user".into(),
                "http://127.0.0.1".into(),
                "wrong".into(),
                permissions()
            )
            .is_err());
        login(&b);
        let mut events = vec![];
        b.run_turn(
            "$(touch SHOULD_NOT_EXIST)\nไทย".into(),
            permissions(),
            |v| {
                events.push(v);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(events.last().unwrap()["type"], "turn.completed");
        assert!(events.iter().any(|v| v["text"] == "เรียบร้อยแล้ว"));
        assert!(events.iter().any(|v| v["type"] == "files.changed"));
        let serialized = serde_json::to_string(&events).unwrap();
        assert!(!serialized.contains("TEST_PASSWORD"));
        assert!(!serialized.contains("private"));
        b.shutdown();
        let other = start("success").unwrap();
        login(&other);
        other.shutdown();
    }
    #[test]
    fn failure_boundaries() {
        for scenario in ["nonzero", "malformed", "oversized", "failed"] {
            let b = start(scenario).unwrap();
            login(&b);
            let mut events = vec![];
            let result = b.run_turn("test".into(), permissions(), |v| {
                events.push(v);
                Ok(())
            });
            if scenario == "failed" {
                assert!(result.is_ok());
                assert_eq!(events.last().unwrap()["type"], "turn.failed")
            } else {
                assert!(result.is_err())
            }
            b.shutdown();
        }
    }
    #[test]
    fn session_response_cannot_change_platform_execution_policy() {
        let b = start("wrong-policy").unwrap();
        assert_eq!(
            b.authenticate(
                "synthetic".into(),
                "http://127.0.0.1:9".into(),
                "TEST_PASSWORD_DO_NOT_LEAK".into(),
                permissions(),
            ),
            Err(ErrorCategory::ProtocolError)
        );
        assert!(b
            .run_turn("must not execute".into(), permissions(), |_| Ok(()))
            .is_err());
        b.shutdown();
    }
    #[test]
    fn broken_pipe_is_sanitized() {
        let b = start("broken-pipe").unwrap();
        std::thread::sleep(Duration::from_millis(50));
        let error = b
            .authenticate(
                "synthetic".into(),
                "http://127.0.0.1".into(),
                "TEST_PASSWORD_DO_NOT_LEAK".into(),
                permissions(),
            )
            .unwrap_err();
        assert_eq!(error, ErrorCategory::ProcessFailed);
        b.shutdown();
    }
    #[test]
    fn workspace_restart_and_active_shutdown() {
        let b = Arc::new(start("slow").unwrap());
        let first = b.workspace().to_owned();
        login(&b);
        let worker = b.clone();
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            worker.run_turn("test".into(), permissions(), |v| {
                tx.send(v).unwrap();
                Ok(())
            })
        });
        rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert!(b
            .run_turn("concurrent".into(), permissions(), |_| Ok(()))
            .is_err());
        b.shutdown();
        assert!(handle.join().unwrap().is_err());
        let next = start("success").unwrap();
        assert_ne!(first, next.workspace());
        login(&next);
        next.run_turn("new workspace".into(), permissions(), |_| Ok(()))
            .unwrap();
        next.shutdown();
    }
    #[test]
    fn local_cancel_and_shutdown() {
        let b = Arc::new(start("slow").unwrap());
        login(&b);
        let worker = b.clone();
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            worker.run_turn("test".into(), permissions(), |v| {
                tx.send(v).unwrap();
                Ok(())
            })
        });
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(3)).unwrap()["type"],
            "turn.started"
        );
        b.cancel();
        let end = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(end["type"], "turn.cancelled");
        assert_eq!(end["remote_state"], "unknown");
        handle.join().unwrap().unwrap();
        b.shutdown();
    }
}

#[test]
fn shared_version_and_policy_contract() {
    use jv_desktop_process::{
        bridge::{compatible, policy},
        Access, Permissions,
    };
    assert!(compatible("JV CLI 0.4.9"));
    assert!(compatible("JV CLI 0.4.10"));
    for v in ["JV CLI 0.4.8", "JV CLI 0.4.9-beta", "garbage"] {
        assert!(!compatible(v));
    }
    assert!(policy(Permissions {
        access: Access::ReadOnly,
        network: true
    })
    .is_err());
    for (access, network) in [
        (Access::ReadOnly, false),
        (Access::WorkspaceWrite, true),
        (Access::WorkspaceWrite, false),
    ] {
        let result = policy(Permissions { access, network });
        if cfg!(target_os = "windows") && (access != Access::WorkspaceWrite || !network) {
            assert!(result.is_err());
            continue;
        }
        let p = result.unwrap();
        assert_eq!(p["read_only"], access == Access::ReadOnly);
        assert_eq!(p["allow_network"], network);
    }
}
