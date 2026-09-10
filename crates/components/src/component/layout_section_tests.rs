#[test]
fn validates_section_background_props() {
    let node = container_component_node(
        BuiltinComponent::Section,
        vec![
            string_prop("background", "aurora"),
            string_prop("color", "backgroundText"),
            string_prop("animation", "fadeIn"),
            boolean_prop("boxed", true),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("section");

    match node {
        ViewNode::Section { props, .. } => {
            assert_eq!(
                props.background.as_ref().expect("background").entries[0].value,
                SectionBackground::Aurora
            );
            assert!(props.text.is_some());
            assert_eq!(props.animation(), Some(ViewAnimation::FadeIn));
            assert!(props.boxed);
        }
        _ => panic!("section"),
    }
}

#[test]
fn rejects_invalid_section_background_props() {
    let invalid_background = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("background", "custom")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("background");
    assert_eq!(
        invalid_background,
        ComponentError::invalid_prop(
            "background",
            "aurora, sunrise, ocean, meadow or slate"
        )
    );

    let combined_layers = container_component_node(
        BuiltinComponent::Section,
        vec![
            string_prop("background", "aurora"),
            string_prop("cover", "/hero.jpg"),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("layers");
    assert_eq!(
        combined_layers,
        ComponentError::invalid_prop_combination(
            "`cover` and `background` cannot be used together on `Section`"
        )
    );
}

