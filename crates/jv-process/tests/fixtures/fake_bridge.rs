//! Synthetic protocol peer, no network client or real credentials.
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
fn emit(v: Value) {
    println!("{v}");
    io::stdout().flush().unwrap();
}
fn main() {
    let current_version = jv_desktop_process::bridge::MINIMUM_CLI_VERSION;
    let old_version = if cfg!(target_os = "windows") {
        "0.4.9"
    } else {
        "0.4.8"
    };
    let cwd = jv_desktop_process::canonical_path(&std::env::current_dir().unwrap()).unwrap();
    let scenario = cwd.file_name().unwrap().to_string_lossy();
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--fixture-grandchild"] {
        std::fs::write(
            cwd.join("grandchild.json"),
            json!({"pid":std::process::id(),"job":in_job()}).to_string(),
        )
        .unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    if args == ["--fixture-child"] {
        let spec = jv_desktop_process::process::ProcessSpec {
            program: std::env::current_exe().unwrap(),
            args: vec!["--fixture-grandchild".into()],
            cwd,
            stdin: vec![],
        };
        let _ = jv_desktop_process::process::run_process(
            spec,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            std::time::Duration::from_secs(300),
            |_| {},
        );
        return;
    }
    if args == ["--version"] {
        println!(
            "JV CLI {}",
            if scenario == "old" {
                old_version
            } else {
                current_version
            }
        );
        return;
    }
    #[cfg(target_os = "linux")]
    assert_eq!(args, ["bridge", "--stdio"]);
    #[cfg(target_os = "windows")]
    assert_eq!(
        args,
        ["bridge", "--stdio", "--execution-mode", "trusted-host"]
    );
    assert!(std::env::var_os("JV_API_PASSWORD").is_none());
    for key in [
        "OPENAI_API_KEY",
        "JV_API_TOKEN",
        "JVCLI_ADAPTER_KEY",
        "PARENT_SECRET",
        "PYTHONPATH",
        "NODE_OPTIONS",
        "HTTP_PROXY",
        "HTTPS_PROXY",
    ] {
        assert!(
            std::env::var_os(key).is_none(),
            "unreviewed environment inherited"
        );
    }
    assert!(std::env::var_os("JVCLI_HOME").is_some());
    #[cfg(target_os = "windows")]
    assert!(in_job());
    let mut owned_child = None;
    if scenario == "tree" || scenario == "hung-tree" {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--fixture-child")
            .current_dir(&cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        std::fs::write(
            cwd.join("child.json"),
            json!({"pid":child.id()}).to_string(),
        )
        .unwrap();
        owned_child = Some(child);
        let start = std::time::Instant::now();
        while !cwd.join("grandchild.json").is_file() {
            assert!(start.elapsed().as_secs() < 5);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    let mut ready = json!({"type":"bridge.ready","protocol":if scenario=="mismatch"{2}else{1},"cli_version":current_version,"workspace":cwd});
    if scenario == "wrong-workspace" {
        ready["workspace"] = json!("unselected");
    }
    if scenario == "ready-old" {
        ready["cli_version"] = json!(old_version);
    }
    eprintln!("Synthetic trusted-host diagnostic; stderr is not protocol output");
    if scenario == "fragmented" {
        let bytes = format!("{ready}\r\n");
        for chunk in bytes.as_bytes().chunks(3) {
            io::stdout().write_all(chunk).unwrap();
            io::stdout().flush().unwrap();
        }
    } else {
        emit(ready);
    }
    if scenario == "broken-pipe" {
        return;
    }
    if scenario == "blocked-input" {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    let mut turn = String::new();
    for line in io::stdin().lock().lines() {
        let v: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = v["request_id"].clone();
        match v["type"].as_str().unwrap() {
            "auth.login" => {
                if scenario == "auth-hang" {
                    continue;
                }
                if v["password"] != "TEST_PASSWORD_DO_NOT_LEAK" {
                    emit(
                        json!({"type":"request.error","request_id":id,"code":"authentication_failed","message":"TEST_PASSWORD_DO_NOT_LEAK"}),
                    );
                } else {
                    emit(json!({"type":"auth.succeeded","request_id":id}));
                }
            }
            "session.start" => {
                let mut response = json!({"type":"session.started","request_id":id,"session_id":"fixture","network":v["allow_network"],"sandbox":if cfg!(target_os = "windows") {"danger-full-access"} else if v["read_only"]==true{"read-only"}else{"workspace-write"}});
                if scenario == "wrong-policy" {
                    response["sandbox"] = json!(if cfg!(target_os = "windows") {
                        "workspace-write"
                    } else {
                        "danger-full-access"
                    });
                }
                if scenario == "wrong-network" {
                    response["network"] = json!(!v["allow_network"].as_bool().unwrap());
                }
                if scenario == "wrong-auth-correlation" {
                    response["request_id"] = json!("foreign");
                }
                emit(response);
            }
            "turn.start" => {
                turn = id.as_str().unwrap().into();
                if scenario == "oversized" {
                    emit(json!({"type":"future.event","value":"x".repeat(140_000)}));
                    continue;
                }
                if scenario == "nonzero" {
                    std::process::exit(7)
                }
                if scenario == "malformed" {
                    println!("{{bad");
                    io::stdout().flush().unwrap();
                    continue;
                }
                if scenario == "invalid-utf8" {
                    io::stdout().write_all(&[0xff, b'\n']).unwrap();
                    io::stdout().flush().unwrap();
                    continue;
                }
                if scenario == "stderr-flood" {
                    let _ = io::stderr().write_all(&vec![b'x'; 70 * 1024]);
                    continue;
                }
                if scenario == "flood" {
                    for _ in 0..4100 {
                        emit(json!({"type":"future.event"}));
                    }
                    continue;
                }
                if scenario == "byte-flood" {
                    for _ in 0..100 {
                        emit(json!({"type":"future.event","value":"x".repeat(60000)}));
                    }
                    continue;
                }
                if scenario == "wrong-turn-correlation" {
                    emit(
                        json!({"type":"turn.started","request_id":"foreign","turn_id":"turn_fixture"}),
                    );
                    continue;
                }
                if scenario == "crash" {
                    std::process::exit(9);
                }
                emit(json!({"type":"turn.started","request_id":id,"turn_id":"turn_fixture"}));
                if scenario == "auth-resume" || scenario == "auth-wait" {
                    emit(
                        json!({"type":"turn.status", "turn_id":"turn_fixture", "source":"remote",
                        "status":"waiting_for_auth", "support_code":"JV-SVC-103",
                        "public_message":"The AI service requires administrator authentication.", "private":"SECRET"}),
                    );
                    if scenario == "auth-wait" {
                        continue;
                    }
                    emit(
                        json!({"type":"turn.status", "turn_id":"turn_fixture", "source":"remote", "status":"running"}),
                    );
                }
                if scenario == "slow"
                    || scenario == "tree"
                    || scenario == "hung-tree"
                    || v["text"] == "FIXTURE_SLOW_TURN"
                {
                    continue;
                }
                emit(json!({"type":"future.event","private":"never forwarded"}));
                emit(
                    json!({"type":"command.started","turn_id":"turn_fixture","command":"synthetic check"}),
                );
                emit(
                    json!({"type":"command.completed","turn_id":"turn_fixture","command":"synthetic check","output":"ok","exit_code":0}),
                );
                emit(
                    json!({"type":"files.changed","turn_id":"turn_fixture","changes":[{"path":"ไฟล์.txt","kind":"update"}]}),
                );
                emit(json!({"type":"tool.started","turn_id":"turn_fixture","name":"synthetic"}));
                emit(
                    json!({"type":"assistant.message","turn_id":"turn_fixture","text":"เรียบร้อยแล้ว"}),
                );
                if scenario == "wrong-event-correlation" {
                    emit(
                        json!({"type":"assistant.message","turn_id":"foreign","text":"not this turn"}),
                    );
                    continue;
                }
                if scenario == "final-line" {
                    print!(
                        "{}",
                        json!({"type":"turn.completed","request_id":id,"turn_id":"turn_fixture"})
                    );
                    io::stdout().flush().unwrap();
                    return;
                }
                emit(
                    json!({"type":if scenario=="failed"{"turn.failed"}else{"turn.completed"},"request_id":id,"turn_id":"turn_fixture"}),
                );
            }
            "turn.cancel" => {
                cleanup(&mut owned_child);
                emit(
                    json!({"type":"turn.cancel.accepted","request_id":id,"turn_id":"turn_fixture"}),
                );
                emit(
                    json!({"type":"turn.cancelled","request_id":turn,"turn_id":"turn_fixture","local":true,"remote_state":"unknown"}),
                );
            }
            "bridge.shutdown" => {
                if scenario == "hung-tree" {
                    continue;
                }
                cleanup(&mut owned_child);
                emit(json!({"type":"bridge.stopping","request_id":id}));
                return;
            }
            _ => panic!("unexpected synthetic request"),
        }
    }
    cleanup(&mut owned_child);
}

fn cleanup(child: &mut Option<std::process::Child>) {
    if let Some(mut child) = child.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn in_job() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::{
            System::JobObjects::IsProcessInJob, System::Threading::GetCurrentProcess,
        };
        let mut member = 0;
        // SAFETY: query only this fixture's own process using the pseudo handle.
        unsafe {
            IsProcessInJob(GetCurrentProcess(), std::ptr::null_mut(), &mut member) != 0
                && member != 0
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
