fn render_swift_rich_display_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    match node {
        ViewNode::AvatarGroup { props, items } => {
            render_swift_avatar_group(props, items, indent, output, context)
        }
        ViewNode::ChatBox { props } => render_swift_chat_box(props, indent, output, context),
        ViewNode::Empty { props } => render_swift_empty(props, indent, output, context),
        ViewNode::Marquee { props, children } => render_swift_marquee(
            props,
            children,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::TypeWriter { props, items } => {
            render_swift_type_writer(props, items, indent, output)
        }
        ViewNode::RichText { props, marks } => {
            render_swift_rich_text(props, marks, indent, output, inherited_font, default_family)
        }
        ViewNode::Record { props } => render_swift_record(props, indent, output, context),
        ViewNode::ToggleGroup { props, items } => {
            render_swift_toggle_group(props, items, indent, output, context)
        }
        ViewNode::Collapsible { props, children } => render_swift_collapsible(
            props,
            children,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Countdown { props } => render_swift_countdown(props, indent, output, context),
        ViewNode::Map {
            props,
            markers,
            waypoints,
        } => render_swift_map(props, markers, waypoints, indent, output, context),
        _ => unreachable!(),
    }
}
