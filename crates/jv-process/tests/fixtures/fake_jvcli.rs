//! Deterministic, offline executable. It cannot submit jobs or make HTTP calls.
use serde_json::json;
use std::{
    io::{self, Read, Write},
    time::Duration,
};

fn emit(value: serde_json::Value) {
    println!("{value}");
    io::stdout().flush().unwrap();
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        println!("JV CLI fixture-1");
        return;
    }
    let scenario = args
        .windows(2)
        .find(|a| a[0] == "--fixture")
        .map(|a| a[1].as_str())
        .unwrap_or("");
    if scenario.is_empty() {
        std::process::exit(90);
    }
    if scenario == "descendant" {
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    if scenario == "no-stdin" {
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    let mut prompt = String::new();
    io::stdin().read_to_string(&mut prompt).unwrap();
    if scenario == "inspect" {
        // Synthetic prompts only in tests. This output is not a production log.
        emit(
            json!({"args": args, "cwd": std::env::current_dir().unwrap(), "prompt": prompt,
            "env_keys": std::env::vars().map(|(k,_)| k).collect::<Vec<_>>() }),
        );
        return;
    }
    if scenario == "malformed" {
        println!("not JSON");
        return;
    }
    if scenario == "unknown" {
        emit(json!({"type":"future.event","payload":"ignored"}));
        return;
    }
    if scenario == "oversized" {
        println!("{}", "x".repeat(70 * 1024));
        emit(json!({"type":"turn.completed"}));
        return;
    }
    if scenario == "invalid-utf8" {
        io::stdout().write_all(&[0xff, b'\n']).unwrap();
        return;
    }
    if scenario == "flood" {
        for _ in 0..5000 {
            println!("{{\"type\":\"turn.started\"}}");
        }
        return;
    }
    if scenario == "flood-no-newline" {
        for _ in 0..4000 {
            println!("{{\"type\":\"turn.started\"}}");
        }
        print!("{{\"type\":\"turn.completed\"}}");
        return;
    }
    if scenario == "stderr-flood" {
        let _ = io::stderr().write_all(&vec![b'x'; 5 * 1024 * 1024]);
        return;
    }
    if scenario == "nonzero" {
        eprintln!("fixture diagnostic must not reach the UI");
        std::process::exit(7);
    }
    if scenario == "slow" || scenario == "tree" {
        if scenario == "tree" {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--fixture", "descendant"])
                .spawn()
                .unwrap();
            emit(json!({"fixture_child_pid":child.id()}));
            let _ = child.wait();
            return;
        }
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    emit(json!({"type":"thread.started","thread_id":"thread_test"}));
    emit(json!({"type":"turn.started"}));
    emit(
        json!({"type":"item.started","item":{"id":"1","type":"command_execution","command":"pytest"}}),
    );
    emit(
        json!({"type":"item.completed","item":{"id":"1","type":"command_execution","command":"pytest","exit_code":if scenario=="command-fails" {1} else {0},"aggregated_output":"fixture checks complete"}}),
    );
    emit(
        json!({"type":"item.completed","item":{"id":"2","type":"file_change","changes":[{"path":"example.txt","kind":"update"}]}}),
    );
    emit(
        json!({"type":"item.completed","item":{"id":"3","type":"agent_message","text":"เรียบร้อยแล้ว 🟢"}}),
    );
    emit(json!({"type":"turn.completed"}));
}
