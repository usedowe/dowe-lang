#[test]
fn rejects_non_boolean_section_boxed_prop() {
    let error = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("boxed", "true")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("boxed");

    assert_eq!(error, ComponentError::invalid_prop("boxed", "boolean"));
}

