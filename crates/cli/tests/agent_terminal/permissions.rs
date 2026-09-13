#[test]
fn agent_harness_json_cannot_enable_full_permission_mode() {
    let (home, server) = conversation_fixture(vec![]);
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
    use std::io::Write;
    writeln!(
        child.stdin.take().unwrap(),
        "/permissions full\n/permissions status\n/exit"
    )
    .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("never auto-approve"));
    let events = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        events
            .iter()
            .all(|event| event["result"]["mode"] != "full_access")
    );
    assert!(server.join().unwrap().is_empty());
}

#[test]
fn agent_harness_full_permission_mode_applies_unclassified_file_without_prompt() {
    let response = serde_json::json!({
        "output":[{"type":"function_call","call_id":"full-write","name":"write_file","arguments":serde_json::json!({"path":"Dockerfile","content":"FROM scratch\n","skill":"full-access","reason":"create application image definition"}).to_string()}]
    });
    let (home, server) = conversation_fixture(vec![
        (200, response),
        (200, conversation_reply("Full access applied.")),
    ]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/permissions full\r");
    session.until("full_access");
    session.send("Create the application image definition\r");
    let output = session.until("Full access applied.");
    assert!(!output.contains("Approve this exact operation once?"));
    let home = session.stop();
    assert_eq!(
        std::fs::read_to_string(home.path().join("Dockerfile")).unwrap(),
        "FROM scratch\n"
    );
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    assert!(store.sessions().unwrap().into_iter().any(|id| {
        store.load_session(&id).unwrap().events.iter().any(|event| {
            event["event"] == "operation_finished"
                && event["call_id"] == "full-write"
                && event["failed"] == false
        })
    }));
    assert_eq!(server.join().unwrap().len(), 2);
}

#[test]
fn agent_permissions_mode_is_reversible_and_resets_on_new_session() {
    let (home, server) = conversation_fixture(vec![]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/permissions full\r");
    session.until("full_access");
    session.send("/permissions confirm\r");
    session.until("mode: confirm");
    session.send("/permissions full\r");
    session.until("full_access");
    session.send("/new\r");
    session.until("Started a new conversation.");
    session.send("/permissions status\r");
    session.until("mode: confirm");
    let _home = session.stop();
    assert!(server.join().unwrap().is_empty());
}

#[test]
fn agent_permissions_menu_selects_full_access_for_session() {
    let (home, server) = conversation_fixture(vec![]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/permissions\r");
    session.until("Full access for this session");
    session.send("\u{1b}[B\r");
    session.until("mode: full_access");
    let _home = session.stop();
    assert!(server.join().unwrap().is_empty());
}
