fn render_swift_side_nav_data(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let reactive_text = |path: &str, fallback: &str| {
        format!(
            "state.text(\"{}\", fallback: \"{fallback}\")",
            escape_swift(&context.signal_path(path))
        )
    };
    let variant = props
        .style
        .reactive
        .variant
        .as_ref()
        .map(|path| reactive_text(path, "solid"));
    let scheme = props
        .style
        .reactive
        .scheme
        .as_ref()
        .map(|path| reactive_text(path, "primary"));
    let size = props
        .style
        .reactive
        .size
        .as_ref()
        .map(|path| reactive_text(path, "md"));
    let wide = swift_side_nav_wide(props, context);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        if let Some(size) = size.as_ref() {
            (
                format!("doweSideNavMetric({size}, small: 8, medium: 12, large: 16)"),
                format!("doweSideNavMetric({size}, small: 6, medium: 8, large: 12)"),
                format!("doweSideNavMetric({size}, small: 6, medium: 8, large: 12)"),
                format!("doweSideNavMetric({size}, small: 12, medium: 14, large: 16)"),
                format!("doweSideNavMetric({size}, small: 10, medium: 12, large: 14)"),
            )
        } else {
            let values = swift_side_nav_metrics(props.size);
            (
                values.0.to_string(),
                values.1.to_string(),
                values.2.to_string(),
                values.3.to_string(),
                values.4.to_string(),
            )
        };
    let container = match (&variant, &scheme) {
        (None, None) => variant_container(&props.style).to_string(),
        _ => format!(
            "doweButtonContainer({}, {})",
            variant.as_deref().unwrap_or("\"solid\""),
            scheme.as_deref().unwrap_or("\"primary\"")
        ),
    };
    let content = match (&variant, &scheme) {
        (None, None) => variant_content(&props.style).to_string(),
        _ => format!(
            "doweButtonContent({}, {})",
            variant.as_deref().unwrap_or("\"solid\""),
            scheme.as_deref().unwrap_or("\"primary\"")
        ),
    };
    let active_content = content.clone();
    let title = match (&variant, &scheme) {
        (None, None) => side_nav_header_content(&props.style).to_string(),
        _ => format!(
            "doweSideNavHeaderColor({})",
            props
                .style
                .reactive
                .scheme
                .as_ref()
                .map(|path| reactive_text(path, "muted"))
                .unwrap_or_else(|| "\"muted\"".to_string())
        ),
    };
    let border = if let Some(variant) = variant.as_ref() {
        format!("{variant} == \"outlined\" ? Optional({content}) : nil")
    } else if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
        format!("Optional({content})")
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweSideNav(items: {}, stateKey: \"{}\", activePath: activePath, wide: {}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), gap: CGFloat({gap}), labelFont: {}, descriptionFont: {}, backgroundColor: {}, contentColor: {}, titleColor: {}, activeContentColor: {}, borderColor: {border}, navigate: navigate)\n",
        swift_side_nav_entries(items, indent),
        escape_swift(&side_nav_memory_key(props, items)),
        wide,
        swift_font_value(
            current_font,
            &format!("CGFloat({label_size})"),
            default_family
        ),
        swift_font_value(
            current_font,
            &format!("CGFloat({description_size})"),
            default_family
        ),
        container,
        content,
        title,
        active_content,
    ));
    let mut modifiers = swift_modifiers_for_container_style(&props.style.style, flow);
    if props.wide || props.reactive_wide.is_some() {
        modifiers.push(format!(
            ".frame(maxWidth: {wide} ? .infinity : nil, alignment: .leading)"
        ));
    }
    append_swift_modifiers(output, indent, &modifiers);
}

fn swift_side_nav_wide(props: &SideNavProps, context: &SwiftReactiveContext) -> String {
    props
        .reactive_wide
        .as_ref()
        .map(|path| {
            format!(
                "state.bool(\"{}\", fallback: false)",
                escape_swift(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| props.wide.to_string())
}

fn swift_side_nav_can_use_data_renderer(items: &[SideNavItem]) -> bool {
    items.iter().all(|item| match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => {
            swift_side_nav_item_can_use_data_renderer(props, true)
        }
        SideNavItem::Divider => true,
        SideNavItem::Submenu { props, items, .. } => {
            swift_side_nav_item_can_use_data_renderer(props, false)
                && items
                    .iter()
                    .all(|item| swift_side_nav_item_can_use_data_renderer(item, true))
        }
    })
}

fn swift_side_nav_item_can_use_data_renderer(
    props: &SideNavItemProps,
    allow_navigation: bool,
) -> bool {
    props
        .icon
        .as_ref()
        .is_none_or(swift_side_nav_icon_can_use_data_renderer)
        && props.on_click.is_none()
        && (allow_navigation || props.navigation.is_none())
        && props
            .navigation
            .as_ref()
            .is_none_or(swift_side_nav_navigation_supported)
}

fn swift_side_nav_icon_can_use_data_renderer(icon: &SideNavIcon) -> bool {
    let supported = StyleProps {
        text: icon.props.style.text.clone(),
        sizing: icon.props.style.sizing.clone(),
        ..Default::default()
    };
    icon.props.style == supported
}

fn swift_side_nav_navigation_supported(action: &NavigationAction) -> bool {
    matches!(
        action,
        NavigationAction::Internal { .. } | NavigationAction::Section { .. }
    )
}

fn swift_side_nav_entries(items: &[SideNavItem], indent: usize) -> String {
    swift_side_nav_entries_with_prefix(items, indent, "item")
}

fn swift_side_nav_entries_with_prefix(
    items: &[SideNavItem],
    indent: usize,
    prefix: &str,
) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let pad = " ".repeat(indent);
    let item_pad = " ".repeat(indent + 4);
    let mut output = "[\n".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("{prefix}-{index}");
        output.push_str(&format!(
            "{item_pad}{},\n",
            swift_side_nav_entry(item, indent + 4, &id)
        ));
    }
    output.push_str(&format!("{pad}]"));
    output
}

