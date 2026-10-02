#![cfg(all(
    any(target_os = "linux", target_os = "windows"),
    feature = "test-fixture"
))]
use jv_desktop_process::{
    exec_args,
    process::{detect, run_process, Frame, ProcessSpec},
    Access, ErrorCategory, Permissions,
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

fn spec(scenario: &str) -> ProcessSpec {
    let mut args = exec_args(Permissions {
        access: Access::ReadOnly,
        network: false,
    })
    .unwrap();
    args.extend(["--fixture".into(), scenario.into()]);
    ProcessSpec {
        program: PathBuf::from(env!("CARGO_BIN_EXE_fake-jvcli")),
        args: args.into_iter().map(Into::into).collect(),
        cwd: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        stdin: "ช่วยตรวจ project นี้\n$(not executable)".as_bytes().to_vec(),
    }
}
fn run(
    scenario: &str,
) -> (
    Result<jv_desktop_process::process::ProcessResult, ErrorCategory>,
    Vec<Frame>,
) {
    let mut frames = vec![];
    let result = run_process(
        spec(scenario),
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(3),
        |f| frames.push(f),
    );
    (result, frames)
}
#[test]
fn success_and_thai_and_structured_activity() {
    let (result, frames) = run("success");
    assert_eq!(result.unwrap().exit_code, Some(0));
    let lines: Vec<_> = frames
        .into_iter()
        .filter_map(|f| {
            if let Frame::Line(s) = f {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines.len(), 7);
    assert!(lines.iter().any(|s| s.contains("เรียบร้อยแล้ว")));
    for line in lines {
        assert!(serde_json::from_str::<serde_json::Value>(&line).is_ok());
    }
}
#[test]
fn argv_cwd_stdin_and_minimal_environment() {
    let mut s = spec("inspect");
    let cwd = s.cwd.join("../../.cache/tmp/work space;$(not-a-command)");
    std::fs::create_dir_all(&cwd).unwrap();
    s.cwd = jv_desktop_process::canonical_path(&cwd).unwrap();
    let expected_cwd = s.cwd.clone();
    let mut frames = vec![];
    run_process(
        s,
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(3),
        |f| frames.push(f),
    )
    .unwrap();
    let Frame::Line(line) = &frames[0] else {
        panic!("missing fixture output")
    };
    let value: serde_json::Value = serde_json::from_str(line).unwrap();
    assert_eq!(value["cwd"], expected_cwd.to_string_lossy().as_ref());
    assert_eq!(value["prompt"], "ช่วยตรวจ project นี้\n$(not executable)");
    assert_eq!(
        value["args"],
        serde_json::json!(["exec", "--json", "--read-only", "--fixture", "inspect"])
    );
    let keys = value["env_keys"].as_array().unwrap();
    let allowed: &[&str] = if cfg!(target_os = "windows") {
        &[
            "PATH",
            "SYSTEMROOT",
            "WINDIR",
            "USERPROFILE",
            "LOCALAPPDATA",
            "TEMP",
            "TMP",
        ]
    } else {
        &[
            "LANG",
            "PATH",
            "PYTHONDONTWRITEBYTECODE",
            "PYTHONNOUSERSITE",
        ]
    };
    assert!(keys.iter().all(|k| allowed.contains(&k.as_str().unwrap())));
}
#[test]
fn framing_discards_oversize_and_invalid_utf8_then_recovers() {
    let (result, frames) = run("oversized");
    assert!(result.is_ok());
    assert_eq!(frames[0], Frame::Diagnostic("oversized-event"));
    assert!(matches!(&frames[1], Frame::Line(line) if line.contains("turn.completed")));
    assert_eq!(run("invalid-utf8").1, [Frame::Diagnostic("invalid-utf8")]);
    assert!(matches!(&run("malformed").1[0], Frame::Line(line) if line == "not JSON"));
    assert!(matches!(&run("unknown").1[0], Frame::Line(line) if line.contains("future.event")));
}
#[test]
fn failures_limits_and_detection() {
    assert_eq!(run("nonzero").0, Err(ErrorCategory::ProcessFailed));
    assert!(run("nonzero").1.is_empty());
    assert_eq!(run("flood").0, Err(ErrorCategory::StreamLimit));
    assert_eq!(run("flood-no-newline").0, Err(ErrorCategory::StreamLimit));
    assert_eq!(run("stderr-flood").0, Err(ErrorCategory::StreamLimit));
    let s = spec("success");
    assert!(detect(s.program, s.cwd).is_ok());
    let mut s = spec("success");
    s.program = s.cwd.join("does-not-exist");
    assert_eq!(
        run_process(
            s,
            Arc::new(AtomicBool::new(false)),
            Duration::from_secs(1),
            |_| {}
        ),
        Err(ErrorCategory::Unavailable)
    );
    assert!(run("command-fails")
        .1
        .iter()
        .any(|f| matches!(f, Frame::Line(s) if s.contains("\"exit_code\":1"))));
}
#[test]
fn timeout_and_pre_cancel_are_bounded() {
    let start = Instant::now();
    assert_eq!(
        run_process(
            spec("slow"),
            Arc::new(AtomicBool::new(false)),
            Duration::from_millis(60),
            |_| {}
        ),
        Err(ErrorCategory::Timeout)
    );
    assert!(start.elapsed() < Duration::from_secs(3));
    assert!(
        run_process(
            spec("slow"),
            Arc::new(AtomicBool::new(true)),
            Duration::from_secs(1),
            |_| {}
        )
        .unwrap()
        .cancelled
    );
    let mut blocked = spec("no-stdin");
    blocked.stdin = vec![b'x'; 32 * 1024];
    assert_eq!(
        run_process(
            blocked,
            Arc::new(AtomicBool::new(false)),
            Duration::from_millis(60),
            |_| {}
        ),
        Err(ErrorCategory::Timeout)
    );
}
#[test]
fn cancellation_stops_process_tree_and_reaps_leader() {
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let mut child_id = None;
    let start = Instant::now();
    let result = run_process(spec("tree"), cancel, Duration::from_secs(3), |frame| {
        if let Frame::Line(line) = frame {
            let value: serde_json::Value = serde_json::from_str(&line).unwrap();
            child_id = value["fixture_child_pid"].as_u64();
            flag.store(true, Ordering::Release);
        }
    })
    .unwrap();
    assert!(result.cancelled);
    assert!(start.elapsed() < Duration::from_secs(3));
    let pid = child_id.expect("fixture descendant");
    // The container's init may retain an orphan zombie. Z is not a running child.
    #[cfg(target_os = "linux")]
    if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        assert_eq!(stat.rsplit_once(") ").unwrap().1.chars().next(), Some('Z'));
    }
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_OBJECT_0},
            System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE},
        };
        // SAFETY: query a fixture PID; never terminate a caller-selected process.
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid as u32) };
        if !handle.is_null() {
            assert_eq!(unsafe { WaitForSingleObject(handle, 1000) }, WAIT_OBJECT_0);
            unsafe {
                CloseHandle(handle);
            }
        }
    }
}
