fn bar_props(floating: bool) -> BarProps {
    BarProps {
        style: VariantProps {
            variant: Some(ComponentVariant::Soft),
            color: Some(ColorFamily::Surface),
            ..Default::default()
        },
        bordered: true,
        blurred: true,
        boxed: true,
        floating,
    }
}

fn side_nav_item(label: &str, path: &str) -> SideNavItemProps {
    SideNavItemProps {
        label: label.to_string(),
        description: None,
        status: None,
        icon: None,
        on_click: None,
        navigation: Some(NavigationAction::Internal {
            path: path.to_string(),
            fragment: None,
            operation: NavigationOperation::Push,
        }),
    }
}

fn navigation_shell_tree() -> ViewNode {
    ViewNode::Scaffold {
        props: ScaffoldProps {
            style: StyleProps::default(),
            boxed: true,
        },
        app_bar: vec![ViewNode::NavMenu {
            props: NavMenuProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Ghost),
                    color: Some(ColorFamily::Muted),
                    ..Default::default()
                },
                size: SideNavSize::Md,
            },
            items: vec![
                NavMenuItem::Item(NavMenuItemProps {
                    label: "Home".to_string(),
                    description: None,
                    icon: None,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                }),
                NavMenuItem::Submenu {
                    props: NavMenuItemProps {
                        label: "Docs".to_string(),
                        description: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    items: vec![NavMenuItemProps {
                        label: "Guide".to_string(),
                        description: Some("Start here".to_string()),
                        icon: None,
                        on_click: None,
                        navigation: Some(NavigationAction::Internal {
                            path: "/docs".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                    }],
                },
                NavMenuItem::Megamenu {
                    props: NavMenuItemProps {
                        label: "Resources".to_string(),
                        description: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    content: vec![text("Resource hub")],
                },
            ],
        }],
        start: vec![ViewNode::Sidebar {
            props: SideNavProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(ColorFamily::Surface),
                    style: StyleProps {
                        sizing: dowe_components::SizingProps {
                            w: Some(ResponsiveValue::scalar(
                                dowe_components::SizeValue::Scale(ScaleValue::from_half_steps(
                                    192,
                                )),
                            )),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                size: SideNavSize::Md,
                wide: true,
            },
            items: vec![SideNavItem::Item(SideNavItemProps {
                label: "Side Home".to_string(),
                description: None,
                status: None,
                icon: None,
                on_click: None,
                navigation: Some(NavigationAction::Internal {
                    path: "/".to_string(),
                    fragment: None,
                    operation: NavigationOperation::Push,
                }),
            })],
        }],
        main: vec![text("Main content")],
        end: Vec::new(),
        bottom_bar: vec![text("Bottom")],
    }
}

fn tabs_tree() -> ViewNode {
    ViewNode::Tabs {
        props: TabsProps {
            style: StyleProps::default(),
            variant: TabsVariant::Line,
            color: ColorFamily::Primary,
            position: TabsPosition::Start,
        },
        tabs: vec![
            TabItem {
                id: "overview".to_string(),
                label: "Overview".to_string(),
                children: vec![text("Overview content")],
            },
            TabItem {
                id: "details".to_string(),
                label: "Details".to_string(),
                children: vec![text("Details content")],
            },
        ],
    }
}
