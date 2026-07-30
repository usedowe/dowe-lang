fn render_compose_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    overlays: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let persistent_app_bar = app_bar.iter().any(|node| {
        matches!(node, ViewNode::AppBar { props, .. } if props.position != BarPosition::Static)
    });
    output.push_str(&format!(
        "{pad}Column(modifier = {}) {{\n",
        modifier_for_container_style(&props.style, flow)
    ));
    let color_scope = compose_content_color(&props.style);
    if let Some(color) = color_scope.as_ref() {
        output.push_str(&format!(
            "{pad}    CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
        ));
    }
    let content_indent = indent + if color_scope.is_some() { 8 } else { 4 };
    let content_pad = " ".repeat(content_indent);
    for child in app_bar {
        render_compose_node_in_flow(
            child,
            content_indent,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    let body_modifier = if props.boxed {
        "Modifier.fillMaxWidth().weight(1f)"
    } else if persistent_app_bar {
        "Modifier.fillMaxWidth().weight(1f).verticalScroll(scrollState)"
    } else {
        "Modifier.fillMaxWidth().weight(1f)"
    };
    if props.boxed {
        output.push_str(&format!(
            "{content_pad}Box(modifier = {body_modifier}, contentAlignment = Alignment.TopCenter) {{\n"
        ));
    }
    let row_indent = if props.boxed { content_indent + 4 } else { content_indent };
    let row_pad = " ".repeat(row_indent);
    let row_modifier = if props.boxed && persistent_app_bar {
        "Modifier.widthIn(max = 1536.dp).fillMaxSize().verticalScroll(scrollState)"
    } else if props.boxed {
        "Modifier.widthIn(max = 1536.dp).fillMaxSize()"
    } else {
        body_modifier
    };
    output.push_str(&format!("{row_pad}Row(modifier = {row_modifier}) {{\n"));
    if !start.is_empty() {
        output.push_str(&format!("{row_pad}    Column {{\n"));
        for child in start {
            render_compose_node_in_flow(
                child,
                row_indent + 8,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{row_pad}    }}\n"));
    }
    output.push_str(&format!(
        "{row_pad}    Column(modifier = Modifier.weight(1f)) {{\n"
    ));
    for child in main {
        render_compose_node_in_flow(
            child,
            row_indent + 8,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{row_pad}    }}\n"));
    if !end.is_empty() {
        output.push_str(&format!("{row_pad}    Column {{\n"));
        for child in end {
            render_compose_node_in_flow(
                child,
                row_indent + 8,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{row_pad}    }}\n"));
    }
    output.push_str(&format!("{row_pad}}}\n"));
    if props.boxed {
        output.push_str(&format!("{content_pad}}}\n"));
    }
    for child in bottom_bar {
        render_compose_node_in_flow(
            child,
            content_indent,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    for child in overlays {
        render_compose_node_in_flow(
            child,
            content_indent,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    if color_scope.is_some() {
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_sidebar(
    props: &SidebarProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let mut modifier = modifier_for_container_style(&props.style.style, flow);
    if props.style.style.sizing.h.is_none() {
        modifier.push_str(".heightIn(max = LocalConfiguration.current.screenHeightDp.dp)");
    }
    let modifier = format!("{}.background({})", modifier, variant_container(&props.style));
    output.push_str(&format!("{pad}Column(modifier = {modifier}) {{\n"));
    output.push_str(&format!(
        "{pad}    CompositionLocalProvider(LocalContentColor provides {}) {{\n",
        variant_content(&props.style)
    ));
    if !header.is_empty() {
        output.push_str(&format!(
            "{pad}        Column(modifier = Modifier.fillMaxWidth()) {{\n"
        ));
        for child in header {
            render_compose_node_in_flow(
                child,
                indent + 12,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!(
        "{pad}        Column(modifier = Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState())) {{\n"
    ));
    for child in body {
        render_compose_node_in_flow(
            child,
            indent + 12,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}        }}\n"));
    if !footer.is_empty() {
        output.push_str(&format!(
            "{pad}        Column(modifier = Modifier.fillMaxWidth()) {{\n"
        ));
        for child in footer {
            render_compose_node_in_flow(
                child,
                indent + 12,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}    }}\n"));
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_side_nav(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    if compose_side_nav_can_use_data_renderer(items) {
        render_compose_side_nav_data(
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
    let wide = compose_side_nav_wide(props, context);
    let modifier = compose_side_nav_modifier(props, flow, &wide);
    output.push_str(&format!(
        "{pad}Column(modifier = {}, verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n",
        modifier
    ));
    for item in items {
        render_compose_side_nav_item(
            item,
            indent + 4,
            output,
            props,
            &wide,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_rail_nav(
    props: &RailNavProps,
    items: &[RailNavItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (rail_width, item_size, icon_size, label_size) = compose_rail_nav_metrics(props.size);
    let modifier = modifier_for_container_style(&props.style.style, flow);
    output.push_str(&format!(
        "{pad}Column(modifier = {modifier}.width({rail_width}.dp).padding(vertical = 4.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(4.dp)) {{\n"
    ));
    for item in items {
        match item {
            RailNavItem::Divider => output.push_str(&format!(
                "{pad}    Box(modifier = Modifier.width({item_size}.dp).padding(vertical = 6.dp).height(1.dp).background(DoweDesign.muted))\n"
            )),
            RailNavItem::Item(item) => {
                let action = item
                    .on_click
                    .as_deref()
                    .and_then(|name| context.action_id(name))
                    .map(|id| format!("{{ actionScope.launch {{ state.run(\"{}\") }} }}", escape_kotlin(id)))
                    .or_else(|| item.navigation.as_ref().map(|action| compose_navigation_action(Some(action))))
                    .unwrap_or_else(|| "null".to_string());
                let active = compose_side_nav_active(item.navigation.as_ref());
                let border = if props.style.variant.unwrap_or(ComponentVariant::Ghost)
                    == ComponentVariant::Outlined
                {
                    variant_content(&props.style)
                } else {
                    "null"
                };
                output.push_str(&format!(
                    "{pad}    DoweRailNavItem(label = {}, showLabel = {}, active = {active}, itemSize = {item_size}.dp, labelSize = {label_size}f, backgroundColor = {}, contentColor = {}, borderColor = {border}, onClick = {action}) {{\n",
                    compose_localized_literal(&item.label, item.i18n.as_deref()),
                    props.show_labels,
                    variant_container(&props.style),
                    variant_content(&props.style),
                ));
                output.push_str(&format!(
                    "{pad}        DoweSvg(viewBox = {}, modifier = Modifier.size({icon_size}.dp), color = {}, paths = {})\n",
                    compose_svg_view_box(&item.icon.props.view_box),
                    compose_svg_color(&item.icon.props.style),
                    compose_svg_paths(&item.icon.paths)
                ));
                output.push_str(&format!("{pad}    }}\n"));
            }
        }
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_rail_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (56, 40, 20, 10),
        SideNavSize::Md => (64, 48, 24, 11),
        SideNavSize::Lg => (72, 56, 28, 12),
    }
}

fn render_compose_side_nav_data(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let reactive_text = |path: &str, fallback: &str| {
        format!(
            "state.text(\"{}\", \"{fallback}\")",
            escape_kotlin(&context.signal_path(path))
        )
    };
    let variant = props.style.reactive.variant.as_ref().map(|path| reactive_text(path, "ghost"));
    let scheme = props.style.reactive.scheme.as_ref().map(|path| reactive_text(path, "muted"));
    let size = props.style.reactive.size.as_ref().map(|path| reactive_text(path, "md"));
    let wide = compose_side_nav_wide(props, context);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) = if let Some(size) = size.as_ref() {
        (
            format!("doweSideNavMetric({size}, 8, 12, 16)"),
            format!("doweSideNavMetric({size}, 6, 8, 12)"),
            format!("doweSideNavMetric({size}, 6, 8, 12)"),
            format!("doweSideNavMetric({size}, 12, 14, 16)"),
            format!("doweSideNavMetric({size}, 10, 12, 14)"),
        )
    } else {
        let values = compose_side_nav_metrics(props.size);
        (values.0.to_string(), values.1.to_string(), values.2.to_string(), values.3.to_string(), values.4.to_string())
    };
    let container = match (&variant, &scheme) { (None, None) => variant_container(&props.style).to_string(), _ => format!("doweButtonContainer({}, {})", variant.as_deref().unwrap_or("\"ghost\""), scheme.as_deref().unwrap_or("\"muted\"")) };
    let content = match (&variant, &scheme) { (None, None) => variant_content(&props.style).to_string(), _ => format!("doweButtonContent({}, {})", variant.as_deref().unwrap_or("\"ghost\""), scheme.as_deref().unwrap_or("\"muted\"")) };
    let active_content = content.clone();
    let border = if let Some(variant) = variant.as_ref() { format!("if ({variant} == \"outlined\") {content} else null") } else if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined { content.clone() } else { "null".to_string() };
    let modifier = compose_side_nav_modifier(props, flow, &wide);
    output.push_str(&format!(
        "{pad}DoweSideNav(items = {}, modifier = {}, activePath = activePath, wide = {}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, gap = {gap}.dp, labelSize = {label_size}f, descriptionSize = {description_size}f, fontFamily = {}, backgroundColor = {}, contentColor = {}, activeContentColor = {}, borderColor = {border}, navigate = navigate)\n",
        compose_side_nav_entries(items, indent),
        modifier,
        wide,
        compose_font_value(current_font, default_family),
        container,
        content,
        active_content,
    ));
}

fn compose_side_nav_wide(props: &SideNavProps, context: &ComposeReactiveContext) -> String {
    props
        .reactive_wide
        .as_ref()
        .map(|path| {
            format!(
                "state.bool(\"{}\", false)",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| props.wide.to_string())
}

fn compose_side_nav_modifier(props: &SideNavProps, flow: ComposeFlow, wide: &str) -> String {
    let modifier = modifier_for_container_style(&props.style.style, flow);
    format!("{modifier}.then(if ({wide}) Modifier.fillMaxWidth() else Modifier)")
}

fn compose_side_nav_can_use_data_renderer(items: &[SideNavItem]) -> bool {
    items.iter().all(|item| match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => {
            compose_side_nav_item_can_use_data_renderer(props, true)
        }
        SideNavItem::Divider => true,
        SideNavItem::Submenu { props, items, .. } => {
            compose_side_nav_item_can_use_data_renderer(props, false)
                && items
                    .iter()
                    .all(|item| compose_side_nav_item_can_use_data_renderer(item, true))
        }
    })
}

fn compose_side_nav_item_can_use_data_renderer(
    props: &SideNavItemProps,
    allow_navigation: bool,
) -> bool {
    props.icon.is_none()
        && props.on_click.is_none()
        && (allow_navigation || props.navigation.is_none())
        && props
            .navigation
            .as_ref()
            .is_none_or(compose_side_nav_navigation_supported)
}

fn compose_side_nav_navigation_supported(action: &NavigationAction) -> bool {
    matches!(
        action,
        NavigationAction::Internal { .. } | NavigationAction::Section { .. }
    )
}

fn compose_side_nav_entries(items: &[SideNavItem], indent: usize) -> String {
    compose_side_nav_entries_with_prefix(items, indent, "item")
}

fn compose_side_nav_entries_with_prefix(
    items: &[SideNavItem],
    indent: usize,
    prefix: &str,
) -> String {
    if items.is_empty() {
        return "emptyList()".to_string();
    }
    let pad = " ".repeat(indent);
    let item_pad = " ".repeat(indent + 4);
    let mut output = "listOf(\n".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("{prefix}-{index}");
        output.push_str(&format!(
            "{item_pad}{},\n",
            compose_side_nav_entry(item, indent + 4, &id)
        ));
    }
    output.push_str(&format!("{pad})"));
    output
}

fn compose_side_nav_child_entries(
    items: &[SideNavItemProps],
    indent: usize,
    prefix: &str,
) -> String {
    if items.is_empty() {
        return "emptyList()".to_string();
    }
    let pad = " ".repeat(indent);
    let item_pad = " ".repeat(indent + 4);
    let mut output = "listOf(\n".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("{prefix}-{index}");
        output.push_str(&format!(
            "{item_pad}{},\n",
            compose_side_nav_entry_props("item", item, false, false, "", &id)
        ));
    }
    output.push_str(&format!("{pad})"));
    output
}

fn compose_side_nav_entry(item: &SideNavItem, indent: usize, id: &str) -> String {
    match item {
        SideNavItem::Header(props) => {
            compose_side_nav_entry_props("header", props, false, false, "", id)
        }
        SideNavItem::Item(props) => compose_side_nav_entry_props("item", props, false, false, "", id),
        SideNavItem::Divider => format!(
            "DoweSideNavEntry(id = \"{}\", kind = \"divider\", label = \"\", description = null, status = null, operation = null, path = null, fragment = null, bordered = false)",
            escape_kotlin(id)
        ),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            let children = compose_side_nav_child_entries(items, indent + 4, id);
            compose_side_nav_entry_props("submenu", props, *open, *bordered, &children, id)
        }
    }
}

fn compose_side_nav_entry_props(
    kind: &str,
    props: &SideNavItemProps,
    open: bool,
    bordered: bool,
    children: &str,
    id: &str,
) -> String {
    let (operation, path, fragment) = compose_side_nav_navigation_values(props.navigation.as_ref());
    let children = if children.is_empty() {
        "emptyList()"
    } else {
        children
    };
    format!(
        "DoweSideNavEntry(id = \"{}\", kind = \"{}\", label = {}, description = {}, status = {}, operation = {}, path = {}, fragment = {}, open = {}, bordered = {}, children = {})",
        escape_kotlin(id),
        kind,
        compose_localized_literal(&props.label, props.i18n.as_deref()),
        props.description.as_deref().map(|value| compose_localized_literal(value, props.description_i18n.as_deref())).unwrap_or_else(|| "null".to_string()),
        props.status.as_deref().map(|value| compose_localized_literal(value, props.status_i18n.as_deref())).unwrap_or_else(|| "null".to_string()),
        compose_side_nav_optional_string(operation),
        compose_side_nav_optional_string(path),
        compose_side_nav_optional_string(fragment),
        open,
        bordered,
        children
    )
}

fn compose_side_nav_navigation_values(
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

fn compose_side_nav_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_kotlin(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn render_compose_side_nav_item(
    item: &SideNavItem,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    wide: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match item {
        SideNavItem::Header(props) => render_compose_side_nav_row(
            props,
            true,
            compose_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Item(props) => render_compose_side_nav_row(
            props,
            false,
            compose_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Divider => output.push_str(&format!(
            "{pad}Box(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp).height(1.dp).background(DoweDesign.muted))\n"
        )),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            output.push_str(&format!("{pad}DoweSideNavSubmenu(open = {open}, bordered = {bordered}, wide = {wide}, trigger = {{ expanded, toggle ->\n"));
            render_compose_side_nav_row(
                props,
                true,
                "toggle".to_string(),
                indent + 4,
                output,
                nav,
                wide,
                inherited_font,
                default_family,
                Some("expanded"),
            );
            output.push_str(&format!("{pad}}}) {{\n"));
            for item in items {
                render_compose_side_nav_row(
                    item,
                    false,
                    compose_side_nav_action(item, context),
                    indent + 4,
                    output,
                    nav,
                    wide,
                    inherited_font,
                    default_family,
                    None,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_compose_side_nav_row(
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
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        compose_side_nav_metrics(nav.size);
    let border =
        if nav.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            variant_content(&nav.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweSideNavRow(active = {}, wide = {}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, gap = {gap}.dp, backgroundColor = {}, contentColor = {}, borderColor = {border}, onClick = {action}) {{\n",
        compose_side_nav_active(props.navigation.as_ref()),
        wide,
        variant_container(&nav.style),
        variant_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
            compose_svg_view_box(&icon.props.view_box),
            modifier_for_style(&icon.props.style),
            compose_svg_color(&icon.props.style),
            compose_svg_paths(&icon.paths)
        ));
    }
    output.push_str(&format!(
        "{pad}    Column(modifier = Modifier.weight(1f)) {{\n"
    ));
    output.push_str(&format!(
        "{pad}        Text(text = {}, fontSize = {label_size}.sp, fontFamily = {}, fontWeight = {})\n",
        compose_localized_literal(&props.label, props.i18n.as_deref()),
        compose_font_value(inherited_font, default_family),
        if header {
            "FontWeight.SemiBold"
        } else {
            "FontWeight.Normal"
        }
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text(text = {}, fontSize = {description_size}.sp, fontFamily = {}, color = LocalContentColor.current.copy(alpha = 0.72f))\n",
            compose_localized_literal(description, props.description_i18n.as_deref()),
            compose_font_value(inherited_font, default_family),
        ));
    }
    output.push_str(&format!("{pad}    }}\n"));
    if props.status.is_some() || submenu_expanded.is_some() {
        output.push_str(&format!(
            "{pad}    Row(horizontalArrangement = Arrangement.spacedBy({gap}.dp), verticalAlignment = Alignment.CenterVertically) {{\n"
        ));
        if let Some(status) = props.status.as_deref() {
            output.push_str(&format!(
                "{pad}        DoweSideNavStatus(text = {}, descriptionSize = {description_size}f, fontFamily = {})\n",
                compose_localized_literal(status, props.status_i18n.as_deref()),
                compose_font_value(inherited_font, default_family)
            ));
        }
        if let Some(expanded) = submenu_expanded {
            output.push_str(&format!(
                "{pad}        DoweSideNavArrow(expanded = {expanded})\n"
            ));
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_side_nav_action(props: &SideNavItemProps, context: &ComposeReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "{{ actionScope.launch {{ state.run(\"{}\") }} }}",
                escape_kotlin(id)
            )
        })
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| compose_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "null".to_string())
}

fn compose_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_kotlin(path))
        }
        _ => "false".to_string(),
    }
}

fn compose_side_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 8, 12, 10),
        SideNavSize::Md => (12, 8, 10, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}
