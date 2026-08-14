fn render_compose_bar(
    props: &BarProps,
    boxed_max_width: u16,
    boxed_regions: bool,
    top: &[ViewNode],
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    bottom: &[ViewNode],
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
        "{pad}Column(modifier = {}) {{\n",
        modifier_for_bar(props, flow)
    ));
    output.push_str(&format!(
        "{pad}    CompositionLocalProvider(LocalContentColor provides {}) {{\n",
        variant_content(&props.style)
    ));
    if props.boxed && boxed_regions {
        output.push_str(&format!(
            "{pad}        Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {{\n"
        ));
        output.push_str(&format!(
            "{pad}            Column(modifier = Modifier.widthIn(max = {boxed_max_width}.dp).fillMaxWidth()) {{\n"
        ));
        render_compose_bar_edge_region(
            top,
            indent + 16,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!(
            "{pad}                Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {{\n"
        ));
        render_compose_bar_regions(
            start,
            center,
            end,
            indent + 20,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}                }}\n"));
        render_compose_bar_edge_region(
            bottom,
            indent + 16,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}            }}\n"));
        output.push_str(&format!("{pad}        }}\n"));
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!("{pad}}}\n"));
        return;
    }
    render_compose_bar_edge_region(
        top,
        indent + 8,
        output,
        current_font,
        default_family,
        context,
    );
    if props.boxed {
        output.push_str(&format!(
            "{pad}        Box(modifier = {}, contentAlignment = Alignment.Center) {{\n",
            "Modifier.fillMaxWidth()"
        ));
        output.push_str(&format!(
            "{pad}            Row(modifier = Modifier.widthIn(max = {boxed_max_width}.dp).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {{\n"
        ));
        render_compose_bar_regions(
            start,
            center,
            end,
            indent + 16,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}            }}\n"));
        output.push_str(&format!("{pad}        }}\n"));
    } else {
        output.push_str(&format!(
            "{pad}        Row(modifier = {}, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {{\n",
            "Modifier.fillMaxWidth()"
        ));
        render_compose_bar_regions(
            start,
            center,
            end,
            indent + 12,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}        }}\n"));
    }
    render_compose_bar_edge_region(
        bottom,
        indent + 8,
        output,
        current_font,
        default_family,
        context,
    );
    output.push_str(&format!("{pad}    }}\n"));
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_bar_edge_region(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    for child in children {
        render_compose_node_in_flow(
            child,
            indent,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn render_compose_bottom_bar(
    props: &BarProps,
    tabs: &[BottomBarTab],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    _context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    if props.boxed {
        output.push_str(&format!(
            "{pad}Box(modifier = {}, contentAlignment = Alignment.BottomCenter) {{\n",
            modifier_for_bar(props, flow)
        ));
    }
    let content_indent = if props.boxed { indent + 4 } else { indent };
    let content_pad = " ".repeat(content_indent);
    let content_modifier = if props.boxed {
        "Modifier.widthIn(max = 1536.dp).fillMaxWidth()".to_string()
    } else {
        modifier_for_bar(props, flow)
    };
    output.push_str(&format!(
        "{content_pad}Row(modifier = {content_modifier}, verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.SpaceEvenly) {{\n"
    ));
    for tab in tabs {
        let action = compose_navigation_action(Some(&tab.navigation));
        let active = compose_side_nav_active(Some(&tab.navigation));
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
            compose_svg_color(&tab.icon.props.style)
        };
        output.push_str(&format!(
            "{content_pad}    DoweRailNavItem(label = {}, showLabel = true, active = {active}, itemSize = {size}.dp, labelSize = 10f, backgroundColor = {background}, contentColor = {content}, borderColor = null, featured = {featured}, onClick = {action}) {{\n",
            compose_localized_literal(&tab.label, tab.i18n.as_deref()),
        ));
        output.push_str(&format!(
            "{content_pad}        DoweSvg(viewBox = {}, modifier = Modifier.size(20.dp), color = {}, paths = {}, animated = {})\n",
            compose_svg_view_box(&tab.icon.props.view_box),
            icon_color,
            compose_svg_paths(&tab.icon.paths),
            tab.icon.props.is_animated()
        ));
        output.push_str(&format!("{content_pad}    }}\n"));
    }
    output.push_str(&format!("{content_pad}}}\n"));
    if props.boxed {
        output.push_str(&format!("{pad}}}\n"));
    }
}

fn render_compose_bar_regions(
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    render_compose_bar_region(
        start,
        indent,
        output,
        "Arrangement.Start",
        "Modifier",
        inherited_font,
        default_family,
        context,
    );
    render_compose_bar_region(
        center,
        indent,
        output,
        "Arrangement.Center",
        "Modifier.weight(1f)",
        inherited_font,
        default_family,
        context,
    );
    render_compose_bar_region(
        end,
        indent,
        output,
        "Arrangement.End",
        "Modifier",
        inherited_font,
        default_family,
        context,
    );
}

fn render_compose_bar_region(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    arrangement: &str,
    modifier: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    if children.is_empty() {
        return;
    }
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}Row(modifier = {modifier}.padding(8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = {arrangement}) {{\n"
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Inline,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}
