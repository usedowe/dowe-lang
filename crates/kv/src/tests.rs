use crate::{
    KvError, KvServerConfig, RemoteKvClient, RemoteKvConfig, clear_memory, create_user,
    init_database, list_databases, open_database, start_kv_server, verify_user,
};
use serde_json::json;

#[test]
fn creates_multiple_persistent_databases() {
    let temp = tempfile::tempdir().expect("temp");

    init_database(temp.path(), "db1").expect("db1");
    init_database(temp.path(), "db2").expect("db2");

    assert!(temp.path().join(".dowe/kv/db1/metadata.bin").exists());
    assert!(temp.path().join(".dowe/kv/db2/metadata.bin").exists());
    assert_eq!(
        list_databases(temp.path()).expect("list"),
        vec!["db1".to_string(), "db2".to_string()]
    );
}

#[test]
fn reads_memory_before_persistence_fallback() {
    let temp = tempfile::tempdir().expect("temp");
    let db = open_database(temp.path(), "cache", true).expect("db");

    db.set("greeting", json!("persisted")).expect("set");
    clear_memory(temp.path(), "cache").expect("clear memory");
    assert_eq!(
        db.get("greeting").expect("fallback"),
        Some(json!("persisted"))
    );

    db.set("greeting", json!("memory")).expect("set memory");
    assert_eq!(db.get("greeting").expect("memory"), Some(json!("memory")));
}

#[test]
fn volatile_database_does_not_read_persistence() {
    let temp = tempfile::tempdir().expect("temp");
    let persistent = open_database(temp.path(), "cache", true).expect("persistent");
    persistent.set("greeting", json!("persisted")).expect("set");
    clear_memory(temp.path(), "cache").expect("clear memory");
    let volatile = open_database(temp.path(), "cache", false).expect("volatile");

    assert_eq!(volatile.get("greeting").expect("get"), None);
}

#[test]
fn keys_delete_and_clear_apply_persistence() {
    let temp = tempfile::tempdir().expect("temp");
    let db = open_database(temp.path(), "cache", true).expect("db");

    db.set("appointment:1", json!({ "id": 1 })).expect("set 1");
    db.set("user:1", json!({ "id": 1 })).expect("set 2");
    clear_memory(temp.path(), "cache").expect("clear memory");

    assert_eq!(
        db.keys(Some("appointment:")).expect("keys"),
        vec!["appointment:1".to_string()]
    );
    assert!(db.delete("appointment:1").expect("delete"));
    assert_eq!(db.get("appointment:1").expect("get"), None);
    assert_eq!(db.clear().expect("clear"), 1);
    assert!(db.keys(None).expect("keys").is_empty());
}

#[test]
fn creates_and_verifies_database_users() {
    let temp = tempfile::tempdir().expect("temp");
    let created = create_user(temp.path(), "clinic", "clinic-api", Some("secret")).expect("user");

    assert_eq!(created.database, "clinic");
    assert!(!created.generated);
    assert!(temp.path().join(".dowe/kv/clinic/metadata.bin").exists());
    assert!(temp.path().join(".dowe/kv/_auth/users.bin").exists());
    verify_user(temp.path(), "clinic", "clinic-api", "secret").expect("verify");
    let error = verify_user(temp.path(), "other", "clinic-api", "secret").expect_err("authz");
    assert!(matches!(error, KvError::Authorization(_)));
}

#[tokio::test]
async fn remote_server_requires_auth_and_executes_kv_operations() {
    let temp = tempfile::tempdir().expect("temp");
    create_user(temp.path(), "clinic", "clinic-api", Some("secret")).expect("user");
    let server = start_kv_server(KvServerConfig {
        root: temp.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("server");
    let client = RemoteKvClient::new(RemoteKvConfig {
        host: format!("http://{}", server.addr),
        database: "clinic".to_string(),
        user: "clinic-api".to_string(),
        credential: "secret".to_string(),
        persist: true,
    })
    .expect("client");

    assert_eq!(
        client
            .set("appointment:1", json!({ "id": "1" }))
            .await
            .expect("set"),
        json!({ "ok": true, "key": "appointment:1" })
    );
    assert_eq!(
        client.get("appointment:1", true).await.expect("get"),
        json!({ "id": "1" })
    );
    assert_eq!(
        client.keys(Some("appointment:")).await.expect("keys"),
        json!(["appointment:1"])
    );
    assert_eq!(
        client.delete("appointment:1").await.expect("delete"),
        json!({ "deleted": true })
    );

    let bad_client = RemoteKvClient::new(RemoteKvConfig {
        host: format!("http://{}", server.addr),
        database: "clinic".to_string(),
        user: "clinic-api".to_string(),
        credential: "bad".to_string(),
        persist: true,
    })
    .expect("bad client");
    let error = bad_client
        .get("appointment:1", false)
        .await
        .expect_err("auth");
    assert!(matches!(error, KvError::Authentication(_)));

    server.shutdown().await.expect("shutdown");
}
