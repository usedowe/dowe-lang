pub fn avatar_component_node(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut name = None;
    let mut alt = "Avatar".to_string();
    let mut size = ButtonSize::Md;
    let mut status = None;
    let mut bordered = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_avatar_src(&prop.name, &prop.value)?),
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "alt" => alt = parse_required_string(&prop.name, &prop.value)?,
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "status" => status = Some(parse_avatar_status(&prop.name, &prop.value)?),
            "bordered" => bordered = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Avatar)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Avatar, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Avatar, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Avatar {
        props: AvatarProps {
            style,
            src,
            name,
            alt,
            size,
            status,
            bordered,
        },
        icon,
    })
}

pub fn badge_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Badge requires at least one child",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Badge, &children, allow_children)?;
    let mut text = None;
    let mut position = OverlayCornerPosition::TopRight;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "text" => text = Some(parse_required_string(&prop.name, &prop.value)?),
            "position" => position = parse_overlay_corner_position(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Badge)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Badge, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Badge, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Badge {
        props: BadgeProps {
            style,
            text: text.ok_or_else(|| ComponentError::invalid_prop("text", "non-empty string"))?,
            position,
        },
        children,
    })
}

pub fn chip_component_node(
    props: Vec<ComponentProp>,
    value: impl AsRef<str>,
    start: Option<SideNavIcon>,
    end: Option<SideNavIcon>,
) -> ComponentResult<ViewNode> {
    let value = static_text(value, BuiltinComponent::Chip)?;
    let mut on_close = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "onClose" => on_close = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => style_props.push(prop),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Chip)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Chip, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Soft);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Chip {
        props: ChipProps { style, on_close },
        value,
        start,
        end,
    })
}

pub fn skeleton_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut variant = SkeletonVariant::Text;
    let mut animation = SkeletonAnimation::Wave;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "variant" => variant = parse_skeleton_variant(&prop.name, &prop.value)?,
            "animation" => animation = parse_skeleton_animation(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::Skeleton {
        props: SkeletonProps {
            style: parse_style_props(
                BuiltinComponent::Skeleton,
                &style_props,
                StylePropMode::Variant,
            )?,
            variant,
            animation,
        },
    })
}

pub fn modal_component_node(
    props: Vec<ComponentProp>,
    header: Vec<ViewNode>,
    body: Vec<ViewNode>,
    footer: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if body.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Modal requires body children",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Modal, &header, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Modal, &body, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Modal, &footer, allow_children)?;
    let mut open = None;
    let mut on_close = None;
    let mut disable_overlay_close = false;
    let mut hide_close_button = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => {
                open = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal bool path",
                )?)
            }
            "onClose" => on_close = Some(parse_required_string(&prop.name, &prop.value)?),
            "disableOverlayClose" => {
                disable_overlay_close = parse_static_bool(&prop.name, &prop.value)?
            }
            "hideCloseButton" => hide_close_button = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Modal)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Modal, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Modal {
        props: ModalProps {
            style,
            open: open.ok_or_else(|| ComponentError::invalid_prop("open", "signal bool path"))?,
            on_close,
            disable_overlay_close,
            hide_close_button,
        },
        header,
        body,
        footer,
    })
}

pub fn alert_dialog_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut open = None;
    let mut title = "Are you sure?".to_string();
    let mut description = "This action cannot be undone.".to_string();
    let mut confirm_text = "Confirm".to_string();
    let mut cancel_text = "Cancel".to_string();
    let mut on_confirm = None;
    let mut on_cancel = None;
    let mut loading = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => {
                open = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal bool path",
                )?)
            }
            "title" => title = parse_required_string(&prop.name, &prop.value)?,
            "description" => description = parse_required_string(&prop.name, &prop.value)?,
            "confirmText" => confirm_text = parse_required_string(&prop.name, &prop.value)?,
            "cancelText" => cancel_text = parse_required_string(&prop.name, &prop.value)?,
            "onConfirm" => on_confirm = Some(parse_required_string(&prop.name, &prop.value)?),
            "onCancel" => on_cancel = Some(parse_required_string(&prop.name, &prop.value)?),
            "loading" => loading = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::AlertDialog)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::AlertDialog, &style_props)?;
    style.color.get_or_insert(ColorFamily::Danger);
    style.variant.get_or_insert(ComponentVariant::Solid);
    Ok(ViewNode::AlertDialog {
        props: AlertDialogProps {
            style,
            open: open.ok_or_else(|| ComponentError::invalid_prop("open", "signal bool path"))?,
            title,
            description,
            confirm_text,
            cancel_text,
            on_confirm,
            on_cancel,
            loading,
        },
    })
}

