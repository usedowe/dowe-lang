fn render_dev_android_display_node(
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
        ViewNode::Accordion { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::Code { .. }
        | ViewNode::Image { .. }
        | ViewNode::Table { .. }
        | ViewNode::Video { .. } => render_dev_android_display_media_data_node(
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
        ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Collapsible { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Empty { .. }
        | ViewNode::Map { .. }
        | ViewNode::Marquee { .. }
        | ViewNode::Record { .. }
        | ViewNode::RichText { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::TypeWriter { .. } => render_dev_android_display_rich_controls_node(
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
        ViewNode::Alert { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Text { .. }
        | ViewNode::Title { .. } => render_dev_android_display_text_svg_node(
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
