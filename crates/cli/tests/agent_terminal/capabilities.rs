#[test]
fn agent_harness_capabilities_exposes_offline_provenance_and_override_precedence() {
    use std::io::Write;
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
    child.stdin.take().unwrap().write_all(b"/capabilities\n/capabilities anthropic/claude-sonnet-4-5\n/capabilities anthropic/claude-sonnet-4-5 tools:false images:false\n/capabilities anthropic/claude-sonnet-4-5\n/capabilities anthropic/claude-sonnet-4-5 inherit\n/capabilities anthropic/claude-sonnet-4-5\n/capabilities google/gemini-3.1-flash-tts-preview\n/capabilities google/unverified-model\n/exit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let events: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let inventory = events
        .iter()
        .map(|event| &event["result"])
        .find(|value| value["builtin_models"].is_array())
        .unwrap();
    assert_eq!(inventory["catalog_version"], 1);
    let models = inventory["builtin_models"].as_array().unwrap();
    assert_eq!(models.len(), 29);
    assert!(
        models
            .windows(2)
            .all(|pair| pair[0].as_str() < pair[1].as_str())
    );
    let inspections: Vec<_> = events
        .iter()
        .map(|event| &event["result"])
        .filter(|value| value["effective_source"].is_string())
        .collect();
    assert_eq!(inspections.len(), 5);
    for index in [0, 2] {
        assert_eq!(inspections[index]["effective_source"], "builtin_snapshot");
        assert_eq!(inspections[index]["effective"]["tools"], true);
        assert_eq!(inspections[index]["effective"]["images"], true);
        assert_eq!(
            inspections[index]["builtin_evidence"]["tools_evidence"]["sha256"],
            "87e737a50fa73fe6b55fad499a2b23f16e6c4e1fffad818197e0c112ecb5aae7"
        );
        assert_eq!(
            inspections[index]["builtin_evidence"]["images_evidence"]["retrieved_at"],
            "2026-09-07"
        );
    }
    assert_eq!(inspections[1]["effective_source"], "local_declaration");
    assert_eq!(inspections[1]["effective"]["tools"], false);
    assert_eq!(inspections[1]["builtin"]["tools"], true);
    assert_eq!(inspections[3]["effective"]["tools"], false);
    assert_eq!(inspections[3]["effective"]["images"], false);
    assert_eq!(inspections[4]["effective_source"], "unknown");
    assert!(inspections[4]["effective"].is_null());
    assert!(inspections[4]["builtin_evidence"].is_null());
    assert!(
        inspections
            .iter()
            .all(|value| value["execution_approval"] == false)
    );
    assert!(server.join().unwrap().is_empty());
}

#[test]
fn agent_harness_documented_limitations_fail_before_request_attachment_or_turn() {
    for (model, images, message) in [
        (
            "google/gemini-3.1-flash-tts-preview",
            false,
            "does not support harness tools",
        ),
        (
            "minimax/MiniMax-M2.7",
            true,
            "does not support images; no attachment sent",
        ),
    ] {
        let (home, server) = conversation_fixture(vec![]);
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"));
        command.args(["agent", "--json", "--model", model]);
        if images {
            command.args(["--image", "missing.png"]);
        }
        let output = command
            .arg("Create theme")
            .env_clear()
            .env("HOME", home.path())
            .current_dir(home.path())
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(server.join().unwrap().is_empty());
        let store = dowe_agent::native_harness::HarnessStore::new(
            home.path().join(".dowe/agent"),
            home.path(),
        )
        .unwrap();
        for id in store.sessions().unwrap() {
            assert!(store.load_session(&id).unwrap().turns.is_empty());
        }
        assert!(!home.path().join("main.dowe").exists());
    }
}
