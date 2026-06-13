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
            let view = next_dev_view(counter);
            let horizontal = !props.vertical;
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer({horizontal});\n        {view}.setPadding(doweDp(4), doweDp(4), doweDp(4), doweDp(4));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for item in items {
                let button = next_dev_view(counter);
                let active = item.id == props.selected;
                output.push_str(&format!(
                                            "        TextView {button} = doweText(\"{}\", {}, 14f, 600, 0f, 1.2f, {});\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {button}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
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
                                        "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
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
                                        "        TextView {view} = doweText(\"00 {}\", {}, 18f, 700, 0f, 1.2f, {});\n        {view}.setPadding(doweDp(12), doweDp(10), doweDp(12), doweDp(10));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
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
                                        "        TextView {view} = doweText(\"{}\", {}, 14f, 600, 0f, 1.2f, {});\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setPadding(doweDp(16), doweDp(40), doweDp(16), doweDp(40));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                                        escape_java(&label),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::AvatarGroup { props, items } => {
            let label = if items.is_empty() {
                "A".to_string()
            } else {
                format!("+{}", items.len())
            };
            render_dev_android_variant_label(
                &label,
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
        _ => {}
    }
}
