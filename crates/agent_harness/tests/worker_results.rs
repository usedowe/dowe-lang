use dowe_agent_harness::{
    AllowedEditSurface, capture_worker_result, create_isolated_worktree,
    create_worktree_from_results,
};
use std::{path::Path, process::Command};

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    git(root.path(), &["config", "user.name", "Test"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    std::fs::write(root.path().join("base.txt"), "original").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    root
}

fn scopes(path: &str) -> Vec<AllowedEditSurface> {
    vec![AllowedEditSurface::new(path).unwrap()]
}

#[test]
fn independent_results_form_an_isolated_candidate_without_touching_host() {
    let root = repository();
    let first = create_isolated_worktree(root.path(), "first").unwrap();
    let second = create_isolated_worktree(root.path(), "second").unwrap();
    std::fs::write(Path::new(&first.path).join("a.txt"), "alpha").unwrap();
    std::fs::write(Path::new(&second.path).join("b.txt"), "beta").unwrap();
    let a = capture_worker_result(root.path(), &first, "a", &[], &scopes("a.txt")).unwrap();
    let b = capture_worker_result(root.path(), &second, "b", &[], &scopes("b.txt")).unwrap();
    std::fs::write(
        Path::new(&first.path).join("a.txt"),
        "later unrecorded edit",
    )
    .unwrap();
    let candidate = create_worktree_from_results(root.path(), "candidate", &[a, b]).unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&candidate.path).join("a.txt")).unwrap(),
        "alpha"
    );
    assert_eq!(
        std::fs::read_to_string(Path::new(&candidate.path).join("b.txt")).unwrap(),
        "beta"
    );
    assert!(!root.path().join("a.txt").exists());
    assert!(!root.path().join("b.txt").exists());
}

#[test]
fn diamond_dependencies_inherit_once_and_allow_descendant_updates() {
    let root = repository();
    let first = create_isolated_worktree(root.path(), "first").unwrap();
    std::fs::write(Path::new(&first.path).join("base.txt"), "ancestor").unwrap();
    let a = capture_worker_result(root.path(), &first, "a", &[], &scopes("base.txt")).unwrap();
    let left = create_worktree_from_results(root.path(), "left", &[a.clone()]).unwrap();
    let right = create_worktree_from_results(root.path(), "right", &[a.clone()]).unwrap();
    std::fs::write(Path::new(&left.path).join("left.txt"), "left").unwrap();
    std::fs::write(Path::new(&right.path).join("right.txt"), "right").unwrap();
    let b =
        capture_worker_result(root.path(), &left, "b", &[a.clone()], &scopes("left.txt")).unwrap();
    let c = capture_worker_result(root.path(), &right, "c", &[a], &scopes("right.txt")).unwrap();
    let joined =
        create_worktree_from_results(root.path(), "joined", &[b.clone(), c.clone()]).unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&joined.path).join("base.txt")).unwrap(),
        "ancestor"
    );
    std::fs::write(Path::new(&joined.path).join("base.txt"), "descendant").unwrap();
    let d = capture_worker_result(root.path(), &joined, "d", &[b, c], &scopes("base.txt")).unwrap();
    let final_tree = create_worktree_from_results(root.path(), "final", &[d]).unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&final_tree.path).join("base.txt")).unwrap(),
        "descendant"
    );
    assert!(Path::new(&final_tree.path).join("left.txt").is_file());
    assert!(Path::new(&final_tree.path).join("right.txt").is_file());
    assert_eq!(
        std::fs::read_to_string(root.path().join("base.txt")).unwrap(),
        "original"
    );
}

#[test]
fn independent_conflicts_and_out_of_scope_changes_are_rejected() {
    let root = repository();
    let left = create_isolated_worktree(root.path(), "left").unwrap();
    let right = create_isolated_worktree(root.path(), "right").unwrap();
    std::fs::write(Path::new(&left.path).join("base.txt"), "left").unwrap();
    std::fs::write(Path::new(&right.path).join("base.txt"), "right").unwrap();
    assert!(capture_worker_result(root.path(), &left, "a", &[], &scopes("other.txt")).is_err());
    let a = capture_worker_result(root.path(), &left, "a", &[], &scopes("base.txt")).unwrap();
    let b = capture_worker_result(root.path(), &right, "b", &[], &scopes("base.txt")).unwrap();
    assert!(create_worktree_from_results(root.path(), "conflict", &[a, b]).is_err());
    assert!(!root.path().join(".dowe/agent-worktrees/conflict").exists());
}

