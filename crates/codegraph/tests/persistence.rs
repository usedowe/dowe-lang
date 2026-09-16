use dowe_codegraph::{
    GraphFreshness, ensure_persistent_codegraph, read_persistent_codegraph,
    refresh_persistent_codegraph,
};
use std::fs;
use tempfile::TempDir;

const STORE: &str = ".dowe/codegraph";
const LEGACY_STORE: &str = ".agents/codegraph";

#[test]
fn generation_round_trip_and_incremental_reuse() {
    let d = TempDir::new().unwrap();
    fs::create_dir_all(d.path().join("src")).unwrap();
    fs::write(d.path().join("src/a.rs"), "pub fn a() {}\n").unwrap();
    let first = ensure_persistent_codegraph(d.path()).unwrap();
    assert_eq!(first.freshness, GraphFreshness::Initialized);
    let second = refresh_persistent_codegraph(d.path()).unwrap();
    assert_eq!(second.freshness, GraphFreshness::Fresh);
    assert_eq!(second.manifest.revision, first.manifest.revision);
    let current = fs::read_to_string(d.path().join(STORE).join("CURRENT")).unwrap();
    let generation = d.path().join(STORE).join(current.trim());
    for file in ["manifest.json", "nodes.json", "edges.json", "index.json"] {
        assert!(generation.join(file).is_file());
    }
    fs::write(generation.join("manifest.json"), "{}").unwrap();
    assert!(refresh_persistent_codegraph(d.path()).is_err());
}

#[test]
fn refresh_keeps_only_the_current_generation() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("main.py"), "print(1)\n").unwrap();
    let first = ensure_persistent_codegraph(d.path()).unwrap();
    fs::write(d.path().join("main.py"), "print(2)\n").unwrap();
    let second = refresh_persistent_codegraph(d.path()).unwrap();
    fs::write(d.path().join("main.py"), "print(3)\n").unwrap();
    let third = refresh_persistent_codegraph(d.path()).unwrap();
    let store = d.path().join(STORE);
    let generations = fs::read_dir(&store)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("generation-")
        })
        .collect::<Vec<_>>();
    assert_eq!(generations.len(), 1);
    assert_eq!(
        fs::read_to_string(store.join("CURRENT")).unwrap().trim(),
        third.generation.as_deref().unwrap()
    );
    assert_ne!(first.generation, second.generation);
    assert_ne!(second.generation, third.generation);
    let stale = store.join("generation-stale");
    fs::create_dir(&stale).unwrap();
    fs::write(store.join("CURRENT.tmp"), "generation-stale\n").unwrap();
    refresh_persistent_codegraph(d.path()).unwrap();
    assert!(!stale.exists());
    assert!(!store.join("CURRENT.tmp").exists());
}

#[test]
fn legacy_store_is_read_and_next_refresh_publishes_to_dowe() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("main.py"), "print(1)\n").unwrap();
    let snapshot = ensure_persistent_codegraph(d.path()).unwrap();
    let current = fs::read_to_string(d.path().join(STORE).join("CURRENT")).unwrap();
    let legacy = d.path().join(LEGACY_STORE);
    fs::create_dir_all(&legacy).unwrap();
    fs::rename(
        d.path().join(STORE).join(current.trim()),
        legacy.join(current.trim()),
    )
    .unwrap();
    fs::rename(d.path().join(STORE).join("CURRENT"), legacy.join("CURRENT")).unwrap();
    fs::remove_dir(d.path().join(STORE)).unwrap();
    let read = read_persistent_codegraph(d.path()).unwrap();
    assert_eq!(read.manifest.revision, snapshot.manifest.revision);
    let refreshed = refresh_persistent_codegraph(d.path()).unwrap();
    assert!(d.path().join(STORE).join("CURRENT").is_file());
    assert_eq!(refreshed.manifest.revision, snapshot.manifest.revision + 1);
}
