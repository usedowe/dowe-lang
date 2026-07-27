use crate::engine::{StoreRecord, init_database, open_database};
use crate::error::StoreResult;
use crate::value::StoreValue;
use serde_json::Value;
use std::path::Path;
use std::time::Instant;

pub fn run_bench(project_root: &Path) -> StoreResult<Value> {
    let database = "bench";
    let _ = init_database(project_root, database)?;
    let db = open_database(project_root, database)?;
    db.create_index("users", "roleId")?;
    let started = Instant::now();
    for index in 0..100 {
        let mut record = StoreRecord::new();
        record.insert(
            "name".to_string(),
            StoreValue::String(format!("User {index}")),
        );
        record.insert(
            "roleId".to_string(),
            StoreValue::String("admin".to_string()),
        );
        let _ = db.insert("users", record);
    }
    let insert_ms = started.elapsed().as_millis();
    let query_started = Instant::now();
    let rows = db.query("select * from users where roleId = \"admin\" limit 10")?;
    let query_ms = query_started.elapsed().as_millis();
    Ok(serde_json::json!({
        "database": database,
        "profiles": {
            "insertBatch": { "rows": 100, "elapsedMs": insert_ms },
            "indexedFilter": { "rows": rows.len(), "elapsedMs": query_ms }
        },
        "postgresBaseline": "not configured"
    }))
}