#[test]
fn restoring_ancestor_changes_and_deleting_files_are_not_lost() {
    let root = repository();
    let first = create_isolated_worktree(root.path(), "first").unwrap();
    std::fs::write(Path::new(&first.path).join("base.txt"), "ancestor").unwrap();
    std::fs::write(Path::new(&first.path).join("new.txt"), "new").unwrap();
    let a = capture_worker_result(root.path(), &first, "a", &[], &scopes(".")).unwrap();
    let restored = create_worktree_from_results(root.path(), "restored", &[a.clone()]).unwrap();
    std::fs::write(Path::new(&restored.path).join("base.txt"), "original").unwrap();
    std::fs::remove_file(Path::new(&restored.path).join("new.txt")).unwrap();
    assert!(
        capture_worker_result(root.path(), &restored, "b", &[a.clone()], &scopes("other")).is_err()
    );
    let b = capture_worker_result(root.path(), &restored, "b", &[a.clone()], &scopes(".")).unwrap();
    assert_eq!(b.changed_files(), ["base.txt", "new.txt"]);
    let final_tree = create_worktree_from_results(root.path(), "final", &[a.clone(), b]).unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&final_tree.path).join("base.txt")).unwrap(),
        "original"
    );
    assert!(!Path::new(&final_tree.path).join("new.txt").exists());
    let deleted = create_worktree_from_results(root.path(), "deleted", &[a.clone()]).unwrap();
    std::fs::remove_file(Path::new(&deleted.path).join("base.txt")).unwrap();
    let c = capture_worker_result(root.path(), &deleted, "c", &[a], &scopes("base.txt")).unwrap();
    let final_deleted = create_worktree_from_results(root.path(), "final-deleted", &[c]).unwrap();
    assert!(!Path::new(&final_deleted.path).join("base.txt").exists());
    assert!(Path::new(&final_deleted.path).join("new.txt").exists());
}

#[test]
fn identical_writes_coalesce_but_task_identity_and_git_base_cannot_be_reused() {
    let root = repository();
    let first = create_isolated_worktree(root.path(), "first").unwrap();
    let second = create_isolated_worktree(root.path(), "second").unwrap();
    for worker in [&first, &second] {
        std::fs::write(Path::new(&worker.path).join("base.txt"), "same").unwrap();
    }
    let a = capture_worker_result(root.path(), &first, "a", &[], &scopes("base.txt")).unwrap();
    let b = capture_worker_result(root.path(), &second, "b", &[], &scopes("base.txt")).unwrap();
    assert!(create_worktree_from_results(root.path(), "coalesced", &[a.clone(), b]).is_ok());
    std::fs::write(Path::new(&second.path).join("base.txt"), "different").unwrap();
    let reused =
        capture_worker_result(root.path(), &second, "a", &[], &scopes("base.txt")).unwrap();
    assert!(create_worktree_from_results(root.path(), "reused", &[a.clone(), reused]).is_err());
    std::fs::write(root.path().join("base.txt"), "next revision").unwrap();
    git(root.path(), &["add", "base.txt"]);
    git(root.path(), &["commit", "-qm", "next"]);
    assert!(create_worktree_from_results(root.path(), "stale", &[a]).is_err());
    assert!(!root.path().join(".dowe/agent-worktrees/stale").exists());
}

#[cfg(unix)]
#[test]
fn symlinks_are_rejected_and_executable_bits_survive_composition() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = repository();
    let first = create_isolated_worktree(root.path(), "first").unwrap();
    let source = Path::new(&first.path).join("run.sh");
    std::fs::write(&source, "exit 0\n").unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o755)).unwrap();
    let a = capture_worker_result(root.path(), &first, "a", &[], &scopes("run.sh")).unwrap();
    let candidate = create_worktree_from_results(root.path(), "candidate", &[a]).unwrap();
    assert_ne!(
        std::fs::metadata(Path::new(&candidate.path).join("run.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o111,
        0
    );
    symlink(
        root.path().join("base.txt"),
        Path::new(&first.path).join("link.txt"),
    )
    .unwrap();
    assert!(capture_worker_result(root.path(), &first, "b", &[], &scopes(".")).is_err());
    let prepared =
        dowe_agent_harness::prepare_isolated_worktrees(root.path(), &[candidate.clone()]).unwrap();
    assert_eq!(
        prepared.approval_manifest()["newFiles"][0]["executable"],
        true
    );
    let candidate_script = Path::new(&candidate.path).join("run.sh");
    std::fs::set_permissions(&candidate_script, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(prepared.apply().is_err());
    assert!(!root.path().join("run.sh").exists());
    std::fs::set_permissions(&candidate_script, std::fs::Permissions::from_mode(0o755)).unwrap();
    dowe_agent_harness::prepare_isolated_worktrees(root.path(), &[candidate])
        .unwrap()
        .apply()
        .unwrap();
    assert_ne!(
        std::fs::metadata(root.path().join("run.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o111,
        0
    );
}
