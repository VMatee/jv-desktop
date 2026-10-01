//! Synthetic protocol peer, no network client or real credentials.
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
fn emit(v: Value) {
    println!("{v}");
    io::stdout().flush().unwrap();
}
fn main() {
    let cwd = std::env::current_dir().unwrap();
    let scenario = cwd.file_name().unwrap().to_string_lossy();
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        println!(
            "JV CLI {}",
            if scenario == "old" { "0.4.8" } else { "0.4.9" }
        );
        return;
    }
    assert_eq!(args, ["bridge", "--stdio"]);
    assert!(std::env::var_os("JV_API_PASSWORD").is_none());
    assert!(std::env::var_os("JVCLI_HOME").is_some());
    emit(
        json!({"type":"bridge.ready","protocol":if scenario=="mismatch"{2}else{1},"cli_version":"0.4.9","workspace":cwd}),
    );
    if scenario == "broken-pipe" {
        return;
    }
    let mut turn = String::new();
    for line in io::stdin().lock().lines() {
        let v: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = v["request_id"].clone();
        match v["type"].as_str().unwrap() {
            "auth.login" => {
                if v["password"] != "TEST_PASSWORD_DO_NOT_LEAK" {
                    emit(
                        json!({"type":"request.error","request_id":id,"code":"authentication_failed","message":"TEST_PASSWORD_DO_NOT_LEAK"}),
                    );
                } else {
                    emit(json!({"type":"auth.succeeded","request_id":id}));
                }
            }
            "session.start" => emit(
                json!({"type":"session.started","request_id":id,"session_id":"fixture","network":v["allow_network"],"sandbox":if v["read_only"]==true{"read-only"}else{"workspace-write"}}),
            ),
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
                emit(json!({"type":"turn.started","request_id":id,"turn_id":"turn_fixture"}));
                if scenario == "slow" {
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
                emit(
                    json!({"type":if scenario=="failed"{"turn.failed"}else{"turn.completed"},"request_id":id,"turn_id":"turn_fixture"}),
                );
            }
            "turn.cancel" => {
                emit(
                    json!({"type":"turn.cancel.accepted","request_id":id,"turn_id":"turn_fixture"}),
                );
                emit(
                    json!({"type":"turn.cancelled","request_id":turn,"turn_id":"turn_fixture","local":true,"remote_state":"unknown"}),
                );
            }
            "bridge.shutdown" => {
                emit(json!({"type":"bridge.stopping","request_id":id}));
                return;
            }
            _ => panic!("unexpected synthetic request"),
        }
    }
}
