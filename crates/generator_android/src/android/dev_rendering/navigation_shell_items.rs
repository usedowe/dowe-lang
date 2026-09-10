fn render_dev_android_side_nav_item(
    item: &SideNavItem,
    parent: &str,
    nav: &SideNavProps,
    wide: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    memory_key: &str,
) {
    match item {
        SideNavItem::Header(props) => {
            render_dev_android_side_nav_row(
                props,
                true,
                parent,
                nav,
                wide,
                counter,
                output,
                inherited_font,
                context,
                None,
            );
        }
        SideNavItem::Item(props) => {
            render_dev_android_side_nav_row(
                props,
                false,
                parent,
                nav,
                wide,
                counter,
                output,
                inherited_font,
                context,
                None,
            );
        }
        SideNavItem::Divider => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setBackgroundColor(DOWE_MUTED);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));\n        doweAdd({parent}, {view}, 8, false);\n"
            ));
        }
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            let expanded = format!(
                "doweSideNavExpanded(\"{}\", {open})",
                escape_java(memory_key)
            );
            let trigger = render_dev_android_side_nav_row(
                props,
                false,
                parent,
                nav,
                wide,
                counter,
                output,
                inherited_font,
                context,
                Some(*open),
            );
            let submenu = next_dev_view(counter);
            let submenu_content = next_dev_view(counter);
            output.push_str(&format!(
                "        boolean {submenu}Expanded = {expanded};\n        {trigger}Arrow.setRotation({submenu}Expanded ? 90f : 0f);\n        LinearLayout {submenu} = doweContainer({bordered});\n        {submenu}.setLayoutParams(new LinearLayout.LayoutParams({}, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {submenu}.setPadding(doweDp(16), 0, 0, 0);\n        {submenu}.setVisibility({submenu}Expanded ? View.VISIBLE : View.GONE);\n        doweAdd({parent}, {submenu});\n        LinearLayout {submenu_content} = doweSideNavSubmenuContent({submenu}, {bordered});\n        {trigger}.setOnClickListener(v -> doweToggleSideNavSubmenu({submenu}, {trigger}Arrow, \"{}\"));\n",
                format!("{wide} ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT"),
                escape_java(memory_key)
            ));
            for (index, item) in items.iter().enumerate() {
                let renderer = format!("{submenu_content}Item{index}");
                let item_parent = format!("{renderer}Parent");
                output.push_str(&format!(
                    "        Consumer<LinearLayout> {renderer} = {item_parent} -> {{\n"
                ));
                render_dev_android_side_nav_row(
                    item,
                    false,
                    &item_parent,
                    nav,
                    wide,
                    counter,
                    output,
                    inherited_font,
                    context,
                    None,
                );
                output.push_str(&format!(
                    "        }};\n        {renderer}.accept({submenu_content});\n"
                ));
            }
        }
    }
}

