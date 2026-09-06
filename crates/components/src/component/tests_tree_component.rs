#[test]
fn validates_tree_props_and_defaults() {
    let node = tree_component_node(vec![
        string_prop("data", "fileTree"),
        string_prop("bind", "selectedFile"),
        boolean_prop("defaultOpen", false),
        string_prop("emptyLabel", "No project files"),
        string_prop("ariaLabel", "Application files"),
        string_prop("onSelect", "selectFile"),
        string_prop("variant", "outlined"),
        string_prop("scheme", "surface"),
    ])
    .expect("tree");

    match node {
        ViewNode::Tree { props } => {
            assert_eq!(props.data, "fileTree");
            assert_eq!(props.bind.as_deref(), Some("selectedFile"));
            assert!(!props.default_open);
            assert_eq!(props.empty_label, "No project files");
            assert_eq!(props.aria_label, "Application files");
            assert_eq!(props.on_select.as_deref(), Some("selectFile"));
            assert_eq!(props.style.variant, Some(ComponentVariant::Outlined));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
        }
        _ => panic!("tree"),
    }

    let default_node = tree_component_node(vec![string_prop("data", "fileTree")]).expect("default tree");
    match default_node {
        ViewNode::Tree { props } => {
            assert_eq!(props.bind, None);
            assert!(props.default_open);
            assert_eq!(props.empty_label, "No files");
            assert_eq!(props.aria_label, "File tree");
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
        }
        _ => panic!("default tree"),
    }
}

#[test]
fn rejects_invalid_tree_props() {
    assert_eq!(
        tree_component_node(Vec::new()).expect_err("data"),
        ComponentError::invalid_prop("data", "signal object or array path")
    );
    assert_eq!(
        tree_component_node(vec![
            string_prop("data", "fileTree"),
            string_prop("color", "primary"),
        ])
        .expect_err("color"),
        ComponentError::new("unknown prop `color` on `Tree`; use `scheme` for visual family")
    );
    assert_eq!(
        tree_component_node(vec![
            string_prop("data", "fileTree"),
            string_prop("bind", "selectedFile"),
            string_prop("defaultOpen", "yes"),
        ])
        .expect_err("defaultOpen"),
        ComponentError::invalid_prop("defaultOpen", "boolean")
    );
}
