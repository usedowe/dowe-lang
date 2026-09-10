fn render_compose_flow_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    if render_compose_structural_flow_node(
        node, indent, output, flow, inherited_font, default_family, context,
    ) {
        return;
    }
    render_compose_button_flow_node(
        node, indent, output, flow, inherited_font, default_family, context,
    );
}
