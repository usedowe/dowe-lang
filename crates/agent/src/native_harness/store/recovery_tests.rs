use super::*;

#[test]
fn session_metadata_is_bounded_and_inventory_falls_back_for_legacy_turns() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();

    let mut current = store.create_session().unwrap();
    current.set_initial_prompt_metadata("Build a dashboard\nwith safe colors\u{1b}[2J");
    store.save_session(&mut current).unwrap();
    let saved = store.load_session(&current.id).unwrap();
    assert_eq!(saved.title.as_deref(), Some("Build a dashboard"));
    assert!(!saved.initial_prompt_preview.as_deref().unwrap().contains('\u{1b}'));

    let mut legacy = store.create_session().unwrap();
    legacy.turns.push(HarnessTurn {
        message: Some(crate::AgentMessage {
            role: "user".into(),
            content: crate::AgentMessageContent::Text("Legacy session prompt".into()),
        }),
        ..Default::default()
    });
    store.save_session(&mut legacy).unwrap();
    let row = store.session_inventory().unwrap().into_iter().find(|row| row["id"] == legacy.id).unwrap();
    assert_eq!(row["title"], "Legacy session prompt");
    assert_eq!(row["initial_prompt_preview"], "Legacy session prompt");
}

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

#[test]
fn dynamic_registry_authority_is_stable_per_registry_and_session_bound() {
    let definition = |base_url: &str| crate::DynamicProviderDefinition {
        name: "local".into(),
        base_url: base_url.into(),
        protocol: crate::AgentProviderProtocol::OpenAiCompletions,
        models: vec!["model-a".into()],
        api_key_env: Some("DOWE_TEST_PROVIDER_KEY".into()),
    };
    let mut registry_a = crate::ProviderRegistry::default();
    registry_a.providers.insert("custom/local".into(), definition("https://a.example"));
    let mut registry_b = registry_a.clone();
    registry_b.providers.get_mut("custom/local").unwrap().base_url = "https://b.example".into();
    assert_eq!(registry_a.canonical_material().unwrap(), registry_a.canonical_material().unwrap());
    assert_ne!(
        crate::native_harness::catalog::authority_fingerprint_with_registry(&registry_a).unwrap(),
        crate::native_harness::catalog::authority_fingerprint_with_registry(&registry_b).unwrap()
    );
    assert!(!registry_a.canonical_material().unwrap().contains("secret-value"));
    let auth = registry_a.resolve_auth("custom/local", Some("secret-value")).unwrap().unwrap();
    assert_eq!(auth.secret.as_deref(), Some("secret-value"));
    assert_eq!(
        crate::native_harness::catalog::authority_fingerprint_with_registry(&registry_a).unwrap(),
        crate::native_harness::catalog::authority_fingerprint_with_registry(&registry_a).unwrap()
    );

    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut config = HarnessConfig::default();
    config.providers = registry_a;
    store.save_config(&config).unwrap();
    let session = store.create_session().unwrap();
    config.providers = registry_b;
    store.save_config(&config).unwrap();
    assert!(store.load_session(&session.id).is_err());
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