fn render_dev_android_rail_nav(
    props: &RailNavProps,
    items: &[RailNavItem],
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let rail = next_dev_view(counter);
    let (rail_width, item_size, icon_size, label_size) = compose_rail_nav_metrics(props.size);
    output.push_str(&format!(
        "        LinearLayout {rail} = doweContainer(false);\n        {rail}.setGravity(Gravity.CENTER_HORIZONTAL);\n        {rail}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({rail_width}), ViewGroup.LayoutParams.WRAP_CONTENT));\n        {rail}.setPadding(0, doweDp(4), 0, doweDp(4));\n        doweAdd({parent}, {rail});\n"
    ));
    for item in items {
        match item {
            RailNavItem::Divider => {
                let divider = next_dev_view(counter);
                output.push_str(&format!(
                    "        View {divider} = new View(this);\n        {divider}.setBackgroundColor(DOWE_MUTED);\n        {divider}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({item_size}), doweDp(1)));\n        doweAdd({rail}, {divider}, 6, false);\n"
                ));
            }
            RailNavItem::Item(item) => {
                let view = next_dev_view(counter);
                let active = dev_side_nav_active(item.navigation.as_ref());
                let active_content = dev_nav_active_content(&props.style);
                let content = format!("({active}) ? {active_content} : DOWE_BACKGROUND_TEXT");
                output.push_str(&format!(
                    "        LinearLayout {view} = doweContainer(false);\n        {view}.setGravity(Gravity.CENTER_HORIZONTAL);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({item_size}), doweDp({item_size})));\n        {view}.setPadding(doweDp(6), doweDp(6), doweDp(6), doweDp(6));\n        {view}.setContentDescription({});\n        if ({active}) {{ {view}.setBackground(doweBackground({}, DOWE_RADIUS)); }}\n        doweAdd({rail}, {view}, 4, false);\n",
                    dev_localized_literal(&item.label, item.i18n.as_deref()),
                    dev_variant_container(&props.style)
                ));
                let icon = render_dev_android_icon_view(
                    &item.icon,
                    counter,
                    output,
                    Some(&content),
                );
                output.push_str(&format!(
                    "        {icon}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({icon_size}), doweDp({icon_size})));\n        doweAdd({view}, {icon});\n"
                ));
                if props.show_labels {
                    output.push_str(&format!(
                        "        TextView {view}Label = doweText({}, {content}, {label_size}f, 600, 0f, {label_size}f, {});\n        {view}Label.setGravity(Gravity.CENTER);\n        {view}Label.setMaxLines(1);\n        doweAdd({view}, {view}Label, 4, false);\n",
                        dev_localized_literal(&item.label, item.i18n.as_deref()),
                        dev_font_value(inherited_font)
                    ));
                }
                let action = item
                    .on_click
                    .as_deref()
                    .and_then(|name| context.action_id(name))
                    .map(|id| format!("doweRunAction(\"{}\", null)", escape_java(id)))
                    .or_else(|| dev_android_navigation_action(item.navigation.as_ref()));
                if let Some(action) = action {
                    output.push_str(&format!(
                        "        {view}.setOnClickListener(v -> {action});\n"
                    ));
                }
            }
        }
    }
}

fn render_dev_android_bottom_bar(
    props: &BarProps,
    tabs: &[BottomBarTab],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
) {
    let bar = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {bar} = doweContainer(false);\n        {bar}.setGravity(Gravity.CENTER_VERTICAL);\n        {bar}.setMinimumHeight(doweDp(48));\n        {bar}.setBackground(doweBackground({}, {}));\n",
        dev_variant_container(&props.style),
        if props.floating { "DOWE_RADIUS" } else { "0" }
    ));
    output.push_str(&dev_add(parent, &bar, parent_gap, parent_horizontal));
    let content = next_dev_view(counter);
    let content_constructor = if props.boxed {
        "doweBoxedContainer(true, 1536)"
    } else {
        "doweContainer(true)"
    };
    output.push_str(&format!(
        "        LinearLayout {content} = {content_constructor};\n        {content}.setGravity(Gravity.CENTER_VERTICAL);\n        doweAdd({bar}, {content});\n"
    ));
    for tab in tabs {
        let view = next_dev_view(counter);
        let active = dev_side_nav_active(Some(&tab.navigation));
        let size = if tab.featured { 56 } else { 48 };
        let background = if tab.featured {
            "DOWE_PRIMARY"
        } else {
            dev_variant_container(&props.style)
        };
        let radius = if tab.featured { "999f" } else { "DOWE_RADIUS" };
        output.push_str(&format!(
            "        LinearLayout {view} = doweContainer(false);\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, doweDp({size}), 1f));\n        {view}.setContentDescription({});\n        if ({active} || {}) {{ {view}.setBackground(doweBackground({background}, {radius})); }}\n        {content}.addView({view});\n",
            dev_localized_literal(&tab.label, tab.i18n.as_deref()),
            tab.featured,
        ));
        if tab.featured {
            output.push_str(&format!("        {view}.setElevation(doweDp(8));\n"));
        }
        let icon = render_dev_android_icon_view(
            &tab.icon,
            counter,
            output,
            tab.featured.then_some("DOWE_PRIMARY_TEXT"),
        );
        output.push_str(&format!(
            "        {icon}.setLayoutParams(new LinearLayout.LayoutParams(doweDp(20), doweDp(20)));\n        doweAdd({view}, {icon});\n"
        ));
        output.push_str(&format!(
            "        TextView {view}Label = doweText({}, {}, 10f, 600, 0f, 10f, {});\n        {view}Label.setGravity(Gravity.CENTER);\n        {view}Label.setMaxLines(1);\n        doweAdd({view}, {view}Label, 2, false);\n",
            dev_localized_literal(&tab.label, tab.i18n.as_deref()),
            if tab.featured { "DOWE_PRIMARY_TEXT" } else { "DOWE_BACKGROUND_TEXT" },
            dev_font_value(inherited_font),
        ));
        if let Some(action) = dev_android_navigation_action(Some(&tab.navigation)) {
            output.push_str(&format!("        {view}.setOnClickListener(v -> {action});\n"));
        }
    }
}

