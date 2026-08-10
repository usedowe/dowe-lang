use super::{
    DatabaseServiceConfig, DatabaseTransactionInsert, DoweDatabaseClient, DoweDatabaseConfig,
    StoreError, StoreValue, create_account, init_database, open_database, start_database_service,
    verify_account,
};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn initializes_database_with_stable_ulid_metadata() {
    let temp = TempDir::new().expect("tempdir");
    let metadata = init_database(temp.path(), "db1").expect("metadata");
    let reopened = open_database(temp.path(), "db1").expect("database");

    assert_eq!(metadata.database_id, reopened.metadata().database_id);
    assert!(temp.path().join(".dowe/db/db1/metadata.bin").exists());
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
    assert!(temp.path().join(".dowe/db/db1/users/segments").exists());
    assert!(temp.path().join(".dowe/db/db1/users/wal").exists());
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
fn secondary_indexes_follow_wal_commits_and_recovery() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    {
        let db = open_database(temp.path(), "db1").expect("database");
        db.create_index("messages", "status").expect("index");
        let mut queued = BTreeMap::new();
        queued.insert(
            "status".to_string(),
            StoreValue::String("queued".to_string()),
        );
        let queued = db.insert("messages", queued).expect("queued");
        let mut sent = BTreeMap::new();
        sent.insert("status".to_string(), StoreValue::String("sent".to_string()));
        db.insert("messages", sent).expect("sent");

        let (rows, plan) = db
            .query_with_plan("select * from messages where status = \"queued\"")
            .expect("indexed query");
        assert!(plan.indexed);
        assert_eq!(rows.len(), 1);

        let mut patch = BTreeMap::new();
        patch.insert("status".to_string(), StoreValue::String("sent".to_string()));
        db.update("messages", "id", queued.get("id").expect("id"), patch)
            .expect("update");
        assert!(
            db.query("select * from messages where status = \"queued\"")
                .expect("queued after update")
                .is_empty()
        );
    }

    let reopened = open_database(temp.path(), "db1").expect("reopen");
    let (rows, plan) = reopened
        .query_with_plan("select * from messages where status = \"sent\"")
        .expect("recovered index");

    assert!(plan.indexed);
    assert_eq!(rows.len(), 2);
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
fn transaction_conflict_never_leaves_partial_writes() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    let duplicate_id = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let mut existing = BTreeMap::new();
    existing.insert(
        "id".to_string(),
        StoreValue::String(duplicate_id.to_string()),
    );
    db.insert("users", existing).expect("existing");

    let mut tx = db.transaction();
    let mut first = BTreeMap::new();
    first.insert("name".to_string(), StoreValue::String("First".to_string()));
    tx.insert("users", first).expect("first");
    let mut duplicate = BTreeMap::new();
    duplicate.insert(
        "id".to_string(),
        StoreValue::String(duplicate_id.to_string()),
    );
    tx.insert("users", duplicate).expect("duplicate");

    let error = tx.commit().expect_err("conflict");

    assert!(matches!(
        error,
        StoreError::AlreadyExists(_) | StoreError::TransactionConflict(_)
    ));
    assert_eq!(db.records("users").expect("records").len(), 1);
}

#[test]
fn concurrent_transactions_detect_write_conflicts() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    let barrier = Arc::new(Barrier::new(3));
    let id = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let handles = (0..2)
        .map(|index| {
            let db = db.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let mut tx = db.transaction();
                let mut record = BTreeMap::new();
                record.insert("id".to_string(), StoreValue::String(id.to_string()));
                record.insert("worker".to_string(), StoreValue::String(index.to_string()));
                tx.insert("users", record).expect("stage");
                barrier.wait();
                tx.commit()
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().expect("join"))
        .collect::<Vec<_>>();

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
    assert_eq!(db.records("users").expect("records").len(), 1);
}

