fn render_dev_android_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    overlays: &[ViewNode],
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
    let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
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
            current_color.clone(),
            context,
            children_method,
        );
    }
    let body = next_dev_view(counter);
    let body_constructor = if props.boxed {
        "doweBoxedContainer(true, 1536)"
    } else {
        "doweContainer(true)"
    };
    output.push_str(&format!("        LinearLayout {body} = {body_constructor};\n"));
    if props.boxed {
        output.push_str(&format!(
            "        LinearLayout.LayoutParams {body}Params = new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);\n        {body}Params.gravity = Gravity.CENTER_HORIZONTAL;\n        {body}.setLayoutParams({body}Params);\n"
        ));
    } else {
        output.push_str(&format!(
            "        {body}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
        ));
    }
    output.push_str(&format!("        doweAdd({view}, {body});\n"));
    for child in start {
        render_dev_android_node(
            child,
            &body,
            None,
            true,
            counter,
            output,
            current_font,
            current_color.clone(),
            context,
            children_method,
        );
    }
    let main_view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {main_view} = doweContainer(false);\n        {main_view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({body}, {main_view});\n"
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
            current_color.clone(),
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
            current_color.clone(),
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
            current_color.clone(),
            context,
            children_method,
        );
    }
    for child in overlays {
        render_dev_android_node(
            child,
            &view,
            None,
            false,
            counter,
            output,
            current_font,
            current_color.clone(),
            context,
            children_method,
        );
    }
}

fn render_dev_android_sidebar(
    props: &SidebarProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let view = next_dev_view(counter);
    let body_scroll = next_dev_view(counter);
    let body_content = next_dev_view(counter);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    if props.style.style.sizing.h.is_none() {
        output.push_str(&format!(
            "        int {view}ShellHeight = Math.max(0, getResources().getDisplayMetrics().heightPixels - scrollView.getPaddingTop() - scrollView.getPaddingBottom());\n        {view}.setLayoutParams(new LinearLayout.LayoutParams({view}.getLayoutParams().width, {view}ShellHeight));\n"
        ));
    }
    output.push_str(&format!(
        "        {view}.setBackgroundColor({});\n",
        dev_variant_container(&props.style)
    ));
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    if !header.is_empty() {
        let header_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {header_content} = doweContainer(false);\n        doweAdd({view}, {header_content});\n"
        ));
        for child in header {
            render_dev_android_node(
                child,
                &header_content,
                None,
                false,
                counter,
                output,
                current_font,
                Some(dev_content_colors(
                    dev_scheme_title(&props.style),
                    dev_scheme_title(&props.style),
                )),
                context,
                children_method,
            );
        }
    }
    output.push_str(&format!(
        "        ScrollView {body_scroll} = new ScrollView(this);\n        {body_scroll}.setFillViewport(true);\n        LinearLayout {body_content} = doweContainer(false);\n        {body_scroll}.addView({body_content}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.addView({body_scroll}, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));\n"
    ));
    for child in body {
        render_dev_android_node(
            child,
            &body_content,
            None,
            false,
            counter,
            output,
            current_font,
            Some(dev_content_colors(
                dev_variant_content(&props.style),
                dev_scheme_title(&props.style),
            )),
            context,
            children_method,
        );
    }
    if !footer.is_empty() {
        let footer_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {footer_content} = doweContainer(false);\n        doweAdd({view}, {footer_content});\n"
        ));
        for child in footer {
            render_dev_android_node(
                child,
                &footer_content,
                None,
                false,
                counter,
                output,
                current_font,
                Some(dev_content_colors(
                    dev_variant_content(&props.style),
                    dev_scheme_title(&props.style),
                )),
                context,
                children_method,
            );
        }
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
    let wide = dev_side_nav_wide(props, context);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&format!(
        "        if ({wide}) {{ {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT)); }}\n"
    ));
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    if dev_side_nav_can_use_data_renderer(items) {
        render_dev_android_side_nav_data(
            props,
            items,
            &view,
            output,
            current_font,
            context,
        );
        return;
    }
    for (index, item) in items.iter().enumerate() {
        let renderer = format!("{view}Item{index}");
        let item_parent = format!("{renderer}Parent");
        output.push_str(&format!(
            "        Consumer<LinearLayout> {renderer} = {item_parent} -> {{\n"
        ));
        render_dev_android_side_nav_item(
            item,
            &item_parent,
            props,
            &wide,
            counter,
            output,
            current_font,
            context,
            &format!("{}:{index}", side_nav_memory_key(props, items)),
        );
        output.push_str(&format!(
            "        }};\n        {renderer}.accept({view});\n"
        ));
    }
}

