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

