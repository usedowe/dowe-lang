use crate::engine::{StoreRecord, init_database, open_database};
use crate::error::StoreResult;
use crate::value::StoreValue;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

const TARGET_TRANSACTIONS_PER_MINUTE: usize = 20_000;
const SINGLE_TRANSACTION_COUNT: usize = 2_000;
const CONCURRENT_WORKERS: usize = 16;
const BATCH_COUNT: usize = 200;
const RECORDS_PER_BATCH: usize = 100;

pub fn run_bench(project_root: &Path) -> StoreResult<Value> {
    let database = "bench";
    let _ = init_database(project_root, database)?;
    let db = open_database(project_root, database)?;
    db.create_index("messages", "status")?;

    let mut latencies = Vec::with_capacity(SINGLE_TRANSACTION_COUNT);
    let barrier = Arc::new(Barrier::new(CONCURRENT_WORKERS + 1));
    let transactions_per_worker = SINGLE_TRANSACTION_COUNT / CONCURRENT_WORKERS;
    let handles = (0..CONCURRENT_WORKERS)
        .map(|worker| {
            let db = db.clone();
            let barrier = barrier.clone();
            thread::spawn(move || -> StoreResult<Vec<Duration>> {
                barrier.wait();
                let mut latencies = Vec::with_capacity(transactions_per_worker);
                for index in 0..transactions_per_worker {
                    let started = Instant::now();
                    let mut transaction = db.transaction();
                    transaction.insert(
                        "messages",
                        message_record("concurrent", worker * transactions_per_worker + index),
                    )?;
                    transaction.commit()?;
                    latencies.push(started.elapsed());
                }
                Ok(latencies)
            })
        })
        .collect::<Vec<_>>();
    let single_started = Instant::now();
    barrier.wait();
    for handle in handles {
        latencies.extend(handle.join().map_err(|_| {
            crate::error::StoreError::DurabilityError(
                "Database benchmark worker failed".to_string(),
            )
        })??);
    }
    let single_elapsed = single_started.elapsed();
    latencies.sort_unstable();
    let single_throughput = throughput(SINGLE_TRANSACTION_COUNT, single_elapsed);

    let batch_started = Instant::now();
    for batch in 0..BATCH_COUNT {
        let mut transaction = db.transaction();
        for index in 0..RECORDS_PER_BATCH {
            transaction.insert(
                "messages",
                message_record("batch", batch * RECORDS_PER_BATCH + index),
            )?;
        }
        transaction.commit()?;
    }
    let batch_elapsed = batch_started.elapsed();
    let batch_records = BATCH_COUNT * RECORDS_PER_BATCH;
    let batch_throughput = throughput(batch_records, batch_elapsed);

    let query_started = Instant::now();
    let rows = db.query("select * from messages where status = \"queued\" limit 100")?;
    let query_elapsed = query_started.elapsed();
    let required_per_second = TARGET_TRANSACTIONS_PER_MINUTE as f64 / 60.0;

    Ok(serde_json::json!({
        "database": database,
        "target": {
            "transactionsPerMinute": TARGET_TRANSACTIONS_PER_MINUTE,
            "transactionsPerSecond": round(required_per_second),
            "meetsTarget": single_throughput >= required_per_second
        },
        "profiles": {
            "durableConcurrentTransactions": {
                "transactions": SINGLE_TRANSACTION_COUNT,
                "workers": CONCURRENT_WORKERS,
                "elapsedMs": single_elapsed.as_millis(),
                "throughputPerSecond": round(single_throughput),
                "throughputPerMinute": round(single_throughput * 60.0),
                "latencyMs": {
                    "p50": round(duration_ms(percentile(&latencies, 50))),
                    "p95": round(duration_ms(percentile(&latencies, 95)))
                }
            },
            "durableBatchTransactions": {
                "transactions": BATCH_COUNT,
                "records": batch_records,
                "recordsPerTransaction": RECORDS_PER_BATCH,
                "elapsedMs": batch_elapsed.as_millis(),
                "recordThroughputPerSecond": round(batch_throughput)
            },
            "indexedFilter": {
                "rows": rows.len(),
                "elapsedMs": query_elapsed.as_millis()
            }
        },
        "durability": "wal-fsync-per-commit-group",
        "postgresBaseline": "not configured"
    }))
}

fn message_record(profile: &str, index: usize) -> StoreRecord {
    let mut record = StoreRecord::new();
    record.insert(
        "recipient".to_string(),
        StoreValue::String(format!("+1555{index:07}")),
    );
    record.insert(
        "profile".to_string(),
        StoreValue::String(profile.to_string()),
    );
    record.insert(
        "status".to_string(),
        StoreValue::String("queued".to_string()),
    );
    record
}

fn throughput(count: usize, elapsed: Duration) -> f64 {
    count as f64 / elapsed.as_secs_f64().max(f64::EPSILON)
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn percentile(values: &[Duration], percentile: usize) -> Duration {
    let index = values
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1)
        .min(values.len().saturating_sub(1));
    values.get(index).copied().unwrap_or_default()
}

fn round(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}
