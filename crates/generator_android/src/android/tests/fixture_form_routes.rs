fn form_route() -> ViewRoute {
    ViewRoute {
        id: "form".to_string(),
        route_path: "/form".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Input {
                    props: VariantProps {
                        label: Some("Email".to_string()),
                        placeholder: Some("Email address".to_string()),
                        variant: Some(ComponentVariant::Outlined),
                        icon_start: Some(solar_control_icon("magnifier").expect("start icon")),
                        icon_end: Some(solar_control_icon("close-circle").expect("end icon")),
                        ..Default::default()
                    },
                },
                ViewNode::Input {
                    props: VariantProps {
                        label: Some("Name".to_string()),
                        placeholder: Some("Full name".to_string()),
                        label_floating: true,
                        size: Some(ButtonSize::Sm),
                        variant: Some(ComponentVariant::Outlined),
                        icon_start: Some(solar_control_icon("magnifier").expect("start icon")),
                        icon_end: Some(solar_control_icon("close-circle").expect("end icon")),
                        ..Default::default()
                    },
                },
                ViewNode::Select {
                    props: VariantProps {
                        label: Some("Department".to_string()),
                        placeholder: Some("Choose department".to_string()),
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                    options: vec![SelectOption {
                        value: "design".to_string(),
                        label: "Design".to_string(),
                        description: None,
                    }],
                    option_each: None,
                },
                ViewNode::Select {
                    props: VariantProps {
                        label: Some("Role".to_string()),
                        placeholder: Some("Choose role".to_string()),
                        label_floating: true,
                        size: Some(ButtonSize::Lg),
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                    options: vec![SelectOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                        description: Some("Manages users".to_string()),
                    }],
                    option_each: None,
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

