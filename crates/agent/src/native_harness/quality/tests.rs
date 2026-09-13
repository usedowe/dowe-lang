use super::*;

#[test]
fn flags_redundant_component_defaults_and_allows_differences() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("main.dowe"),
        "import view from \"views/page.dowe\"\nmain {}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("theme.dowe"),
        "theme\n    Card variant:\"solid\" scheme:\"surface\"\n",
    )
    .unwrap();
    fs::write(
        root.path().join("views.dowe"),
        "Card variant:\"solid\" scheme:\"primary\"\n",
    )
    .unwrap();
    let report = audit_dowe_project(root.path()).unwrap();
    assert_eq!(report["status"], "failed");
    assert_eq!(report["findings"].as_array().unwrap().len(), 1);
    assert_eq!(report["findings"][0]["prop"], "variant");
}

#[test]
fn reads_canonical_two_space_design_defaults() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"reference\"\n    Card variant:\"solid\" scheme:\"surface\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.path().join("views")).unwrap();
    fs::write(
        root.path().join("views/page.dowe"),
        "page Page\n  Card variant:\"solid\" scheme:\"primary\"\n",
    )
    .unwrap();
    let report = audit_dowe_project(root.path()).unwrap();
    assert_eq!(report["defaults_loaded"], 1);
    assert_eq!(report["status"], "failed");
    assert_eq!(report["findings"].as_array().unwrap().len(), 1);
    assert_eq!(report["findings"][0]["prop"], "variant");
}

#[test]
fn reports_not_run_when_no_design_defaults_exist() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    let report = audit_dowe_project(root.path()).unwrap();
    assert_eq!(report["status"], "not_run");
    assert_eq!(report["defaults_loaded"], 0);
}

#[test]
fn strict_ui_audit_flags_typography_and_missing_mobile_navigation() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    Card variant:\"solid\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.path().join("views")).unwrap();
    fs::write(
        root.path().join("views/page.dowe"),
        "page Home\n  Scaffold\n    appBar\n      AppBar\n        center\n          NavMenu\n            item label:\"Home\" href:\"/\"\n    main\n      Section\n        Text size:\"xs\" color:\"muted\" weight:\"bold\"\n          \"Caption\"\n        Title as:\"h1\" size:{ xs:\"4xl\" md:\"6xl\" } weight:\"black\"\n          \"Home\"\n",
    )
    .unwrap();
    fs::write(
        root.path().join("views/navigation.dowe"),
        "component Navigation\n  NavMenu variant:\"solid\"\n    item label:\"Home\" href:\"/\"\n",
    )
    .unwrap();

    let report = audit_dowe_project_for_ui(root.path()).unwrap();
    assert_eq!(report["status"], "failed");
    let findings = report["findings"].as_array().unwrap();
    assert!(findings.iter().any(|finding| finding["prop"] == "color"));
    assert!(findings.iter().any(|finding| finding["prop"] == "weight"));
    assert!(findings.iter().any(|finding| finding["prop"] == "size"));
    assert!(findings.iter().any(|finding| finding["prop"] == "variant"));
    assert!(
        findings
            .iter()
            .any(|finding| finding["prop"] == "mobileNavigation")
    );
}

#[test]
fn strict_ui_audit_accepts_default_first_responsive_shell() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    Card variant:\"solid\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.path().join("views")).unwrap();
    fs::write(
        root.path().join("views/page.dowe"),
        "page Home\n  Scaffold\n    appBar\n      AppBar\n        start\n          IconButton show:{ xs:true md:false } icon:\"menu-dots\" label:\"Open menu\"\n        center\n          NavMenu show:{ xs:false md:true }\n            item label:\"Home\" href:\"/\"\n    main\n      Section\n        Text\n          \"Body\"\n        Title as:\"h1\" size:\"6xl\"\n          \"Home\"\n    overlays\n      Drawer bind:openNavigation\n        body\n          SideNav\n            item label:\"Home\" href:\"/\"\n",
    )
    .unwrap();

    let report = audit_dowe_project_for_ui(root.path()).unwrap();
    assert_eq!(report["status"], "passed", "{report}");
    assert!(report["findings"].as_array().unwrap().is_empty());
}

#[test]
fn strict_ui_audit_keeps_repeated_defaults_as_advisory_warnings() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    Card variant:\"solid\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.path().join("views")).unwrap();
    fs::write(
        root.path().join("views/page.dowe"),
        "page Home\n  Card variant:\"solid\"\n",
    )
    .unwrap();

    let report = audit_dowe_project_for_ui(root.path()).unwrap();
    assert_eq!(report["status"], "passed", "{report}");
    assert!(report["findings"].as_array().unwrap().is_empty());
    assert_eq!(report["warnings"][0]["prop"], "variant");
}
