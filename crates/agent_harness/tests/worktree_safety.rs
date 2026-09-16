use dowe_agent_harness::prepare_isolated_worktrees;
use dowe_agent_harness::{
    IntegrationFaultPoint, IntegrationRecovery, apply_isolated_worktrees,
    cleanup_isolated_worktrees, create_isolated_worktree, recover_pending_integration,
    remove_isolated_worktree,
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
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Dowe Test"]);
    std::fs::write(root.path().join("main.dowe"), "app old {}\n").unwrap();
    std::fs::write(root.path().join(".gitignore"), "ignored.txt\n.dowe/\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    root
}

#[test]
fn cleanup_preserves_ignored_worker_data() {
    let root = repository();
    let worker = create_isolated_worktree(root.path(), "worker").unwrap();
    let data = Path::new(&worker.path).join("ignored.txt");
    std::fs::write(&data, "irreplaceable").unwrap();
    assert!(remove_isolated_worktree(root.path(), "worker").is_err());
    assert_eq!(std::fs::read_to_string(data).unwrap(), "irreplaceable");
}

#[test]
fn cleanup_rejects_mismatched_identity_before_removing_anything() {
    let root = repository();
    let first = create_isolated_worktree(root.path(), "first").unwrap();
    let mut second = create_isolated_worktree(root.path(), "second").unwrap();
    second.path = first.path.clone();
    assert!(cleanup_isolated_worktrees(root.path(), &[first.clone(), second]).is_err());
    assert!(Path::new(&first.path).exists());
}

#[test]
fn integration_rejects_a_worker_that_changed_its_commit() {
    let root = repository();
    let worker = create_isolated_worktree(root.path(), "worker").unwrap();
    std::fs::write(
        Path::new(&worker.path).join("main.dowe"),
        "app changed {}\n",
    )
    .unwrap();
    git(Path::new(&worker.path), &["add", "."]);
    git(Path::new(&worker.path), &["commit", "-qm", "unexpected"]);
    assert!(apply_isolated_worktrees(root.path(), &[worker]).is_err());
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app old {}\n"
    );
}

#[test]
fn durable_knowledge_edits_are_not_ignored_as_cache() {
    let root = repository();
    std::fs::create_dir(root.path().join(".agent")).unwrap();
    std::fs::write(root.path().join(".agent/knowledge.json"), "{}").unwrap();
    assert!(create_isolated_worktree(root.path(), "worker").is_err());
}

#[test]
fn prepared_approval_rejects_changed_tracked_and_new_bytes() {
    for file in ["main.dowe", "new.dowe"] {
        let root = repository();
        let worker = create_isolated_worktree(root.path(), "worker").unwrap();
        let path = Path::new(&worker.path).join(file);
        std::fs::write(&path, "approved bytes").unwrap();
        let prepared = prepare_isolated_worktrees(root.path(), &[worker]).unwrap();
        assert!(
            prepared.approval_manifest()["patchSha256"]
                .as_str()
                .unwrap()
                .len()
                == 64
        );
        std::fs::write(&path, "unapproved bytes").unwrap();
        assert!(
            prepared
                .apply()
                .unwrap_err()
                .to_string()
                .contains("new approval")
        );
        assert_eq!(
            std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
            "app old {}\n"
        );
        assert!(!root.path().join("new.dowe").exists());
    }
}

