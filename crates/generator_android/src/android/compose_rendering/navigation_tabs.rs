fn render_compose_tabs(
    props: &TabsProps,
    tabs: &[TabItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let initial = tabs.first().map(|tab| tab.id.as_str()).unwrap_or_default();
    output.push_str(&format!(
        "{pad}DoweTabs(items = {}, initialId = {}, modifier = {}, position = {}, variant = {}, backgroundColor = {}, contentColor = {}, activeBackgroundColor = {}, activeContentColor = {}, accentColor = {}, borderColor = {}, radius = {}, fontFamily = {}) {{ activeTab ->\n",
        compose_tabs_items(tabs),
        compose_string_literal(initial),
        modifier_for_container_style(&props.style, flow),
        compose_string_literal(props.position.as_str()),
        compose_string_literal(props.variant.as_str()),
        tabs_list_background(props),
        tabs_list_content(props),
        tabs_active_background(props),
        tabs_active_content(props),
        tabs_accent(props),
        tabs_border(props),
        compose_control_radius(&props.style),
        compose_font_value(current_font, default_family),
    ));
    for (index, tab) in tabs.iter().enumerate() {
        output.push_str(&format!(
            "{pad}    {} (activeTab == {}) {{\n",
            if index == 0 { "if" } else { "else if" },
            compose_string_literal(&tab.id)
        ));
        for child in &tab.children {
            render_compose_node_in_flow(
                child,
                indent + 8,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}
