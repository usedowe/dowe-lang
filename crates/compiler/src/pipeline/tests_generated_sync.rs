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

#[test]
fn selected_web_development_compile_skips_unselected_app_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Text\n    \"Home\"",
    );

    let project = super::compile_dev_for_platforms(
        temp.path(),
        [crate::model::ViewPlatform::Web],
    )
    .expect("web compile");

    assert!(project.apps.files.is_empty());
    assert!(!project.view_routes.web.is_empty());
    assert!(project.view_routes.desktop.is_empty());
    assert!(project.view_routes.android.is_empty());
    assert!(project.view_routes.ios.is_empty());
    assert!(temp.path().join(".dowe/web/manifest.json").is_file());
    assert!(!temp.path().join(".dowe/apps").exists());
}

#[test]
fn selected_android_development_compile_writes_only_android_app_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Text\n    \"Home\"",
    );

    let project = super::compile_dev_for_platforms(
        temp.path(),
        [crate::model::ViewPlatform::Android],
    )
    .expect("android compile");

    assert!(project.apps.files.iter().any(|file| file.target == "android"));
    assert!(project.view_routes.web.is_empty());
    assert!(project.view_routes.desktop.is_empty());
    assert!(!project.view_routes.android.is_empty());
    assert!(project.view_routes.ios.is_empty());
    assert!(project
        .apps
        .files
        .iter()
        .all(|file| matches!(file.target.as_str(), "android" | "apps")));
    assert!(temp.path().join(".dowe/apps/android").is_dir());
    assert!(!temp.path().join(".dowe/apps/desktop").exists());
    assert!(!temp.path().join(".dowe/apps/ios").exists());
    assert!(!temp.path().join(".dowe/web").exists());
}

#[test]
fn selected_views_rebuild_does_not_parse_server_modules() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Text\n    \"Home\"",
    );
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    unsupported"#,
    )
    .expect("main");

    let project = super::compile_dev_views_for_platforms(
        temp.path(),
        [crate::model::ViewPlatform::Web],
    )
    .expect("views compile");

    assert!(!project.view_routes.web.is_empty());
    assert!(project.backend.endpoints.is_empty());
    assert!(super::compile_dev_for_platforms(
        temp.path(),
        [crate::model::ViewPlatform::Web],
    )
    .is_err());
}
