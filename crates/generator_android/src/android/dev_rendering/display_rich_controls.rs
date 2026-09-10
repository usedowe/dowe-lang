fn dev_tree_icon_spec(name: &str, paths_name: &str, icon_name: &str) -> String {
    let icon = solar_control_icon(name).expect("bundled Tree icon");
    let mut output = format!(
        "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
    );
    for path in &icon.paths {
        output.push_str(&format!(
            "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}, {}, {}));\n",
            escape_java(&path.data),
            dev_svg_path_current_color(path.fill),
            dev_svg_path_color(path.fill),
            dev_svg_path_details(path.fill),
            dev_svg_path_transform(path.transform.as_ref()),
        ));
    }
    output.push_str(&format!(
        "        DoweTreeIcon {icon_name} = new DoweTreeIcon({}f, {}f, {}f, {}f, {paths_name});\n",
        icon.props.view_box.min_x,
        icon.props.view_box.min_y,
        icon.props.view_box.width,
        icon.props.view_box.height,
    ));
    output
}

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
            let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
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
                    current_color.clone(),
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
                dev_svg_color(&props.style, inherited_color.as_deref()),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::RichText { props, marks } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        DoweFlexLayout {view} = doweFlex(DOWE_DIRECTION_ROW, true, DOWE_JUSTIFY_CENTER, DOWE_ALIGN_CENTER, 4);\n"
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for mark in marks {
                let mark_view = next_dev_view(counter);
                output.push_str(&format!(
                    "        TextView {mark_view} = doweRichTextView(\"{}\", {}, {}, {}, {}, {}, {});\n        doweRichTextMark({mark_view}, \"{}\", \"{}\");\n        doweAdd({view}, {mark_view});\n",
                    escape_java(&mark.text),
                    dev_text_color(props.title, props, inherited_color.as_deref()),
                    dev_text_size(props.title, props),
                    dev_text_weight(props.title, props),
                    dev_text_spacing(props.title, props),
                    dev_text_line_height(props.title, props),
                    dev_font_value(props.style.font.as_ref().or(inherited_font)),
                    mark.style.as_str(),
                    mark.color.as_str(),
                ));
            }
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
            if !props.wide {
                output.push_str(&format!("        doweWrapContentWidth({view});\n"));
            }
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
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
            let selected_value = if path.is_empty() {
                format!("\"{}\"", escape_java(&props.selected))
            } else {
                format!("doweTextValue(\"{path}\", null)")
            };
            for (index, item) in items.iter().enumerate() {
                let button = next_dev_view(counter);
                let is_first = index == 0;
                output.push_str(&format!(
                                            "        String {button}Value = {selected_value};\n        boolean {button}Active = \"{}\".equals({button}Value) || ({button}Value.isEmpty() && {is_first});\n        TextView {button} = doweText(\"{}\", {button}Active ? {} : {}, {}, 600, 0f, 1.2f, {});\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setPadding(doweDp({}), 0, doweDp({}), 0);\n        {button}.setBackground(doweBackground({button}Active ? {} : Color.TRANSPARENT, DOWE_RADIUS));\n        {button}.setEnabled({});\n",
                                            escape_java(&item.id),
                                            escape_java(&item.label),
                                            dev_variant_container(&props.style),
                                            dev_variant_content(&props.style),
                                            match props.size {
                                                ButtonSize::Xs => "12f",
                                                ButtonSize::Sm => "13f",
                                                ButtonSize::Lg => "18f",
                                                _ => "14f",
                                            },
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                            match props.size {
                                                ButtonSize::Xs => 8,
                                                ButtonSize::Sm => 10,
                                                ButtonSize::Lg => 16,
                                                _ => 12,
                                            },
                                            match props.size {
                                                ButtonSize::Xs => 8,
                                                ButtonSize::Sm => 10,
                                                ButtonSize::Lg => 16,
                                                _ => 12,
                                            },
                                            dev_variant_content(&props.style),
                                            !props.disabled,
                                        ));
                let write = if path.is_empty() {
                    String::new()
                } else {
                    format!("doweWrite(\"{path}\", \"{}\"); ", escape_java(&item.id))
                };
                if !props.disabled && (!path.is_empty() || !action.is_empty()) {
                    output.push_str(&format!(
                        "        {button}.setOnClickListener(v -> {{ if (!{button}Active) {{ {write}{action}renderCurrentRoute(false); }} }});\n"
                    ));
                }
                let button_width = if props.wide && horizontal {
                    "0"
                } else if props.wide {
                    "ViewGroup.LayoutParams.MATCH_PARENT"
                } else {
                    "ViewGroup.LayoutParams.WRAP_CONTENT"
                };
                let button_height = match props.size {
                    ButtonSize::Xs => 24,
                    ButtonSize::Sm => 32,
                    ButtonSize::Lg => 48,
                    _ => 40,
                };
                output.push_str(&format!(
                    "        LinearLayout.LayoutParams {button}Params = new LinearLayout.LayoutParams({button_width}, doweDp({button_height}));\n        {button}Params.setMargins(doweDp(4), 0, 0, 0);\n{}        {view}.addView({button}, {button}Params);\n",
                    if props.wide && horizontal {
                        format!("        {button}Params.weight = 1f;\n")
                    } else {
                        String::new()
                    }
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
            let on_complete = props
                .on_complete
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| format!("() -> doweRunAction(\"{}\", null)", escape_java(id)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                                        "        HorizontalScrollView {view} = doweCountdown(\"{}\", {}, {}, {}, {}, \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", {}, {}, null, {}, {on_complete});\n",
                                        escape_java(&props.target),
                                        props.show_days,
                                        props.show_hours,
                                        props.show_minutes,
                                        props.show_seconds,
                                        props.size.as_str(),
                                        escape_java(&props.days_label),
                                        escape_java(&props.hours_label),
                                        escape_java(&props.minutes_label),
                                        escape_java(&props.seconds_label),
                                        dev_variant_container(&props.style),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Tree { props } => {
            let view = next_dev_view(counter);
            let border = if props.style.variant == Some(ComponentVariant::Outlined) {
                format!("{}", dev_variant_content(&props.style))
            } else {
                "null".to_string()
            };
            let arrow_paths = format!("{view}TreeArrowPaths");
            let folder_paths = format!("{view}TreeFolderPaths");
            let file_paths = format!("{view}TreeFilePaths");
            let arrow_icon = format!("{view}TreeArrow");
            let folder_icon = format!("{view}TreeFolder");
            let file_icon = format!("{view}TreeFile");
            output.push_str(&dev_tree_icon_spec("alt-arrow-down", &arrow_paths, &arrow_icon));
            output.push_str(&dev_tree_icon_spec("folder-with-files", &folder_paths, &folder_icon));
            output.push_str(&dev_tree_icon_spec("file-text", &file_paths, &file_icon));
            output.push_str(&format!(
                "        LinearLayout {view} = doweTree(\"{}\", {}, {}, \"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {});\n",
                escape_java(&context.signal_path(&props.data)),
                props.bind.as_deref().map(|path| format!("\"{}\"", escape_java(&context.signal_path(path)))).unwrap_or_else(|| "null".to_string()),
                props.default_open,
                escape_java(&props.empty_label),
                escape_java(&props.aria_label),
                props.on_select.as_deref().and_then(|value| context.action_id(value)).map(|value| format!("\"{}\"", escape_java(&value))).unwrap_or_else(|| "null".to_string()),
                dev_variant_container(&props.style),
                dev_variant_content(&props.style),
                border,
                dev_style_radius(&props.style.style),
                folder_icon,
                file_icon,
                arrow_icon,
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

