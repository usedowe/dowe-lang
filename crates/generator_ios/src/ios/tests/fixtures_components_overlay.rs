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