#[test]
fn recovers_every_transaction_from_a_concurrent_commit_group() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    {
        let db = open_database(temp.path(), "db1").expect("database");
        let barrier = Arc::new(Barrier::new(33));
        let handles = (0..32)
            .map(|index| {
                let db = db.clone();
                let barrier = barrier.clone();
                thread::spawn(move || {
                    let mut tx = db.transaction();
                    let mut record = BTreeMap::new();
                    record.insert("worker".to_string(), StoreValue::String(index.to_string()));
                    tx.insert("messages", record).expect("stage");
                    barrier.wait();
                    tx.commit()
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        for handle in handles {
            handle.join().expect("join").expect("commit");
        }
        assert_eq!(db.records("messages").expect("records").len(), 32);
    }

    let reopened = open_database(temp.path(), "db1").expect("reopen");

    assert_eq!(reopened.records("messages").expect("records").len(), 32);
}

#[test]
fn transaction_reads_use_a_consistent_snapshot() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let db = open_database(temp.path(), "db1").expect("database");
    let mut record = BTreeMap::new();
    record.insert("name".to_string(), StoreValue::String("Before".to_string()));
    let inserted = db.insert("users", record).expect("insert");
    let id = inserted.get("id").expect("id").clone();
    let tx = db.transaction();
    let before = tx.records("users").expect("snapshot");
    let mut patch = BTreeMap::new();
    patch.insert("name".to_string(), StoreValue::String("After".to_string()));
    db.update("users", "id", &id, patch).expect("update");
    let after = tx.records("users").expect("same snapshot");

    assert_eq!(before, after);
    assert_eq!(
        after[0].get("name"),
        Some(&StoreValue::String("Before".to_string()))
    );
}

#[test]
fn recovers_durable_commits_from_database_wal() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    {
        let db = open_database(temp.path(), "db1").expect("database");
        let mut tx = db.transaction();
        let mut first = BTreeMap::new();
        first.insert("name".to_string(), StoreValue::String("First".to_string()));
        tx.insert("users", first).expect("first");
        let mut second = BTreeMap::new();
        second.insert("name".to_string(), StoreValue::String("Second".to_string()));
        tx.insert("users", second).expect("second");
        tx.commit().expect("commit");
    }

    let reopened = open_database(temp.path(), "db1").expect("reopen");

    assert_eq!(reopened.records("users").expect("records").len(), 2);
}

#[test]
fn recovery_discards_an_incomplete_wal_tail() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    {
        let db = open_database(temp.path(), "db1").expect("database");
        let mut record = BTreeMap::new();
        record.insert("name".to_string(), StoreValue::String("Safe".to_string()));
        db.insert("users", record).expect("insert");
    }
    let wal = temp.path().join(".dowe/db/db1/wal/transactions-v2.bin");
    let valid_length = std::fs::metadata(&wal).expect("metadata").len();
    let mut file = OpenOptions::new().append(true).open(&wal).expect("wal");
    file.write_all(b"DOWE_DB_TX_V2\n\xff\xff")
        .expect("partial frame");
    file.sync_all().expect("sync");

    let reopened = open_database(temp.path(), "db1").expect("reopen");

    assert_eq!(reopened.records("users").expect("records").len(), 1);
    assert_eq!(
        std::fs::metadata(wal).expect("metadata").len(),
        valid_length
    );
}

#[test]
fn recovery_rejects_a_corrupted_committed_wal_frame() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    {
        let db = open_database(temp.path(), "db1").expect("database");
        let mut record = BTreeMap::new();
        record.insert("name".to_string(), StoreValue::String("Safe".to_string()));
        db.insert("users", record).expect("insert");
    }
    let wal = temp.path().join(".dowe/db/db1/wal/transactions-v2.bin");
    let mut bytes = std::fs::read(&wal).expect("wal");
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0x5a;
    std::fs::write(&wal, bytes).expect("corrupt");

    let error = open_database(temp.path(), "db1").expect_err("corruption");

    assert_eq!(error.category(), "Corruption");
}

