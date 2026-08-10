fn render_swift_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    overlays: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let docking_app_bar = app_bar
        .iter()
        .any(|node| matches!(node, ViewNode::AppBar { props, .. } if props.dock_on_scroll));
    if docking_app_bar {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}DoweDockingScaffold {{\n"));
        render_swift_scaffold_content(
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
            true,
        );
        output.push_str(&format!("{pad}}}\n"));
    } else {
        render_swift_scaffold_content(
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
            false,
        );
    }
}

fn render_swift_scaffold_content(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    overlays: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
    docking_app_bar: bool,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let persistent_app_bar = app_bar.iter().any(|node| {
        matches!(node, ViewNode::AppBar { props, .. } if props.position != BarPosition::Static)
    });
    output.push_str(&format!("{pad}VStack(spacing: CGFloat(0)) {{\n"));
    for child in app_bar {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    if persistent_app_bar {
        output.push_str(&format!("{pad}    ScrollView {{\n"));
    }
    output.push_str(&format!(
        "{pad}    HStack(alignment: .top, spacing: CGFloat(0)) {{\n"
    ));
    if !start.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
        ));
        for child in start {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!(
        "{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    for child in main {
        render_swift_node_in_flow(
            child,
            indent + 12,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}        }}\n"));
    if persistent_app_bar {
        output.push_str(&format!(
            "{pad}        .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    } else {
        output.push_str(&format!("{pad}        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"));
    }
    if !end.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
        ));
        for child in end {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}    }}\n"));
    if props.boxed && persistent_app_bar {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: CGFloat(1536), alignment: .topLeading)\n{pad}    .frame(maxWidth: .infinity, alignment: .top)\n"
        ));
    } else if props.boxed {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: CGFloat(1536), maxHeight: .infinity, alignment: .topLeading)\n{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)\n"
        ));
    } else if persistent_app_bar {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    } else {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
        ));
    }
    if persistent_app_bar {
        if docking_app_bar {
            output.push_str(&format!(
                "{pad}    .background(DoweDockingScrollObserver())\n"
            ));
        }
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
        ));
    }
    for child in bottom_bar {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    for child in overlays {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style, flow),
    );
}

