fn render_dev_android_display_media_data_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    if render_dev_android_basic_display_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        _inherited_color.clone(), context, children_method,
    ) {
        return;
    }
    if render_dev_android_interactive_display_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        _inherited_color.clone(), context, children_method,
    ) {
        return;
    }
    if render_dev_android_media_display_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        _inherited_color.clone(), context, children_method,
    ) {
        return;
    }
    if render_dev_android_visualization_display_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        _inherited_color.clone(), context, children_method,
    ) {
        return;
    }
    if render_dev_android_table_display_node(
        node, parent, parent_gap, parent_horizontal, counter, output, inherited_font,
        _inherited_color.clone(), context, children_method,
    ) {
        return;
    }
}
