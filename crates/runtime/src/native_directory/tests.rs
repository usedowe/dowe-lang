use super::project_files::*;
use super::studio_agent::*;
use super::studio_apps::*;
use super::studio_changes::*;
use super::studio_context::*;
use super::*;
use dowe_database::{init_database, open_database};
use serde_json::json;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn creates_and_lists_folders() {
    let temp = TempDir::new().expect("tempdir");
    let created = create_folder(&json!({ "parent": temp.path(), "name": "workspace" }))
        .await
        .expect("create");
    assert_eq!(
        created.as_str().map(|value| value.ends_with("workspace")),
        Some(true)
    );
    let folders = list_folders(&json!({ "parent": temp.path() }))
        .await
        .expect("list");
    assert_eq!(folders, json!(["workspace"]));
}

#[test]
fn accepts_only_public_github_repository_urls() {
    assert_eq!(
        normalized_github_repository_url("https://github.com/dowe-lang/example")
            .expect("github url"),
        "https://github.com/dowe-lang/example.git"
    );
    assert_eq!(
        normalized_github_repository_url("https://www.github.com/dowe-lang/example.git")
            .expect("www github url"),
        "https://github.com/dowe-lang/example.git"
    );
    assert!(normalized_github_repository_url("git@github.com:dowe-lang/example.git").is_err());
    assert!(normalized_github_repository_url("https://example.com/dowe-lang/example").is_err());
    assert!(
        normalized_github_repository_url("https://github.com/dowe-lang/example?token=secret")
            .is_err()
    );
}

#[tokio::test]
async fn classifies_only_structurally_valid_dowe_projects() {
    let temp = TempDir::new().expect("tempdir");
    assert_eq!(
        inspect_dowe_project(&json!({ "path": temp.path() }))
            .await
            .expect("empty"),
        "empty"
    );
    fs::write(temp.path().join("main.dowe"), "not a main block\n").expect("invalid main");
    assert_eq!(
        inspect_dowe_project(&json!({ "path": temp.path() }))
            .await
            .expect("invalid"),
        "invalid"
    );
}

#[tokio::test]
async fn initializes_a_blank_project_without_agents() {
    let temp = TempDir::new().expect("project");
    let result = initialize_dowe_project(&json!({ "path": temp.path(), "template": "blank" }))
        .await
        .expect("initialize");
    assert_eq!(
        result,
        json!(
            temp.path()
                .canonicalize()
                .expect("canonical project")
                .to_string_lossy()
                .to_string()
        )
    );
    assert!(temp.path().join("main.dowe").is_file());
    assert!(temp.path().join("views/pages/home.dowe").is_file());
    assert!(!temp.path().join("AGENTS.md").exists());
    assert!(!temp.path().join(".agents").exists());
}

#[tokio::test]
async fn registers_an_app_with_its_source() {
    let studio = TempDir::new().expect("studio root");
    let app = TempDir::new().expect("app root");
    fs::write(app.path().join("main.dowe"), "main\n").expect("main");
    init_database(studio.path(), "dowe-studio-apps").expect("initialize database");

    let inserted = register_studio_app(
        studio.path(),
        &json!({ "path": app.path(), "name": "Imported app", "source": "github" }),
    )
    .await
    .expect("register");

    assert_eq!(inserted["source"], "github");
    assert_eq!(
        open_database(studio.path(), "dowe-studio-apps")
            .expect("database")
            .records("workspace_apps")
            .expect("records")
            .len(),
        1
    );
}

#[tokio::test]
async fn deletes_only_the_local_app_record() {
    let studio = TempDir::new().expect("studio root");
    let app = TempDir::new().expect("app root");
    fs::write(app.path().join("main.dowe"), "main\n").expect("main");
    init_database(studio.path(), "dowe-studio-apps").expect("initialize database");

    let inserted = save_studio_app(
        studio.path(),
        &json!({ "path": app.path(), "name": "Test app" }),
    )
    .await
    .expect("save");
    let id = inserted
        .get("id")
        .and_then(|value| value.as_str())
        .expect("id");

    let deleted = delete_studio_app(studio.path(), &json!({ "id": id }))
        .await
        .expect("delete");

    assert_eq!(deleted, json!({ "changed": 1 }));
    assert!(app.path().join("main.dowe").is_file());
    let database = open_database(studio.path(), "dowe-studio-apps").expect("database");
    assert!(
        database
            .records("workspace_apps")
            .expect("records")
            .is_empty()
    );
}

