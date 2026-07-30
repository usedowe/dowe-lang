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
            top,
            start,
            center,
            end,
            bottom,
        } => {
            render_dev_android_bar(
                props,
                1536,
                top,
                start,
                center,
                end,
                bottom,
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
        ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
        } => {
            render_dev_android_bar(
                props,
                1536,
                top,
                start,
                center,
                end,
                bottom,
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
        ViewNode::BottomBar { props, tabs, .. } => {
            render_dev_android_bottom_bar(
                props,
                tabs,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
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
        ViewNode::RailNav { props, items } => {
            render_dev_android_rail_nav(
                props,
                items,
                parent,
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
            overlays,
        } => {
            render_dev_android_scaffold(
                props,
                app_bar,
                start,
                main,
                end,
                bottom_bar,
                overlays,
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

fn render_dev_android_bar(
    props: &BarProps,
    boxed_max_width: u16,
    top: &[ViewNode],
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    bottom: &[ViewNode],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let current_color = Some(dev_variant_content(&props.style).to_string());
    let surface = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {surface} = doweContainer(false);\n        {surface}.setMinimumHeight(doweDp(48));\n        {surface}.setBackground(doweBackground({}, {}));\n",
        dev_variant_container(&props.style),
        if props.floating {
            "DOWE_RADIUS"
        } else {
            "0"
        }
    ));
    apply_dev_android_style(&props.style.style, &surface, true, output);
    if props.position != BarPosition::Static {
        output.push_str(&format!(
            "        {surface}.setElevation(doweDp(4));\n"
        ));
    }
    if props.position == BarPosition::Static {
        output.push_str(&dev_add(
            parent,
            &surface,
            parent_gap,
            parent_horizontal,
        ));
    }
    render_dev_android_bar_edge_region(
        top,
        &surface,
        counter,
        output,
        current_font,
        current_color.clone(),
        context,
        children_method,
    );
    let content = next_dev_view(counter);
    let content_constructor = if props.boxed {
        format!("doweBoxedContainer(true, {boxed_max_width})")
    } else {
        "doweContainer(true)".to_string()
    };
    output.push_str(&format!(
        "        LinearLayout {content} = {content_constructor};\n        {content}.setGravity(Gravity.CENTER_VERTICAL);\n        doweAdd({surface}, {content});\n"
    ));
    render_dev_android_bar_region(
        start,
        &content,
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
        render_dev_android_bar_spacer(&content, counter, output);
    }
    render_dev_android_bar_region(
        center,
        &content,
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
        &content,
        "Gravity.END",
        false,
        counter,
        output,
        current_font,
        current_color.clone(),
        context,
        children_method,
    );
    render_dev_android_bar_edge_region(
        bottom,
        &surface,
        counter,
        output,
        current_font,
        current_color,
        context,
        children_method,
    );
    if props.position != BarPosition::Static {
        output.push_str(&format!(
            "        dowePinAppBar({parent}, {surface});\n"
        ));
    }
}

fn render_dev_android_bar_edge_region(
    children: &[ViewNode],
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    for child in children {
        render_dev_android_node(
            child,
            parent,
            Some("8"),
            false,
            counter,
            output,
            inherited_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
}
