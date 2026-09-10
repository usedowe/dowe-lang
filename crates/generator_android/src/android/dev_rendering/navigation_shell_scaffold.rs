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
    let container = match (&variant, &scheme) { (None, None) => dev_variant_container(&props.style).to_string(), _ => format!("doweButtonContainer({}, {})", variant.as_deref().unwrap_or("\"solid\""), scheme.as_deref().unwrap_or("\"primary\"")) };
    let content = match (&variant, &scheme) { (None, None) => dev_nav_active_content(&props.style).to_string(), _ => format!("doweButtonContent({}, {})", variant.as_deref().unwrap_or("\"solid\""), scheme.as_deref().unwrap_or("\"primary\"")) };
    let title = dev_side_nav_header_content(&props.style).to_string();
    let wide = dev_side_nav_wide(props, context);
    let entries = dev_side_nav_entries(items);
    if let Some(path) = props.style.reactive.variant.as_ref() {
        output.push_str(&format!("        {parent}.setTag(DOWE_VARIANT_TAG, \"{}\");\n", escape_java(path)));
    }
    if let Some(path) = props.style.reactive.scheme.as_ref() {
        output.push_str(&format!("        {parent}.setTag(DOWE_SCHEME_TAG, \"{}\");\n", escape_java(path)));
    }
    if let Some(path) = props.style.reactive.size.as_ref() {
        output.push_str(&format!("        {parent}.setTag(DOWE_SIZE_TAG, \"{}\");\n", escape_java(path)));
    }
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

