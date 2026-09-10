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
        "{pad}    CompositionLocalProvider(LocalContentColor provides {}, LocalDoweTitleColor provides {}) {{\n",
        variant_content(&props.style),
        scheme_title(&props.style)
    ));
    if !header.is_empty() {
        output.push_str(&format!(
            "{pad}        CompositionLocalProvider(LocalContentColor provides {}) {{\n{pad}        Column(modifier = Modifier.fillMaxWidth().zIndex(1f)) {{\n",
            scheme_title(&props.style)
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
        output.push_str(&format!("{pad}        }}\n{pad}        }}\n"));
    }
    output.push_str(&format!(
        "{pad}        Box(modifier = Modifier.fillMaxWidth().weight(1f).clipToBounds()) {{\n{pad}            Column(modifier = Modifier.fillMaxWidth().verticalScroll(rememberScrollState())) {{\n"
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
    output.push_str(&format!("{pad}            }}\n{pad}        }}\n"));
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
    let memory_key = side_nav_memory_key(props, items);
    output.push_str(&format!(
        "{pad}Column(modifier = {}, verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n",
        modifier
    ));
    for (index, item) in items.iter().enumerate() {
        let item_memory_key = format!("{memory_key}:{index}");
        render_compose_side_nav_item(
            item,
            indent + 4,
            output,
            props,
            &wide,
            current_font,
            default_family,
            context,
            &item_memory_key,
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
    let variant = props.style.bindings().iter().find(|binding| binding.property == dowe_components::VariantBindingProperty::Variant).map(|binding| reactive_text(&binding.binding.path, "solid"));
    let scheme = props.style.bindings().iter().find(|binding| binding.property == dowe_components::VariantBindingProperty::Scheme).map(|binding| reactive_text(&binding.binding.path, "primary"));
    let size = props.style.bindings().iter().find(|binding| binding.property == dowe_components::VariantBindingProperty::Size).map(|binding| reactive_text(&binding.binding.path, "md"));
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
    let container = match (&variant, &scheme) { (None, None) => variant_container(&props.style).to_string(), _ => format!("doweButtonContainer({}, {})", variant.as_deref().unwrap_or("\"solid\""), scheme.as_deref().unwrap_or("\"primary\"")) };
    let content = match (&variant, &scheme) { (None, None) => variant_content(&props.style).to_string(), _ => format!("doweButtonContent({}, {})", variant.as_deref().unwrap_or("\"solid\""), scheme.as_deref().unwrap_or("\"primary\"")) };
    let title = side_nav_header_content(&props.style).to_string();
    let active_content = content.clone();
    let border = if let Some(variant) = variant.as_ref() { format!("if ({variant} == \"outlined\") {content} else null") } else if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined { content.clone() } else { "null".to_string() };
    let modifier = compose_side_nav_modifier(props, flow, &wide);
    output.push_str(&format!(
        "{pad}DoweSideNav(items = {}, stateKey = \"{}\", modifier = {}, activePath = activePath, wide = {}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, gap = {gap}.dp, labelSize = {label_size}f, descriptionSize = {description_size}f, fontFamily = {}, backgroundColor = {}, contentColor = {}, titleColor = {}, activeContentColor = {}, borderColor = {border}, navigate = navigate)\n",
        compose_side_nav_entries(items, indent),
        escape_kotlin(&side_nav_memory_key(props, items)),
        modifier,
        wide,
        compose_font_value(current_font, default_family),
        container,
        content,
        title,
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

