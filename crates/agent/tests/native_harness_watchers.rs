#![cfg(any(target_os = "macos", target_os = "linux"))]
use dowe_agent::native_harness::*;
use serde_json::json;
use std::time::{Duration, Instant};

fn approve(tools: &mut HarnessTools, command: &str, resource: &str) -> Approval {
    tools.prepare(&ToolCall::new("watch-call", "shell", json!({"command":command,"cwd":".","reason":"test watcher","resource":resource,"watch":true})), HarnessRole::Execute).unwrap().unwrap()
}

#[test]
fn watchers_reject_duplicates_and_stop_only_owned_handles() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let config = HarnessConfig {
        shell: Some("/bin/sh".into()),
        shell_timeout_ms: 10000,
        ..Default::default()
    };
    let mut tools = HarnessTools::new(root.path(), &session.id, config).unwrap();
    let mut watchers = HarnessWatchers::default();
    let approval = approve(&mut tools, "printf 'ready\\n'; sleep 10", "dev:web");
    let id = watchers.start(&store, &mut tools, approval).unwrap();
    let duplicate = approve(&mut tools, "printf 'ready\\n'; sleep 10", "different-key");
    assert!(watchers.start(&store, &mut tools, duplicate).is_err());
    let second = approve(&mut tools, "printf 'other\\n'; sleep 10", "dev:server");
    let other = watchers.start(&store, &mut tools, second).unwrap();
    assert!(store.delete_session(&session.id).is_err());
    assert!(watchers.stop("unowned-pid").is_err());
    watchers.stop(&id).unwrap();
    let status = watchers.snapshot(&other).unwrap();
    assert_eq!(status["running"], true);
    let started = Instant::now();
    watchers.stop_all();
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(store.processes().unwrap().is_empty());
}

#[test]
fn watcher_capture_reports_truncation_exit_failure_and_timeout() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let config = HarnessConfig {
        shell: Some("/bin/sh".into()),
        max_output_bytes: 1024,
        shell_timeout_ms: 500,
        ..Default::default()
    };
    let mut tools = HarnessTools::new(root.path(), &session.id, config).unwrap();
    let mut watchers = HarnessWatchers::default();
    for (command, resource, timed_out) in [
        (
            "i=0; while [ $i -lt 500 ]; do printf 'abcdefghijk\\n'; i=$((i+1)); done; exit 3",
            "failure",
            false,
        ),
        ("sleep 10", "timeout", true),
    ] {
        let approval = approve(&mut tools, command, resource);
        let id = watchers.start(&store, &mut tools, approval).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let state = watchers.snapshot(&id).unwrap();
            if state["running"] == false {
                assert_eq!(state["result"]["timed_out"], timed_out);
                assert_eq!(state["result"]["success"], false);
                if !timed_out {
                    assert_eq!(state["result"]["exit_code"], 3);
                    assert_eq!(state["truncated"], true);
                    assert!(state["stdout"].as_str().unwrap().len() <= 1024);
                }
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    assert!(store.processes().unwrap().is_empty());
}

#[test]
fn watcher_output_is_bounded_redacted_and_drop_reaps_processes() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(".env"), "TOKEN=watch-secret-value\n").unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let config = HarnessConfig {
        shell: Some("/bin/sh".into()),
        max_output_bytes: 1024,
        shell_timeout_ms: 10000,
        ..Default::default()
    };
    let mut tools = HarnessTools::new(root.path(), &session.id, config).unwrap();
    let mut watchers = HarnessWatchers::default();
    let approval = approve(
        &mut tools,
        "cat .env; printf 'done\\n'; sleep 10",
        "dev:web",
    );
    let id = watchers.start(&store, &mut tools, approval).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let status = watchers.snapshot(&id).unwrap();
        let text = status.to_string();
        assert!(!text.contains("watch-secret-value"));
        if !status["stdout"].as_str().unwrap().is_empty() {
            assert!(text.contains("[REDACTED]") || text.contains("withheld"));
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(watchers);
    assert!(store.processes().unwrap().is_empty());
}
