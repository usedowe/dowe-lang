#[test]
fn every_target_generates_only_resolved_computed_icon_names() {
    let temp = tempfile::tempdir().unwrap();
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        r#"page loginPage
  const icons value:[{ id:"one" name:"home" } { id:"two" name:"svg-logos:apple" } { id:"three" name:"home" }]
  Text
    "Selected icons"
  each in:icons as:icon key:icon.id
    Icon name:icon.name"#,
    );
    let project = super::compile_dev_views_for_platforms(
        temp.path(),
        crate::model::ViewPlatform::all().iter().copied(),
    )
    .unwrap();
    for web in [&project.web, &project.desktop_web] {
        assert!(web.router_js.contains("svg-logos:apple"));
        assert!(!web.router_js.contains("svg-logos:apache-flink"));
        assert!(!web.router_js.contains("country-flags:CO"));
    }
    for (needle, entry) in [
        ("DoweDevDynamicIcons", "values.put("),
        ("DoweDynamicIconCatalogShard", "(\""),
    ] {
        let catalog = project
            .apps
            .files
            .iter()
            .filter(|file| file.relative_path.to_string_lossy().contains(needle))
            .map(|file| file.content.as_str())
            .collect::<String>();
        assert_eq!(catalog.matches(entry).count(), 2, "{needle}");
        assert!(catalog.len() < 20_000, "{needle}: {} bytes", catalog.len());
    }
    let compose = project
        .apps
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .unwrap();
    let catalog = compose
        .content
        .split_once("private val DoweDynamicIconCatalog = mapOf(\n")
        .unwrap()
        .1
        .split_once("\n)\n")
        .unwrap()
        .0;
    assert_eq!(catalog.lines().count(), 2);
}

#[test]
fn unbounded_icon_names_fail_before_any_app_artifacts_are_generated() {
    let temp = tempfile::tempdir().unwrap();
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  signal chosen value:\"home\"\n  Text\n    \"Selected icons\"\n  Icon name:chosen",
    );
    let error = super::compile_dev_views_for_platforms(
        temp.path(),
        crate::model::ViewPlatform::all().iter().copied(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("chosen"));
    assert!(error.to_string().contains("const"));
    assert!(!temp.path().join(".dowe/apps").exists());
}
