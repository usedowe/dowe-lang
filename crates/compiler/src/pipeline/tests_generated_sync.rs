#[test]
fn generated_tree_preserves_unchanged_files_and_removes_obsolete_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    let tree = temp.path().join(".dowe/web");
    let current = crate::model::GeneratedFile {
        relative_path: std::path::PathBuf::from("web/current.js"),
        content: "current".to_string(),
        kind: "JavaScript".to_string(),
        target: "web".to_string(),
    };
    super::sync_generated_tree(temp.path(), &tree, std::slice::from_ref(&current))
        .expect("first sync");
    let path = tree.join("current.js");
    let first_modified = fs::metadata(&path)
        .expect("metadata")
        .modified()
        .expect("modified");
    fs::write(tree.join("obsolete.js"), "obsolete").expect("obsolete");
    std::thread::sleep(std::time::Duration::from_millis(20));

    super::sync_generated_tree(temp.path(), &tree, &[current]).expect("second sync");

    let second_modified = fs::metadata(&path)
        .expect("metadata")
        .modified()
        .expect("modified");
    assert_eq!(first_modified, second_modified);
    assert!(!tree.join("obsolete.js").exists());
}
