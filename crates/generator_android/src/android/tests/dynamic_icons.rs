fn android_bound_icon(name: &str) -> ViewNode {
    icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String(format!("@icon-binding:{name}")),
    }])
    .expect("dynamic icon")
}

fn android_dynamic_icon_catalogs(output: &AndroidOutput) -> (String, String) {
    let java = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevDynamicIcons"))
        })
        .map(|file| file.content.as_str())
        .collect::<String>();
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("Compose views");
    let kotlin = views
        .content
        .split_once("private val DoweDynamicIconCatalog = mapOf(\n")
        .map(|(_, catalog)| catalog.split_once("\n)\n").unwrap().0.to_string())
        .unwrap_or_default();
    (java, kotlin)
}

#[test]
fn android_dynamic_icons_include_only_constant_demand_across_routes() {
    let mut first = route();
    first.page_tree = ViewNode::Scope {
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
                children: vec![android_bound_icon("platform.icon")],
            },
            ViewNode::Each {
                item: "row".to_string(),
                collection: "runtimeRows".to_string(),
                key: "row.id".to_string(),
                children: vec![text("Unrelated mutable list")],
            },
        ],
    };
    first.layout_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "brand".to_string(),
            name: "brand".to_string(),
            value: ViewSignalValue::String("svg-logos:github-icon".to_string()),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![android_bound_icon("brand"), ViewNode::Children],
    };
    let mut second = first.clone();
    second.id = "second".to_string();
    second.route_path = "/second".to_string();
    let output = generate_android(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let (java, kotlin) = android_dynamic_icon_catalogs(&output);
    assert_eq!(java.matches("values.put(").count(), 3);
    assert_eq!(kotlin.lines().count(), 3);
    for name in [
        "route-bold-duotone",
        "svg-logos:apple",
        "svg-logos:github-icon",
    ] {
        assert_eq!(java.matches(&format!("values.put(\"{name}\",")).count(), 1);
        assert_eq!(kotlin.matches(&format!("\"{name}\" to ")).count(), 1);
    }
    assert!(!java.contains("svg-logos:apache-flink"));
    assert!(!kotlin.contains("svg-logos:apache-flink"));
    assert!(java.len() < 25_000, "{} Java catalog bytes", java.len());
    assert!(
        kotlin.len() < 20_000,
        "{} Kotlin catalog bytes",
        kotlin.len()
    );
}

#[test]
#[should_panic(expected = "computed icon names must be validated before generation")]
fn android_dynamic_icons_reject_unbounded_values_without_a_full_catalog() {
    let mut page = route();
    page.layout_tree = ViewNode::Children;
    page.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "selectedIcon".to_string(),
            name: "selectedIcon".to_string(),
            storage_key: "selectedIcon".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("route-bold-duotone".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![android_bound_icon("selectedIcon")],
    };
    generate_android(
        &[page],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
}

#[test]
fn android_static_icons_emit_no_dynamic_catalog() {
    let mut page = route();
    page.layout_tree = ViewNode::Children;
    page.page_tree = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-logos:apple".to_string()),
    }])
    .unwrap();
    let output = generate_android(
        &[page],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let (java, kotlin) = android_dynamic_icon_catalogs(&output);
    assert!(java.is_empty());
    assert!(kotlin.is_empty());
}
