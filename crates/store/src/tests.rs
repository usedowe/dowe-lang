use super::{
    RemoteStoreClient, RemoteStoreConfig, StoreError, StoreServerConfig, StoreValue, create_user,
    init_database, open_database, start_store_server, verify_user,
};
use std::collections::BTreeMap;
use tempfile::TempDir;

#[test]
fn initializes_database_with_stable_ulid_metadata() {
    let temp = TempDir::new().expect("tempdir");
    let metadata = init_database(temp.path(), "db1").expect("metadata");
    let reopened = open_database(temp.path(), "db1").expect("database");

    assert_eq!(metadata.database_id, reopened.metadata().database_id);
    assert!(temp.path().join(".dowe/store/db1/metadata.bin").exists());
}

#[test]
fn inserts_records_and_creates_table_layout() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    let mut record = BTreeMap::new();
    record.insert("name".to_string(), StoreValue::String("Ana".to_string()));
    let inserted = db.insert("users", record).expect("insert");

    assert!(matches!(inserted.get("id"), Some(StoreValue::Ulid(_))));
    assert!(temp.path().join(".dowe/store/db1/users/segments").exists());
    assert!(temp.path().join(".dowe/store/db1/users/wal").exists());
    assert_eq!(db.records("users").expect("records").len(), 1);
}

#[test]
fn rejects_invalid_primary_keys() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    let mut record = BTreeMap::new();
    record.insert("id".to_string(), StoreValue::String("bad".to_string()));

    let error = db.insert("users", record).expect_err("error");

    assert_eq!(error.category(), "InvalidUlid");
}

#[test]
fn queries_filters_and_joins() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    db.create_index("users", "roleId").expect("index");
    let mut role = BTreeMap::new();
    role.insert(
        "id".to_string(),
        StoreValue::String("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
    );
    role.insert("name".to_string(), StoreValue::String("Admin".to_string()));
    db.insert("roles", role).expect("role");
    let mut user = BTreeMap::new();
    user.insert("name".to_string(), StoreValue::String("Ana".to_string()));
    user.insert(
        "roleId".to_string(),
        StoreValue::String("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
    );
    db.insert("users", user).expect("user");

    let rows = db
        .query("select users.name, roles.name from users join roles on users.roleId = roles.id")
        .expect("join");

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].get("users.name"),
        Some(&StoreValue::String("Ana".to_string()))
    );
}

#[test]
fn rolls_back_uncommitted_transaction() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    let mut tx = db.transaction();
    let mut record = BTreeMap::new();
    record.insert("name".to_string(), StoreValue::String("Ana".to_string()));
    tx.insert("users", record).expect("stage");
    tx.rollback();

    assert!(db.records("users").expect("records").is_empty());
}

#[test]
fn commits_transaction() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    let mut tx = db.transaction();
    let mut record = BTreeMap::new();
    record.insert("name".to_string(), StoreValue::String("Ana".to_string()));
    tx.insert("users", record).expect("stage");
    tx.commit().expect("commit");

    assert_eq!(db.records("users").expect("records").len(), 1);
}

#[test]
fn creates_database_scoped_users_without_plaintext_credentials() {
    let temp = TempDir::new().expect("tempdir");
    let user = create_user(temp.path(), "clinic", "clinic-api", Some("secret-token"))
        .expect("created user");

    assert!(!user.generated);
    assert_eq!(user.database, "clinic");
    assert!(temp.path().join(".dowe/store/clinic/metadata.bin").exists());
    assert!(temp.path().join(".dowe/store/_auth/users.bin").exists());
    let auth = std::fs::read(temp.path().join(".dowe/store/_auth/users.bin")).expect("auth");
    assert!(!String::from_utf8_lossy(&auth).contains("secret-token"));
    verify_user(temp.path(), "clinic", "clinic-api", "secret-token").expect("verify");

    let invalid = verify_user(temp.path(), "clinic", "clinic-api", "wrong").expect_err("error");
    assert_eq!(invalid.category(), "Authentication");

    create_user(temp.path(), "billing", "billing-api", Some("billing-token"))
        .expect("billing user");
    let forbidden =
        verify_user(temp.path(), "clinic", "billing-api", "billing-token").expect_err("error");
    assert_eq!(forbidden.category(), "Authorization");
}

#[tokio::test]
async fn remote_server_requires_auth_and_executes_store_operations() {
    let temp = TempDir::new().expect("tempdir");
    create_user(temp.path(), "clinic", "clinic-api", Some("secret-token")).expect("user");
    let server = start_store_server(StoreServerConfig {
        root: temp.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("server");
    let host = format!("http://{}", server.addr);
    let client = RemoteStoreClient::new(RemoteStoreConfig {
        host: host.clone(),
        database: "clinic".to_string(),
        user: "clinic-api".to_string(),
        credential: "secret-token".to_string(),
    })
    .expect("client");

    let inserted = client
        .insert("appointments", serde_json::json!({"patientName":"Ana"}))
        .await
        .expect("insert");
    let id = inserted["id"].as_str().expect("id").to_string();

    let list = client.list("appointments").await.expect("list");
    assert_eq!(list.as_array().expect("array").len(), 1);

    let read = client
        .read(
            "appointments",
            "id",
            serde_json::Value::String(id.clone()),
            true,
        )
        .await
        .expect("read");
    assert_eq!(read["patientName"], "Ana");

    let changed = client
        .update(
            "appointments",
            "id",
            serde_json::Value::String(id.clone()),
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

    let deleted = client
        .delete("appointments", "id", serde_json::Value::String(id), true)
        .await
        .expect("delete");
    assert_eq!(deleted["changed"], 1);

    let bad_client = RemoteStoreClient::new(RemoteStoreConfig {
        host,
        database: "clinic".to_string(),
        user: "clinic-api".to_string(),
        credential: "wrong".to_string(),
    })
    .expect("bad client");
    let error = bad_client
        .list("appointments")
        .await
        .expect_err("auth error");
    assert!(matches!(error, StoreError::Authentication(_)));

    server.shutdown().await.expect("shutdown");
}