fn render_swift_sidebar(
    props: &SidebarProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "{pad}VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    if !header.is_empty() {
        output.push_str(&format!(
            "{pad}    VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
        ));
        for child in header {
            render_swift_node_in_flow(
                child,
                indent + 8,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    }
    output.push_str(&format!(
        "{pad}    ScrollView {{\n{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    for child in body {
        render_swift_node_in_flow(
            child,
            indent + 12,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!(
        "{pad}        }}\n{pad}        .frame(maxWidth: .infinity, alignment: .topLeading)\n{pad}    }}\n{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
    ));
    if !footer.is_empty() {
        output.push_str(&format!(
            "{pad}    VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
        ));
        for child in footer {
            render_swift_node_in_flow(
                child,
                indent + 8,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
    let mut modifiers = swift_modifiers_for_container_style(&props.style.style, flow);
    if props.style.style.sizing.h.is_none() {
        modifiers.push(
            ".frame(maxHeight: UIScreen.main.bounds.height, alignment: .topLeading)".to_string(),
        );
        modifiers.push(".clipped()".to_string());
    }
    modifiers.push(format!(".background({})", variant_container(&props.style)));
    modifiers.push(format!(
        ".foregroundStyle({})",
        variant_content(&props.style)
    ));
    append_swift_modifiers(output, indent, &modifiers);
}

fn render_swift_side_nav(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if swift_side_nav_can_use_data_renderer(items) {
        render_swift_side_nav_data(
            props,
            items,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        return;
    }
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let wide = swift_side_nav_wide(props, context);
    let memory_key = side_nav_memory_key(props, items);
    output.push_str(&format!(
        "{pad}VStack(alignment: .leading, spacing: CGFloat(2)) {{\n"
    ));
    for (index, item) in items.iter().enumerate() {
        render_swift_side_nav_item(
            item,
            indent + 4,
            output,
            props,
            &wide,
            current_font,
            default_family,
            context,
            &format!("{memory_key}:{index}"),
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    let mut modifiers = swift_modifiers_for_container_style(&props.style.style, flow);
    if props.wide || props.reactive_wide.is_some() {
        modifiers.push(format!(
            ".frame(maxWidth: {wide} ? .infinity : nil, alignment: .leading)"
        ));
    }
    append_swift_modifiers(output, indent, &modifiers);
}

fn render_swift_rail_nav(
    props: &RailNavProps,
    items: &[RailNavItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (rail_width, item_size, icon_size, label_size) = swift_rail_nav_metrics(props.size);
    output.push_str(&format!(
        "{pad}VStack(alignment: .center, spacing: CGFloat(4)) {{\n"
    ));
    for item in items {
        match item {
            RailNavItem::Divider => output.push_str(&format!(
                "{pad}    Divider()\n{pad}        .frame(width: CGFloat({item_size}))\n{pad}        .padding(.vertical, CGFloat(6))\n"
            )),
            RailNavItem::Item(item) => {
                let action = item
                    .on_click
                    .as_deref()
                    .and_then(|name| context.action_id(name))
                    .map(|id| format!("{{ state.run(\"{}\") }}", escape_swift(id)))
                    .or_else(|| item.navigation.as_ref().map(|action| swift_navigation_action(Some(action))))
                    .unwrap_or_else(|| "{}".to_string());
                let active = swift_side_nav_active(item.navigation.as_ref());
                let border = if props.style.variant.unwrap_or(ComponentVariant::Ghost)
                    == ComponentVariant::Outlined
                {
                    variant_content(&props.style)
                } else {
                    "nil"
                };
                output.push_str(&format!(
                    "{pad}    DoweRailNavItem(label: {}, showLabel: {}, active: {active}, itemSize: CGFloat({item_size}), iconSize: CGFloat({icon_size}), labelSize: CGFloat({label_size}), backgroundColor: {}, contentColor: {}, borderColor: {border}, icon: DoweRailNavIcon(viewBox: {}, color: {}, paths: {}), action: {action})\n",
                    swift_localized_literal(&item.label, item.i18n.as_deref()),
                    props.show_labels,
                    variant_container(&props.style),
                    variant_content(&props.style),
                    swift_svg_view_box(&item.icon.props.view_box),
                    swift_svg_color(&item.icon.props.style),
                    swift_svg_paths(&item.icon.paths)
                ));
            }
        }
    }
    output.push_str(&format!("{pad}}}\n"));
    let mut modifiers = swift_modifiers_for_container_style(&props.style.style, flow);
    modifiers.push(format!(
        ".frame(width: CGFloat({rail_width}), alignment: .top)"
    ));
    modifiers.push(".padding(.vertical, CGFloat(4))".to_string());
    append_swift_modifiers(output, indent, &modifiers);
}

fn swift_rail_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (56, 40, 20, 10),
        SideNavSize::Md => (64, 48, 24, 11),
        SideNavSize::Lg => (72, 56, 28, 12),
    }
}

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
        .map(|path| reactive_text(path, "ghost"));
    let scheme = props
        .style
        .reactive
        .scheme
        .as_ref()
        .map(|path| reactive_text(path, "muted"));
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
            variant.as_deref().unwrap_or("\"ghost\""),
            scheme.as_deref().unwrap_or("\"muted\"")
        ),
    };
    let content = match (&variant, &scheme) {
        (None, None) => variant_content(&props.style).to_string(),
        _ => format!(
            "doweButtonContent({}, {})",
            variant.as_deref().unwrap_or("\"ghost\""),
            scheme.as_deref().unwrap_or("\"muted\"")
        ),
    };
    let active_content = content.clone();
    let border = if let Some(variant) = variant.as_ref() {
        format!("{variant} == \"outlined\" ? Optional({content}) : nil")
    } else if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
        format!("Optional({content})")
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweSideNav(items: {}, stateKey: \"{}\", activePath: activePath, wide: {}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), gap: CGFloat({gap}), labelFont: {}, descriptionFont: {}, backgroundColor: {}, contentColor: {}, activeContentColor: {}, borderColor: {border}, navigate: navigate)\n",
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

fn render_swift_side_nav_item(
    item: &SideNavItem,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    wide: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
    memory_key: &str,
) {
    let pad = " ".repeat(indent);
    match item {
        SideNavItem::Header(props) => render_swift_side_nav_row(
            props,
            true,
            swift_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Item(props) => render_swift_side_nav_row(
            props,
            false,
            swift_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Divider => {
            output.push_str(&format!(
                "{pad}Divider()\n{pad}    .padding(.vertical, CGFloat(8))\n"
            ));
        }
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            output.push_str(&format!(
                "{pad}DoweSideNavSubmenu(stateKey: \"{}\", open: {open}, bordered: {bordered}, wide: {wide}) {{\n",
                escape_swift(memory_key)
            ));
            for item in items {
                render_swift_side_nav_row(
                    item,
                    false,
                    swift_side_nav_action(item, context),
                    indent + 4,
                    output,
                    nav,
                    wide,
                    inherited_font,
                    default_family,
                    None,
                );
            }
            output.push_str(&format!("{pad}}} label: {{ expanded in\n"));
            render_swift_side_nav_row(
                props,
                true,
                "nil".to_string(),
                indent + 4,
                output,
                nav,
                wide,
                inherited_font,
                default_family,
                Some("expanded"),
            );
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_swift_side_nav_row(
    props: &SideNavItemProps,
    header: bool,
    action: String,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    wide: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    submenu_expanded: Option<&str>,
) {
    let pad = " ".repeat(indent);
    let active = swift_side_nav_active(props.navigation.as_ref());
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        swift_side_nav_metrics(nav.size);
    let border =
        if nav.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            format!("Optional({})", variant_content(&nav.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweSideNavRow(active: {}, wide: {}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), gap: CGFloat({gap}), backgroundColor: {}, contentColor: {}, borderColor: {border}, action: {action}) {{\n",
        active,
        wide,
        variant_container(&nav.style),
        variant_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
            swift_svg_view_box(&icon.props.view_box),
            swift_side_nav_explicit_icon_color(icon, nav, &active),
            swift_svg_paths(&icon.paths)
        ));
        append_swift_modifiers(
            output,
            indent + 4,
            &swift_modifiers_for_style(&icon.props.style),
        );
    }
    output.push_str(&format!(
        "{pad}    VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    output.push_str(&format!(
        "{pad}        Text({})\n{pad}            .font({})\n{pad}            .fontWeight({})\n",
        swift_localized_literal(&props.label, props.i18n.as_deref()),
        swift_font_value(
            inherited_font,
            &format!("CGFloat({label_size})"),
            default_family
        ),
        if header { ".semibold" } else { ".regular" }
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text({})\n{pad}            .font({})\n{pad}            .opacity(0.72)\n",
            swift_localized_literal(description, props.description_i18n.as_deref()),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
    output.push_str(&format!(
        "{pad}    }}\n{pad}    .frame(maxWidth: {wide} ? .infinity : nil, alignment: .leading)\n"
    ));
    if props.status.is_some() || submenu_expanded.is_some() {
        output.push_str(&format!("{pad}    HStack(spacing: CGFloat({gap})) {{\n"));
        if let Some(status) = props.status.as_deref() {
            output.push_str(&format!(
                "{pad}        DoweSideNavStatus(text: {}, font: {})\n",
                swift_localized_literal(status, props.status_i18n.as_deref()),
                swift_font_value(
                    inherited_font,
                    &format!("CGFloat({description_size})"),
                    default_family
                )
            ));
        }
        if let Some(expanded) = submenu_expanded {
            output.push_str(&format!(
                "{pad}        DoweSideNavArrow(expanded: {expanded})\n"
            ));
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn swift_side_nav_action(props: &SideNavItemProps, context: &SwiftReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("{{ state.run(\"{}\") }}", escape_swift(id)))
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| swift_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_swift(path))
        }
        _ => "false".to_string(),
    }
}

fn swift_side_nav_data_icon_color(icon: &SideNavIcon) -> String {
    if icon.props.style.text.is_some() {
        format!("Optional({})", swift_svg_color(&icon.props.style))
    } else {
        "nil".to_string()
    }
}

fn swift_side_nav_explicit_icon_color(
    icon: &SideNavIcon,
    nav: &SideNavProps,
    active: &str,
) -> String {
    if icon.props.style.text.is_some() {
        swift_svg_color(&icon.props.style)
    } else {
        format!(
            "{active} ? {} : DoweDesign.onBackground",
            nav_active_content(&nav.style)
        )
    }
}

fn swift_side_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 8, 12, 10),
        SideNavSize::Md => (12, 8, 10, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}

fn render_swift_bar(
    props: &BarProps,
    top: &[ViewNode],
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    bottom: &[ViewNode],
    options: SwiftBarOptions,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let content_width = if props.boxed {
        if options.boxed_regions {
            ".infinity".to_string()
        } else {
            format!("CGFloat({})", options.boxed_width)
        }
    } else {
        ".infinity".to_string()
    };
    let boxed_regions = props.boxed && options.boxed_regions;
    let content_indent = if boxed_regions { indent + 4 } else { indent };
    let content_pad = " ".repeat(content_indent);
    output.push_str(&format!("{pad}VStack(spacing: CGFloat(0)) {{\n"));
    if boxed_regions {
        output.push_str(&format!("{pad}    VStack(spacing: CGFloat(0)) {{\n"));
    }
    render_swift_bar_edge_region(
        top,
        content_indent + 4,
        output,
        current_font,
        default_family,
        context,
    );
    output.push_str(&format!("{content_pad}    ZStack {{\n"));
    output.push_str(&format!(
        "{content_pad}    HStack(alignment: .center, spacing: 0) {{\n"
    ));
    render_swift_bar_region(
        start,
        content_indent + 8,
        output,
        ".leading",
        false,
        options.start_padding,
        current_font,
        default_family,
        context,
    );
    if center.is_empty() && !start.is_empty() && !end.is_empty() {
        output.push_str(&format!(
            "{content_pad}        Spacer(minLength: CGFloat(0))\n"
        ));
    }
    render_swift_bar_region(
        center,
        content_indent + 8,
        output,
        ".center",
        true,
        options.center_padding,
        current_font,
        default_family,
        context,
    );
    render_swift_bar_region(
        end,
        content_indent + 8,
        output,
        ".trailing",
        false,
        options.end_padding,
        current_font,
        default_family,
        context,
    );
    output.push_str(&format!("{content_pad}    }}\n"));
    output.push_str(&format!(
        "{content_pad}    .frame(maxWidth: {content_width}, alignment: .center)\n"
    ));
    output.push_str(&format!("{content_pad}    }}\n"));
    render_swift_bar_edge_region(
        bottom,
        content_indent + 4,
        output,
        current_font,
        default_family,
        context,
    );
    if boxed_regions {
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: CGFloat({}), alignment: .center)\n",
            options.boxed_width
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(output, indent, &swift_modifiers_for_bar(props, flow));
}

fn render_swift_bar_edge_region(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    for child in children {
        render_swift_node_in_flow(
            child,
            indent,
            output,
            NativeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn render_swift_bottom_bar(
    props: &BarProps,
    tabs: &[BottomBarTab],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    _context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let content_width = if props.boxed {
        "CGFloat(1536)"
    } else {
        ".infinity"
    };
    output.push_str(&format!("{pad}VStack(spacing: CGFloat(0)) {{\n"));
    output.push_str(&format!(
        "{pad}    HStack(alignment: .bottom, spacing: CGFloat(4)) {{\n"
    ));
    for tab in tabs {
        let action = swift_navigation_action(Some(&tab.navigation));
        let active = swift_side_nav_active(Some(&tab.navigation));
        let size = if tab.featured { 56 } else { 48 };
        let featured = tab.featured;
        let background = if featured {
            "DoweDesign.primary"
        } else {
            variant_container(&props.style)
        };
        let content = if featured {
            "DoweDesign.onPrimary"
        } else {
            variant_content(&props.style)
        };
        let icon_color = if featured {
            "DoweDesign.onPrimary".to_string()
        } else {
            swift_svg_color(&tab.icon.props.style)
        };
        output.push_str(&format!(
            "{pad}        DoweRailNavItem(label: {}, showLabel: true, active: {active}, itemSize: CGFloat({size}), iconSize: CGFloat(20), labelSize: CGFloat(10), backgroundColor: {background}, contentColor: {content}, borderColor: nil, featured: {featured}, icon: DoweRailNavIcon(viewBox: {}, color: {icon_color}, paths: {}, animated: {}), action: {action})\n",
            swift_localized_literal(&tab.label, tab.i18n.as_deref()),
            swift_svg_view_box(&tab.icon.props.view_box),
            swift_svg_paths(&tab.icon.paths),
            tab.icon.props.is_animated()
        ));
    }
    output.push_str(&format!("{pad}    }}\n"));
    output.push_str(&format!(
        "{pad}    .frame(maxWidth: {content_width}, alignment: .center)\n"
    ));
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(output, indent, &swift_modifiers_for_bar(props, flow));
}

fn render_swift_bar_region(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    alignment: &str,
    fill: bool,
    padding: usize,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if children.is_empty() {
        return;
    }
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}HStack(alignment: .center, spacing: CGFloat({padding})) {{\n"
    ));
    for child in children {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Inline,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    output.push_str(&format!(
        "{pad}    .padding(.horizontal, CGFloat({padding}))\n"
    ));
    output.push_str(&format!(
        "{pad}    .padding(.vertical, CGFloat({padding}))\n"
    ));
    if fill {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, alignment: {alignment})\n"
        ));
    } else {
        output.push_str(&format!("{pad}    .frame(alignment: {alignment})\n"));
        output.push_str(&format!("{pad}    .lineLimit(1)\n"));
        output.push_str(&format!("{pad}    .layoutPriority(1)\n"));
        output.push_str(&format!(
            "{pad}    .fixedSize(horizontal: true, vertical: false)\n"
        ));
    }
}
