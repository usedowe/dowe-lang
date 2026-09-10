include!("form_actions_theme_and_fab.rs");
include!("form_actions_basic_controls.rs");
include!("form_actions_date_and_selection.rs");

fn render_dev_android_form_actions_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) {
    match node {
        ViewNode::ToggleTheme { .. } => render_dev_android_form_actions_toggle_theme(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::SelectTheme { .. } => render_dev_android_form_actions_select_theme(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::Fab { .. } => render_dev_android_form_actions_fab(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::Slider { .. } => render_dev_android_form_actions_slider(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::Dropzone { .. } => render_dev_android_form_actions_dropzone(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::Checkbox { .. } => render_dev_android_form_actions_checkbox(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::Color { .. } => render_dev_android_form_actions_color(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::Date { .. } => render_dev_android_form_actions_date(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::DateRange { .. } => render_dev_android_form_actions_date_range(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::RadioGroup { .. } => render_dev_android_form_actions_radio_group(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        ViewNode::Toggle { .. } => render_dev_android_form_actions_toggle(node, parent, parent_gap, parent_horizontal, counter, output, inherited_font, inherited_color, context, _children_method),
        _ => {}
    }
}
