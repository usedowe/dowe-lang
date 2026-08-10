fn render_dev_android_form_node(
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
        ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Fab { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. } => render_dev_android_form_actions_node(
            node,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            inherited_color,
            context,
            children_method,
        ),
        ViewNode::Password { props } => render_dev_android_password(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Textarea { props } => render_dev_android_textarea(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Phone { props } => render_dev_android_phone(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Pin { props } => render_dev_android_pin(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Input { .. } | ViewNode::Select { .. } => render_dev_android_form_fields_node(
            node,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            inherited_color,
            context,
            children_method,
        ),
        _ => {}
    }
}
