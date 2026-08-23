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
            "DoweDesign.primaryText"
        } else {
            variant_content(&props.style)
        };
        let icon_color = if featured {
            "DoweDesign.primaryText".to_string()
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
