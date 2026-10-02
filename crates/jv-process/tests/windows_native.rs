#![cfg(all(target_os = "windows", feature = "test-fixture"))]
use jv_desktop_process::{
    bridge::{policy, BridgeSupervisor},
    canonical_path, validate_workspace, Access, ErrorCategory as E, Permissions,
};
use std::{
    fs,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::WAIT_OBJECT_0,
    System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE},
};
fn workspace(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.cache/tmp/windows-native")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
        .join(name);
    fs::create_dir_all(&root).unwrap();
    canonical_path(&root).unwrap()
}
fn permissions() -> Permissions {
    Permissions {
        access: Access::WorkspaceWrite,
        network: true,
    }
}
fn start(w: &Path) -> BridgeSupervisor {
    BridgeSupervisor::start(
        env!("CARGO_BIN_EXE_fake-bridge").into(),
        w.into(),
        w.parent().unwrap().join("state"),
    )
    .unwrap()
}
fn login(b: &BridgeSupervisor) {
    b.authenticate(
        "synthetic".into(),
        "http://127.0.0.1:9".into(),
        "TEST_PASSWORD_DO_NOT_LEAK".into(),
        permissions(),
    )
    .unwrap();
}
struct Watch(OwnedHandle);
impl Watch {
    fn new(pid: u32) -> Self {
        // SAFETY: only fixture PIDs produced by this test; handle reserves identity.
        let h = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
        assert!(!h.is_null(), "fixture must still be running");
        Self(unsafe { OwnedHandle::from_raw_handle(h) })
    }
    fn exited(&self) -> bool {
        // SAFETY: owned process handle, never a reused PID.
        unsafe { WaitForSingleObject(self.0.as_raw_handle(), 0) == WAIT_OBJECT_0 }
    }
    fn assert_exit(&self) {
        // SAFETY: owned process handle with synchronize access.
        assert_eq!(
            unsafe { WaitForSingleObject(self.0.as_raw_handle(), 5000) },
            WAIT_OBJECT_0
        );
    }
}
fn record(w: &Path, name: &str, key: &str) -> u32 {
    let start = Instant::now();
    loop {
        if let Ok(bytes) = fs::read(w.join(name)) {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                return v[key].as_u64().unwrap() as u32;
            }
        }
        assert!(start.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn sentinel(w: &Path) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_fake-bridge"))
        .arg("--fixture-grandchild")
        .current_dir(w)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

#[test]
fn windows_trusted_host_rejects_conflicting_policies_before_execution() {
    let w = workspace("success");
    let b = start(&w);
    for p in [
        Permissions {
            access: Access::ReadOnly,
            network: false,
        },
        Permissions {
            access: Access::ReadOnly,
            network: true,
        },
        Permissions {
            access: Access::WorkspaceWrite,
            network: false,
        },
    ] {
        assert_eq!(policy(p), Err(E::PermissionConflict));
        assert_eq!(
            b.authenticate(
                "synthetic".into(),
                "http://127.0.0.1:9".into(),
                "TEST_PASSWORD_DO_NOT_LEAK".into(),
                p
            ),
            Err(E::PermissionConflict)
        );
        assert_eq!(
            b.run_turn("test".into(), p, |_| Ok(())),
            Err(E::PermissionConflict)
        );
    }
    login(&b);
    b.shutdown();
}
#[test]
fn readiness_validates_workspace_and_version() {
    for name in ["wrong-workspace", "ready-old"] {
        let w = workspace(name);
        assert!(matches!(
            BridgeSupervisor::start(
                env!("CARGO_BIN_EXE_fake-bridge").into(),
                w.clone(),
                w.parent().unwrap().join("state")
            ),
            Err(E::ProtocolError)
        ));
    }
}
#[test]
fn trusted_host_session_must_confirm_full_access_and_network() {
    for name in ["wrong-policy", "wrong-network", "wrong-auth-correlation"] {
        let w = workspace(name);
        let b = start(&w);
        assert_eq!(
            b.authenticate(
                "synthetic".into(),
                "http://127.0.0.1:9".into(),
                "TEST_PASSWORD_DO_NOT_LEAK".into(),
                permissions()
            ),
            Err(E::ProtocolError)
        );
        assert_eq!(
            b.run_turn("test".into(), permissions(), |_| Ok(())),
            Err(E::AuthenticationRequired)
        );
        b.shutdown();
    }
}
#[test]
fn pipe_fragmentation_crlf_unicode_and_final_line() {
    for name in ["fragmented", "final-line"] {
        let w = workspace(name);
        let b = start(&w);
        login(&b);
        let mut events = vec![];
        b.run_turn(
            "สวัสดี\n$(literal prompt)".into(),
            permissions(),
            |v| {
                events.push(v);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(events.last().unwrap()["type"], "turn.completed");
        b.shutdown();
    }
}
#[test]
fn multiple_turns_keep_one_session_and_correlation() {
    let w = workspace("success");
    let b = start(&w);
    login(&b);
    for text in ["first", "second ไทย"] {
        let mut events = vec![];
        b.run_turn(text.into(), permissions(), |v| {
            events.push(v);
            Ok(())
        })
        .unwrap();
        assert_eq!(events.first().unwrap()["type"], "turn.started");
        assert_eq!(events.last().unwrap()["type"], "turn.completed");
    }
    b.shutdown();
}
#[test]
fn malformed_correlation_limits_crash_and_utf8_close_safely() {
    for name in [
        "invalid-utf8",
        "stderr-flood",
        "flood",
        "byte-flood",
        "wrong-turn-correlation",
        "wrong-event-correlation",
        "crash",
    ] {
        let w = workspace(name);
        let b = start(&w);
        login(&b);
        let watch = Watch::new(b.bridge_pid());
        let result = b.run_turn("synthetic".into(), permissions(), |_| Ok(()));
        assert!(
            matches!(
                result,
                Err(E::ProtocolError | E::StreamLimit | E::ProcessFailed)
            ),
            "{name}: {result:?}"
        );
        b.shutdown();
        watch.assert_exit();
    }
}
#[test]
fn blocked_stdin_deadline_releases_writer_and_owned_process() {
    let w = workspace("blocked-input");
    let b = start(&w);
    let watch = Watch::new(b.bridge_pid());
    let start = Instant::now();
    assert_eq!(
        b.authenticate(
            "synthetic".into(),
            "http://127.0.0.1:9".into(),
            "x".repeat(16384),
            permissions()
        ),
        Err(E::Timeout)
    );
    assert!(start.elapsed() < Duration::from_secs(5));
    b.shutdown();
    watch.assert_exit();
}
#[test]
fn shutdown_interrupts_auth_and_prevents_late_session_installation() {
    let w = workspace("auth-hang");
    let b = Arc::new(start(&w));
    let watch = Watch::new(b.bridge_pid());
    let worker = b.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    let handle = std::thread::spawn(move || {
        tx.send(worker.authenticate(
            "synthetic".into(),
            "http://127.0.0.1:9".into(),
            "TEST_PASSWORD_DO_NOT_LEAK".into(),
            permissions(),
        ))
        .unwrap();
    });
    std::thread::sleep(Duration::from_millis(50));
    let start = Instant::now();
    b.shutdown();
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        Err(E::ProcessFailed)
    );
    handle.join().unwrap();
    assert!(start.elapsed() < Duration::from_secs(3));
    watch.assert_exit();
    assert!(b
        .run_turn("late".into(), permissions(), |_| Ok(()))
        .is_err());
}
#[test]
fn cooperative_cleanup_covers_nested_child_grandchild_and_not_sentinel() {
    let w = workspace("tree");
    let sw = workspace("sentinel");
    let mut sentinel = sentinel(&sw);
    let b = start(&w);
    let root = Watch::new(b.bridge_pid());
    let child = Watch::new(record(&w, "child.json", "pid"));
    let grandchild = Watch::new(record(&w, "grandchild.json", "pid"));
    let v: serde_json::Value =
        serde_json::from_slice(&fs::read(w.join("grandchild.json")).unwrap()).unwrap();
    assert_eq!(v["job"], true);
    b.shutdown();
    root.assert_exit();
    child.assert_exit();
    grandchild.assert_exit();
    assert!(sentinel.try_wait().unwrap().is_none());
    sentinel.kill().unwrap();
    sentinel.wait().unwrap();
}
#[test]
fn forced_cleanup_covers_owned_tree_after_shutdown_deadline() {
    let w = workspace("hung-tree");
    let b = start(&w);
    let child = Watch::new(record(&w, "child.json", "pid"));
    let grandchild = Watch::new(record(&w, "grandchild.json", "pid"));
    let start = Instant::now();
    b.shutdown();
    assert!(start.elapsed() < Duration::from_secs(11));
    child.assert_exit();
    grandchild.assert_exit();
}
#[test]
fn parent_death_closes_job_and_cleans_bridge_and_nested_descendants() {
    let w = workspace("tree");
    let sw = workspace("unrelated");
    let mut sentinel = sentinel(&sw);
    let mut owner = Command::new(env!("CARGO_BIN_EXE_bridge-owner"))
        .args([
            Path::new(env!("CARGO_BIN_EXE_fake-bridge")),
            &w,
            &w.parent().unwrap().join("state"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let bridge = Watch::new(record(&w, "owner.json", "bridge"));
    let child = Watch::new(record(&w, "child.json", "pid"));
    let grandchild = Watch::new(record(&w, "grandchild.json", "pid"));
    assert!(!bridge.exited());
    owner.kill().unwrap();
    owner.wait().unwrap();
    bridge.assert_exit();
    child.assert_exit();
    grandchild.assert_exit();
    assert!(sentinel.try_wait().unwrap().is_none());
    sentinel.kill().unwrap();
    sentinel.wait().unwrap();
}
#[test]
fn eof_of_owner_causes_cooperative_shutdown() {
    let w = workspace("tree");
    let mut owner = Command::new(env!("CARGO_BIN_EXE_bridge-owner"))
        .args([
            Path::new(env!("CARGO_BIN_EXE_fake-bridge")),
            &w,
            &w.parent().unwrap().join("state"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let bridge = Watch::new(record(&w, "owner.json", "bridge"));
    let child = Watch::new(record(&w, "child.json", "pid"));
    let grandchild = Watch::new(record(&w, "grandchild.json", "pid"));
    owner.stdin.take();
    bridge.assert_exit();
    child.assert_exit();
    grandchild.assert_exit();
    assert!(owner.wait().unwrap().success());
}
#[test]
fn cancelled_turn_is_local_and_stops_owned_descendants_before_shutdown() {
    let w = workspace("tree");
    let b = Arc::new(start(&w));
    login(&b);
    let child = Watch::new(record(&w, "child.json", "pid"));
    let grandchild = Watch::new(record(&w, "grandchild.json", "pid"));
    let worker = b.clone();
    let (tx, rx) = mpsc::sync_channel(4);
    let turn = std::thread::spawn(move || {
        worker.run_turn("synthetic".into(), permissions(), |v| {
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
    turn.join().unwrap().unwrap();
    child.assert_exit();
    grandchild.assert_exit();
    b.shutdown();
}
#[test]
fn workspace_handles_drive_roots_spaces_thai_and_aliases() {
    let w = workspace("work space ไทย");
    let app = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(validate_workspace(&w, None, app).unwrap(), w);
    assert_eq!(validate_workspace(&w.join("."), None, app).unwrap(), w);
    assert_eq!(
        validate_workspace(
            Path::new(&w.to_str().unwrap().to_ascii_lowercase()),
            None,
            app
        )
        .unwrap(),
        w
    );
    assert_eq!(validate_workspace(&w, None, &w), Err(E::InvalidWorkspace));
    assert_eq!(
        validate_workspace(&w, Some(&w), app),
        Err(E::InvalidWorkspace)
    );
    assert!(validate_workspace(Path::new("C:\\"), None, app).is_err());
    for path in [
        r"relative\project",
        r"\\server\share\project",
        r"\\.\C:\project",
    ] {
        assert!(validate_workspace(Path::new(path), None, app).is_err());
    }
    let file = w.join("file.txt");
    fs::write(&file, "test").unwrap();
    assert!(validate_workspace(&file, None, app).is_err());
}
#[test]
fn junction_workspace_and_state_paths_fail_closed() {
    let w = workspace("junction");
    let target = w.join("target");
    fs::create_dir(&target).unwrap();
    let link = w.join("link");
    let ps = PathBuf::from(std::env::var_os("SYSTEMROOT").unwrap())
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let quote = |p: &Path| p.to_str().unwrap().replace('\'', "''");
    // Test setup only: fixed synthetic filesystem paths, no prompt/credential.
    let status = Command::new(ps).args(["-NoProfile","-NonInteractive","-Command", &format!("$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path '{}' -Value '{}' | Out-Null",quote(&link),quote(&target))]).status().unwrap();
    assert!(status.success());
    assert!(validate_workspace(&link, None, Path::new(env!("CARGO_MANIFEST_DIR"))).is_err());
    assert!(BridgeSupervisor::start(
        env!("CARGO_BIN_EXE_fake-bridge").into(),
        w.clone(),
        link.join("state")
    )
    .is_err());
    fs::remove_dir(&link).unwrap();
}
