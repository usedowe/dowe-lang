fn side_nav_route() -> ViewRoute {
    ViewRoute {
        id: "side-nav".to_string(),
        route_path: "/bars".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::SideNav {
            props: SideNavProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(ColorFamily::Surface),
                    ..Default::default()
                },
                size: SideNavSize::Md,
                wide: true,
            },
            items: vec![
                SideNavItem::Header(SideNavItemProps {
                    label: "Workspace".to_string(),
                    description: None,
                    status: None,
                    icon: None,
                    on_click: None,
                    navigation: None,
                }),
                SideNavItem::Submenu {
                    props: SideNavItemProps {
                        label: "Content".to_string(),
                        description: None,
                        status: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    open: true,
                    items: vec![side_nav_item("Blogs", "/bars")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn navigation_shell_route() -> ViewRoute {
    ViewRoute {
        id: "shell".to_string(),
        route_path: "/".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scaffold {
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
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn tabs_route() -> ViewRoute {
    ViewRoute {
        id: "tabs".to_string(),
        route_path: "/tabs".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Tabs {
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
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn drawer_route() -> ViewRoute {
    ViewRoute {
        id: "drawer".to_string(),
        route_path: "/drawer".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            signals: vec![ViewSignal {
                id: "drawer01".to_string(),
                name: "drawerOpen".to_string(),
                initial: ViewSignalValue::Bool(false),
                schema: None,
            }],
            actions: Vec::new(),
            children: vec![ViewNode::Drawer {
                props: DrawerProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    open: "drawerOpen".to_string(),
                    position: DrawerPosition::End,
                    disable_overlay_close: true,
                    hide_close_button: false,
                },
                children: vec![text("Navigation")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
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

fn responsive_route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[
                        (Breakpoint::Xs, 4),
                        (Breakpoint::Md, 8),
                    ])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Children],
        },
        page_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[(Breakpoint::Md, 8)])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Text {
                props: TextProps {
                    size: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: TextSize::Lg,
                    }])),
                    weight: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: TextWeight::Thin,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: TextWeight::Extralight,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Lg,
                            value: TextWeight::Black,
                        },
                    ])),
                    ..Default::default()
                },
                value: "Login".to_string(),
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn show_route() -> ViewRoute {
    ViewRoute {
        id: "ready".to_string(),
        route_path: "/ready".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            signals: vec![
                ViewSignal {
                    id: "ready01".to_string(),
                    name: "isReady".to_string(),
                    initial: ViewSignalValue::Bool(false),
                    schema: None,
                },
                ViewSignal {
                    id: "rows01".to_string(),
                    name: "rows".to_string(),
                    initial: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                        ("id".to_string(), ViewSignalValue::String("1".to_string())),
                        ("ready".to_string(), ViewSignalValue::Bool(true)),
                    ])]),
                    schema: None,
                },
            ],
            actions: Vec::new(),
            children: vec![
                ViewNode::Box {
                    props: StyleProps {
                        element: ElementProps {
                            show: Some(VisibilityCondition::Static(responsive_bool(&[
                                (Breakpoint::Xs, false),
                                (Breakpoint::Md, true),
                            ]))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    children: vec![ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                element: ElementProps {
                                    show: Some(VisibilityCondition::Signal("isReady".to_string())),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Ready".to_string(),
                    }],
                },
                ViewNode::Each {
                    item: "row".to_string(),
                    collection: "rows".to_string(),
                    key: "row.id".to_string(),
                    children: vec![ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                element: ElementProps {
                                    show: Some(VisibilityCondition::Signal(
                                        "row.ready".to_string(),
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Row".to_string(),
                    }],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
