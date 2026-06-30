fn render_compose_overlay_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    match node {
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            render_compose_drawer(
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
        ViewNode::Avatar { props, icon } => {
            render_compose_avatar(props, icon.as_ref(), indent, output, context);
        }
        ViewNode::Badge { props, children } => {
            render_compose_badge(
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
            render_compose_chip(
                props,
                value,
                start.as_ref(),
                end.as_ref(),
                indent,
                output,
                context,
            );
        }
        ViewNode::Skeleton { props } => {
            render_compose_skeleton(props, indent, output, flow);
        }
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            render_compose_modal(
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
            render_compose_alert_dialog(props, indent, output, context);
        }
        ViewNode::Tooltip { props, children } => {
            render_compose_tooltip(
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
        ViewNode::Toast { props } => {
            render_compose_toast(props, indent, output, context);
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            render_compose_dropdown(
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
            render_compose_command(
                props,
                entries,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        }
        _ => {}
    }
}
