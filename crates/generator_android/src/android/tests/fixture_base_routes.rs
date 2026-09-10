fn parity_route() -> ViewRoute {
    let input = || ViewNode::Input {
        props: VariantProps {
            variant: Some(ComponentVariant::Outlined),
            color: Some(ColorFamily::Secondary),
            ..Default::default()
        },
    };
    ViewRoute {
        id: "parity".to_string(),
        route_path: "/parity".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Flex {
                    props: LayoutProps {
                        gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                            ScaleValue::from_half_steps(4),
                        )))),
                        ..Default::default()
                    },
                    children: vec![input(), input()],
                },
                ViewNode::Grid {
                    props: GridProps {
                        columns: Some(ResponsiveValue::ordered(vec![
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Xs,
                                value: GridTracks::Count(1),
                            },
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: GridTracks::Count(2),
                            },
                        ])),
                        gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                            ScaleValue::from_half_steps(8),
                        )))),
                        ..Default::default()
                    },
                    children: vec![
                        ViewNode::Card {
                            props: VariantProps {
                                variant: Some(ComponentVariant::Solid),
                                color: Some(ColorFamily::Muted),
                                ..Default::default()
                            },
                            children: vec![text("Card")],
                        },
                        ViewNode::Card {
                            props: VariantProps {
                                variant: Some(ComponentVariant::Outlined),
                                color: Some(ColorFamily::Surface),
                                ..Default::default()
                            },
                            children: vec![text("Surface")],
                        },
                    ],
                },
                ViewNode::Button {
                    props: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    children: vec![text("Outlined")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn flex_alignment_route() -> ViewRoute {
    ViewRoute {
        id: "flex_alignment".to_string(),
        route_path: "/flex-alignment".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Flex {
            props: LayoutProps {
                direction: ResponsiveValue::ordered(vec![
                    dowe_components::ResponsiveEntry {
                        breakpoint: dowe_components::Breakpoint::Xs,
                        value: dowe_components::FlexDirection::Column,
                    },
                    dowe_components::ResponsiveEntry {
                        breakpoint: dowe_components::Breakpoint::Md,
                        value: dowe_components::FlexDirection::Row,
                    },
                ]),
                wrap: true,
                justify: Some(ResponsiveValue::scalar(Justify::End)),
                align: Some(ResponsiveValue::scalar(Align::Center)),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(6),
                )))),
                style: StyleProps {
                    sizing: dowe_components::SizingProps {
                        w: Some(ResponsiveValue::scalar(SizeValue::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            children: vec![
                ViewNode::Button {
                    props: VariantProps {
                        icon_start: Some(SideNavIcon {
                            props: SvgProps {
                                style: Default::default(),
                                view_box: SvgViewBox {
                                    min_x: "0".to_string(),
                                    min_y: "0".to_string(),
                                    width: "24".to_string(),
                                    height: "24".to_string(),
                                },
                                data: None,
                                icon_name: None,
                                icon_fallback: None,
                                icon_fill: None,
            icon_fill_binding: None,
                                icon_stroke: None,
            icon_stroke_binding: None,
                                motion: None,
                            },
                            paths: vec![SvgPath {
                                data: "M4 12h16".to_string(),
                                fill: SvgPathFill::CurrentColor,
                                transform: None,
                            }],
                        }),
                        ..Default::default()
                    },
                    children: vec![text("One")],
                },
                text("Two"),
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn flex_box_theme_route() -> ViewRoute {
    ViewRoute {
        id: "flex_box_theme".to_string(),
        route_path: "/flex-box-theme".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Flex {
            props: LayoutProps {
                justify: Some(ResponsiveValue::scalar(Justify::Between)),
                align: Some(ResponsiveValue::scalar(Align::Center)),
                style: StyleProps {
                    sizing: dowe_components::SizingProps {
                        w: Some(ResponsiveValue::scalar(SizeValue::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![
                ViewNode::Box {
                    props: Default::default(),
                    children: vec![text("Dowe Hues")],
                },
                ViewNode::SelectTheme {
                    props: ThemeSelectProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Outlined),
                            color: Some(ColorFamily::Surface),
                            ..Default::default()
                        },
                        label: "Palette".to_string(),
                        placeholder: "Choose a palette".to_string(),
                        themes: vec!["light".to_string(), "dark".to_string()],
                        default_theme: "light".to_string(),
                    },
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn flex_theme_button_route() -> ViewRoute {
    ViewRoute {
        id: "flex_theme_button".to_string(),
        route_path: "/flex-theme-button".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Flex {
            props: LayoutProps {
                align: Some(ResponsiveValue::scalar(Align::Center)),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(6),
                )))),
                ..Default::default()
            },
            children: vec![
                ViewNode::SelectTheme {
                    props: ThemeSelectProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Outlined),
                            color: Some(ColorFamily::Surface),
                            ..Default::default()
                        },
                        label: "Theme palette".to_string(),
                        placeholder: "Choose a palette".to_string(),
                        themes: vec!["light".to_string(), "dark".to_string()],
                        default_theme: "light".to_string(),
                    },
                },
                ViewNode::Button {
                    props: VariantProps {
                        variant: Some(ComponentVariant::Solid),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    children: vec![text("See it in context")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

