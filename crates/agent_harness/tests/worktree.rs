use dowe_agent_harness::{
    apply_isolated_worktrees, cleanup_isolated_worktrees, create_isolated_worktree,
    remove_isolated_worktree,
};
use std::process::Command;

fn git(root: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "git failed: {:?}", output);
}

#[test]
fn creates_and_removes_a_detached_isolation() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Dowe Test"]);
    std::fs::write(root.path().join("main.dowe"), "app demo {}\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);

    let worktree = create_isolated_worktree(root.path(), "worker-1").unwrap();
    assert!(
        std::path::Path::new(&worktree.path)
            .join("main.dowe")
            .is_file()
    );
    assert!(!worktree.base_revision.is_empty());
    remove_isolated_worktree(root.path(), "worker-1").unwrap();
    assert!(!std::path::Path::new(&worktree.path).exists());
}

#[test]
fn refuses_a_dirty_checkout_and_invalid_ids() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Dowe Test"]);
    std::fs::write(root.path().join("main.dowe"), "app demo {}\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    std::fs::write(root.path().join("dirty.txt"), "uncommitted\n").unwrap();
    assert!(create_isolated_worktree(root.path(), "worker-1").is_err());
    assert!(create_isolated_worktree(root.path(), "../escape").is_err());
}

#[test]
fn applies_tracked_worktree_changes_after_preflight() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Dowe Test"]);
    std::fs::write(root.path().join("main.dowe"), "app old {}\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    let worktree = create_isolated_worktree(root.path(), "worker-1").unwrap();
    std::fs::write(
        std::path::Path::new(&worktree.path).join("main.dowe"),
        "app new {}\n",
    )
    .unwrap();
    let files = apply_isolated_worktrees(root.path(), &[worktree.clone()]).unwrap();
    assert_eq!(files.files, vec!["main.dowe"]);
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app new {}\n"
    );
}

#[test]
fn applies_new_worktree_files_and_keeps_existing_content() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Dowe Test"]);
    std::fs::write(root.path().join("main.dowe"), "app old {}\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    let worktree = create_isolated_worktree(root.path(), "worker-1").unwrap();
    std::fs::write(
        std::path::Path::new(&worktree.path).join("new.dowe"),
        "app new {}\n",
    )
    .unwrap();
    let files = apply_isolated_worktrees(root.path(), &[worktree.clone()]).unwrap();
    assert_eq!(files.files, vec!["new.dowe"]);
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app old {}\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("new.dowe")).unwrap(),
        "app new {}\n"
    );
    let path = worktree.path.clone();
    assert!(cleanup_isolated_worktrees(root.path(), &[worktree]).is_err());
    assert_eq!(
        std::fs::read_to_string(std::path::Path::new(&path).join("new.dowe")).unwrap(),
        "app new {}\n"
    );
}

#[test]
fn failed_host_preflight_does_not_modify_the_checkout() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Dowe Test"]);
    std::fs::write(root.path().join("main.dowe"), "app old {}\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    let worktree = create_isolated_worktree(root.path(), "worker-1").unwrap();
    std::fs::write(
        std::path::Path::new(&worktree.path).join("main.dowe"),
        "app new {}\n",
    )
    .unwrap();
    std::fs::write(root.path().join("user-change.txt"), "keep me\n").unwrap();
    assert!(apply_isolated_worktrees(root.path(), &[worktree]).is_err());
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app old {}\n"
    );
}
