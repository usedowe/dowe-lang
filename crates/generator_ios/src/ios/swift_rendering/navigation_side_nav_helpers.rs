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
        render_swift_region_children(
            header,
            indent + 8,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
        output.push_str(&format!(
            "{pad}    .foregroundStyle({})\n",
            scheme_title(&props.style)
        ));
    }
    output.push_str(&format!(
        "{pad}    ScrollView {{\n{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    render_swift_region_children(
        body,
        indent + 12,
        output,
        current_font,
        default_family,
        context,
    );
    output.push_str(&format!(
        "{pad}        }}\n{pad}        .frame(maxWidth: .infinity, alignment: .topLeading)\n{pad}    }}\n{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
    ));
    if !footer.is_empty() {
        output.push_str(&format!(
            "{pad}    VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
        ));
        render_swift_region_children(
            footer,
            indent + 8,
            output,
            current_font,
            default_family,
            context,
        );
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
    modifiers.push(format!(
        ".environment(\\.doweTitleColor, {})",
        scheme_title(&props.style)
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

