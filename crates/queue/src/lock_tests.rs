use crate::{QueueError, open_namespace};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const CHILD_ROOT: &str = "DOWE_QUEUE_NAMESPACE_LOCK_CHILD_ROOT";
const NAMESPACE: &str = "orders";
const READY_FILE: &str = "queue-lock-ready";
const RELEASE_FILE: &str = "queue-lock-release";

#[test]
fn namespace_lock_child_probe() {
    let Ok(root) = env::var(CHILD_ROOT) else {
        return;
    };
    let root = PathBuf::from(root);
    let queue = open_namespace(&root, NAMESPACE).expect("child namespace");
    queue.declare("workers").expect("declare");
    queue.bind("workers", "orders.#").expect("bind");
    queue
        .publish("orders.created", serde_json::json!({"id": "locked"}))
        .expect("publish");
    let mut subscription = queue.subscribe("workers", "child").expect("subscribe");
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let _delivery = runtime
        .block_on(subscription.next())
        .expect("delivery")
        .expect("message");
    fs::write(root.join(READY_FILE), "ready").expect("ready");
    if wait_for_path(&root.join(RELEASE_FILE), Duration::from_secs(5)) {
        std::process::exit(0);
    }
    std::process::exit(1);
}

#[test]
fn namespace_lock_rejects_second_process_and_recovers_after_owner_exit() {
    let root = TempDir::new().expect("tempdir");
    let mut child = spawn_lock_owner(root.path());
    let ready = root.path().join(READY_FILE);
    if !wait_for_path(&ready, Duration::from_secs(5)) {
        let _ = child.kill();
        panic!("child did not acquire namespace lock");
    }
    let state_path = root.path().join(".dowe/queue/orders/state.json");
    let before = fs::metadata(&state_path).expect("state before blocked open");
    let blocked = match open_namespace(root.path(), NAMESPACE) {
        Ok(queue) => {
            drop(queue);
            None
        }
        Err(error) => Some(error),
    };
    let after = fs::metadata(&state_path).expect("state after blocked open");
    fs::write(root.path().join(RELEASE_FILE), "release").expect("release child");
    let exited = wait_for_exit(&mut child, Duration::from_secs(5));
    assert!(exited, "child did not exit cleanly");
    assert!(matches!(
        blocked,
        Some(QueueError::DurabilityError(message)) if message == "Queue namespace is already in use"
    ));
    assert_eq!(before.len(), after.len());
    assert_eq!(
        before.modified().expect("before modified"),
        after.modified().expect("after modified")
    );

    let queue = open_namespace(root.path(), NAMESPACE).expect("reopen after child exit");
    let inspection = queue.inspect().expect("inspect");
    let queues = inspection.queues.expect("authoritative queues");
    assert_eq!(queues[0].ready, 1);
    assert_eq!(queues[0].in_flight, 0);
    let mut subscription = queue.subscribe("workers", "parent").expect("subscribe");
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let mut delivery = runtime
        .block_on(subscription.next())
        .expect("delivery")
        .expect("message");
    assert!(delivery.message.redelivered);
    runtime.block_on(delivery.ack()).expect("ack");
}

fn spawn_lock_owner(root: &Path) -> Child {
    Command::new(env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg("lock_tests::namespace_lock_child_probe")
        .arg("--nocapture")
        .env(CHILD_ROOT, root)
        .spawn()
        .expect("spawn lock owner")
}

fn wait_for_path(path: &Path, duration: Duration) -> bool {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        if path.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    path.exists()
}

fn wait_for_exit(child: &mut Child, duration: Duration) -> bool {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => return false,
        }
    }
    let _ = child.kill();
    false
}