#[test]
fn prepared_integration_applies_exact_combined_content() {
    let root = repository();
    let worker = create_isolated_worktree(root.path(), "worker").unwrap();
    std::fs::write(Path::new(&worker.path).join("main.dowe"), "updated").unwrap();
    std::fs::write(Path::new(&worker.path).join("new.dowe"), "created").unwrap();
    let prepared = prepare_isolated_worktrees(root.path(), &[worker]).unwrap();
    let manifest = prepared.approval_manifest();
    assert_eq!(manifest["newFiles"][0]["bytes"], 7);
    let report = prepared.apply().unwrap();
    assert_eq!(report.files, ["main.dowe", "new.dowe"]);
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "updated"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("new.dowe")).unwrap(),
        "created"
    );
    let receipt = std::fs::read_dir(root.path().join(".dowe"))
        .unwrap()
        .map(Result::unwrap)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("integration-applied-")
        })
        .unwrap()
        .path();
    assert_eq!(
        std::fs::read_to_string(receipt.join("state")).unwrap(),
        "applied\n"
    );
    assert!(receipt.join("tracked.patch").is_file());
    assert_eq!(
        std::fs::read_to_string(receipt.join("new/new.dowe")).unwrap(),
        "created"
    );
}

#[test]
fn fault_injected_process_stop_recovers_patch_and_new_files() {
    let root = repository();
    let worker = create_isolated_worktree(root.path(), "worker").unwrap();
    std::fs::write(Path::new(&worker.path).join("main.dowe"), "updated").unwrap();
    std::fs::write(Path::new(&worker.path).join("new.dowe"), "created").unwrap();

    let prepared = prepare_isolated_worktrees(root.path(), &[worker]).unwrap();
    let error = prepared
        .apply_with_fault_injection(Some(IntegrationFaultPoint::AfterPatch))
        .unwrap_err();
    assert!(error.to_string().contains("journal retained"));
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "updated"
    );
    assert!(!root.path().join("new.dowe").exists());

    let recovered = recover_pending_integration(root.path()).unwrap();
    assert!(
        matches!(recovered, IntegrationRecovery::RolledBack { .. }),
        "unexpected recovery result: {recovered:?}"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app old {}\n"
    );
    assert!(!root.path().join("new.dowe").exists());
    assert!(!root.path().join(".dowe/integration-pending").exists());
}

#[test]
fn process_kill_after_patch_is_recovered_by_a_new_process() {
    if let Ok(root) = std::env::var("DOWE_INTEGRATION_CHILD_ROOT") {
        let root = Path::new(&root);
        let worker = create_isolated_worktree(root, "worker").unwrap();
        std::fs::write(Path::new(&worker.path).join("main.dowe"), "updated").unwrap();
        std::fs::write(Path::new(&worker.path).join("new.dowe"), "created").unwrap();
        let prepared = prepare_isolated_worktrees(root, &[worker]).unwrap();
        let _ = prepared
            .apply_with_fault_injection(Some(IntegrationFaultPoint::TerminateProcessAfterPatch));
        panic!("the fault-injection child must terminate before returning");
    }

    let root = repository();
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process_kill_after_patch_is_recovered_by_a_new_process",
            "--nocapture",
        ])
        .env("DOWE_INTEGRATION_CHILD_ROOT", root.path())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(75));

    let recovered = recover_pending_integration(root.path()).unwrap();
    assert!(matches!(recovered, IntegrationRecovery::RolledBack { .. }));
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app old {}\n"
    );
    assert!(!root.path().join("new.dowe").exists());
    assert!(!root.path().join(".dowe/integration-pending").exists());
    git(
        root.path(),
        &[
            "-C",
            ".dowe/agent-worktrees/worker",
            "reset",
            "--hard",
            "HEAD",
        ],
    );
    git(
        root.path(),
        &["-C", ".dowe/agent-worktrees/worker", "clean", "-fd"],
    );
    remove_isolated_worktree(root.path(), "worker").unwrap();
}

#[test]
fn pending_journal_prevents_replaying_uncertain_integration() {
    let root = repository();
    let worker = create_isolated_worktree(root.path(), "worker").unwrap();
    std::fs::write(Path::new(&worker.path).join("main.dowe"), "updated").unwrap();
    let journal = root.path().join(".dowe/integration-pending");
    std::fs::create_dir(&journal).unwrap();
    std::fs::write(journal.join("state"), "uncertain").unwrap();
    assert!(
        apply_isolated_worktrees(root.path(), &[worker])
            .unwrap_err()
            .to_string()
            .contains("journal")
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app old {}\n"
    );
    assert_eq!(
        std::fs::read_to_string(journal.join("state")).unwrap(),
        "uncertain"
    );
}

