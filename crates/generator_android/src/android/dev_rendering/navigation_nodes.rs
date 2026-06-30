fn render_dev_android_navigation_node(
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
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                            "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setMinimumHeight(doweDp(48));\n        {view}.setBackground(doweBackground({}, {}));\n",
                            dev_variant_container(&props.style),
                            if props.floating {
                                "DOWE_RADIUS_BOX"
                            } else {
                                "0"
                            }
                        ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            render_dev_android_bar_region(
                start,
                &view,
                "Gravity.START",
                false,
                counter,
                output,
                current_font,
                current_color.clone(),
                context,
                children_method,
            );
            if center.is_empty() && !end.is_empty() {
                render_dev_android_bar_spacer(&view, counter, output);
            }
            render_dev_android_bar_region(
                center,
                &view,
                "Gravity.CENTER",
                true,
                counter,
                output,
                current_font,
                current_color.clone(),
                context,
                children_method,
            );
            render_dev_android_bar_region(
                end,
                &view,
                "Gravity.END",
                false,
                counter,
                output,
                current_font,
                current_color,
                context,
                children_method,
            );
        }
        ViewNode::SideNav { props, items } => {
            render_dev_android_side_nav(
                props,
                items,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            render_dev_android_sidebar(
                props,
                header,
                body,
                footer,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::NavMenu { props, items } => {
            render_dev_android_nav_menu(
                props,
                items,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
                children_method,
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
            render_dev_android_scaffold(
                props,
                app_bar,
                start,
                main,
                end,
                bottom_bar,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                inherited_color,
                context,
                children_method,
            );
        }
        ViewNode::Tabs { props, tabs } => {
            render_dev_android_tabs(
                props,
                tabs,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        _ => {}
    }
}
