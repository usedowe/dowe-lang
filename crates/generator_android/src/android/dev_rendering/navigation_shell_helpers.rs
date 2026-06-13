fn render_dev_android_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
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
    let view = next_dev_view(counter);
    let current_font = props.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    for child in app_bar {
        render_dev_android_node(
            child,
            &view,
            None,
            false,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    let body = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {body} = doweContainer(true);\n        {body}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));\n        doweAdd({view}, {body});\n"
    ));
    for child in start {
        render_dev_android_node(
            child,
            &body,
            None,
            true,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    let main_view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {main_view} = doweContainer(false);\n        {main_view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f));\n        doweAdd({body}, {main_view});\n"
    ));
    for child in main {
        render_dev_android_node(
            child,
            &main_view,
            None,
            false,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    for child in end {
        render_dev_android_node(
            child,
            &body,
            None,
            true,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    for child in bottom_bar {
        render_dev_android_node(
            child,
            &view,
            None,
            false,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
}

fn render_dev_android_side_nav(
    props: &SideNavProps,
    items: &[SideNavItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    for item in items {
        render_dev_android_side_nav_item(
            item,
            &view,
            props,
            counter,
            output,
            current_font,
            context,
        );
    }
}

fn render_dev_android_side_nav_item(
    item: &SideNavItem,
    parent: &str,
    nav: &SideNavProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    match item {
        SideNavItem::Header(props) => {
            render_dev_android_side_nav_row(
                props,
                true,
                parent,
                nav,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        SideNavItem::Item(props) => {
            render_dev_android_side_nav_row(
                props,
                false,
                parent,
                nav,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        SideNavItem::Divider => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setBackgroundColor(DOWE_MUTED);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));\n        doweAdd({parent}, {view}, 8, false);\n"
            ));
        }
        SideNavItem::Submenu { props, open, items } => {
            let trigger = render_dev_android_side_nav_row(
                props,
                true,
                parent,
                nav,
                counter,
                output,
                inherited_font,
                context,
            );
            let submenu = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {submenu} = doweContainer(false);\n        {submenu}.setPadding(doweDp(16), 0, 0, 0);\n        {submenu}.setVisibility({});\n        doweAdd({parent}, {submenu});\n        {trigger}.setOnClickListener(v -> doweToggleSideNavSubmenu({submenu}));\n",
                if *open { "View.VISIBLE" } else { "View.GONE" }
            ));
            for item in items {
                render_dev_android_side_nav_row(
                    item,
                    false,
                    &submenu,
                    nav,
                    counter,
                    output,
                    inherited_font,
                    context,
                );
            }
        }
    }
}

fn render_dev_android_side_nav_row(
    props: &SideNavItemProps,
    header: bool,
    parent: &str,
    nav: &SideNavProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) -> String {
    let view = next_dev_view(counter);
    let (padding_horizontal, padding_vertical, _, label_size, description_size) =
        compose_side_nav_metrics(nav.size);
    let content = dev_nav_active_content(&nav.style);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp({padding_horizontal}), doweDp({padding_vertical}), doweDp({padding_horizontal}), doweDp({padding_vertical}));\n        if ({}) {{ {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI)); }}\n",
        dev_side_nav_active(props.navigation.as_ref()),
        dev_variant_container(&nav.style)
    ));
    output.push_str(&format!("        doweAdd({parent}, {view});\n"));
    if let Some(icon) = props.icon.as_ref() {
        render_dev_android_side_nav_icon(icon, &view, counter, output, Some(content));
    }
    let copy = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {copy} = doweContainer(false);\n        {copy}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({view}, {copy});\n        TextView {copy}Label = doweText(\"{}\", {content}, {label_size}f, {}, 0f, {label_size}f, {});\n        doweAdd({copy}, {copy}Label);\n",
        escape_java(&props.label),
        if header { "600" } else { "400" },
        dev_font_value(inherited_font)
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "        TextView {copy}Description = doweText(\"{}\", {content}, {description_size}f, 400, 0f, {description_size}f, {});\n        {copy}Description.setAlpha(0.72f);\n        doweAdd({copy}, {copy}Description);\n",
            escape_java(description),
            dev_font_value(inherited_font)
        ));
    }
    if let Some(status) = props.status.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Status = doweText(\"{}\", {content}, {description_size}f, 600, 0f, {description_size}f, {});\n        doweAdd({view}, {view}Status);\n",
            escape_java(status),
            dev_font_value(inherited_font)
        ));
    }
    if let Some(action) = dev_side_nav_action(props, context) {
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> {action});\n"
        ));
    }
    view
}

fn render_dev_android_side_nav_icon(
    icon: &SideNavIcon,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_color: Option<&str>,
) {
    let view = next_dev_view(counter);
    let paths_name = format!("{view}Paths");
    output.push_str(&format!(
        "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
    ));
    for path in &icon.paths {
        output.push_str(&format!(
            "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}));\n",
            escape_java(&path.data),
            dev_svg_path_current_color(path.fill),
            dev_svg_path_color(path.fill)
        ));
    }
    output.push_str(&format!(
        "        DoweSvgView {view} = new DoweSvgView(this, {}f, {}f, {}f, {}f, {}, {paths_name});\n",
        icon.props.view_box.min_x,
        icon.props.view_box.min_y,
        icon.props.view_box.width,
        icon.props.view_box.height,
        dev_svg_color(&icon.props.style, inherited_color)
    ));
    apply_dev_android_style(&icon.props.style, &view, false, output);
    output.push_str(&format!("        doweAdd({parent}, {view}, 8, true);\n"));
}

fn dev_side_nav_action(
    props: &SideNavItemProps,
    context: &ComposeReactiveContext,
) -> Option<String> {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null)", escape_java(id)))
        .or_else(|| dev_android_navigation_action(props.navigation.as_ref()))
}

fn dev_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("\"{}\".equals(currentPath)", escape_java(path))
        }
        _ => "false".to_string(),
    }
}
