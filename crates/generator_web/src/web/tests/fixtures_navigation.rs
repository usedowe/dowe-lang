fn bar_props(floating: bool) -> BarProps {
    BarProps {
        style: VariantProps {
            variant: Some(ComponentVariant::Solid),
            color: Some(ColorFamily::Surface),
            ..Default::default()
        },
        bordered: true,
        blurred: true,
        boxed: true,
        floating,
        position: BarPosition::Static,
        hide_on_scroll: false,
        dock_on_scroll: false,
    }
}

fn side_nav_item(label: &str, path: &str) -> SideNavItemProps {
    SideNavItemProps {
        label: label.to_string(),
        i18n: None,
        description: None,
        description_i18n: None,
        status: None,
        status_i18n: None,
        icon: None,
        on_click: None,
        navigation: Some(NavigationAction::Internal {
            path: path.to_string(),
            fragment: None,
            operation: NavigationOperation::Push,
        }),
    }
}

fn shell_sidebar(label: &str) -> ViewNode {
    ViewNode::Sidebar {
        props: SidebarProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Solid),
                color: Some(ColorFamily::Surface),
                style: StyleProps {
                    sizing: dowe_components::SizingProps {
                        w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                            ScaleValue::from_half_steps(192),
                        ))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        header: Vec::new(),
        body: vec![ViewNode::SideNav {
            props: SideNavProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Ghost),
                    color: Some(ColorFamily::Muted),
                    ..Default::default()
                },
                size: SideNavSize::Md,
                wide: true,
                reactive_wide: None,
            },
            items: vec![SideNavItem::Item(SideNavItemProps {
                label: label.to_string(),
                i18n: None,
                description: None,
                description_i18n: None,
                status: None,
                status_i18n: None,
                icon: None,
                on_click: None,
                navigation: Some(NavigationAction::Internal {
                    path: "/".to_string(),
                    fragment: None,
                    operation: NavigationOperation::Push,
                }),
            })],
        }],
        footer: Vec::new(),
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
                    i18n: Some("home.hero.title".to_string()),
                    description: None,
                    description_i18n: None,
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
                        i18n: None,
                        description: None,
                        description_i18n: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    items: vec![NavMenuItemProps {
                        label: "Guide".to_string(),
                        i18n: None,
                        description: Some("Start here".to_string()),
                        description_i18n: None,
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
                        i18n: None,
                        description: None,
                        description_i18n: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    content: vec![text("Resource hub")],
                },
            ],
        }],
        start: vec![shell_sidebar("Start home")],
        main: vec![text("Main content")],
        end: vec![shell_sidebar("End home")],
        bottom_bar: vec![text("Bottom")],
        overlays: vec![text("Shell overlay")],
    }
}

fn tabs_tree() -> ViewNode {
    ViewNode::Tabs {
        props: TabsProps {
            style: StyleProps::default(),
            variant: TabsVariant::Line,
            color: ColorFamily::Primary,
            position: TabsPosition::Start,
            variant_explicit: true,
            color_explicit: true,
        },
        tabs: vec![
            TabItem {
                id: "overview".to_string(),
                label: "Overview".to_string(),
                i18n: None,
                children: vec![text("Overview content")],
            },
            TabItem {
                id: "details".to_string(),
                label: "Details".to_string(),
                i18n: None,
                children: vec![text("Details content")],
            },
        ],
    }
}
