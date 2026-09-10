#[test]
fn validates_studio_agent_urls_and_run_ids() {
    assert_eq!(
        studio_agent_websocket_url("http://127.0.0.1:7755", "run_1").expect("loopback"),
        "ws://127.0.0.1:7755/api/studio/runs/run_1/stream"
    );
    assert_eq!(
        studio_agent_websocket_url("https://studio.example.com/base", "run-1")
            .expect("secure backend"),
        "wss://studio.example.com/base/api/studio/runs/run-1/stream"
    );
    assert!(studio_agent_websocket_url("http://studio.example.com", "run-1").is_err());
    assert!(validate_studio_run_id("run/1").is_err());
    assert!(validate_studio_run_id("run-1").is_ok());
}

#[test]
fn rejects_nested_folder_names() {
    assert!(validate_folder_name("workspace").is_ok());
    assert!(validate_folder_name("../workspace").is_err());
    assert!(validate_folder_name("workspace/child").is_err());
}

#[test]
fn validates_change_ownership_and_bounds_diff() {
    let plan = r#"{"files":[{"path":"views/pages/home.dowe","owner":"views","operation":"upsert","content":"page Home\n"}]}"#;
    assert_eq!(parse_studio_change_plan(plan).expect("plan").len(), 1);
    let invalid = r#"{"files":[{"path":"server/handlers/home.dowe","owner":"views","content":"handler home\n"}]}"#;
    assert!(parse_studio_change_plan(invalid).is_err());
    let (patch, added, removed) = studio_unified_diff("main.dowe", b"old\n", b"new\n");
    assert!(patch.contains("--- a/main.dowe"));
    assert_eq!((added, removed), (1, 1));
}

#[tokio::test]
async fn stages_applies_and_rolls_back_an_authorized_change() {
    if Command::new("dowe").arg("--version").status().is_err() {
        return;
    }

    let temp = TempDir::new().expect("workspace");
    fs::write(
        temp.path().join("main.dowe"),
        "main\n  app name:\"Before\" bundle:\"dev.dowe.before\"\n",
    )
    .expect("main");
    let context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "",
        "selectedPaths": [],
        "detail": "compact"
    }))
    .await
    .expect("context");
    let fingerprint = context["sourceFingerprint"].as_str().expect("fingerprint");
    let plan = json!({
        "files": [{
            "path": "main.dowe",
            "owner": "core",
            "operation": "upsert",
            "content": "main\n  app name:\"After\" bundle:\"dev.dowe.after\"\n"
        }]
    });
    assert!(
        stage_studio_changes(&json!({
            "root": temp.path(),
            "runId": "run-stage-1",
            "sourceFingerprint": "wrong",
            "query": "",
            "selectedPaths": [],
            "changePlan": plan.to_string()
        }))
        .await
        .is_err()
    );
    let staged = stage_studio_changes(&json!({
        "root": temp.path(),
        "runId": "run-stage-1",
        "sourceFingerprint": fingerprint,
        "query": "",
        "selectedPaths": [],
        "changePlan": plan.to_string()
    }))
    .await
    .expect("stage");
    assert_eq!(staged["ok"], true);
    assert_eq!(staged["status"], "staged");
    assert!(
        staged["validation"]["compiler"]["ok"]
            .as_bool()
            .unwrap_or(false)
    );
    let stage_id = staged["stageId"].as_str().expect("stage id");
    let applied = apply_studio_changes(&json!({
        "root": temp.path(),
        "stageId": stage_id,
        "sourceFingerprint": fingerprint
    }))
    .await
    .expect("apply");
    assert_eq!(applied["status"], "applied");
    assert_eq!(
        apply_studio_changes(&json!({
            "root": temp.path(),
            "stageId": stage_id,
            "sourceFingerprint": fingerprint
        }))
        .await
        .expect("idempotent apply")["status"],
        "applied"
    );
    assert!(
        fs::read_to_string(temp.path().join("main.dowe"))
            .expect("updated")
            .contains("After")
    );
    let rolled_back = rollback_studio_changes(&json!({
        "root": temp.path(),
        "stageId": stage_id
    }))
    .await
    .expect("rollback");
    assert_eq!(rolled_back["status"], "rolled_back");
    assert!(
        fs::read_to_string(temp.path().join("main.dowe"))
            .expect("restored")
            .contains("Before")
    );
    assert_eq!(
        rollback_studio_changes(&json!({
            "root": temp.path(),
            "stageId": stage_id
        }))
        .await
        .expect("idempotent rollback")["status"],
        "rolled_back"
    );
}
