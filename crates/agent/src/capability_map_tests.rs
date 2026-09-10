#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_sources_by_project_domain() {
        assert_eq!(capability_id("src/lib.rs"), "src");
        assert_eq!(capability_id("main.dowe"), "main-dowe");
    }

    #[test]
    fn rejects_generated_and_map_files() {
        assert!(!should_index(".dowe/codegraph/graph.json"));
        assert!(!should_index(".agents/capabilities/index.md"));
        assert!(should_index("src/main.rs"));
    }

    #[test]
    fn sync_is_bounded_and_idempotent() {
        let root = tempfile::tempdir().expect("root");
        let first = sync_capability_map(root.path(), &["src/main.rs".into()]).expect("first");
        assert!(first.changed.iter().any(|path| path.ends_with("src.md")));
        assert!(root.path().join(".agents/capabilities/index.md").is_file());
        let second = sync_capability_map(root.path(), &["src/main.rs".into()]).expect("second");
        assert!(second.changed.is_empty());
    }

    #[test]
    fn preserves_sources_when_a_domain_changes_again() {
        let root = tempfile::tempdir().expect("root");
        sync_capability_map(root.path(), &["src/main.rs".into()]).expect("first");
        sync_capability_map(root.path(), &["src/lib.rs".into()]).expect("second");
        let page = std::fs::read_to_string(root.path().join(".agents/capabilities/src.md"))
            .expect("page");
        assert!(page.contains("src/main.rs"));
        assert!(page.contains("src/lib.rs"));
    }

    #[test]
    fn preserves_human_status_when_refreshing_sources() {
        let root = tempfile::tempdir().expect("root");
        sync_capability_map(root.path(), &["src/main.rs".into()]).expect("first");
        let page_path = root.path().join(".agents/capabilities/src.md");
        let page = std::fs::read_to_string(&page_path).expect("page");
        std::fs::write(&page_path, page.replace("## Status\n\nproposed", "## Status\n\nvalidated"))
            .expect("status");
        sync_capability_map(root.path(), &["src/lib.rs".into()]).expect("second");
        let refreshed = std::fs::read_to_string(page_path).expect("refreshed page");
        assert!(refreshed.contains("## Status\n\nvalidated"));
    }

    #[test]
    fn external_source_change_marks_capability_pending() {
        let root = tempfile::tempdir().expect("root");
        std::fs::create_dir_all(root.path().join("src")).expect("src");
        std::fs::write(root.path().join("src/main.rs"), "before").expect("source");
        sync_capability_map(root.path(), &["src/main.rs".into()]).expect("sync");
        std::fs::write(root.path().join("src/main.rs"), "after").expect("external change");
        let stale = refresh_capability_map(root.path()).expect("refresh");
        assert_eq!(stale, ["src/main.rs"]);
        let page = std::fs::read_to_string(root.path().join(".agents/capabilities/src.md"))
            .expect("page");
        assert!(page.contains("## Status\n\npending"));
        assert!(refresh_capability_map(root.path()).expect("second refresh").is_empty());
    }

    #[test]
    fn bootstrap_seeds_only_supported_project_files_and_is_idempotent() {
        let root = tempfile::tempdir().expect("root");
        fs::create_dir_all(root.path().join("src")).expect("src");
        fs::write(root.path().join("src/main.rs"), "fn main() {}\n").expect("source");
        fs::write(root.path().join("notes.txt"), "ignore").expect("unsupported");
        let first = bootstrap_capability_map(root.path()).expect("bootstrap");
        assert!(first.changed.iter().any(|path| path.ends_with("src.md")));
        assert!(root.path().join(".agents/capabilities/index.md").is_file());
        assert!(!std::fs::read_to_string(root.path().join(".agents/capabilities/src.md"))
            .unwrap()
            .contains("notes.txt"));
        assert!(bootstrap_capability_map(root.path())
            .expect("second bootstrap")
            .changed
            .is_empty());
    }
}
