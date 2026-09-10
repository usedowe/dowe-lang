include!("display_media_nodes.rs");
include!("display_canvas_and_charts.rs");
include!("display_data_and_rich.rs");
include!("display_text_and_svg.rs");

fn render_compose_display_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    match node {
        ViewNode::Audio { .. } => render_compose_display_audio(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Camera { .. } => render_compose_display_camera(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Microphone { .. } => render_compose_display_microphone(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Image { .. } => render_compose_display_image(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Accordion { .. } => render_compose_display_accordion(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Carousel { .. } => render_compose_display_carousel(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Code { .. } => render_compose_display_code(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Video { .. } => render_compose_display_video(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Iframe { .. } => render_compose_display_iframe(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Device { .. } => render_compose_display_device(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Canvas { .. } => render_compose_display_canvas(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Diagram { .. } => render_compose_display_diagram(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Candlestick { .. } => render_compose_display_candlestick(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::ArcChart { .. } => render_compose_display_arc_chart(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::AreaChart { .. } => render_compose_display_area_chart(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::BarChart { .. } => render_compose_display_bar_chart(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::LineChart { .. } => render_compose_display_line_chart(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::PieChart { .. } => render_compose_display_pie_chart(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Table { .. } => render_compose_display_table(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Tree { .. } => render_compose_display_tree(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::AvatarGroup { .. } => render_compose_display_avatar_group(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::ChatBox { .. } => render_compose_display_chat_box(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Empty { .. } => render_compose_display_empty(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Marquee { .. } => render_compose_display_marquee(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::TypeWriter { .. } => render_compose_display_type_writer(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::RichText { .. } => render_compose_display_rich_text(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Record { .. } => render_compose_display_record(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::ToggleGroup { .. } => render_compose_display_toggle_group(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Collapsible { .. } => render_compose_display_collapsible(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Countdown { .. } => render_compose_display_countdown(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Map { .. } => render_compose_display_map(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Divider { .. } => render_compose_display_divider(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Title { .. } => render_compose_display_title(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Text { .. } => render_compose_display_text(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Alert { .. } => render_compose_display_alert(node, indent, output, flow, inherited_font, default_family, context),
        ViewNode::Svg { .. } => render_compose_display_svg(node, indent, output, flow, inherited_font, default_family, context),
        _ => {}
    }
}
