#[test]
fn swiftui_shared_runtime_keeps_fragment_type_declarations() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let runtime = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("shared runtime");
    for declaration in [
        "struct DoweEmpty: View {",
        "struct DoweTabItem: Identifiable {",
        "struct DoweSvgViewBox {",
    ] {
        assert_eq!(
            runtime.content.matches(declaration).count(),
            1,
            "{declaration}"
        );
    }
}

#[test]
#[cfg(target_os = "macos")]
fn swiftui_shared_runtime_parses_with_swift() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let runtime = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("shared runtime");
    let directory =
        std::env::temp_dir().join(format!("dowe-ios-cold-start-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("DowePages.swift");
    std::fs::write(&source, &runtime.content).unwrap();
    let parsed = std::process::Command::new("xcrun")
        .args(["swiftc", "-frontend", "-parse"])
        .arg(&source)
        .output()
        .expect("Swift parser");
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        parsed.status.success(),
        "{}",
        String::from_utf8_lossy(&parsed.stderr)
    );
}

#[test]
fn swiftui_dynamic_icons_generate_only_referenced_constant_names() {
    let icon = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("@icon-binding:platform.icon".to_string()),
    }])
    .expect("dynamic icon");
    let mut page = route();
    page.layout_tree = ViewNode::Children;
    page.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "platforms".to_string(),
            name: "platforms".to_string(),
            value: ViewSignalValue::Array(
                [
                    "route-bold-duotone",
                    "svg-logos:apple",
                    "route-bold-duotone",
                ]
                .into_iter()
                .map(|name| {
                    ViewSignalValue::Object(vec![(
                        "icon".to_string(),
                        ViewSignalValue::String(name.to_string()),
                    )])
                })
                .collect(),
            ),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            ViewNode::Each {
                item: "platform".to_string(),
                collection: "platforms".to_string(),
                key: "platform.icon".to_string(),
                children: vec![icon],
            },
            ViewNode::Each {
                item: "entry".to_string(),
                collection: "runtimeEntries".to_string(),
                key: "entry.id".to_string(),
                children: vec![text("Unrelated mutable list")],
            },
        ],
    };
    let mut second = page.clone();
    second.route_path = "/second".to_string();
    let output = generate_ios(
        &[page, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let catalogs = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .to_string_lossy()
                .contains("DoweDynamicIconCatalog")
        })
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(catalogs.contains("catalog.reserveCapacity(2)"));
    assert_eq!(catalogs.matches("(\"route-bold-duotone\",").count(), 1);
    assert_eq!(catalogs.matches("(\"svg-logos:apple\",").count(), 1);
    assert!(!catalogs.contains("country-flags:CO"));
    assert!(!catalogs.contains("svg-logos:github-icon"));
    assert!(catalogs.len() < 20_000, "{} catalog bytes", catalogs.len());
}

#[test]
fn swiftui_static_icons_do_not_generate_a_dynamic_catalog() {
    let output = generate_ios(
        &[side_nav_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    assert!(!output.files.iter().any(|file| {
        file.relative_path
            .to_string_lossy()
            .contains("DoweDynamicIconCatalog")
    }));
}

#[test]
fn swiftui_runtime_fragment_boundaries_do_not_repeat_members() {
    let mut side_nav = side_nav_route();
    side_nav.route_path = "/side-nav".to_string();
    let output = generate_ios(
        &[bar_route(), side_nav],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let runtime = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("shared runtime");
    let lines = runtime.content.lines().map(str::trim).collect::<Vec<_>>();
    for pair in lines.windows(2) {
        if pair[0].starts_with("let ") || pair[0].starts_with("case ") || pair[0].starts_with('@') {
            assert_ne!(pair[0], pair[1], "repeated Swift member: {}", pair[0]);
        }
    }
    assert!(!swift_content(&output).contains(".frame(alignment:"));
}

#[test]
#[cfg(target_os = "macos")]
fn swiftui_cold_start_typechecks_with_simulator_sdk() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let sdk = Command::new("xcrun")
        .args(["--sdk", "iphonesimulator", "--show-sdk-path"])
        .output()
        .expect("Xcode SDK query");
    if !sdk.status.success() {
        eprintln!("skipping iOS cold-start typecheck: simulator SDK unavailable");
        return;
    }
    let mut side_nav = side_nav_route();
    side_nav.route_path = "/side-nav".to_string();
    let output = generate_ios(
        &[bar_route(), side_nav],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let directory = std::env::temp_dir().join(format!("dowe-ios-typecheck-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let mut sources = Vec::new();
    for file in &output.files {
        if file
            .relative_path
            .extension()
            .is_none_or(|extension| extension != "swift")
            || file.relative_path.ends_with("DoweIosApp.swift")
            || file.relative_path.ends_with("DoweIosDevHost.swift")
        {
            continue;
        }
        let path = directory.join(file.relative_path.file_name().unwrap());
        std::fs::write(&path, &file.content).unwrap();
        sources.push(path);
    }
    let architecture = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86_64"
    };
    let target = format!("{architecture}-apple-ios17.0-simulator");
    let diagnostics = directory.join("diagnostics.log");
    let mut child = Command::new("xcrun")
        .args([
            "swiftc",
            "-frontend",
            "-typecheck",
            "-parse-as-library",
            "-sdk",
        ])
        .arg(String::from_utf8_lossy(&sdk.stdout).trim())
        .args(["-target", &target, "-module-name", "DoweIosViewModule"])
        .args(sources)
        .stdout(Stdio::null())
        .stderr(std::fs::File::create(&diagnostics).unwrap())
        .spawn()
        .expect("Swift typechecker");
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if start.elapsed() > Duration::from_secs(60) {
            child.kill().unwrap();
            child.wait().unwrap();
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let diagnostics = std::fs::read_to_string(diagnostics).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        status.is_some_and(|status| status.success()),
        "Swift cold-start typecheck: {status:?}\n{diagnostics}"
    );
}
