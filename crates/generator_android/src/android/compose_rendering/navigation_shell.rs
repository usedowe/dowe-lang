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
        ),
        SideNavItem::Divider => output.push_str(&format!(
            "{pad}Box(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp).height(1.dp).background(DoweDesign.muted))\n"
        )),
        SideNavItem::Submenu { props, open, items } => {
            output.push_str(&format!("{pad}DoweSideNavSubmenu(open = {open}, trigger = {{ toggle ->\n"));
            render_compose_side_nav_row(
                props,
                true,
                "toggle".to_string(),
                indent + 4,
                output,
                nav,
                inherited_font,
                default_family,
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