pub fn tooltip_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Tooltip requires at least one child",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Tooltip, &children, allow_children)?;
    let mut label = None;
    let mut position = OverlayPosition::Top;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "position" => position = parse_overlay_position(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Tooltip)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Tooltip, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Tooltip, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Tooltip {
        props: TooltipProps {
            style,
            label: label
                .ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
            position,
        },
        children,
    })
}

pub fn toast_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut source = None;
    let mut kind = ToastKind::Info;
    let mut title = None;
    let mut description = String::new();
    let mut position = OverlayCornerPosition::BottomLeft;
    let mut show_icon = false;
    let mut style_props = Vec::new();
    let mut explicit_scheme = false;
    for prop in props {
        match prop.name.as_str() {
            "source" => {
                source = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal object path",
                )?)
            }
            "type" => kind = parse_toast_kind(&prop.name, &prop.value)?,
            "title" => title = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = parse_static_string(&prop.name, &prop.value)?,
            "position" => position = parse_overlay_corner_position(&prop.name, &prop.value)?,
            "showIcon" => show_icon = parse_static_bool(&prop.name, &prop.value)?,
            "scheme" => {
                explicit_scheme = true;
                style_props.push(prop);
            }
            "color" => return Err(scheme_prop_error(BuiltinComponent::Toast)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Toast, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Toast, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    if !explicit_scheme {
        style.color.get_or_insert(kind.color());
    }
    Ok(ViewNode::Toast {
        props: ToastProps {
            style,
            source,
            kind,
            title,
            description,
            position,
            show_icon,
        },
    })
}

pub fn dropdown_component_node(
    props: Vec<ComponentProp>,
    trigger: Vec<ViewNode>,
    header: Vec<ViewNode>,
    entries: Vec<OverlayEntry>,
    footer: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if trigger.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Dropdown requires a trigger region",
        ));
    }
    if entries.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Dropdown requires at least one item",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Dropdown, &trigger, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Dropdown, &header, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Dropdown, &footer, allow_children)?;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "variant" => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Dropdown,
                    &prop.name,
                ));
            }
            "color" => return Err(scheme_prop_error(BuiltinComponent::Dropdown)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Dropdown, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Dropdown {
        props: DropdownProps { style },
        trigger,
        header,
        entries,
        footer,
    })
}

pub fn command_component_node(
    props: Vec<ComponentProp>,
    entries: Vec<CommandEntry>,
) -> ComponentResult<ViewNode> {
    if entries.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Command requires at least one item or group",
        ));
    }
    let mut open = None;
    let mut placeholder = "Search...".to_string();
    let mut empty_text = "No results found".to_string();
    let mut close_text = "to close".to_string();
    let mut navigate_text = "Navigate".to_string();
    let mut select_text = "Select".to_string();
    let mut toggle_text = "Toggle".to_string();
    let mut shortcut = "k".to_string();
    let mut disable_global_shortcut = false;
    let mut show_footer = true;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => {
                open = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal bool path",
                )?)
            }
            "placeholder" => placeholder = parse_required_string(&prop.name, &prop.value)?,
            "emptyText" => empty_text = parse_required_string(&prop.name, &prop.value)?,
            "closeText" => close_text = parse_required_string(&prop.name, &prop.value)?,
            "navigateText" => navigate_text = parse_required_string(&prop.name, &prop.value)?,
            "selectText" => select_text = parse_required_string(&prop.name, &prop.value)?,
            "toggleText" => toggle_text = parse_required_string(&prop.name, &prop.value)?,
            "shortcut" => shortcut = parse_command_shortcut(&prop.name, &prop.value)?,
            "disableGlobalShortcut" => {
                disable_global_shortcut = parse_static_bool(&prop.name, &prop.value)?
            }
            "showFooter" => show_footer = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Command)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Command, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Command {
        props: CommandProps {
            style,
            open,
            placeholder,
            empty_text,
            close_text,
            navigate_text,
            select_text,
            toggle_text,
            shortcut,
            disable_global_shortcut,
            show_footer,
        },
        entries,
    })
}
