use super::{StoreValue, init_database, open_database};
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
