#[tokio::test]
async fn websocket_service_requires_auth_and_executes_database_operations() {
    let temp = TempDir::new().expect("tempdir");
    create_account(temp.path(), "clinic", "clinic-api", Some("secret-token")).expect("account");
    let server = start_database_service(DatabaseServiceConfig {
        root: temp.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("server");
    let client = DoweDatabaseClient::new(DoweDatabaseConfig {
        host: "127.0.0.1".to_string(),
        port: server.addr.port(),
        database: "clinic".to_string(),
        account: "clinic-api".to_string(),
        secret: "secret-token".to_string(),
    })
    .expect("client");

    let inserted = tokio::time::timeout(
        Duration::from_secs(5),
        client.insert("appointments", serde_json::json!({"patientName":"Ana"})),
    )
    .await
    .expect("insert timeout")
    .expect("insert");
    let id = inserted["id"].as_str().expect("id").to_string();

    let list = client.list("appointments").await.expect("list");
    assert_eq!(list.as_array().expect("array").len(), 1);

    let read = client
        .read(
            "appointments",
            vec![("id".to_string(), serde_json::Value::String(id.clone()))],
            true,
        )
        .await
        .expect("read");
    assert_eq!(read["patientName"], "Ana");

    let changed = client
        .update(
            "appointments",
            vec![("id".to_string(), serde_json::Value::String(id.clone()))],
            serde_json::json!({"patientName":"Bea"}),
            true,
        )
        .await
        .expect("update");
    assert_eq!(changed["changed"], 1);

    let rows = client
        .query("select * from appointments")
        .await
        .expect("query");
    assert_eq!(rows[0]["patientName"], "Bea");

    let transaction = client
        .transaction(&[
            DatabaseTransactionInsert {
                table: "messages".to_string(),
                value: serde_json::json!({"recipient":"+15550000001"}),
            },
            DatabaseTransactionInsert {
                table: "messages".to_string(),
                value: serde_json::json!({"recipient":"+15550000002"}),
            },
        ])
        .await
        .expect("transaction");
    assert_eq!(transaction.as_array().expect("array").len(), 2);
    assert_eq!(
        client
            .list("messages")
            .await
            .expect("messages")
            .as_array()
            .expect("array")
            .len(),
        2
    );

    let existing_id = transaction[0]["id"].as_str().expect("transaction id");
    let failed = client
        .transaction(&[
            DatabaseTransactionInsert {
                table: "messages".to_string(),
                value: serde_json::json!({"recipient":"+15550000003"}),
            },
            DatabaseTransactionInsert {
                table: "messages".to_string(),
                value: serde_json::json!({"id":existing_id,"recipient":"+15550000004"}),
            },
        ])
        .await
        .expect_err("transaction conflict");
    assert!(matches!(
        failed,
        StoreError::AlreadyExists(_) | StoreError::TransactionConflict(_)
    ));
    assert_eq!(
        client
            .list("messages")
            .await
            .expect("messages after conflict")
            .as_array()
            .expect("array")
            .len(),
        2
    );

    let deleted = client
        .delete(
            "appointments",
            vec![("id".to_string(), serde_json::Value::String(id))],
            true,
        )
        .await
        .expect("delete");
    assert_eq!(deleted["changed"], 1);

    let bad_client = DoweDatabaseClient::new(DoweDatabaseConfig {
        host: "127.0.0.1".to_string(),
        port: server.addr.port(),
        database: "clinic".to_string(),
        account: "clinic-api".to_string(),
        secret: "wrong".to_string(),
    })
    .expect("bad client");
    let error = bad_client
        .list("appointments")
        .await
        .expect_err("auth error");
    assert!(matches!(error, StoreError::Authentication(_)));

    drop(client);
    drop(bad_client);
    server.shutdown().await.expect("shutdown");
}