fn swift_side_nav_child_entries(items: &[SideNavItemProps], indent: usize, prefix: &str) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let pad = " ".repeat(indent);
    let item_pad = " ".repeat(indent + 4);
    let mut output = "[\n".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("{prefix}-{index}");
        output.push_str(&format!(
            "{item_pad}{},\n",
            swift_side_nav_entry_props("item", item, false, false, "", &id)
        ));
    }
    output.push_str(&format!("{pad}]"));
    output
}

fn swift_side_nav_entry(item: &SideNavItem, indent: usize, id: &str) -> String {
    match item {
        SideNavItem::Header(props) => {
            swift_side_nav_entry_props("header", props, false, false, "", id)
        }
        SideNavItem::Item(props) => swift_side_nav_entry_props("item", props, false, false, "", id),
        SideNavItem::Divider => format!(
            "DoweSideNavEntry(id: \"{}\", kind: \"divider\", label: \"\", description: nil, status: nil, icon: nil, operation: nil, path: nil, fragment: nil, open: false, bordered: false, children: [])",
            escape_swift(id)
        ),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            let children = swift_side_nav_child_entries(items, indent + 4, id);
            swift_side_nav_entry_props("submenu", props, *open, *bordered, &children, id)
        }
    }
}

fn swift_side_nav_entry_props(
    kind: &str,
    props: &SideNavItemProps,
    open: bool,
    bordered: bool,
    children: &str,
    id: &str,
) -> String {
    let (operation, path, fragment) = swift_side_nav_navigation_values(props.navigation.as_ref());
    let children = if children.is_empty() { "[]" } else { children };
    format!(
        "DoweSideNavEntry(id: \"{}\", kind: \"{}\", label: {}, description: {}, status: {}, icon: {}, operation: {}, path: {}, fragment: {}, open: {}, bordered: {}, children: {})",
        escape_swift(id),
        kind,
        swift_localized_literal(&props.label, props.i18n.as_deref()),
        props
            .description
            .as_deref()
            .map(|value| swift_localized_literal(value, props.description_i18n.as_deref()))
            .unwrap_or_else(|| "nil".to_string()),
        props
            .status
            .as_deref()
            .map(|value| swift_localized_literal(value, props.status_i18n.as_deref()))
            .unwrap_or_else(|| "nil".to_string()),
        swift_side_nav_data_icon(props.icon.as_ref()),
        swift_side_nav_optional_string(operation),
        swift_side_nav_optional_string(path),
        swift_side_nav_optional_string(fragment),
        open,
        bordered,
        children
    )
}

fn swift_side_nav_data_icon(icon: Option<&SideNavIcon>) -> String {
    let Some(icon) = icon else {
        return "nil".to_string();
    };
    let sizing = &icon.props.style.sizing;
    let width = sizing
        .w
        .as_ref()
        .map(|value| format!("doweFixedSize({})", swift_size_value(value)))
        .unwrap_or_else(|| "nil".to_string());
    let max_width = sizing
        .w
        .as_ref()
        .map(|value| format!("doweMaxSize({})", swift_size_value(value)))
        .unwrap_or_else(|| "nil".to_string());
    let height = sizing
        .h
        .as_ref()
        .map(|value| {
            format!(
                "doweFixedSize({}, viewportHeight: viewportHeight)",
                swift_size_value(value)
            )
        })
        .unwrap_or_else(|| "nil".to_string());
    let max_height = sizing
        .h
        .as_ref()
        .map(|value| format!("doweMaxSize({})", swift_size_value(value)))
        .unwrap_or_else(|| "nil".to_string());
    let min_width = sizing
        .min_w
        .as_ref()
        .map(|value| format!("doweFixedSize({})", swift_size_value(value)))
        .unwrap_or_else(|| "nil".to_string());
    let min_height = sizing
        .min_h
        .as_ref()
        .map(|value| {
            format!(
                "doweFixedSize({}, viewportHeight: viewportHeight)",
                swift_size_value(value)
            )
        })
        .unwrap_or_else(|| "nil".to_string());
    format!(
        "DoweSideNavIcon(viewBox: {}, color: {}, paths: {}, width: {width}, maxWidth: {max_width}, height: {height}, maxHeight: {max_height}, minWidth: {min_width}, minHeight: {min_height})",
        swift_svg_view_box(&icon.props.view_box),
        swift_side_nav_data_icon_color(icon),
        swift_svg_paths(&icon.paths),
    )
}

fn swift_side_nav_navigation_values(
    action: Option<&NavigationAction>,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    match action {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => (
            Some(operation.as_str()),
            Some(path.as_str()),
            fragment.as_deref(),
        ),
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => (Some(operation.as_str()), Some(""), Some(fragment.as_str())),
        _ => (None, None, None),
    }
}

fn swift_side_nav_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_swift(value)))
        .unwrap_or_else(|| "nil".to_string())
}

