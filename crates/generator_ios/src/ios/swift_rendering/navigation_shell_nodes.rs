fn render_swift_navigation_shell_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    match node {
        ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => render_swift_bar(
            props,
            top,
            start,
            center,
            end,
            bottom,
            SwiftBarOptions {
                start_padding: 12,
                center_padding: 12,
                end_padding: 12,
                boxed_width: 1536,
            },
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        => render_swift_bar(
            props,
            &[],
            start,
            center,
            end,
            &[],
            SwiftBarOptions {
                start_padding: 8,
                center_padding: 8,
                end_padding: 8,
                boxed_width: 1536,
            },
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::BottomBar { props, tabs, .. } => {
            render_swift_bottom_bar(props, tabs, indent, output, flow, context)
        }
        ViewNode::SideNav { props, items } => {
            render_swift_side_nav(
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
            render_swift_rail_nav(props, items, indent, output, flow, context);
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            render_swift_sidebar(
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
            render_swift_nav_menu(
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
            render_swift_scaffold(
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
            render_swift_tabs(
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
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            render_swift_drawer(
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
        _ => unreachable!(),
    }
}
