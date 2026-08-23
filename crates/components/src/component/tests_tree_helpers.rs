#[test]
fn composes_children_with_page_tree() {
    let layout = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Text {
                props: Default::default(),
                value: "Before".to_string(),
            },
            ViewNode::Children,
            ViewNode::Text {
                props: Default::default(),
                value: "After".to_string(),
            },
        ],
    };
    let page = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Text {
            props: Default::default(),
            value: "Login".to_string(),
        }],
    };

    assert_eq!(
        compose_tree(&layout, &page),
        ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Text {
                    props: Default::default(),
                    value: "Before".to_string()
                },
                page,
                ViewNode::Text {
                    props: Default::default(),
                    value: "After".to_string()
                }
            ]
        }
    );
}

#[test]
fn finds_only_fixed_fabs_in_nested_trees() {
    let fixed = ViewNode::Fab {
        props: FabProps {
            style: Default::default(),
            position: OverlayCornerPosition::BottomRight,
            fixed: true,
            offset_x: ScaleValue::from_half_steps(8),
            offset_y: ScaleValue::from_half_steps(8),
            icon: ViewIcon::Plus,
            label: "Open actions".to_string(),
        },
        actions: Vec::new(),
    };
    let inline = ViewNode::Fab {
        props: FabProps {
            fixed: false,
            ..match &fixed {
                ViewNode::Fab { props, .. } => props.clone(),
                _ => unreachable!(),
            }
        },
        actions: Vec::new(),
    };
    let tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Section {
                props: Default::default(),
                children: vec![fixed],
            },
            inline,
        ],
    };

    let fabs = fixed_fab_nodes(&tree);
    assert_eq!(fabs.len(), 1);
    assert!(matches!(fabs[0], ViewNode::Fab { props, .. } if props.fixed));
}

#[test]
fn finds_first_text() {
    let tree = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Text {
                props: Default::default(),
                value: "Login".to_string(),
            }],
        }],
    };

    assert_eq!(first_text(&tree), Some("Login".to_string()));
}
