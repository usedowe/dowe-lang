fn route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![text("Layout"), ViewNode::Children],
        },
        page_tree: ViewNode::Card {
            props: Default::default(),
            children: vec![text("Login")],
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
                        text: Some(ResponsiveValue::scalar(ColorToken::BackgroundText)),
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
                mobile_menu: None,
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
                mobile_menu: None,
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

fn appbar_divider_route() -> ViewRoute {
    ViewRoute {
        id: "appbar-divider".to_string(),
        route_path: "/appbar-divider".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::AppBar {
                    props: bar_props(false),
                    top: Vec::new(),
                    start: vec![text("Menu")],
                    center: vec![text("Brand")],
                    end: vec![text("Account")],
                    bottom: Vec::new(),
                mobile_menu: None,
                },
                ViewNode::Children,
            ],
        },
        page_tree: text("Page"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
