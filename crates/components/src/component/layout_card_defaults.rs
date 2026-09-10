#[test]
fn normalizes_card_padding_default_and_author_override() {
    let default_card = container_component_node(
        BuiltinComponent::Card,
        Vec::new(),
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("card");

    match default_card {
        ViewNode::Card { props, .. } => {
            let padding = props.style.spacing.p.expect("default padding");
            assert_eq!(padding.entries.len(), 2);
            assert_eq!(padding.entries[0].breakpoint, Breakpoint::Xs);
            assert_eq!(padding.entries[0].value, ScaleValue::from_half_steps(8));
            assert_eq!(padding.entries[1].breakpoint, Breakpoint::Lg);
            assert_eq!(padding.entries[1].value, ScaleValue::from_half_steps(10));
            assert!(props.style.spacing.px.is_none());
            assert!(props.style.spacing.py.is_none());
        }
        _ => panic!("card"),
    }

    let padded_card = container_component_node(
        BuiltinComponent::Card,
        vec![number_prop("p", 4)],
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("padded card");

    match padded_card {
        ViewNode::Card { props, .. } => {
            assert_eq!(
                props.style.spacing.p.expect("p").entries[0].value,
                ScaleValue::from_half_steps(8)
            );
        }
        _ => panic!("card"),
    }

    let vertical_card = container_component_node(
        BuiltinComponent::Card,
        vec![number_prop("py", 6)],
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("vertical card");

    match vertical_card {
        ViewNode::Card { props, .. } => {
            assert!(props.style.spacing.p.is_none());
            assert_eq!(
                props.style.spacing.py.expect("py").entries[0].value,
                ScaleValue::from_half_steps(12)
            );
            let horizontal = props.style.spacing.px.expect("default px");
            assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
            assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(10));
        }
        _ => panic!("card"),
    }
}

