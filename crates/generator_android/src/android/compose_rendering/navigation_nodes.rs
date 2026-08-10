fn render_compose_navigation_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    match node {
        ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
        } => {
            render_compose_bar(
                props,
                1536,
                false,
                top,
                start,
                center,
                end,
                bottom,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
        } => {
            render_compose_bar(
                props,
                1536,
                true,
                top,
                start,
                center,
                end,
                bottom,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::BottomBar { props, tabs, .. } => {
            render_compose_bottom_bar(props, tabs, indent, output, flow, context);
        }
        ViewNode::SideNav { props, items } => {
            render_compose_side_nav(
                props,
                items,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::RailNav { props, items } => {
            render_compose_rail_nav(props, items, indent, output, flow, context);
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            render_compose_sidebar(
                props,
                header,
                body,
                footer,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::NavMenu { props, items } => {
            render_compose_nav_menu(
                props,
                items,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => {
            render_compose_scaffold(
                props,
                app_bar,
                start,
                main,
                end,
                bottom_bar,
                overlays,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Tabs { props, tabs } => {
            render_compose_tabs(
                props,
                tabs,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        _ => {}
    }
}
