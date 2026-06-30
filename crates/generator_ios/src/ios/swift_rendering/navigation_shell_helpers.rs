fn render_swift_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
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
    output.push_str(&format!("{pad}        }}\n{pad}        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"));
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
    output.push_str(&format!("{pad}    }}\n{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"));
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
    modifiers.push(format!(".foregroundStyle({})", variant_content(&props.style)));
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
        );
        return;
    }
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "{pad}VStack(alignment: .leading, spacing: CGFloat(2)) {{\n"
    ));
    for item in items {
        render_swift_side_nav_item(
            item,
            indent + 4,
            output,
            props,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style.style, flow),
    );
}

fn render_swift_side_nav_data(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        swift_side_nav_metrics(props.size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            format!("Optional({})", variant_content(&props.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweSideNav(items: {}, activePath: activePath, wide: {}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), gap: CGFloat({gap}), labelFont: {}, descriptionFont: {}, backgroundColor: {}, contentColor: {}, activeContentColor: {}, borderColor: {border}, navigate: navigate)\n",
        swift_side_nav_entries(items, indent),
        props.wide,
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
        variant_container(&props.style),
        variant_content(&props.style),
        nav_active_content(&props.style),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style.style, flow),
    );
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
    props.icon.is_none()
        && props.on_click.is_none()
        && (allow_navigation || props.navigation.is_none())
        && props
            .navigation
            .as_ref()
            .is_none_or(swift_side_nav_navigation_supported)
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

fn swift_side_nav_child_entries(
    items: &[SideNavItemProps],
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
            "DoweSideNavEntry(id: \"{}\", kind: \"divider\", label: \"\", description: nil, status: nil, operation: nil, path: nil, fragment: nil, open: false, bordered: false, children: [])",
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
        "DoweSideNavEntry(id: \"{}\", kind: \"{}\", label: \"{}\", description: {}, status: {}, operation: {}, path: {}, fragment: {}, open: {}, bordered: {}, children: {})",
        escape_swift(id),
        kind,
        escape_swift(&props.label),
        swift_side_nav_optional_string(props.description.as_deref()),
        swift_side_nav_optional_string(props.status.as_deref()),
        swift_side_nav_optional_string(operation),
        swift_side_nav_optional_string(path),
        swift_side_nav_optional_string(fragment),
        open,
        bordered,
        children
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
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
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
            output.push_str(&format!("{pad}DoweSideNavSubmenu(open: {open}, bordered: {bordered}) {{\n"));
            for item in items {
                render_swift_side_nav_row(
                    item,
                    false,
                    swift_side_nav_action(item, context),
                    indent + 4,
                    output,
                    nav,
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
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    submenu_expanded: Option<&str>,
) {
    let pad = " ".repeat(indent);
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
        swift_side_nav_active(props.navigation.as_ref()),
        nav.wide,
        variant_container(&nav.style),
        variant_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
            swift_svg_view_box(&icon.props.view_box),
            swift_side_nav_icon_color(icon, nav),
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
        "{pad}        Text(\"{}\")\n{pad}            .font({})\n{pad}            .fontWeight({})\n",
        escape_swift(&props.label),
        swift_font_value(
            inherited_font,
            &format!("CGFloat({label_size})"),
            default_family
        ),
        if header { ".semibold" } else { ".regular" }
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text(\"{}\")\n{pad}            .font({})\n{pad}            .opacity(0.72)\n",
            escape_swift(description),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
    output.push_str(&format!(
        "{pad}    }}\n{pad}    .frame(maxWidth: .infinity, alignment: .leading)\n"
    ));
    if let Some(status) = props.status.as_deref() {
        output.push_str(&format!(
            "{pad}    Text(\"{}\")\n{pad}        .font({})\n{pad}        .fontWeight(.semibold)\n",
            escape_swift(status),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
    if let Some(expanded) = submenu_expanded {
        output.push_str(&format!(
            "{pad}    DoweSideNavArrow(expanded: {expanded})\n"
        ));
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

fn swift_side_nav_icon_color(icon: &SideNavIcon, nav: &SideNavProps) -> String {
    if icon.props.style.text.is_some() {
        swift_svg_color(&icon.props.style)
    } else {
        nav_active_content(&nav.style).to_string()
    }
}

fn swift_side_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 6, 12, 10),
        SideNavSize::Md => (12, 8, 8, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}

fn render_swift_bar(
    props: &BarProps,
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
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
        "CGFloat(1152)"
    } else {
        ".infinity"
    };
    output.push_str(&format!("{pad}ZStack {{\n"));
    output.push_str(&format!(
        "{pad}    HStack(alignment: .center, spacing: 0) {{\n"
    ));
    render_swift_bar_region(
        start,
        indent + 8,
        output,
        ".leading",
        false,
        options.start_padding,
        current_font,
        default_family,
        context,
    );
    if center.is_empty() && !start.is_empty() && !end.is_empty() {
        output.push_str(&format!("{pad}        Spacer(minLength: CGFloat(0))\n"));
    }
    render_swift_bar_region(
        center,
        indent + 8,
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
        indent + 8,
        output,
        ".trailing",
        false,
        options.end_padding,
        current_font,
        default_family,
        context,
    );
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
    }
}
