#[test]
fn validates_display_and_overlay_component_props() {
    let avatar = super::avatar_component_node(
        vec![
            string_prop("name", "Ada"),
            string_prop("scheme", "success"),
            string_prop("variant", "solid"),
            string_prop("size", "lg"),
            string_prop("status", "online"),
            boolean_prop("bordered", true),
        ],
        None,
    )
    .expect("avatar");
    match avatar {
        ViewNode::Avatar { props, .. } => {
            assert_eq!(props.style.color, Some(ColorFamily::Success));
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.size, AvatarSize::Lg);
            assert_eq!(props.status, Some(super::AvatarStatus::Online));
            assert!(props.bordered);
        }
        _ => panic!("avatar"),
    }

    let badge = super::badge_component_node(
        vec![
            string_prop("text", "3"),
            string_prop("position", "bottom-right"),
        ],
        vec![text_node("Inbox").expect("text")],
        false,
    )
    .expect("badge");
    assert!(matches!(
        badge,
        ViewNode::Badge {
            props: super::BadgeProps {
                position: super::OverlayCornerPosition::BottomRight,
                ..
            },
            ..
        }
    ));

    let chip = super::chip_component_node(
        vec![string_prop("onClose", "close"), string_prop("size", "sm")],
        "Filter",
        None,
        None,
    )
    .expect("chip");
    assert!(matches!(
        chip,
        ViewNode::Chip {
            props: super::ChipProps {
                on_close: Some(_),
                ..
            },
            ..
        }
    ));

    let skeleton = super::skeleton_component_node(vec![
        string_prop("variant", "rounded"),
        string_prop("animation", "pulse"),
    ])
    .expect("skeleton");
    assert!(matches!(
        skeleton,
        ViewNode::Skeleton {
            props: super::SkeletonProps {
                variant: super::SkeletonVariant::Rounded,
                animation: super::SkeletonAnimation::Pulse,
                ..
            }
        }
    ));

    let modal = super::modal_component_node(
        vec![
            string_prop("bind", "open"),
            string_prop("scheme", "surface"),
        ],
        vec![text_node("Header").expect("text")],
        vec![text_node("Body").expect("text")],
        vec![text_node("Footer").expect("text")],
        false,
    )
    .expect("modal");
    assert!(matches!(
        modal,
        ViewNode::Modal {
            props: super::ModalProps { open, .. },
            header,
            body,
            footer,
        } if open == "open" && header.len() == 1 && body.len() == 1 && footer.len() == 1
    ));

    let dialog = super::alert_dialog_component_node(vec![
        string_prop("bind", "open"),
        string_prop("title", "Delete?"),
        string_prop("description", "Cannot undo."),
        string_prop("onConfirm", "confirm"),
    ])
    .expect("dialog");
    assert!(matches!(
        dialog,
        ViewNode::AlertDialog {
            props: super::AlertDialogProps {
                open,
                on_confirm: Some(_),
                ..
            },
        } if open == "open"
    ));

    let tooltip = super::tooltip_component_node(
        vec![
            string_prop("label", "More"),
            string_prop("position", "end"),
            string_prop("scheme", "muted"),
        ],
        vec![text_node("Trigger").expect("text")],
        false,
    )
    .expect("tooltip");
    assert!(matches!(
        tooltip,
        ViewNode::Tooltip {
            props: super::TooltipProps {
                position: super::OverlayPosition::End,
                ..
            },
            ..
        }
    ));

    let toast = super::toast_component_node(vec![
        string_prop("type", "success"),
        string_prop("description", "Saved"),
        string_prop("position", "top-right"),
        string_prop("variant", "outlined"),
        string_prop("scheme", "surface"),
        boolean_prop("showIcon", true),
    ])
    .expect("toast");
    assert!(matches!(
        toast,
        ViewNode::Toast {
            props: super::ToastProps {
                kind: super::ToastKind::Success,
                position: super::OverlayCornerPosition::TopRight,
                show_icon: true,
                style: super::VariantProps {
                    variant: Some(super::ComponentVariant::Outlined),
                    color: Some(super::ColorFamily::Surface),
                    ..
                },
                ..
            },
        }
    ));

    let dropdown = super::dropdown_component_node(
        vec![string_prop("scheme", "surface")],
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        vec![super::OverlayEntry::Item(
            super::overlay_item_component(
                BuiltinComponent::Dropdown,
                vec![string_prop("label", "Profile")],
                None,
            )
            .expect("item"),
        )],
        Vec::new(),
        false,
    )
    .expect("dropdown");
    assert!(matches!(dropdown, ViewNode::Dropdown { entries, .. } if entries.len() == 1));

    let command = super::command_component_node(
        vec![string_prop("bind", "open"), string_prop("shortcut", "p")],
        vec![super::CommandEntry::Item(
            super::overlay_item_component(
                BuiltinComponent::Command,
                vec![string_prop("label", "Home")],
                None,
            )
            .expect("item"),
        )],
    )
    .expect("command");
    assert!(matches!(
        command,
        ViewNode::Command {
            props: super::CommandProps {
                open: Some(open),
                shortcut,
                ..
            },
            ..
        } if open == "open" && shortcut == "p"
    ));

    let error = super::avatar_component_node(vec![string_prop("color", "primary")], None)
        .expect_err("color");
    assert_eq!(
        error,
        ComponentError::new("unknown prop `color` on `Avatar`; use `scheme` for visual family")
    );
}

#[test]
fn rejects_invalid_design_props() {
    let error = container_component_node(
        BuiltinComponent::Flex,
        vec![number_prop("py", 13)],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect_err("error");

    assert_eq!(
        error,
        ComponentError::invalid_prop("py", "Dowe scale value from 0 to 96")
    );

    let error = input_node(vec![string_prop("scheme", "primaryText")]).expect_err("error");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "scheme",
            "primary, secondary, accent, muted, success, info, warning or danger"
        )
    );

    let error = input_node(vec![string_prop("color", "primary")]).expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Input, "color")
    );

    let card = container_component_node(
        BuiltinComponent::Card,
        vec![string_prop("color", "white")],
        Vec::new(),
        false,
    )
    .expect("Card color override");
    let ViewNode::Card { props, .. } = card else {
        panic!("expected Card");
    };
    assert_eq!(props.style.text.expect("Card color").entries[0].value, ColorToken::White);

    let error = container_component_node(
        BuiltinComponent::Alert,
        vec![
            string_prop("type", "success"),
            string_prop("message", "Saved"),
            string_prop("color", "primary"),
        ],
        Vec::new(),
        false,
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Alert, "color")
    );

    let error = text_component_node(
        BuiltinComponent::Text,
        vec![string_prop("font", "Inter")],
        "Hello",
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "font",
            "system, inter, roboto, montserrat, lato, poppins, manrope, quicksand, lora, syne, jost or puritan"
        )
    );
}
