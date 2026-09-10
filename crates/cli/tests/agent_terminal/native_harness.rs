#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn agent_harness_watchers_survive_turns_but_close_on_new_and_exit() {
    let (home, server) = conversation_fixture(vec![]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    for index in 0..2 {
        session.send("/watch start {\"command\":\"sleep 10\",\"cwd\":\".\",\"reason\":\"test watcher\",\"resource\":\"dev:web\"}\r");
        session.until("Approve this exact session watcher once?");
        session.send("y\r");
        session.until("watch_started");
        session.send("/watch\r");
        session.until("\"running\": true");
        if index == 0 {
            session.send("/new\r");
            session.until("Started a new conversation.");
            let store = dowe_agent::native_harness::HarnessStore::new(
                session.home.path().join(".dowe/agent"),
                session.home.path(),
            )
            .unwrap();
            assert!(store.processes().unwrap().is_empty());
        }
    }
    let home = session.stop();
    assert!(server.join().unwrap().is_empty());
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    assert!(store.processes().unwrap().is_empty());
    let stopped = store
        .sessions()
        .unwrap()
        .into_iter()
        .flat_map(|id| store.load_session(&id).unwrap().events)
        .filter(|event| event["event"] == "watch_stopped")
        .count();
    assert_eq!(stopped, 2);
}

#[test]
fn agent_harness_json_watch_start_never_spawns() {
    let (home, server) = conversation_fixture(vec![]);
    use std::io::Write;
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "--json"])
        .env_clear()
        .env("HOME", home.path())
        .current_dir(home.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"/shell /bin/sh\n/watch start {\"command\":\"touch forbidden\",\"cwd\":\".\",\"reason\":\"test approval\",\"resource\":\"dev:web\"}\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let events: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        events
            .iter()
            .any(|event| event["result"]["event"] == "approval_required")
    );
    assert!(!home.path().join("forbidden").exists());
    assert!(server.join().unwrap().is_empty());
}

#[test]
fn agent_harness_recovery_requires_confirmation_and_preserves_interrupted_history() {
    let (home, server) = conversation_fixture(vec![]);
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    let mut source = store.create_session().unwrap();
    source.interrupted = true;
    store.save_session(&mut source).unwrap();
    let report = store.inspect_session(&source.id, 0).unwrap();
    let command = format!(
        "/recover {} {} Inspect current files",
        source.id,
        report["ticket"].as_str().unwrap()
    );
    {
        use std::io::Write;
        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
            .args(["agent", "--json"])
            .env_clear()
            .env("HOME", home.path())
            .current_dir(home.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        writeln!(child.stdin.take().unwrap(), "{command}").unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("approval_required")
        );
        assert!(store.sessions().unwrap().iter().all(|id| {
            store
                .load_session(id)
                .unwrap()
                .events
                .iter()
                .all(|event| event["event"] != "session_recovered")
        }));
    }
    let session = Session::start_at(home, false, &["agent"]);
    session.send(&format!("{command}\r"));
    session.until("Create a fresh recovery session");
    session.send("y\r");
    session.until("recovered_session");
    let home = session.stop();
    assert!(server.join().unwrap().is_empty());
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    assert_eq!(
        store.load_session(&source.id).unwrap().revision,
        source.revision
    );
    assert!(store.load_session(&source.id).unwrap().interrupted);
    let recovered = store
        .sessions()
        .unwrap()
        .into_iter()
        .map(|id| store.load_session(&id).unwrap())
        .find(|session| {
            session
                .events
                .iter()
                .any(|event| event["event"] == "session_recovered")
        })
        .unwrap();
    assert!(recovered.turns.is_empty());
    assert!(!recovered.interrupted);
}

