fn render_swift_overlay_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    match node {
        ViewNode::Avatar { props, icon } => {
            render_swift_avatar(props, icon.as_ref(), indent, output, context);
        }
        ViewNode::Badge { props, children } => {
            render_swift_badge(
                props,
                children,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Chip {
            props,
            value,
            start,
            end,
        } => {
            render_swift_chip(
                props,
                value,
                start.as_ref(),
                end.as_ref(),
                indent,
                output,
                context,
            );
        }
        ViewNode::Skeleton { props } => render_swift_skeleton(props, indent, output, flow),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            render_swift_modal(
                props,
                header,
                body,
                footer,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::AlertDialog { props } => {
            render_swift_alert_dialog(props, indent, output, context);
        }
        ViewNode::Tooltip { props, children } => {
            render_swift_tooltip(
                props,
                children,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Toast { props } => render_swift_toast(props, indent, output, context),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            render_swift_dropdown(
                props,
                trigger,
                header,
                entries,
                footer,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Command { props, entries } => {
            render_swift_command(
                props,
                entries,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        }
        _ => unreachable!(),
    }
}