#[test]
fn compaction_checkpoints_state_and_resets_the_wal() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    {
        let db = open_database(temp.path(), "db1").expect("database");
        let mut record = BTreeMap::new();
        record.insert("name".to_string(), StoreValue::String("Before".to_string()));
        let inserted = db.insert("users", record).expect("insert");
        let id = inserted.get("id").expect("id").clone();
        let mut patch = BTreeMap::new();
        patch.insert("name".to_string(), StoreValue::String("After".to_string()));
        db.update("users", "id", &id, patch).expect("update");
        db.compact().expect("compact");
        assert_eq!(
            std::fs::metadata(temp.path().join(".dowe/db/db1/wal/transactions-v2.bin"))
                .expect("wal")
                .len(),
            0
        );
    }

    let reopened = open_database(temp.path(), "db1").expect("reopen");
    let records = reopened.records("users").expect("records");

    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].get("name"),
        Some(&StoreValue::String("After".to_string()))
    );
}

#[test]
fn rejects_a_second_process_owner_for_the_same_database() {
    let temp = TempDir::new().expect("tempdir");
    init_database(temp.path(), "db1").expect("metadata");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(temp.path().join(".dowe/db/db1/.lock"))
        .expect("lock file");
    lock.try_lock().expect("external lock");

    let error = open_database(temp.path(), "db1").expect_err("lock conflict");

    assert_eq!(error.category(), "TransactionConflict");
}

#[test]
fn creates_database_scoped_accounts_without_plaintext_secrets() {
    let temp = TempDir::new().expect("tempdir");
    let account = create_account(temp.path(), "clinic", "clinic-api", Some("secret-token"))
        .expect("created account");

    assert!(!account.generated);
    assert_eq!(account.database, "clinic");
    assert_eq!(account.account, "clinic-api");
    assert!(temp.path().join(".dowe/db/clinic/metadata.bin").exists());
    assert!(temp.path().join(".dowe/db/_auth/users.bin").exists());
    let auth = std::fs::read(temp.path().join(".dowe/db/_auth/users.bin")).expect("auth");
    assert!(!String::from_utf8_lossy(&auth).contains("secret-token"));
    assert!(String::from_utf8_lossy(&auth).contains("$argon2id$"));
    verify_account(temp.path(), "clinic", "clinic-api", "secret-token").expect("verify");

    let invalid = verify_account(temp.path(), "clinic", "clinic-api", "wrong").expect_err("error");
    assert_eq!(invalid.category(), "Authentication");

    create_account(temp.path(), "billing", "billing-api", Some("billing-token"))
        .expect("billing account");
    let forbidden =
        verify_account(temp.path(), "clinic", "billing-api", "billing-token").expect_err("error");
    assert_eq!(forbidden.category(), "Authorization");

    create_account(
        temp.path(),
        "billing",
        "clinic-api",
        Some("billing-clinic-token"),
    )
    .expect("same account name in another database");
    verify_account(temp.path(), "clinic", "clinic-api", "secret-token")
        .expect("original scoped account");
    verify_account(temp.path(), "billing", "clinic-api", "billing-clinic-token")
        .expect("second scoped account");
}

#[cfg(unix)]
#[test]
fn restricts_database_credentials_and_wal_to_the_current_user() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().expect("tempdir");
    create_account(temp.path(), "clinic", "clinic-api", Some("secret-token")).expect("account");
    let db = open_database(temp.path(), "clinic").expect("database");
    db.insert("messages", BTreeMap::new()).expect("insert");

    let database_root = temp.path().join(".dowe/db/clinic");
    let auth = temp.path().join(".dowe/db/_auth/users.bin");
    let wal = database_root.join("wal/transactions-v2.bin");

    assert_eq!(
        std::fs::metadata(database_root)
            .expect("database root")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(auth).expect("auth").permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        std::fs::metadata(wal).expect("wal").permissions().mode() & 0o777,
        0o600
    );
}

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
