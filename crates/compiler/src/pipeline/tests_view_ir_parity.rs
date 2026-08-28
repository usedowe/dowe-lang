fn collect_generator_consumption(
    node: &dowe_components::ViewNode,
    consume: fn(&dowe_components::ViewNode) -> Vec<dowe_components::ConsumedProp>,
) -> std::collections::BTreeSet<(String, String, String)> {
    let mut entries = consume(node)
        .into_iter()
        .map(|entry| {
            (
                entry.component.as_str().to_string(),
                entry.prop,
                entry.ir_field,
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    for children in dowe_components::node_child_groups(node) {
        for child in children {
            entries.extend(collect_generator_consumption(child, consume));
        }
    }
    entries
}

fn assert_consumption_is_declared(
    entries: &[dowe_components::ConsumedProp],
    manifest: &str,
) {
    let declared = manifest
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<std::collections::BTreeSet<_>>();
    for entry in entries {
        let owner = entry.component.as_str();
        let key = format!("{owner}|{}|{}", entry.prop, entry.ir_field.as_str());
        assert!(declared.contains(key.as_str()), "undeclared target consumption: {key}");
    }
}

fn assert_identical_generator_consumption(node: &dowe_components::ViewNode) {
    let web = dowe_generator_web::consumed_props_for_node(node);
    let android = dowe_generator_android::consumed_props_for_node(node);
    let ios = dowe_generator_ios::consumed_props_for_node(node);
    assert_eq!(web, android);
    assert_eq!(web, ios);
    assert_consumption_is_declared(
        &web,
        include_str!("../../../generator_web/target_props.def"),
    );
    assert_consumption_is_declared(
        &android,
        include_str!("../../../generator_android/target_props.def"),
    );
    assert_consumption_is_declared(
        &ios,
        include_str!("../../../generator_ios/target_props.def"),
    );
    let mut registry = dowe_components::PropConsumptionRegistry::default();
    for entry in web {
        dowe_components::register_consumed_prop(
            &mut registry,
            entry.component,
            entry.prop,
            entry.ir_field,
        );
    }
    assert!(registry.validate().is_ok());
}

#[test]
fn generators_report_identical_consumption_for_shared_ir_nodes() {
    let mut props = dowe_components::VariantProps::default();
    props.variant = Some(dowe_components::ComponentVariant::Outlined);
    props.color = Some(dowe_components::ColorFamily::Secondary);
    props.size = Some(dowe_components::ButtonSize::Lg);
    let node = dowe_components::ViewNode::Button {
        props,
        children: Vec::new(),
    };

    assert_identical_generator_consumption(&node);
    assert_identical_generator_consumption(&dowe_components::ViewNode::Box {
        props: dowe_components::StyleProps::default(),
        children: Vec::new(),
    });
    assert_identical_generator_consumption(&dowe_components::ViewNode::Text {
        props: dowe_components::TextProps::default(),
        value: "Text".to_string(),
    });
}

#[test]
fn generators_preserve_reactive_button_bindings_across_targets() {
    let mut props = dowe_components::VariantProps::default();
    props.reactive.loading = Some("isLoading".to_string());
    props.reactive.disabled = Some("isDisabled".to_string());
    let node = dowe_components::ViewNode::Button {
        props,
        children: Vec::new(),
    };
    let web = dowe_generator_web::consumed_props_for_node(&node);
    let android = dowe_generator_android::consumed_props_for_node(&node);
    let ios = dowe_generator_ios::consumed_props_for_node(&node);
    assert_eq!(web, android);
    assert_eq!(web, ios);
    assert!(web.iter().any(|entry| entry.prop == "loading" && entry.ir_field == "VariantProps.reactive.loading"));
    assert!(web.iter().any(|entry| entry.prop == "disabled" && entry.ir_field == "VariantProps.reactive.disabled"));
}

#[test]
fn generators_preserve_form_bindings_across_targets() {
    let mut input = dowe_components::VariantProps::default();
    input.element.bind = Some("email".to_string());
    let input_node = dowe_components::ViewNode::Input { props: input };
    assert_identical_generator_consumption(&input_node);

    let mut checkbox = dowe_components::CheckboxProps {
        style: dowe_components::VariantProps::default(),
        checked: false,
        disabled: false,
        name: None,
    };
    checkbox.style.element.bind = Some("accepted".to_string());
    let checkbox_node = dowe_components::ViewNode::Checkbox { props: checkbox };
    assert_identical_generator_consumption(&checkbox_node);
}

#[test]
fn generated_targets_preserve_default_button_contract() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Button
    "Default""#,
    );
    let project = compile_dev(temp.path()).expect("project");
    let web = &project.web.pages[0].body_html;
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android output");
    let ios = ios_swift_output(temp.path());
    assert!(web.contains("is-solid") && web.contains("is-primary"));
    assert!(android.contains("DoweDesign.primary"));
    assert!(ios.contains("DoweDesign.primary"));
}

#[test]
fn persists_consumption_reports_for_all_view_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Tabs position:"top"
    tab id:"overview" label:"Overview"
      Text
        "Overview""#,
    );
    compile_dev(temp.path()).expect("project");
    for path in [
        ".dowe/web/view-consumption.json",
        ".dowe/apps/android/view-consumption.json",
        ".dowe/apps/android/dev/view-consumption.json",
        ".dowe/apps/ios/view-consumption.json",
        ".dowe/apps/ios/dev/view-consumption.json",
        ".dowe/apps/desktop/view-consumption.json",
    ] {
        let report = fs::read_to_string(temp.path().join(path)).expect(path);
        assert!(report.contains("\"schemaVersion\":1"));
        assert!(report.contains("\"routes\":[\"/\"]"));
        assert!(report.contains("\"owner\":\"Item:Tab\""));
        assert!(report.contains("TabItem.label"));
    }
}

