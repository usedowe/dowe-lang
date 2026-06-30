fn render_compose_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "{pad}Column(modifier = {}) {{\n",
        modifier_for_container_style(&props.style, flow)
    ));
    for child in app_bar {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!(
        "{pad}    Row(modifier = Modifier.fillMaxWidth().weight(1f)) {{\n"
    ));
    if !start.is_empty() {
        output.push_str(&format!("{pad}        Column {{\n"));
        for child in start {
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
        "{pad}        Column(modifier = Modifier.weight(1f)) {{\n"
    ));
    for child in main {
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
    if !end.is_empty() {
        output.push_str(&format!("{pad}        Column {{\n"));
        for child in end {
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
    for child in bottom_bar {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
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
        );
        return;
    }
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "{pad}Column(modifier = {}, verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n",
        modifier_for_container_style(&props.style.style, flow)
    ));
    for item in items {
        render_compose_side_nav_item(
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
}

fn render_compose_side_nav_data(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        compose_side_nav_metrics(props.size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweSideNav(items = {}, modifier = {}, activePath = activePath, wide = {}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, gap = {gap}.dp, labelSize = {label_size}f, descriptionSize = {description_size}f, fontFamily = {}, backgroundColor = {}, contentColor = {}, activeContentColor = {}, borderColor = {border}, navigate = navigate)\n",
        compose_side_nav_entries(items, indent),
        modifier_for_container_style(&props.style.style, flow),
        props.wide,
        compose_font_value(current_font, default_family),
        variant_container(&props.style),
        variant_content(&props.style),
        nav_active_content(&props.style),
    ));
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
        "DoweSideNavEntry(id = \"{}\", kind = \"{}\", label = \"{}\", description = {}, status = {}, operation = {}, path = {}, fragment = {}, open = {}, bordered = {}, children = {})",
        escape_kotlin(id),
        kind,
        escape_kotlin(&props.label),
        compose_side_nav_optional_string(props.description.as_deref()),
        compose_side_nav_optional_string(props.status.as_deref()),
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
            output.push_str(&format!("{pad}DoweSideNavSubmenu(open = {open}, bordered = {bordered}, trigger = {{ expanded, toggle ->\n"));
            render_compose_side_nav_row(
                props,
                true,
                "toggle".to_string(),
                indent + 4,
                output,
                nav,
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
        nav.wide,
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
        "{pad}        Text(text = \"{}\", fontSize = {label_size}.sp, fontFamily = {}, fontWeight = {})\n",
        escape_kotlin(&props.label),
        compose_font_value(inherited_font, default_family),
        if header {
            "FontWeight.SemiBold"
        } else {
            "FontWeight.Normal"
        }
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text(text = \"{}\", fontSize = {description_size}.sp, fontFamily = {}, color = LocalContentColor.current.copy(alpha = 0.72f))\n",
            escape_kotlin(description),
            compose_font_value(inherited_font, default_family),
        ));
    }
    output.push_str(&format!("{pad}    }}\n"));
    if let Some(status) = props.status.as_deref() {
        output.push_str(&format!(
            "{pad}    Text(text = \"{}\", fontSize = {description_size}.sp, fontWeight = FontWeight.SemiBold)\n",
            escape_kotlin(status)
        ));
    }
    if let Some(expanded) = submenu_expanded {
        output.push_str(&format!("{pad}    DoweSideNavArrow(expanded = {expanded})\n"));
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
        SideNavSize::Sm => (8, 6, 6, 12, 10),
        SideNavSize::Md => (12, 8, 8, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}