fn render_dev_android_side_nav_data(
    props: &SideNavProps,
    items: &[SideNavItem],
    parent: &str,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let reactive_text = |path: &str| format!("doweTextValue(\"{}\", null)", escape_java(&context.signal_path(path)));
    let variant = props.style.reactive.variant.as_ref().map(|path| reactive_text(path));
    let scheme = props.style.reactive.scheme.as_ref().map(|path| reactive_text(path));
    let size = props.style.reactive.size.as_ref().map(|path| reactive_text(path));
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) = if let Some(size) = size.as_ref() {
        (
            format!("doweSideNavMetric({size}, 8, 12, 16)"),
            format!("doweSideNavMetric({size}, 6, 8, 12)"),
            format!("doweSideNavMetric({size}, 8, 10, 12)"),
            format!("doweSideNavMetric({size}, 12, 14, 16)"),
            format!("doweSideNavMetric({size}, 10, 12, 14)"),
        )
    } else {
        let values = compose_side_nav_metrics(props.size);
        (values.0.to_string(), values.1.to_string(), values.2.to_string(), values.3.to_string(), values.4.to_string())
    };
    let container = match (&variant, &scheme) { (None, None) => dev_variant_container(&props.style).to_string(), _ => format!("doweButtonContainer({}, {})", variant.as_deref().unwrap_or("\"ghost\""), scheme.as_deref().unwrap_or("\"muted\"")) };
    let content = match (&variant, &scheme) { (None, None) => dev_nav_active_content(&props.style).to_string(), _ => format!("doweButtonContent({}, {})", variant.as_deref().unwrap_or("\"ghost\""), scheme.as_deref().unwrap_or("\"muted\"")) };
    let title = match (&variant, &scheme) { (None, None) => dev_side_nav_header_content(&props.style).to_string(), _ => format!("doweSideNavHeaderColor({})", scheme.as_deref().unwrap_or("\"muted\"")) };
    let wide = dev_side_nav_wide(props, context);
    let entries = dev_side_nav_entries(items);
    output.push_str(&format!(
        "        doweRenderSideNav({parent}, {entries}, \"{}\", {wide}, {padding_horizontal}, {padding_vertical}, {gap}, {label_size}, {description_size}, {}, {}, {}, {});\n",
        escape_java(&side_nav_memory_key(props, items)),
        container,
        content,
        title,
        dev_font_value(inherited_font)
    ));
}

fn dev_side_nav_wide(props: &SideNavProps, context: &ComposeReactiveContext) -> String {
    props
        .reactive_wide
        .as_ref()
        .map(|path| {
            format!(
                "doweBool(\"{}\", null)",
                escape_java(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| props.wide.to_string())
}

fn dev_side_nav_can_use_data_renderer(items: &[SideNavItem]) -> bool {
    items.iter().all(|item| match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => {
            dev_side_nav_item_can_use_data_renderer(props, true)
        }
        SideNavItem::Divider => true,
        SideNavItem::Submenu { props, items, .. } => {
            dev_side_nav_item_can_use_data_renderer(props, false)
                && items
                    .iter()
                    .all(|item| dev_side_nav_item_can_use_data_renderer(item, true))
        }
    })
}

fn dev_side_nav_item_can_use_data_renderer(
    props: &SideNavItemProps,
    allow_navigation: bool,
) -> bool {
    props.icon.is_none()
        && props.on_click.is_none()
        && (allow_navigation || props.navigation.is_none())
        && props
            .navigation
            .as_ref()
            .is_none_or(dev_side_nav_navigation_supported)
}

fn dev_side_nav_navigation_supported(action: &NavigationAction) -> bool {
    matches!(action, NavigationAction::Internal { .. })
}

fn dev_side_nav_entries(items: &[SideNavItem]) -> String {
    let mut output = "new ArrayList<DoweSideNavEntry>() {{".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("item-{index}");
        output.push_str(&format!(" add({});", dev_side_nav_entry(item, &id)));
    }
    output.push_str(" }}");
    output
}

fn dev_side_nav_submenu_child_entries(items: &[SideNavItemProps], prefix: &str) -> String {
    let mut output = "new ArrayList<DoweSideNavEntry>() {{".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("{prefix}-{index}");
        output.push_str(&format!(
            " add({});",
            dev_side_nav_entry_props("item", item, false, false, "null", &id)
        ));
    }
    output.push_str(" }}");
    output
}

fn dev_side_nav_entry(item: &SideNavItem, id: &str) -> String {
    match item {
        SideNavItem::Header(props) => {
            dev_side_nav_entry_props("header", props, false, false, "null", id)
        }
        SideNavItem::Item(props) => dev_side_nav_entry_props("item", props, false, false, "null", id),
        SideNavItem::Divider => format!(
            "new DoweSideNavEntry(\"{}\", \"divider\", \"\", null, null, null, null, null, false, false, null)",
            escape_java(id)
        ),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            let children = dev_side_nav_submenu_child_entries(items, id);
            dev_side_nav_entry_props("submenu", props, *open, *bordered, &children, id)
        }
    }
}

fn dev_side_nav_entry_props(
    kind: &str,
    props: &SideNavItemProps,
    open: bool,
    bordered: bool,
    children: &str,
    id: &str,
) -> String {
    let (operation, path, fragment) = dev_side_nav_navigation_values(props.navigation.as_ref());
    format!(
        "new DoweSideNavEntry(\"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {}, {})",
        escape_java(id),
        kind,
        dev_localized_literal(&props.label, props.i18n.as_deref()),
        props.description.as_deref().map(|value| dev_localized_literal(value, props.description_i18n.as_deref())).unwrap_or_else(|| "null".to_string()),
        props.status.as_deref().map(|value| dev_localized_literal(value, props.status_i18n.as_deref())).unwrap_or_else(|| "null".to_string()),
        dev_side_nav_optional_string(operation),
        dev_side_nav_optional_string(path),
        dev_side_nav_optional_string(fragment),
        open,
        bordered,
        children
    )
}

fn dev_side_nav_navigation_values(
    action: Option<&NavigationAction>,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    match action {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => (
            Some(operation.as_str()),
            Some(path.as_str()),
            fragment.as_deref(),
        ),
        _ => (None, None, None),
    }
}

fn dev_side_nav_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_java(value)))
        .unwrap_or_else(|| "null".to_string())
}

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
