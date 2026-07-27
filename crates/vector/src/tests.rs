use crate::{
    DoweVectorClient, DoweVectorConfig, VectorError, VectorServerConfig, close_remote_connections,
    create_account, init_database, list_databases, open_database, start_vector_server,
    verify_account,
};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn local_database_persists_and_searches_by_cosine_similarity() {
    let root = tempdir().expect("root");
    let database = init_database(root.path(), "articles").expect("database");
    database
        .upsert("alpha", vec![1.0, 0.0], json!({ "kind": "guide" }))
        .expect("alpha");
    database
        .upsert("beta", vec![0.8, 0.2], json!({ "kind": "guide" }))
        .expect("beta");
    database
        .upsert("gamma", vec![0.0, 1.0], json!({ "kind": "news" }))
        .expect("gamma");

    let reopened = open_database(root.path(), "articles", false).expect("reopen");
    let matches = reopened
        .search(&[1.0, 0.0], 10, 0.0, Some(&json!({ "kind": "guide" })))
        .expect("search");
    assert_eq!(
        matches
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "beta"]
    );
    assert_eq!(reopened.inspect().expect("inspect").dimensions, Some(2));
    assert_eq!(list_databases(root.path()).expect("list"), vec!["articles"]);
}

#[test]
fn local_database_rejects_invalid_vectors_and_dimensions() {
    let root = tempdir().expect("root");
    let database = init_database(root.path(), "articles").expect("database");
    assert!(matches!(
        database.upsert("zero", vec![0.0, 0.0], json!({})),
        Err(VectorError::InvalidRequest(_))
    ));
    assert!(matches!(
        database.upsert("overflow", vec![f32::MAX, f32::MAX], json!({})),
        Err(VectorError::InvalidRequest(_))
    ));
    database
        .upsert("valid", vec![1.0, 0.0], json!({}))
        .expect("valid");
    assert!(matches!(
        database.upsert("wrong", vec![1.0, 0.0, 0.0], json!({})),
        Err(VectorError::InvalidRequest(_))
    ));
}

#[test]
fn account_secrets_are_scoped_to_a_database() {
    let root = tempdir().expect("root");
    create_account(root.path(), "articles", "service", Some("secret")).expect("account");
    create_account(root.path(), "private", "other", Some("other-secret")).expect("other");
    verify_account(root.path(), "articles", "service", "secret").expect("verify");
    assert!(matches!(
        verify_account(root.path(), "private", "service", "secret"),
        Err(VectorError::Authorization(_))
    ));
    assert!(matches!(
        verify_account(root.path(), "articles", "service", "invalid"),
        Err(VectorError::Authentication(_))
    ));
    let auth = std::fs::read_to_string(root.path().join(".dowe/vector/_auth/accounts.json"))
        .expect("auth");
    assert!(!auth.contains("\"secret\""));
}

#[tokio::test]
async fn remote_client_executes_vector_operations() {
    let root = tempdir().expect("root");
    create_account(root.path(), "articles", "service", Some("secret")).expect("account");
    init_database(root.path(), "private").expect("private");
    let server = start_vector_server(VectorServerConfig {
        root: root.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("server");
    let invalid_secret = DoweVectorClient::new(DoweVectorConfig {
        host: "127.0.0.1".to_string(),
        port: server.addr.port(),
        account: "service".to_string(),
        secret: "invalid".to_string(),
        name: "articles".to_string(),
    })
    .expect("invalid secret client");
    assert!(matches!(
        invalid_secret
            .upsert("blocked", vec![1.0, 0.0], json!({}))
            .await,
        Err(VectorError::Authentication(_))
    ));
    let wrong_database = DoweVectorClient::new(DoweVectorConfig {
        host: "127.0.0.1".to_string(),
        port: server.addr.port(),
        account: "service".to_string(),
        secret: "secret".to_string(),
        name: "private".to_string(),
    })
    .expect("wrong database client");
    assert!(matches!(
        wrong_database
            .upsert("blocked", vec![1.0, 0.0], json!({}))
            .await,
        Err(VectorError::Authorization(_))
    ));
    let client = DoweVectorClient::new(DoweVectorConfig {
        host: "127.0.0.1".to_string(),
        port: server.addr.port(),
        account: "service".to_string(),
        secret: "secret".to_string(),
        name: "articles".to_string(),
    })
    .expect("client");
    let inserted = client
        .upsert("alpha", vec![1.0, 0.0], json!({ "kind": "guide" }))
        .await
        .expect("upsert");
    assert_eq!(inserted["id"], "alpha");
    let matches = client
        .search(vec![1.0, 0.0], 10, 0.5, Some(json!({ "kind": "guide" })))
        .await
        .expect("search");
    assert_eq!(matches[0]["id"], "alpha");
    let value = client.read("alpha", true).await.expect("read");
    assert_eq!(value["vector"], json!([1.0, 0.0]));
    let entries = client.list(10, None).await.expect("list");
    assert_eq!(entries[0]["id"], "alpha");
    assert_eq!(
        client.delete("alpha").await.expect("delete")["deleted"],
        true
    );
    drop(invalid_secret);
    drop(wrong_database);
    drop(client);
    close_remote_connections();
    server.shutdown().await.expect("shutdown");
}
