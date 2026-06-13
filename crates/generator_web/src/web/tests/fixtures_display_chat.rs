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