#[tokio::test]
async fn lists_and_reads_project_files() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir(temp.path().join("views")).expect("views");
    fs::create_dir(temp.path().join("target")).expect("target");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(temp.path().join("views/editor.dowe"), "page editor\n").expect("editor");
    fs::write(temp.path().join("target/generated"), "generated\n").expect("generated");
    fs::write(temp.path().join(".hidden"), "hidden\n").expect("hidden");
    fs::write(temp.path().join("AGENTS.md"), "private\n").expect("agents");

    let files = list_project_files(&json!({ "root": temp.path() }))
        .await
        .expect("list");
    assert_eq!(
        files,
        json!({
            "files": [{ "id":"main.dowe", "name":"main.dowe", "path":"main.dowe", "icon":"code-file" }],
            "folders": [{
                "id":"views",
                "name":"views",
                "path":"views",
                "files": [{ "id":"views/editor.dowe", "name":"editor.dowe", "path":"views/editor.dowe", "icon":"code-file" }],
                "folders": []
            }]
        })
    );

    let content = read_project_file(&json!({
        "root": temp.path(),
        "path": "views/editor.dowe"
    }))
    .await
    .expect("read");
    assert_eq!(content, json!("page editor\n"));
    assert!(
        read_project_file(&json!({
            "root": temp.path(),
            "path": "../outside"
        }))
        .await
        .is_err()
    );
}

#[tokio::test]
async fn prepares_bounded_context_without_private_files() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("views/components")).expect("components");
    fs::create_dir_all(temp.path().join("server")).expect("server");
    fs::create_dir(temp.path().join("target")).expect("target");
    fs::write(
        temp.path().join("main.dowe"),
        "import viewRoutes from \"@/views/routes\"\n\nmain\n  views:viewRoutes\n",
    )
    .expect("main");
    fs::write(
            temp.path().join("views/routes.dowe"),
            "import Card from \"@/views/components/card\"\n\nviews viewRoutes\n  group path:\"/\" layout:Card\n",
        )
        .expect("routes");
    fs::write(
        temp.path().join("views/components/card.dowe"),
        "component Card\n  Text\n    \"Card\"\n",
    )
    .expect("component");
    fs::write(temp.path().join("server/private.dowe"), "fn private\n").expect("server");
    fs::write(temp.path().join(".env"), "OPENROUTER_API_KEY=secret\n").expect("env");
    fs::write(temp.path().join("AGENTS.md"), "private\n").expect("agents");
    fs::write(temp.path().join("target/generated.dowe"), "generated\n").expect("generated");

    let context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "component",
        "selectedPaths": ["views/routes.dowe"]
    }))
    .await
    .expect("context");
    assert_eq!(context["protocolVersion"], 1);
    assert_eq!(context["profile"], "views");
    assert_eq!(context["requestType"], "clarify");
    assert_eq!(context["mode"], "dowe");
    assert_eq!(context["compilerVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(context["workspaceId"].as_str().map(str::len), Some(64));
    let files = context["files"].as_array().expect("files");
    let paths = files
        .iter()
        .filter_map(|file| file["path"].as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"main.dowe"));
    assert!(paths.contains(&"views/routes.dowe"));
    assert!(paths.contains(&"views/components/card.dowe"));
    assert!(!paths.iter().any(|path| path.contains("target")));
    assert!(!paths.iter().any(|path| path.contains("AGENTS")));
    assert!(context["imports"].as_array().is_some_and(|imports| {
        imports
            .iter()
            .any(|value| value == "views/components/card.dowe")
    }));
    assert!(
        context["sourceFingerprint"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
    assert!(
        context["workspaceFingerprint"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
    assert!(temp.path().join(".dowe/studio-context-cache").is_dir());
    assert!(
        !context
            .to_string()
            .contains(&temp.path().to_string_lossy().to_string())
    );
    let compact_context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "component",
        "selectedPaths": [],
        "detail": "compact"
    }))
    .await
    .expect("compact context");
    assert_eq!(compact_context["detail"], "compact");
    assert_eq!(compact_context["image"], "");
    assert!(
        compact_context["files"]
            .as_array()
            .is_some_and(|files| { files.iter().all(|file| file["content"] == "") })
    );
    let read_context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "inspect context",
        "selectedPaths": [],
        "detail": "compact"
    }))
    .await
    .expect("read context");
    assert_eq!(read_context["requestType"], "context_read");
    let image_context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "",
        "selectedPaths": [],
        "image": "data:image/png;base64,AA=="
    }))
    .await
    .expect("image context");
    assert_eq!(image_context["profile"], "viewReference");
    assert_eq!(image_context["requestType"], "vision_ui");
    assert_eq!(image_context["image"], "data:image/png;base64,AA==");
    assert!(
        prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "",
            "selectedPaths": [],
            "image": "data:image/png;base64:not-base64"
        }))
        .await
        .is_err()
    );
    assert!(
        prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "",
            "selectedPaths": ["../outside.dowe"]
        }))
        .await
        .is_err()
    );
}

#[tokio::test]
async fn fingerprints_deep_source_tree_without_recursive_directory_calls() {
    let temp = TempDir::new().expect("tempdir");
    let mut directory = temp.path().to_path_buf();
    for _ in 0..256 {
        directory = directory.join("d");
        fs::create_dir(&directory).expect("directory");
    }
    fs::write(directory.join("deep.dowe"), "page deep\n").expect("source");

    let fingerprint = full_studio_source_fingerprint(temp.path())
        .await
        .expect("fingerprint");

    assert_eq!(fingerprint.len(), 64);
}

