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
                    props: bar_props(true),
                    start: vec![text("Menu")],
                    center: vec![text("Brand")],
                    end: vec![text("Account")],
                },
                ViewNode::Children,
                ViewNode::Footer {
                    props: bar_props(false),
                    start: vec![text("Footer")],
                    center: Vec::new(),
                    end: vec![text("Legal")],
                },
            ],
        },
        page_tree: ViewNode::BottomBar {
            props: bar_props(false),
            start: vec![text("Home")],
            center: vec![text("Search")],
            end: vec![text("Profile")],
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
                    start: vec![text("Menu")],
                    center: vec![text("Brand")],
                    end: vec![text("Account")],
                },
                ViewNode::Children,
            ],
        },
        page_tree: text("Page"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
