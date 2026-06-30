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
            start,
            center,
            end,
        }
        | ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        | ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            render_compose_bar(
                props,
                start,
                center,
                end,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
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
        } => {
            render_compose_scaffold(
                props,
                app_bar,
                start,
                main,
                end,
                bottom_bar,
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
