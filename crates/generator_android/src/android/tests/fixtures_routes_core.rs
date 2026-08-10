fn route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![text("Layout"), ViewNode::Children],
        },
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Card {
                props: VariantProps {
                    color: Some(ColorFamily::Primary),
                    ..Default::default()
                },
                children: vec![text("Login")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn section_route() -> ViewRoute {
    ViewRoute {
        id: "sections".to_string(),
        route_path: "/sections".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Section {
                    props: StyleProps {
                        boxed: true,
                        text: Some(ResponsiveValue::scalar(ColorToken::OnBackground)),
                        background: Some(ResponsiveValue::ordered(vec![
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Xs,
                                value: SectionBackground::Aurora,
                            },
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: SectionBackground::Ocean,
                            },
                        ])),
                        ..Default::default()
                    },
                    children: vec![text("Hero")],
                },
                ViewNode::Section {
                    props: StyleProps {
                        cover: Some(ResponsiveValue::scalar(CoverSource(
                            "https://example.com/hero.jpg".to_string(),
                        ))),
                        overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                            "0.35".to_string(),
                        ))),
                        ..Default::default()
                    },
                    children: vec![text("Covered")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn bar_route() -> ViewRoute {
    ViewRoute {
        id: "bars".to_string(),
        route_path: "/bars".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::AppBar {
                    props: BarProps {
                        position: BarPosition::Sticky,
                        ..bar_props(true)
                    },
                    top: Vec::new(),
                    start: vec![text("Menu")],
                    center: vec![text("Brand")],
                    end: vec![text("Account")],
                    bottom: Vec::new(),
                },
                ViewNode::Children,
                ViewNode::Footer {
                    props: bar_props(false),
                    top: vec![text("Directory")],
                    start: vec![text("Footer")],
                    center: Vec::new(),
                    end: vec![text("Legal")],
                    bottom: vec![text("Copyright")],
                },
            ],
        },
        page_tree: ViewNode::BottomBar {
            props: bar_props(false),
            tabs: vec![BottomBarTab {
                label: "Home".to_string(),
                i18n: None,
                featured: true,
                icon: solar_control_icon("home").expect("icon"),
                navigation: NavigationAction::Internal {
                    path: "/bars".to_string(),
                    fragment: None,
                    operation: NavigationOperation::Push,
                },
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn docking_appbar_route() -> ViewRoute {
    ViewRoute {
        id: "docking-appbar".to_string(),
        route_path: "/docking-appbar".to_string(),
        layout_tree: ViewNode::Scaffold {
            props: ScaffoldProps::default(),
            app_bar: vec![ViewNode::AppBar {
                props: BarProps {
                    position: BarPosition::Fixed,
                    floating: true,
                    dock_on_scroll: true,
                    ..bar_props(false)
                },
                top: Vec::new(),
                start: vec![text("Brand")],
                center: vec![text("Navigation")],
                end: vec![text("Account")],
                bottom: Vec::new(),
            }],
            start: Vec::new(),
            main: vec![ViewNode::Children],
            end: Vec::new(),
            bottom_bar: Vec::new(),
            overlays: Vec::new(),
        },
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![text("Scrollable content")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn unbordered_persistent_appbar_route() -> ViewRoute {
    ViewRoute {
        id: "unbordered-appbar".to_string(),
        route_path: "/unbordered-appbar".to_string(),
        layout_tree: ViewNode::AppBar {
            props: BarProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Solid),
                    color: Some(ColorFamily::Surface),
                    ..Default::default()
                },
                position: BarPosition::Sticky,
                ..Default::default()
            },
            top: Vec::new(),
            start: vec![text("Brand")],
            center: Vec::new(),
            end: vec![text("Account")],
            bottom: Vec::new(),
        },
        page_tree: text("Page"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn side_nav_route() -> ViewRoute {
    ViewRoute {
        id: "bars".to_string(),
        route_path: "/bars".to_string(),
        layout_tree: ViewNode::SideNav {
            props: SideNavProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(ColorFamily::Surface),
                    ..Default::default()
                },
                size: SideNavSize::Md,
                wide: false,
                reactive_wide: Some("wideEnabled".to_string()),
            },
            items: vec![
                SideNavItem::Header(side_nav_item("Workspace", None)),
                SideNavItem::Item(side_nav_item(
                    "Home",
                    Some(NavigationAction::Internal {
                        path: "/bars".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                )),
                SideNavItem::Divider,
                SideNavItem::Submenu {
                    props: SideNavItemProps {
                        status: Some("2".to_string()),
                        ..side_nav_item("Content", None)
                    },
                    open: true,
                    bordered: true,
                    items: vec![SideNavItemProps {
                        status: Some("12".to_string()),
                        ..side_nav_item("Blogs", None)
                    }],
                },
            ],
        },
        page_tree: ViewNode::Children,
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
            start: vec![ViewNode::Sidebar {
                props: SidebarProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        style: StyleProps {
                            sizing: dowe_components::SizingProps {
                                w: Some(ResponsiveValue::scalar(SizeValue::Scale(
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
                        label: "Side Home".to_string(),
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
            }],
            main: vec![text("Main content")],
            end: Vec::new(),
            bottom_bar: vec![text("Bottom")],
            overlays: Vec::new(),
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
            constants: Vec::new(),
            signals: vec![ViewSignal {
                id: "drawer01".to_string(),
                name: "drawerOpen".to_string(),
                storage_key: "drawerOpen".to_string(),
                scope: dowe_components::ViewSignalScope::Page,
                storage: dowe_components::ViewSignalStorage::None,
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
                header: vec![text("Menu")],
                body: vec![text("Navigation")],
                footer: vec![text("Footer")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn side_nav_item(label: &str, navigation: Option<NavigationAction>) -> SideNavItemProps {
    SideNavItemProps {
        label: label.to_string(),
        i18n: None,
        description: None,
        description_i18n: None,
        status: (label == "Home").then(|| "New".to_string()),
        status_i18n: None,
        icon: (label == "Home").then(|| SideNavIcon {
            props: SvgProps {
                style: Default::default(),
                view_box: SvgViewBox {
                    min_x: "0".to_string(),
                    min_y: "0".to_string(),
                    width: "24".to_string(),
                    height: "24".to_string(),
                },
                data: None,
                motion: None,
            },
            paths: vec![SvgPath {
                data: "M3 11l9-8 9 8v10H3z".to_string(),
                fill: SvgPathFill::CurrentColor,
                transform: None,
            }],
        }),
        on_click: None,
        navigation,
    }
}
