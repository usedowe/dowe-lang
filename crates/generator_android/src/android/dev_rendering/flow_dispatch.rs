fn render_dev_android_flow_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    if render_dev_android_base_flow_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        inherited_color.clone(), context, children_method,
    ) {
        return;
    }
    if render_dev_android_surface_flow_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        inherited_color.clone(), context, children_method,
    ) {
        return;
    }
    render_dev_android_button_flow_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        inherited_color, context, children_method,
    );
}
