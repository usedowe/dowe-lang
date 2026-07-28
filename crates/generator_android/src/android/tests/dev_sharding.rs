fn dev_module_files(output: &AndroidOutput) -> BTreeMap<String, String> {
    output
        .files
        .iter()
        .filter_map(|file| {
            let name = file.relative_path.file_name()?.to_str()?;
            ((name == "DoweDevActivity.java"
                || name.starts_with("DoweDevRoute")
                || name.starts_with("DoweDevLayout"))
                && name.ends_with(".java"))
            .then(|| (name.to_string(), file.content.clone()))
        })
        .collect()
}

fn changed_dev_modules(before: &AndroidOutput, after: &AndroidOutput) -> Vec<String> {
    let before = dev_module_files(before);
    let after = dev_module_files(after);
    assert_eq!(before.keys().collect::<Vec<_>>(), after.keys().collect::<Vec<_>>());
    before
        .iter()
        .filter_map(|(name, content)| (after.get(name) != Some(content)).then(|| name.clone()))
        .collect()
}

#[test]
fn generates_deterministic_android_dev_modules() {
    let first = route();
    let mut second = route();
    second.id = "signup".to_string();
    second.route_path = "/signup".to_string();
    second.page_tree = text("Signup");
    let routes = [first, second];

    let first_output = generate_android(
        &routes,
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let second_output = generate_android(
        &routes,
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );

    let modules = dev_module_files(&first_output);
    assert_eq!(modules, dev_module_files(&second_output));
    assert!(modules.contains_key("DoweDevActivity.java"));
    assert!(modules.contains_key("DoweDevLayout0.java"));
    assert_eq!(
        modules
            .keys()
            .filter(|name| name.starts_with("DoweDevRoute"))
            .count(),
        2
    );
    assert!(modules["DoweDevActivity.java"].contains("public class DoweDevActivity"));
    assert!(modules["DoweDevActivity.java"].contains(".render(this, root);"));
    assert!(modules["DoweDevActivity.java"].contains("DoweSvgView dowePhoneFlag"));
    assert!(modules["DoweDevActivity.java"].contains("return null;"));
    assert!(!first_output.files.iter().any(|file| {
        file.relative_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("DoweDevPhoneFlags"))
    }));
    assert!(!modules["DoweDevActivity.java"].contains("doweText(\"Login\""));
    assert!(!modules["DoweDevActivity.java"].contains("doweText(\"Signup\""));
    assert!(modules["DoweDevLayout0.java"].contains("doweText(\"Layout\""));
    assert!(!modules["DoweDevLayout0.java"].contains("doweText(\"Login\""));
    let login = modules
        .iter()
        .find(|(name, _)| name.starts_with("DoweDevRouteLogin"))
        .expect("login route module");
    let signup = modules
        .iter()
        .find(|(name, _)| name.starts_with("DoweDevRouteSignup"))
        .expect("signup route module");
    assert!(login.1.contains("doweText(\"Login\""));
    assert!(signup.1.contains("doweText(\"Signup\""));
}

#[test]
fn isolates_android_dev_page_edits_to_one_route_module() {
    let first = route();
    let mut second = route();
    second.id = "signup".to_string();
    second.route_path = "/signup".to_string();
    second.page_tree = text("Signup");
    let before_routes = [first.clone(), second.clone()];

    let mut changed_first = first;
    changed_first.page_tree = text("Login updated");
    let after_routes = [changed_first, second];

    let before = generate_android(
        &before_routes,
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let after = generate_android(
        &after_routes,
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );

    let before_modules = dev_module_files(&before);
    let after_modules = dev_module_files(&after);
    assert_eq!(
        before_modules["DoweDevActivity.java"],
        after_modules["DoweDevActivity.java"]
    );
    assert_eq!(
        before_modules["DoweDevLayout0.java"],
        after_modules["DoweDevLayout0.java"]
    );
    let changed = changed_dev_modules(&before, &after);
    assert_eq!(changed.len(), 1);
    assert!(changed[0].starts_with("DoweDevRouteLogin"));
}

#[test]
fn isolates_android_dev_layout_edits_to_one_layout_module() {
    let layout = |initial| ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "layout.open".to_string(),
            name: "open".to_string(),
            storage_key: "open".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Bool(initial),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![text("Layout"), ViewNode::Children],
        }],
    };
    let mut before_route = route();
    before_route.layout_tree = layout(false);
    let mut after_route = before_route.clone();
    after_route.layout_tree = layout(true);

    let before = generate_android(
        &[before_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let after = generate_android(
        &[after_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );

    assert_eq!(
        changed_dev_modules(&before, &after),
        vec!["DoweDevLayout0.java"]
    );
}

#[test]
fn partitions_large_icon_side_nav_layout_methods() {
    let mut large = route();
    large.layout_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            ViewNode::SideNav {
                props: SideNavProps {
                    style: VariantProps::default(),
                    size: SideNavSize::Md,
                    wide: true,
                    reactive_wide: None,
                },
                items: (0..80)
                    .map(|index| {
                        let mut item = side_nav_item(
                            &format!("Documentation {index}"),
                            Some(NavigationAction::Internal {
                                path: format!("/docs/{index}"),
                                fragment: None,
                                operation: NavigationOperation::Push,
                            }),
                        );
                        item.icon = Some(solar_control_icon("home").expect("icon"));
                        SideNavItem::Item(item)
                    })
                    .collect(),
            },
            ViewNode::Children,
        ],
    };

    let output = generate_android(
        &[large],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let modules = dev_module_files(&output);
    let layout = &modules["DoweDevLayout0.java"];

    assert_eq!(layout.matches("Consumer<LinearLayout> view").count(), 80);
    assert_eq!(layout.matches(".accept(view0);").count(), 80);
    assert!(layout.contains("static void render(DoweDevActivity runtime"));
}

#[test]
fn partitions_large_page_route_methods() {
    let mut large = route();
    large.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: (0..80)
            .map(|index| ViewNode::Section {
                props: StyleProps::default(),
                children: (0..12)
                    .map(|line| text(&format!("Section {index} line {line}")))
                    .collect(),
            })
            .collect(),
    };

    let output = generate_android(
        &[large],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let modules = dev_module_files(&output);
    let route = modules
        .iter()
        .find(|(name, _)| name.starts_with("DoweDevRouteLogin"))
        .map(|(_, content)| content)
        .expect("route module");

    assert_eq!(route.matches("private static void renderPagePart").count(), 80);
    assert_eq!(route.matches("renderPagePart").count(), 160);
    assert!(route.contains("renderPagePart0(runtime, root);"));
    assert!(route.contains("renderPagePart79(runtime, root);"));
}
