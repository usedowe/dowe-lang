fn render_dev_android_display_rich_controls_node(
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
        ViewNode::ChatBox { props } => {
            render_dev_android_variant_label(
                "Chat",
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Empty { props } => {
            let label = props
                .title
                .as_deref()
                .unwrap_or_else(|| match props.kind.as_str() {
                    "playlist" => "No playlist items",
                    "result" => "No results",
                    "template" => "No templates",
                    _ => "No data",
                });
            render_dev_android_variant_label(
                label,
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Marquee { props, children } => {
            let view = next_dev_view(counter);
            let horizontal = props.orientation.as_str() == "horizontal";
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer({});\n",
                horizontal
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    Some("doweDp(8)"),
                    horizontal,
                    counter,
                    output,
                    current_font,
                    inherited_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::TypeWriter { props, items } => {
            let view = next_dev_view(counter);
            let text = items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            output.push_str(&format!(
                "        TextView {view} = doweText(\"{}\", {}, 14f, 500, 0f, 1.2f, {});\n",
                escape_java(&text),
                inherited_color.as_deref().unwrap_or("DOWE_ON_BACKGROUND"),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::RichText { props, marks } => {
            let view = next_dev_view(counter);
            let text = marks
                .iter()
                .map(|mark| mark.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            output.push_str(&format!(
                "        TextView {view} = doweText(\"{}\", {}, {}, {}, {}, {}, {});\n",
                escape_java(&text),
                dev_text_color(props, inherited_color.as_deref()),
                dev_text_size(false, props),
                dev_text_weight(false, props),
                dev_text_spacing(false, props),
                dev_text_line_height(false, props),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Record { props } => {
            render_dev_android_variant_label(
                props.style.label.as_deref().unwrap_or(&props.name),
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::ToggleGroup { props, items } => {
            if props.kind == ToggleGroupKind::Pagination {
                render_dev_android_pagination(
                    props,
                    items,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    context,
                );
                return;
            }
            let view = next_dev_view(counter);
            let horizontal = !props.vertical;
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer({horizontal});\n        {view}.setPadding(doweDp(4), doweDp(4), doweDp(4), doweDp(4));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n",
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for item in items {
                let button = next_dev_view(counter);
                let active = item.id == props.selected;
                output.push_str(&format!(
                                            "        TextView {button} = doweText(\"{}\", {}, 14f, 600, 0f, 1.2f, {});\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {button}.setBackground(doweBackground({}, DOWE_RADIUS));\n",
                                            escape_java(&item.label),
                                            if active { dev_variant_container(&props.style) } else { dev_variant_content(&props.style) },
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                            if active { dev_variant_content(&props.style) } else { "Color.TRANSPARENT" }
                                        ));
                if let Some(action) = props
                    .on_change
                    .as_deref()
                    .and_then(|name| context.action_id(name))
                {
                    output.push_str(&format!(
                        "        {button}.setOnClickListener(v -> doweRunAction(\"{}\", null));\n",
                        escape_java(action)
                    ));
                }
                output.push_str(&format!(
                    "        doweAdd({view}, {button}, doweDp(4), {});\n",
                    if horizontal { "true" } else { "false" }
                ));
            }
        }
        ViewNode::Collapsible { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n",
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            render_dev_android_variant_label(
                &props.label,
                &props.style,
                &view,
                None,
                false,
                counter,
                output,
                current_font,
                context,
            );
            if props.default_open {
                for child in children {
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
        }
        ViewNode::Countdown { props } => {
            let view = next_dev_view(counter);
            let parts = [
                (props.show_days, props.days_label.as_str()),
                (props.show_hours, props.hours_label.as_str()),
                (props.show_minutes, props.minutes_label.as_str()),
                (props.show_seconds, props.seconds_label.as_str()),
            ]
            .iter()
            .filter_map(|(show, label)| show.then_some(*label))
            .collect::<Vec<_>>()
            .join("  ");
            output.push_str(&format!(
                                        "        TextView {view} = doweText(\"00 {}\", {}, 18f, 700, 0f, 1.2f, {});\n        {view}.setPadding(doweDp(12), doweDp(10), doweDp(12), doweDp(10));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n",
                                        escape_java(&parts),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Map { props, markers, .. } => {
            let view = next_dev_view(counter);
            let label = markers
                .iter()
                .filter_map(|marker| marker.label.as_deref().or(marker.popup.as_deref()))
                .collect::<Vec<_>>()
                .join(" · ");
            let label = if label.is_empty() {
                format!("{}, {}", props.center_lat, props.center_lng)
            } else {
                label
            };
            output.push_str(&format!(
                                        "        TextView {view} = doweText(\"{}\", {}, 14f, 600, 0f, 1.2f, {});\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setPadding(doweDp(16), doweDp(40), doweDp(16), doweDp(40));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n",
                                        escape_java(&label),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::AvatarGroup { props, items } => {
            let view = next_dev_view(counter);
            let size = match props.size {
                ButtonSize::Xs => (24, 12),
                ButtonSize::Sm => (32, 14),
                ButtonSize::Lg => (48, 18),
                ButtonSize::Xl => (64, 24),
                ButtonSize::Md => (40, 16),
            };
            let sources = items
                .iter()
                .map(|item| format!("\"{}\"", escape_java(item.src.as_deref().unwrap_or_default())))
                .collect::<Vec<_>>()
                .join(", ");
            let names = items
                .iter()
                .map(|item| format!("\"{}\"", escape_java(item.name.as_deref().unwrap_or_default())))
                .collect::<Vec<_>>()
                .join(", ");
            let alts = items
                .iter()
                .map(|item| format!("\"{}\"", escape_java(item.alt.as_deref().unwrap_or_default())))
                .collect::<Vec<_>>()
                .join(", ");
            let data_path = props
                .items
                .as_deref()
                .map(|path| format!("\"{}\"", escape_java(&context.signal_path(path))))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        LinearLayout {view} = doweAvatarGroup({data_path}, new String[] {{{sources}}}, new String[] {{{names}}}, new String[] {{{alts}}}, {}, {}, {}, {}, {}, {}, {}, {}, {});\n",
                size.0,
                size.1,
                props.max.unwrap_or(0),
                props.inline,
                props.bordered,
                dev_variant_container(&props.style),
                dev_variant_content(&props.style),
                dev_variant_content(&props.style),
                dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}

fn render_dev_android_pagination(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let rendered_pages = items.len().max(1);
    let path = props
        .value
        .as_deref()
        .map(|value| escape_java(&context.signal_path(value)))
        .unwrap_or_default();
    let action = props
        .on_change
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|name| format!("doweRunAction(\"{}\", null); ", escape_java(name)))
        .unwrap_or_default();
    let dimension = match props.size {
        ButtonSize::Xs => 24,
        ButtonSize::Sm => 32,
        ButtonSize::Lg => 48,
        _ => 40,
    };
    let (total_setup, page_count) = props
        .pagination
        .as_ref()
        .map(|pagination| match &pagination.total {
            dowe_components::PaginationTotal::Static(total) => (
                String::new(),
                total.div_ceil(pagination.page_size).max(1).to_string(),
            ),
            dowe_components::PaginationTotal::Signal(total) => {
                let total = escape_java(&context.signal_path(total));
                let offset = pagination.page_size - 1;
                (
                    format!(
                        "        int {view}Total = 0;\n        try {{ {view}Total = Integer.parseInt(doweTextValue(\"{total}\", null)); }} catch (NumberFormatException ignored) {{}}\n"
                    ),
                    format!(
                        "Math.max(1, Math.min(25, (Math.max(0, {view}Total) + {offset}) / {}))",
                        pagination.page_size
                    ),
                )
            }
        })
        .unwrap_or_else(|| (String::new(), rendered_pages.to_string()));
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n{total_setup}        final int {view}Pages = {page_count};\n        int {view}Current = 1;\n        try {{ {view}Current = Integer.parseInt(doweTextValue(\"{path}\", null)); }} catch (NumberFormatException ignored) {{}}\n        {view}Current = Math.max(1, Math.min({view}Pages, {view}Current));\n        final int {view}Selected = {view}Current;\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    render_dev_android_pagination_arrow(
        props,
        "arrow-left",
        "Previous page",
        -1,
        &format!("{view}Pages"),
        dimension,
        &path,
        &action,
        &view,
        counter,
        output,
    );
    for page in 1..=rendered_pages {
        if rendered_pages > 7 && page == 2 {
            let ellipsis = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {ellipsis} = doweText(\"…\", doweAlpha(DOWE_ON_BACKGROUND, 0.6f), 14f, 400, 0f, 1.2f, null);\n        {ellipsis}.setGravity(Gravity.CENTER);\n        {ellipsis}.setVisibility({view}Selected > 3 ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {ellipsis}, doweDp(4), true);\n"
            ));
        }
        if rendered_pages > 7 && page == rendered_pages {
            let ellipsis = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {ellipsis} = doweText(\"…\", doweAlpha(DOWE_ON_BACKGROUND, 0.6f), 14f, 400, 0f, 1.2f, null);\n        {ellipsis}.setGravity(Gravity.CENTER);\n        {ellipsis}.setVisibility({view}Selected < {view}Pages - 2 ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {ellipsis}, doweDp(4), true);\n"
            ));
        }
        let button = next_dev_view(counter);
        let visible = format!(
            "{page} <= {view}Pages && ({view}Pages <= 7 || {page} == 1 || {page} == {view}Pages || Math.abs({page} - {view}Selected) <= 1)"
        );
        output.push_str(&format!(
            "        TextView {button} = doweText(\"{page}\", ({view}Selected == {page}) ? {} : DOWE_ON_BACKGROUND, 14f, 500, 0f, 1.2f, null);\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setContentDescription(\"Page {page}\");\n        {button}.setBackground(doweBackground(({view}Selected == {page}) ? {} : Color.TRANSPARENT, DOWE_RADIUS));\n        {button}.setVisibility({visible} ? View.VISIBLE : View.GONE);\n        {button}.setEnabled({});\n        {button}.setOnClickListener(v -> {{ if ({view}Selected != {page}) {{ doweWrite(\"{path}\", \"{page}\"); {action}renderCurrentRoute(false); }} }});\n        LinearLayout.LayoutParams {button}Params = new LinearLayout.LayoutParams(doweDp({dimension}), doweDp({dimension}));\n        {view}.addView({button}, {button}Params);\n",
            dev_variant_content(&props.style),
            dev_variant_container(&props.style),
            !props.disabled,
        ));
    }
    render_dev_android_pagination_arrow(
        props,
        "arrow-right",
        "Next page",
        1,
        &format!("{view}Pages"),
        dimension,
        &path,
        &action,
        &view,
        counter,
        output,
    );
}

fn render_dev_android_pagination_arrow(
    props: &ToggleGroupProps,
    icon_name: &str,
    label: &str,
    step: i32,
    pages: &str,
    dimension: u16,
    path: &str,
    action: &str,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
) {
    let button = next_dev_view(counter);
    let icon = solar_control_icon(icon_name).expect("bundled Pagination icon");
    let icon_view = render_dev_android_icon_view(
        &icon,
        counter,
        output,
        Some(dev_variant_content(&props.style)),
    );
    let enabled = if step < 0 {
        format!("{parent}Selected > 1")
    } else {
        format!("{parent}Selected < {pages}")
    };
    output.push_str(&format!(
        "        FrameLayout {button} = new FrameLayout(this);\n        {button}.setContentDescription(\"{label}\");\n        {button}.setFocusable(true);\n        {button}.setEnabled({} && {enabled});\n        {button}.setAlpha({enabled} ? 1f : 0.42f);\n        {button}.setBackground(doweBackground({}, DOWE_RADIUS));\n        {button}.setOnClickListener(v -> {{ int page = Math.max(1, Math.min({pages}, {parent}Selected + ({step}))); doweWrite(\"{path}\", String.valueOf(page)); {action}renderCurrentRoute(false); }});\n        {button}.addView({icon_view}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        LinearLayout.LayoutParams {button}Params = new LinearLayout.LayoutParams(doweDp({dimension}), doweDp({dimension}));\n        {button}Params.setMargins(doweDp(4), 0, 0, 0);\n        {parent}.addView({button}, {button}Params);\n",
        !props.disabled,
        dev_variant_container(&props.style),
    ));
}
