#[test]
fn parses_overlay_forms() {
    let rgba = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("cover", "/images/hero.jpg"),
            number_prop("overlay", 1),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("rgba");

    match rgba {
        ViewNode::Box { props, .. } => {
            assert!(matches!(
                props.overlay.expect("overlay").entries[0].value,
                OverlayPaint::BlackOpacity(_)
            ));
        }
        _ => panic!("box"),
    }

    assert!(
        container_component_node(
            BuiltinComponent::Box,
            vec![
                string_prop("cover", "/images/hero.jpg"),
                string_prop("overlay", "blur(4px)"),
            ],
            vec![text_node("Hero").expect("text")],
            false,
        )
        .is_err()
    );
}