#[test]
fn agent_harness_memory_status_requires_review_after_invalidation() {
    use std::io::Write;
    let (home, server) = conversation_fixture(vec![]);
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    let id = store
        .remember("Theme", "Theme original decision", "user", true)
        .unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "--json"])
        .env_clear()
        .env("HOME", home.path())
        .current_dir(home.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    write!(child.stdin.take().unwrap(), "/memory invalidate {id} Superseded\n/memory status\n/memory confirm {id}\n/memory update {id} | Theme | Theme reviewed locally\n/memory status\n/exit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("memory requires review"));
    let events: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let statuses: Vec<_> = events
        .iter()
        .filter_map(|event| event["result"]["memories"].as_array())
        .collect();
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[0][0]["eligible"], false);
    assert_eq!(statuses[0][0]["reason"], "invalidated");
    assert_eq!(statuses[1][0]["eligible"], true);
    assert_eq!(store.recall("Theme").unwrap().len(), 1);
    assert!(server.join().unwrap().is_empty());
}

fn harness_write_response() -> serde_json::Value {
    serde_json::json!({"output":[{"type":"function_call","call_id":"write-1","name":"write_file","arguments":serde_json::json!({"path":"main.dowe","content":"main {}\n","skill":"core","reason":"Create Dowe application root"}).to_string()}]})
}

#[test]
fn agent_harness_unknown_model_capabilities_fail_before_request_or_turn() {
    let (home, server) = conversation_fixture(vec![]);
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    let mut config = store.config().unwrap();
    config.capabilities.clear();
    store.save_config(&config).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "--json", "Create theme"])
        .env_clear()
        .env("HOME", home.path())
        .current_dir(home.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("capabilities"));
    assert!(server.join().unwrap().is_empty());
    for id in store.sessions().unwrap() {
        assert!(store.load_session(&id).unwrap().turns.is_empty());
    }
}

#[test]
fn agent_harness_json_never_autoapproves_file_writes() {
    let (home, server) = conversation_fixture(vec![(200, harness_write_response())]);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "--json", "Create a Dowe application"])
        .env_clear()
        .env("HOME", home.path())
        .current_dir(home.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let events = text
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        events
            .iter()
            .any(|event| event["event"] == "approval_required")
    );
    assert!(!home.path().join("main.dowe").exists());
    assert_eq!(server.join().unwrap().len(), 1);
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    let ids = store.sessions().unwrap();
    assert_eq!(ids.len(), 1);
    assert!(!store.load_session(&ids[0]).unwrap().interrupted);
}

