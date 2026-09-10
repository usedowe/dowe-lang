#[test]
fn generates_compose_form_validation_contract() {
    let mut props = VariantProps {
        label: Some("Email".to_string()),
        variant: Some(ComponentVariant::Outlined),
        ..Default::default()
    };
    let validation = props.element.form_validation_mut();
    validation.help_text = Some("Use your work email".to_string());
    validation.rules = vec![
        dowe_components::form_validation_rule("required", "Email is required").expect("rule"),
        dowe_components::form_validation_rule("email", "Enter a valid email").expect("rule"),
    ];
    let route = ViewRoute {
        id: "validation".to_string(),
        route_path: "/validation".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Input { props },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = all_android_source(&output);

    assert!(source.contains("private data class DoweValidationRule"));
    assert!(source.contains("private fun doweValidationError"));
    assert!(source.contains("message = \"Email is required\""));
    assert!(source.contains("helpText = \"Use your work email\""));
    assert!(source.contains("if (touched) doweValidationError"));
    assert!(source.contains("DoweDesign.danger"));

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("private final class DoweValidationBinding")
    );
    assert!(dev.content.contains("doweTouchedValidations"));
    assert!(
        dev.content
            .contains("new String[]{\"required\", null, \"Email is required\"}")
    );
    assert!(dev.content.contains("Validation.watchText();"));
    assert!(dev.content.contains("DOWE_DANGER"));
}

