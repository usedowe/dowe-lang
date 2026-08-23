fn record_classes(props: &RecordProps) -> Vec<String> {
    let mut classes = variant_classes("media", &props.style);
    classes.push("record".to_string());
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    classes
}

fn toggle_group_classes(props: &ToggleGroupProps) -> Vec<String> {
    let mut classes = variant_classes("toggle-group", &props.style);
    if props.kind == ToggleGroupKind::Pagination {
        classes.push("pagination".to_string());
    }
    classes.push(format!("toggle-group-{}", props.size.as_str()));
    if props.wide {
        classes.push("is-wide".to_string());
    }
    if props.vertical {
        classes.push("is-vertical".to_string());
    }
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    classes
}

fn collapsible_classes(props: &CollapsibleProps) -> Vec<String> {
    let mut classes = variant_classes("collapsible", &props.style);
    if props.default_open {
        classes.push("is-open".to_string());
    }
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    classes
}

fn countdown_classes(props: &CountdownProps) -> Vec<String> {
    let mut classes = variant_classes("countdown", &props.style);
    classes.push(format!("countdown-{}", props.size.as_str()));
    classes
}

fn map_classes(props: &MapProps) -> Vec<String> {
    let mut classes = variant_classes("map", &props.style);
    if !props.interactive {
        classes.push("is-static".to_string());
    }
    classes
}

fn badge_classes(props: &BadgeProps) -> Vec<String> {
    let mut classes = vec!["badge".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    classes.push(format!("is-{}", props.position.as_str()));
    classes
}

fn badge_content_classes(props: &BadgeProps) -> Vec<String> {
    vec![
        "badge-content".to_string(),
        format!(
            "is-{}",
            props
                .style
                .variant
                .unwrap_or(ComponentVariant::Solid)
                .as_str()
        ),
        format!(
            "is-{}",
            props.style.color.unwrap_or(ColorFamily::Primary).as_str()
        ),
    ]
}

fn chip_classes(props: &ChipProps) -> Vec<String> {
    let mut classes = variant_classes("chip", &props.style);
    classes.push(format!(
        "chip-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.on_close.is_some() {
        classes.push("has-close".to_string());
    }
    classes
}

fn skeleton_classes(props: &SkeletonProps) -> Vec<String> {
    let mut classes = vec![
        "skeleton".to_string(),
        format!("is-{}", props.variant.as_str()),
        format!("is-{}", props.animation.as_str()),
    ];
    append_style_classes(&mut classes, &props.style);
    classes
}

fn modal_panel_classes(props: &ModalProps) -> Vec<String> {
    let mut classes = vec!["modal-dialog".to_string()];
    append_show_classes(&mut classes, props.style.element.show.as_ref());
    classes
}

fn modal_classes(props: &ModalProps) -> Vec<String> {
    variant_classes("modal", &props.style)
}

fn alert_dialog_modal_props(props: &AlertDialogProps) -> ModalProps {
    let mut style = props.style.clone();
    style.color = Some(ColorFamily::Surface);
    ModalProps {
        style,
        open: props.open.clone(),
        on_close: props.on_cancel.clone(),
        disable_overlay_close: true,
        hide_close_button: true,
    }
}

fn tooltip_classes(props: &TooltipProps) -> Vec<String> {
    let mut classes = vec!["tooltip".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    classes
}

fn tooltip_popover_classes(props: &TooltipProps) -> Vec<String> {
    vec![
        "tooltip-popover".to_string(),
        format!(
            "is-{}",
            props
                .style
                .variant
                .unwrap_or(ComponentVariant::Solid)
                .as_str()
        ),
        format!(
            "is-{}",
            props.style.color.unwrap_or(ColorFamily::Muted).as_str()
        ),
        format!("position-{}", props.position.as_str()),
    ]
}

fn toast_classes(props: &ToastProps) -> Vec<String> {
    let mut classes = variant_classes("toast", &props.style);
    classes.push(format!("is-{}", props.position.as_str()));
    classes
}

fn dropdown_classes(props: &DropdownProps) -> Vec<String> {
    let mut classes = vec!["dropdown".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    classes
}

fn dropdown_popover_classes(props: &DropdownProps) -> Vec<String> {
    vec![
        "dropdown-popover".to_string(),
        format!(
            "is-{}",
            props
                .style
                .variant
                .unwrap_or(ComponentVariant::Solid)
                .as_str()
        ),
        format!(
            "is-{}",
            props.style.color.unwrap_or(ColorFamily::Primary).as_str()
        ),
    ]
}

fn command_panel_classes(props: &CommandProps) -> Vec<String> {
    let mut classes = vec!["command-dialog".to_string()];
    append_show_classes(&mut classes, props.style.element.show.as_ref());
    classes
}

fn command_classes(props: &CommandProps) -> Vec<String> {
    variant_classes("command", &props.style)
}