#[tokio::test]
async fn writes_project_file_with_relative_path() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(temp.path().join("views.dowe"), "old\n").expect("file");

    let saved = write_project_file(&json!({
        "root": temp.path(),
        "path": "views.dowe",
        "content": "new source\n"
    }))
    .await
    .expect("write");

    assert_eq!(saved, json!(true));
    assert_eq!(
        fs::read_to_string(temp.path().join("views.dowe")).expect("content"),
        "new source\n"
    );
    assert!(
        write_project_file(&json!({
            "root": temp.path(),
            "path": "../outside",
            "content": "unsafe"
        }))
        .await
        .is_err()
    );
    #[cfg(unix)]
    {
        let outside = TempDir::new().expect("outside tempdir");
        fs::write(outside.path().join("source.dowe"), "outside\n").expect("outside file");
        std::os::unix::fs::symlink(
            outside.path().join("source.dowe"),
            temp.path().join("link.dowe"),
        )
        .expect("symlink");
        assert!(
            write_project_file(&json!({
                "root": temp.path(),
                "path": "link.dowe",
                "content": "unsafe"
            }))
            .await
            .is_err()
        );
    }
}

#[tokio::test]
async fn streams_studio_agent_events_through_native_host() {
    use axum::Router;
    use axum::extract::ws::{Message, WebSocketUpgrade};
    use axum::routing::get;
    use futures_util::StreamExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let app = Router::new().route(
            "/api/studio/runs/run_1/stream",
            get(|upgrade: WebSocketUpgrade| async move {
                upgrade.on_upgrade(|mut socket| async move {
                    let Some(Ok(Message::Text(_))) = socket.next().await else {
                        return;
                    };
                    let _ = socket
                        .send(Message::Text(
                            json!({ "event":"started", "requestId":"request-1", "requestType":"implementation", "model":"minimax/minimax-m3", "payload":{} })
                                .to_string()
                                .into(),
                        ))
                        .await;
                    let _ = socket
                        .send(Message::Text(
                            json!({ "event":"delta", "requestId":"request-1", "requestType":"implementation", "model":"minimax/minimax-m3", "content":"done", "payload":{} })
                                .to_string()
                                .into(),
                        ))
                        .await;
                    let _ = socket
                        .send(Message::Text(
                            json!({ "event":"done", "requestId":"request-1", "requestType":"implementation", "model":"minimax/minimax-m3", "payload":{} })
                                .to_string()
                                .into(),
                        ))
                        .await;
                })
            }),
        );
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server");
    });

    let result = studio_agent_stream(&json!({
        "backend": format!("http://{address}"),
        "authorization": "Bearer test-token",
        "runId": "run_1",
        "payload": "{\"requestId\":\"request-1\"}",
    }))
    .await
    .expect("agent stream");
    server.abort();

    assert_eq!(result["ok"], true);
    assert_eq!(result["requestId"], "request-1");
    assert_eq!(result["answer"], "done");
    assert_eq!(result["events"].as_array().map(Vec::len), Some(3));
    assert!(result["events"][0].get("model").is_none());
    assert!(result["events"][0].get("requestType").is_none());
    assert_eq!(result["events"][1]["payload"], "");
}

#[tokio::test]
async fn cancellation_is_safe_when_no_agent_is_active() {
    let result = cancel_studio_agent(&json!({ "runId": "inactive-run" }))
        .await
        .expect("cancel");
    assert_eq!(result["ok"], true);
    assert_eq!(result["active"], false);
    assert_eq!(result["status"], "cancelled");
}

#[tokio::test]
async fn cancellation_closes_an_active_studio_agent_socket() {
    use axum::Router;
    use axum::extract::ws::{Message, WebSocketUpgrade};
    use axum::routing::get;
    use futures_util::StreamExt;
    use std::time::Duration;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let app = Router::new().route(
        "/api/studio/runs/run_cancel/stream",
        get(|upgrade: WebSocketUpgrade| async move {
            upgrade.on_upgrade(|mut socket| async move {
                let _ = socket.next().await;
                tokio::time::sleep(Duration::from_secs(5)).await;
                let _ = socket.send(Message::Close(None)).await;
            })
        }),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server");
    });
    let request_args = json!({
        "backend": format!("http://{address}"),
        "authorization": "Bearer test-token",
        "runId": "run_cancel",
        "payload": "{\"requestId\":\"request-cancel\"}",
    });
    let request = tokio::spawn(async move { studio_agent_stream(&request_args).await });
    tokio::time::sleep(Duration::from_millis(100)).await;
    let cancellation = cancel_studio_agent(&json!({ "runId": "run_cancel" }))
        .await
        .expect("cancel");
    let result = tokio::time::timeout(Duration::from_secs(2), request)
        .await
        .expect("cancelled agent")
        .expect("agent task")
        .expect("agent result");
    server.abort();

    assert_eq!(cancellation["active"], true);
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"], "studio_agent_cancelled");
}

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
