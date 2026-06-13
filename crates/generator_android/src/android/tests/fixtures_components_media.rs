fn svg_route() -> ViewRoute {
    ViewRoute {
        id: "svg".to_string(),
        route_path: "/svg".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: svg_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn motion_route() -> ViewRoute {
    ViewRoute {
        id: "motion".to_string(),
        route_path: "/motion".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                animation: Some(ViewAnimation::FadeIn),
                ..Default::default()
            },
            children: vec![ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        animation: Some(ViewAnimation::SlideUp),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Motion")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn code_route() -> ViewRoute {
    ViewRoute {
        id: "code".to_string(),
        route_path: "/code".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::code_node(
            vec![
                ComponentProp {
                    name: "language".to_string(),
                    value: PropValue::String("dowe".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
            ],
            vec![
                "page docsPage".to_string(),
                "  Card variant:\"soft\" p:4 show:true".to_string(),
                "    Text".to_string(),
                "      Documentation".to_string(),
            ],
        )
        .expect("code"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn video_route() -> ViewRoute {
    ViewRoute {
        id: "video".to_string(),
        route_path: "/video".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::video_node(vec![
            ComponentProp {
                name: "src".to_string(),
                value: PropValue::String(
                    "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string(),
                ),
            },
            ComponentProp {
                name: "poster".to_string(),
                value: PropValue::String("/images/video.jpg".to_string()),
            },
            ComponentProp {
                name: "aspect".to_string(),
                value: PropValue::String("vertical".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
        ])
        .expect("video"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn candlestick_route() -> ViewRoute {
    ViewRoute {
        id: "market".to_string(),
        route_path: "/market".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::candlestick_node(vec![
            ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("candles".to_string()),
            },
            ComponentProp {
                name: "stream".to_string(),
                value: PropValue::String("/api/candles".to_string()),
            },
            ComponentProp {
                name: "variant".to_string(),
                value: PropValue::String("soft".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
            ComponentProp {
                name: "emptyLabel".to_string(),
                value: PropValue::String("Market closed".to_string()),
            },
        ])
        .expect("candlestick"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn table_route() -> ViewRoute {
    ViewRoute {
        id: "users".to_string(),
        route_path: "/users".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::table_node(
            vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("users".to_string()),
                },
                ComponentProp {
                    name: "variant".to_string(),
                    value: PropValue::String("soft".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
                ComponentProp {
                    name: "size".to_string(),
                    value: PropValue::String("lg".to_string()),
                },
                ComponentProp {
                    name: "striped".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "bordered".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "emptyTitle".to_string(),
                    value: PropValue::String("No users".to_string()),
                },
            ],
            vec![
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("name".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Name".to_string()),
                    },
                ])
                .expect("name column"),
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("status".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Status".to_string()),
                    },
                    ComponentProp {
                        name: "align".to_string(),
                        value: PropValue::String("end".to_string()),
                    },
                    ComponentProp {
                        name: "width".to_string(),
                        value: PropValue::String("8rem".to_string()),
                    },
                ])
                .expect("status column"),
            ],
        )
        .expect("table"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn divider_route() -> ViewRoute {
    ViewRoute {
        id: "divider".to_string(),
        route_path: "/divider".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Divider {
            props: DividerProps {
                style: StyleProps::default(),
                orientation: DividerOrientation::Vertical,
                color: ColorFamily::Primary,
            },
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn svg_tree() -> ViewNode {
    ViewNode::Svg {
        props: SvgProps {
            style: StyleProps {
                text: Some(ResponsiveValue::scalar(
                    dowe_components::ColorToken::Tertiary,
                )),
                sizing: dowe_components::SizingProps {
                    w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            view_box: SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
            },
            SvgPath {
                data: "M22 12c0-5.523-4.477-10-10-10".to_string(),
                fill: SvgPathFill::CurrentColor,
            },
        ],
    }
}

fn display_overlay_route() -> ViewRoute {
    ViewRoute {
        id: "overlay".to_string(),
        route_path: "/overlay".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: display_overlay_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn display_overlay_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Avatar {
                props: AvatarProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Ada".to_string()),
                    alt: "Ada Lovelace".to_string(),
                    size: ButtonSize::Lg,
                    status: Some(AvatarStatus::Online),
                    bordered: true,
                },
                icon: None,
            },
            ViewNode::Badge {
                props: BadgeProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    text: "3".to_string(),
                    position: OverlayCornerPosition::BottomRight,
                },
                children: vec![text("Inbox")],
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Info),
                        size: Some(ButtonSize::Sm),
                        ..Default::default()
                    },
                    on_close: Some("close".to_string()),
                },
                value: "Filter".to_string(),
                start: None,
                end: None,
            },
            ViewNode::Skeleton {
                props: SkeletonProps {
                    style: StyleProps::default(),
                    variant: SkeletonVariant::Rounded,
                    animation: SkeletonAnimation::Pulse,
                },
            },
            ViewNode::Modal {
                props: ModalProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    on_close: Some("close".to_string()),
                    disable_overlay_close: false,
                    hide_close_button: false,
                },
                header: vec![text("Settings")],
                body: vec![text("Body")],
                footer: vec![text("Footer")],
            },
            ViewNode::AlertDialog {
                props: AlertDialogProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    title: "Delete?".to_string(),
                    description: "Cannot undo.".to_string(),
                    confirm_text: "Delete".to_string(),
                    cancel_text: "Cancel".to_string(),
                    on_confirm: Some("confirm".to_string()),
                    on_cancel: Some("close".to_string()),
                    loading: false,
                },
            },
            ViewNode::Tooltip {
                props: TooltipProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    label: "More actions".to_string(),
                    position: OverlayPosition::End,
                },
                children: vec![text("Hover")],
            },
            ViewNode::Toast {
                props: ToastProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    source: None,
                    kind: ToastKind::Success,
                    title: Some("Saved".to_string()),
                    description: "Profile updated".to_string(),
                    position: OverlayCornerPosition::TopRight,
                    show_icon: true,
                },
            },
            ViewNode::Dropdown {
                props: DropdownProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                },
                trigger: vec![text("Menu")],
                header: Vec::new(),
                entries: vec![OverlayEntry::Item(OverlayItemProps {
                    label: "Profile".to_string(),
                    description: None,
                    icon: None,
                    on_click: Some("profile".to_string()),
                    navigation: None,
                    disabled: false,
                })],
                footer: Vec::new(),
            },
            ViewNode::Command {
                props: CommandProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    open: Some("modal01".to_string()),
                    placeholder: "Search".to_string(),
                    empty_text: "No results".to_string(),
                    close_text: "to close".to_string(),
                    navigate_text: "Navigate".to_string(),
                    select_text: "Select".to_string(),
                    toggle_text: "Toggle".to_string(),
                    shortcut: "p".to_string(),
                    disable_global_shortcut: false,
                    show_footer: true,
                },
                entries: vec![CommandEntry::Item(OverlayItemProps {
                    label: "Home".to_string(),
                    description: None,
                    icon: None,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                    disabled: false,
                })],
            },
        ],
    }
}
