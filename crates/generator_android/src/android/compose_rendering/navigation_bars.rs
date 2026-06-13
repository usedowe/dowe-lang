fn render_compose_bar(
    props: &BarProps,
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    if props.boxed {
        output.push_str(&format!(
            "{pad}Box(modifier = {}, contentAlignment = Alignment.Center) {{\n",
            modifier_for_bar(props, flow)
        ));
        output.push_str(&format!(
            "{pad}    CompositionLocalProvider(LocalContentColor provides {}) {{\n",
            variant_content(&props.style)
        ));
        output.push_str(&format!(
            "{pad}        Row(modifier = Modifier.fillMaxWidth().widthIn(max = 1152.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {{\n"
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
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!("{pad}}}\n"));
    } else {
        output.push_str(&format!(
            "{pad}Row(modifier = {}, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {{\n",
            modifier_for_bar(props, flow)
        ));
        output.push_str(&format!(
            "{pad}    CompositionLocalProvider(LocalContentColor provides {}) {{\n",
            variant_content(&props.style)
        ));
        render_compose_bar_regions(
            start,
            center,
            end,
            indent + 8,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}    }}\n"));
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
