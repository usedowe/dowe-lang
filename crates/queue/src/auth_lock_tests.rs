use crate::{QueueError, create_account, verify_account};
use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const CHILD_ROOT: &str = "DOWE_QUEUE_AUTH_LOCK_CHILD_ROOT";
const READY_FILE: &str = "auth-lock-ready";
const RELEASE_FILE: &str = "auth-lock-release";

struct ChildGuard {
    child: Child,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

#[test]
fn auth_writer_lock_child_probe() {
    let Ok(root) = env::var(CHILD_ROOT) else {
        return;
    };
    let root = PathBuf::from(root);
    let auth_root = root.join(".dowe/queue/_auth");
    fs::create_dir_all(&auth_root).expect("auth root");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(auth_root.join(".lock"))
        .expect("auth lock file");
    lock.try_lock().expect("auth writer lock");
    fs::write(root.join(READY_FILE), "ready").expect("ready");
    let status = wait_for_path(&root.join(RELEASE_FILE), Duration::from_secs(5));
    drop(lock);
    std::process::exit(if status { 0 } else { 1 });
}

#[test]
fn auth_writer_lock_rejects_other_process_and_preserves_accounts() {
    let root = TempDir::new().expect("tempdir");
    create_account(root.path(), "orders", "existing", Some("existing-secret"))
        .expect("existing account");
    let catalog_path = root.path().join(".dowe/queue/_auth/accounts.json");
    let before = fs::read(&catalog_path).expect("catalog before lock");
    let mut owner = spawn_lock_owner(root.path());
    assert!(
        wait_for_path(&root.path().join(READY_FILE), Duration::from_secs(5)),
        "child did not acquire auth writer lock"
    );

    let blocked = create_account(root.path(), "orders", "second", Some("second-secret"));
    let after = fs::read(&catalog_path).expect("catalog after blocked writer");
    fs::write(root.path().join(RELEASE_FILE), "release").expect("release child");
    assert!(
        wait_for_exit(&mut owner.child, Duration::from_secs(5)),
        "child did not exit cleanly"
    );
    assert!(matches!(
        blocked,
        Err(QueueError::DurabilityError(message)) if message == "Queue auth catalog is already in use"
    ));
    assert_eq!(before, after);

    create_account(root.path(), "orders", "second", Some("second-secret"))
        .expect("create after release");
    verify_account(root.path(), "orders", "existing", "existing-secret")
        .expect("existing account survives");
    verify_account(root.path(), "orders", "second", "second-secret").expect("new account verifies");
}

fn spawn_lock_owner(root: &Path) -> ChildGuard {
    let child = Command::new(env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg("auth_lock_tests::auth_writer_lock_child_probe")
        .arg("--nocapture")
        .env(CHILD_ROOT, root)
        .spawn()
        .expect("spawn auth lock owner");
    ChildGuard { child }
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
    false
}
