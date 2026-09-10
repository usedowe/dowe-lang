#[test]
fn rejects_overlay_without_cover() {
    let error = container_component_node(
        BuiltinComponent::Box,
        vec![number_prop("overlay", 1)],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("overlay error");

    assert_eq!(
        error,
        ComponentError::invalid_prop_combination("`overlay` requires `cover` on `Box`")
    );
}

