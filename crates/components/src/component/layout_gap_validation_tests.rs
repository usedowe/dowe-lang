#[test]
fn rejects_invalid_section_center_values() {
    let string_value = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("centerX", "true")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("center string");
    assert_eq!(
        string_value,
        ComponentError::invalid_prop("centerX", "boolean")
    );

    let invalid_breakpoint = container_component_node(
        BuiltinComponent::Section,
        vec![responsive_boolean_prop("centerX", &[("xxl", true)])],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("center breakpoint");
    assert_eq!(
        invalid_breakpoint,
        ComponentError::invalid_prop("centerX", "valid breakpoint")
    );
}