#[test]
fn recovery_reverses_a_proven_prepared_patch_and_quarantines_evidence() {
    let root = repository();
    let worker = create_isolated_worktree(root.path(), "worker").unwrap();
    std::fs::write(Path::new(&worker.path).join("main.dowe"), "app new {}\n").unwrap();
    let patch = Command::new("git")
        .args(["diff", "--binary", "HEAD"])
        .current_dir(&worker.path)
        .output()
        .unwrap();
    assert!(patch.status.success());
    let journal = root.path().join(".dowe/integration-pending");
    std::fs::create_dir_all(&journal).unwrap();
    std::fs::write(
        journal.join("state"),
        "prepared: application may have started\n",
    )
    .unwrap();
    std::fs::write(journal.join("tracked.patch"), &patch.stdout).unwrap();
    std::fs::write(journal.join("manifest.json"), b"{}\n").unwrap();
    let apply = Command::new("git")
        .args(["apply", "--binary", "-"])
        .current_dir(root.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut apply = apply;
    use std::io::Write;
    apply
        .stdin
        .take()
        .unwrap()
        .write_all(&patch.stdout)
        .unwrap();
    assert!(apply.wait().unwrap().success());
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app new {}\n"
    );
    let result = recover_pending_integration(root.path()).unwrap();
    assert!(matches!(result, IntegrationRecovery::RolledBack { .. }));
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "app old {}\n"
    );
    assert!(!journal.exists());
}

#[test]
fn independent_text_edits_merge_when_their_line_ranges_do_not_overlap() {
    let root = repository();
    std::fs::write(root.path().join("main.dowe"), "one\ntwo\nthree\n").unwrap();
    git(root.path(), &["add", "main.dowe"]);
    git(root.path(), &["commit", "-qm", "three lines"]);
    let left = create_isolated_worktree(root.path(), "left").unwrap();
    let right = create_isolated_worktree(root.path(), "right").unwrap();
    std::fs::write(
        Path::new(&left.path).join("main.dowe"),
        "left\ntwo\nthree\n",
    )
    .unwrap();
    std::fs::write(
        Path::new(&right.path).join("main.dowe"),
        "one\ntwo\nright\n",
    )
    .unwrap();
    let scope = [dowe_agent_harness::AllowedEditSurface::exact("main.dowe").unwrap()];
    let left_result =
        dowe_agent_harness::capture_worker_result(root.path(), &left, "left-task", &[], &scope)
            .unwrap();
    let right_result =
        dowe_agent_harness::capture_worker_result(root.path(), &right, "right-task", &[], &scope)
            .unwrap();
    let merged = dowe_agent_harness::create_worktree_from_results(
        root.path(),
        "merged",
        &[left_result, right_result],
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&merged.path).join("main.dowe")).unwrap(),
        "left\ntwo\nright\n"
    );
}

#[cfg(unix)]
#[test]
fn generated_ancestor_symlink_is_rejected() {
    let root = repository();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join(".dowe")).unwrap();
    assert!(create_isolated_worktree(root.path(), "worker").is_err());
    assert!(!outside.path().join("agent-worktrees").exists());
}

#[cfg(unix)]
#[test]
fn untracked_symlink_is_not_read_or_materialized() {
    let root = repository();
    let worker = create_isolated_worktree(root.path(), "worker").unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(outside.path(), "private").unwrap();
    std::os::unix::fs::symlink(outside.path(), Path::new(&worker.path).join("leak.txt")).unwrap();
    assert!(apply_isolated_worktrees(root.path(), &[worker]).is_err());
    assert!(!root.path().join("leak.txt").exists());
}
