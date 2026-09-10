#[test]
fn rejects_invalid_section_gap_values() {
    let error = container_component_node(
        BuiltinComponent::Section,
        vec![boolean_prop("gap", true)],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("gap boolean");
    assert_eq!(
        error,
        ComponentError::invalid_prop("gap", "Dowe scale value or px value")
    );
}

