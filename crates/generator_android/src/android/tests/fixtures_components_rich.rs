fn display_chat_motion_route() -> ViewRoute {
    ViewRoute {
        id: "display".to_string(),
        route_path: "/display".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: display_chat_motion_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn rich_control_map_route() -> ViewRoute {
    ViewRoute {
        id: "richControls".to_string(),
        route_path: "/rich-controls".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: rich_control_map_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn display_chat_motion_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::AvatarGroup {
                props: AvatarGroupProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Primary),
                        variant: Some(ComponentVariant::Soft),
                        ..Default::default()
                    },
                    items: Some("people".to_string()),
                    size: ButtonSize::Sm,
                    max: Some(3),
                    auto_fit: true,
                    inline: false,
                    bordered: true,
                },
                items: vec![
                    AvatarGroupItem {
                        src: Some("/ada.png".to_string()),
                        name: Some("Ada".to_string()),
                        alt: Some("Ada Lovelace".to_string()),
                        on_click: None,
                        navigation: None,
                    },
                    AvatarGroupItem {
                        src: None,
                        name: Some("Grace".to_string()),
                        alt: Some("Grace Hopper".to_string()),
                        on_click: None,
                        navigation: None,
                    },
                ],
            },
            ViewNode::ChatBox {
                props: ChatBoxProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        variant: Some(ComponentVariant::Soft),
                        ..Default::default()
                    },
                    messages: "messages".to_string(),
                    mode: ChatBoxMode::Conversation,
                    current_user_id: "ada".to_string(),
                    user_name: "Ada".to_string(),
                    user_avatar: Some("/ada.png".to_string()),
                    user_status: "online".to_string(),
                    assistant_name: "Dowe".to_string(),
                    assistant_avatar: Some("/dowe.png".to_string()),
                    show_header: true,
                    placeholder: "Ask Dowe".to_string(),
                    show_attachments: true,
                    show_voice_note: true,
                    show_camera: true,
                    loading: Some("loading".to_string()),
                    sending: Some("sending".to_string()),
                    streaming: Some("streaming".to_string()),
                    has_more: Some("hasMore".to_string()),
                    on_send: None,
                    on_load_more: None,
                    on_stop: None,
                    on_voice_note: None,
                    on_file_attach: None,
                    on_camera_capture: None,
                },
            },
            ViewNode::Empty {
                props: EmptyProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Info),
                        variant: Some(ComponentVariant::Soft),
                        ..Default::default()
                    },
                    kind: EmptyKind::Result,
                    title: Some("Nothing found".to_string()),
                    description: Some("Try again".to_string()),
                    action_label: "Retry".to_string(),
                },
            },
            ViewNode::Marquee {
                props: MarqueeProps {
                    style: StyleProps::default(),
                    speed: MarqueeSpeed::Fast,
                    pause_on_hover: true,
                    reverse: true,
                    orientation: MarqueeOrientation::Horizontal,
                    fade: true,
                    fade_color: ColorToken::Background,
                    gap: ScaleValue::from_half_steps(8),
                },
                children: vec![text("Moving")],
            },
            ViewNode::TypeWriter {
                props: TypeWriterProps {
                    style: StyleProps::default(),
                    type_speed: 10,
                    delete_speed: 5,
                    after_typed: 20,
                    after_deleted: 10,
                    repeat: false,
                },
                items: vec![
                    TypeWriterItem {
                        text: "Hello".to_string(),
                    },
                    TypeWriterItem {
                        text: "World".to_string(),
                    },
                ],
            },
        ],
    }
}

fn rich_control_map_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::RichText {
                props: TextProps::default(),
                marks: vec![
                    RichTextMark {
                        text: "Launch".to_string(),
                        style: RichTextMarkStyle::Grad,
                        color: ColorFamily::Primary,
                    },
                    RichTextMark {
                        text: "ready".to_string(),
                        style: RichTextMarkStyle::Pill,
                        color: ColorFamily::Success,
                    },
                ],
            },
            ViewNode::Record {
                props: RecordProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    name: "voice".to_string(),
                    url: None,
                    disabled: false,
                    max_duration: Some(90),
                    on_start: None,
                    on_pause: None,
                    on_resume: None,
                    on_stop: None,
                    on_discard: None,
                    on_confirm: None,
                },
            },
            ViewNode::ToggleGroup {
                props: ToggleGroupProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Secondary),
                        ..Default::default()
                    },
                    value: Some("mode".to_string()),
                    selected: "map".to_string(),
                    size: ButtonSize::Sm,
                    wide: true,
                    vertical: false,
                    disabled: false,
                    aria_label: Some("Display mode".to_string()),
                    on_change: None,
                },
                items: vec![
                    ToggleGroupItem {
                        id: "list".to_string(),
                        label: "List".to_string(),
                        icon: None,
                    },
                    ToggleGroupItem {
                        id: "map".to_string(),
                        label: "Map".to_string(),
                        icon: None,
                    },
                ],
            },
            ViewNode::Collapsible {
                props: CollapsibleProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    label: "Details".to_string(),
                    default_open: true,
                    disabled: false,
                },
                children: vec![text("Nested content")],
            },
            ViewNode::Countdown {
                props: CountdownProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    target: "2030-01-01T00:00:00Z".to_string(),
                    show_days: true,
                    show_hours: true,
                    show_minutes: true,
                    show_seconds: true,
                    size: CountdownSize::Md,
                    days_label: "Days".to_string(),
                    hours_label: "Hours".to_string(),
                    minutes_label: "Minutes".to_string(),
                    seconds_label: "Seconds".to_string(),
                    on_complete: None,
                },
            },
            ViewNode::Map {
                props: MapProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    center_lat: "4.7109".to_string(),
                    center_lng: "-74.0721".to_string(),
                    zoom: 12,
                    height: "360px".to_string(),
                    width: "100%".to_string(),
                    show_controls: true,
                    show_scale: true,
                    show_location_control: true,
                    interactive: true,
                    route_start_lat: Some("4.7109".to_string()),
                    route_start_lng: Some("-74.0721".to_string()),
                    route_end_lat: Some("4.6500".to_string()),
                    route_end_lng: Some("-74.0900".to_string()),
                    on_location: None,
                    on_location_error: None,
                    on_route: None,
                },
                markers: vec![MapMarker {
                    id: "office".to_string(),
                    lat: "4.7109".to_string(),
                    lng: "-74.0721".to_string(),
                    label: Some("Office".to_string()),
                    popup: Some("Main office".to_string()),
                    icon: MapMarkerIcon::Start,
                    on_click: None,
                }],
                waypoints: vec![MapWaypoint {
                    lat: "4.6800".to_string(),
                    lng: "-74.0800".to_string(),
                }],
            },
        ],
    }
}