#[test]
fn agent_harness_terminal_approves_exact_write_and_returns_real_result() {
    let (home, server) = conversation_fixture(vec![
        (200, harness_write_response()),
        (200, conversation_reply("File applied; validation not run.")),
    ]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("Create a Dowe application\r");
    let approval = session.until("Approve this exact operation once?");
    assert!(approval.contains("File change requested"));
    assert!(approval.contains("main.dowe"));
    assert!(approval.contains("before") && approval.contains("after"));
    assert!(!approval.contains("\"call\""));
    assert!(!approval.contains("\"details\""));
    session.send("y\r");
    session.until("File applied");
    let home = session.stop();
    assert_eq!(
        std::fs::read_to_string(home.path().join("main.dowe")).unwrap(),
        "main {}\n"
    );
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 2);
    let input = requests[1]["input"].as_array().unwrap();
    let result = input
        .iter()
        .find(|item| item["type"] == "function_call_output")
        .unwrap();
    assert_eq!(result["call_id"], "write-1");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(result["output"].as_str().unwrap()).unwrap()["status"],
        "applied"
    );
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn agent_harness_pty_input_is_local_and_not_provider_or_history_data() {
    let response = serde_json::json!({"output":[{"type":"function_call","call_id":"interactive","name":"shell","arguments":serde_json::json!({"command":"printf 'input-ready\\n'; read value; printf '%s' \"$value\"","cwd":".","reason":"interactive app configuration","pty":true}).to_string()}]});
    let (home, server) = conversation_fixture(vec![
        (200, response),
        (200, conversation_reply("Done securely.")),
    ]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("Run interactive app configuration\r");
    session.until("Approve this exact operation once?");
    session.send("y\r");
    session.until("input-ready");
    session.send("only-local-password\r");
    session.until("Done securely.");
    let home = session.stop();
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(
        !serde_json::to_string(&requests)
            .unwrap()
            .contains("only-local-password")
    );
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    for id in store.sessions().unwrap() {
        assert!(
            !serde_json::to_string(&store.load_session(&id).unwrap())
                .unwrap()
                .contains("only-local-password")
        );
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn agent_harness_pty_cancel_stops_the_task_and_releases_owned_processes() {
    let response = serde_json::json!({"output":[{"type":"function_call","call_id":"cancel","name":"shell","arguments":serde_json::json!({"command":"printf 'cancel-ready\\n'; sleep 10","cwd":".","reason":"cancel owned command","pty":true,"resource":"test:cancel"}).to_string()}]});
    let (home, server) = conversation_fixture(vec![(200, response)]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("Run cancellable configuration\r");
    session.until("Approve this exact operation once?");
    session.send("y\r");
    session.until("cancel-ready");
    session.send("\u{3}");
    session.until("Task canceled; no further provider request.");
    let home = session.stop();
    assert_eq!(server.join().unwrap().len(), 1);
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    assert!(store.processes().unwrap().is_empty());
    for id in store.sessions().unwrap() {
        assert!(!store.load_session(&id).unwrap().interrupted);
    }
}

#[test]
fn agent_harness_role_assignment_routes_without_mutating_active_model() {
    let (home, server) = conversation_fixture(vec![(200, conversation_reply("Plan only."))]);
    let path = home.path().join(".dowe/agent/preferences.json");
    let before = std::fs::read(&path).unwrap();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/models plan azure-openai-responses/planner-test\r");
    session.until("precedence");
    session.send("/plan Plan a Dowe theme\r");
    session.until("Plan only.");
    let _home = session.stop();
    assert_eq!(std::fs::read(path).unwrap(), before);
    let requests = server.join().unwrap();
    assert_eq!(requests[0]["model"], "planner-test");
    assert!(
        requests[0]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .all(|tool| tool["name"] != "shell" && tool["name"] != "write_file")
    );
}

#[test]
fn agent_harness_research_routes_to_read_only_tools() {
    let (home, server) = conversation_fixture(vec![(200, conversation_reply("The project uses a Rust CLI."))]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/research Find the CLI entrypoint\r");
    session.until("The project uses a Rust CLI.");
    let home = session.stop();
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 1);
    let body = &requests[0];
    assert!(body["tools"].as_array().unwrap().iter().any(|tool| tool["name"] == "read_file"));
    assert!(body["tools"].as_array().unwrap().iter().all(|tool| {
        !matches!(tool["name"].as_str(), Some("shell" | "write_file" | "edit_file" | "write_asset"))
    }));
    assert!(home.path().join(".agents/capabilities/index.md").is_file());
}

#[test]
fn agent_harness_generic_project_can_apply_a_source_write() {
    let response = serde_json::json!({"output":[{"type":"function_call","call_id":"generic-write","name":"write_file","arguments":serde_json::json!({"path":"src/main.rs","content":"fn main() {}\n","skill":"core","reason":"Create the generic entrypoint"}).to_string()}]});
    let (home, server) = conversation_fixture(vec![(200, response), (200, conversation_reply("Generic source applied."))]);
    std::fs::create_dir_all(home.path().join("src")).unwrap();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("Create the generic Rust entrypoint\r");
    session.until("Approve this exact operation once?");
    session.send("y\r");
    session.until("Generic source applied.");
    let home = session.stop();
    assert_eq!(std::fs::read_to_string(home.path().join("src/main.rs")).unwrap(), "fn main() {}\n");
    assert_eq!(server.join().unwrap().len(), 2);
}
