use super::*;

#[test]
fn active_owners_and_processes_cannot_be_recovered() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let source = store.create_session().unwrap();
    let inspection = store.inspect_session(&source.id, 0).unwrap();
    let owner = store.lease_session(&source.id).unwrap();
    assert_eq!(store.inspect_session(&source.id, 0).unwrap()["busy"], true);
    assert!(
        store
            .recover_session(&source.id, inspection["ticket"].as_str().unwrap(), "Review")
            .is_err()
    );
    drop(owner);
    let process = store.acquire_shell("resource:dev:web", &source.id).unwrap();
    let inspection = store.inspect_session(&source.id, 0).unwrap();
    assert!(
        store
            .recover_session(&source.id, inspection["ticket"].as_str().unwrap(), "Review")
            .is_err()
    );
    drop(process);
    assert!(
        store
            .recover_session(&source.id, inspection["ticket"].as_str().unwrap(), "Review")
            .is_err()
    );
    assert_eq!(store.sessions().unwrap(), vec![source.id]);
}

#[cfg(unix)]
#[test]
fn evidence_does_not_follow_links_or_read_credential_and_generated_paths() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    fs::write(root.path().join("main.dowe"), "main\n").unwrap();
    std::os::unix::fs::symlink("main.dowe", root.path().join("alias.dowe")).unwrap();
    assert!(store.file_fingerprint("alias.dowe").is_err());
    fs::hard_link(root.path().join("main.dowe"), root.path().join("hard.dowe")).unwrap();
    assert!(store.file_fingerprint("hard.dowe").is_err());
    fs::remove_file(root.path().join("hard.dowe")).unwrap();
    assert!(store.file_fingerprint("main.dowe").is_ok());
    fs::write(root.path().join("auth.json"), "private").unwrap();
    fs::create_dir(root.path().join("target")).unwrap();
    fs::write(root.path().join("target/generated.dowe"), "main\n").unwrap();
    assert!(store.file_fingerprint("auth.json").is_err());
    assert!(store.file_fingerprint("target/generated.dowe").is_err());
}