fn render_dev_android_side_nav_row(
    props: &SideNavItemProps,
    header: bool,
    parent: &str,
    nav: &SideNavProps,
    wide: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    submenu_open: Option<bool>,
) -> String {
    let view = next_dev_view(counter);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        compose_side_nav_metrics(nav.size);
    let copy_gap = if props.icon.is_some() {
        format!(", {gap}, true")
    } else {
        String::new()
    };
    let active = dev_side_nav_active(props.navigation.as_ref());
    let active_content = dev_nav_active_content(&nav.style);
    let content = format!("({active}) ? {active_content} : DOWE_BACKGROUND_TEXT");
    let label_content = if header {
        dev_side_nav_header_content(&nav.style).to_string()
    } else {
        content.clone()
    };
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams({}, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp({padding_horizontal}), doweDp({padding_vertical}), doweDp({padding_horizontal}), doweDp({padding_vertical}));\n        if ({}) {{ {view}.setBackground(doweBackground({}, DOWE_RADIUS)); }}\n",
        format!("{wide} ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT"),
        active,
        dev_variant_container(&nav.style)
    ));
    output.push_str(&format!("        doweAdd({parent}, {view});\n"));
    if let Some(icon) = props.icon.as_ref() {
        let icon_content = if header {
            dev_side_nav_header_content(&nav.style)
        } else {
            content.as_str()
        };
        render_dev_android_side_nav_icon(icon, &view, counter, output, Some(icon_content));
    }
    let copy = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {copy} = doweContainer(false);\n        {copy}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({view}, {copy}{copy_gap});\n        TextView {copy}Label = doweText({}, {label_content}, {label_size}f, {}, 0f, {label_size}f, {});\n        doweAdd({copy}, {copy}Label);\n",
        dev_localized_literal(&props.label, props.i18n.as_deref()),
        if header { "600" } else { "400" },
        dev_font_value(inherited_font)
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "        TextView {copy}Description = doweText({}, {content}, {description_size}f, 400, 0f, {description_size}f, {});\n        {copy}Description.setAlpha(0.72f);\n        doweAdd({copy}, {copy}Description);\n",
            dev_localized_literal(description, props.description_i18n.as_deref()),
            dev_font_value(inherited_font)
        ));
    }
    if let Some(status) = props.status.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Status = doweSideNavStatus({}, {description_size}f, {});\n        doweAdd({view}, {view}Status, {gap}, true);\n",
            dev_localized_literal(status, props.status_i18n.as_deref()),
            dev_font_value(inherited_font)
        ));
    }
    if let Some(open) = submenu_open {
        output.push_str(&format!(
            "        DoweSvgView {view}Arrow = doweSideNavArrow({content});\n        {view}Arrow.setRotation({});\n        {view}.setTag({view}Arrow);\n        doweAdd({view}, {view}Arrow, {gap}, true);\n",
            if open { "90f" } else { "0f" }
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
    let view = render_dev_android_icon_view(icon, counter, output, inherited_color);
    output.push_str(&format!("        doweAdd({parent}, {view});\n"));
}

fn render_dev_android_icon_view(
    icon: &SideNavIcon,
    counter: &mut usize,
    output: &mut String,
    inherited_color: Option<&str>,
) -> String {
    let view = next_dev_view(counter);
    let paths_name = format!("{view}Paths");
    output.push_str(&format!(
        "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
    ));
    for path in &icon.paths {
        output.push_str(&format!(
            "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}, {}, {}));\n",
            escape_java(&path.data),
            dev_svg_path_current_color(path.fill),
            dev_svg_path_color(path.fill),
            dev_svg_path_details(path.fill),
            dev_svg_path_transform(path.transform.as_ref())
        ));
    }
    output.push_str(&format!(
        "        DoweSvgView {view} = new DoweSvgView(this, {}f, {}f, {}f, {}f, {}, {paths_name}, {});\n",
        icon.props.view_box.min_x,
        icon.props.view_box.min_y,
        icon.props.view_box.width,
        icon.props.view_box.height,
        dev_svg_color(&icon.props.style, inherited_color),
        icon.props.is_animated()
    ));
    apply_dev_android_style(&icon.props.style, &view, false, output);
    view
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