#[test]
fn preserves_one_shared_view_ir_across_web_android_and_ios() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box p:{ xs:2 md:4 } bg:"primary" rounded:"lg"
    Text size:"lg" color:"white"
      "Shared view contract"
    Button variant:"outlined" scheme:"secondary" rounded:"full"
      "Continue""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let web = project.view_routes.web.first().expect("web route");
    let android = project.view_routes.android.first().expect("android route");
    let ios = project.view_routes.ios.first().expect("ios route");

    assert_eq!(web.layout_tree, android.layout_tree);
    assert_eq!(web.layout_tree, ios.layout_tree);
    assert_eq!(web.page_tree, android.page_tree);
    assert_eq!(web.page_tree, ios.page_tree);
    assert_eq!(web.sections, android.sections);
    assert_eq!(web.sections, ios.sections);
    assert_eq!(web.navigation_actions, android.navigation_actions);
    assert_eq!(web.navigation_actions, ios.navigation_actions);

    let web_consumption = collect_generator_consumption(
        &web.page_tree,
        dowe_generator_web::consumed_props_for_node,
    );
    let android_consumption = collect_generator_consumption(
        &android.page_tree,
        dowe_generator_android::consumed_props_for_node,
    );
    let ios_consumption = collect_generator_consumption(
        &ios.page_tree,
        dowe_generator_ios::consumed_props_for_node,
    );
    assert_eq!(web_consumption, android_consumption);
    assert_eq!(web_consumption, ios_consumption);

    let web_tree = dowe_generator_web::consumed_props_for_tree(&web.page_tree);
    let android_tree = dowe_generator_android::consumed_props_for_tree(&android.page_tree);
    let ios_tree = dowe_generator_ios::consumed_props_for_tree(&ios.page_tree);
    assert_eq!(web_tree, android_tree);
    assert_eq!(web_tree, ios_tree);

    assert!(project
        .apps
        .files
        .iter()
        .any(|file| file.target == "android" && file.relative_path.extension().is_some_and(|extension| extension == "kt")));
    assert!(project
        .apps
        .files
        .iter()
        .any(|file| file.target == "ios" && file.relative_path.extension().is_some_and(|extension| extension == "swift")));
    assert!(project.web.pages.iter().any(|page| {
        page.body_html.contains("Shared view contract")
            && page.body_html.contains("is-outlined")
            && page.body_html.contains("is-secondary")
    }));
}
