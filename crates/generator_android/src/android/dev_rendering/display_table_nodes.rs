fn render_dev_android_table_display_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) -> bool {
    if !matches!(node, ViewNode::Table { .. }) {
        return false;
    }
    match node {
        ViewNode::Table { props } => {
            let view = next_dev_view(counter);
            render_dev_android_table(props, &view, &context.signal_path(&props.data), output);
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
    true
}
