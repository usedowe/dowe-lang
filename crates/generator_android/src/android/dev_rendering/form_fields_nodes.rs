fn render_dev_android_form_fields_node(
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
    match node {
        ViewNode::Input { .. } => render_dev_android_input(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, children_method),
        ViewNode::Select { .. } => render_dev_android_select(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, children_method),
        _ => {}
    }
}
