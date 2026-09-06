fn tree_route() -> ViewRoute {
    ViewRoute {
        id: "tree".to_string(),
        route_path: "/tree".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::tree_component_node(vec![
            ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("fileTree".to_string()),
            },
            ComponentProp {
                name: "bind".to_string(),
                value: PropValue::String("selectedFile".to_string()),
            },
            ComponentProp {
                name: "defaultOpen".to_string(),
                value: PropValue::Boolean(true),
            },
            ComponentProp {
                name: "emptyLabel".to_string(),
                value: PropValue::String("No project files".to_string()),
            },
            ComponentProp {
                name: "ariaLabel".to_string(),
                value: PropValue::String("Application files".to_string()),
            },
            ComponentProp {
                name: "onSelect".to_string(),
                value: PropValue::String("selectFile".to_string()),
            },
        ])
        .expect("tree"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
