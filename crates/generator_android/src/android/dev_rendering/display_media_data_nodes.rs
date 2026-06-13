fn render_dev_android_display_media_data_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    match node {
        ViewNode::Audio { props } => {
            let view = next_dev_view(counter);
            let label = props.subtitle.as_deref().unwrap_or(&props.src);
            output.push_str(&format!(
                                        "        TextView {view} = doweText(\"▶ {}\", {}, 14f, 500, 0f, 1.2f, {});\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {view}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_BOX));\n",
                                        escape_java(label),
                                        dev_card_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        dev_card_variant_container(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Image { props } => {
            let view = next_dev_view(counter);
            let label = if props.alt.is_empty() {
                &props.src
            } else {
                &props.alt
            };
            output.push_str(&format!(
                                        "        TextView {view} = doweText(\"Image: {}\", {}, 14f, 500, 0f, 1.2f, {});\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setMinHeight(doweDp(160));\n        {view}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_BOX));\n",
                                        escape_java(label),
                                        dev_card_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        dev_card_variant_container(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Accordion { props, items } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for item in items {
                render_dev_android_variant_label(
                    &item.label,
                    &props.style,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    context,
                );
                if item.default_open {
                    for child in &item.children {
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
        }
        ViewNode::Carousel { props, slides } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground(Color.TRANSPARENT, DOWE_RADIUS_BOX));\n"
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            if let Some(title) = props.title.as_deref() {
                render_dev_android_variant_label(
                    title,
                    &props.style,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
            if let Some(slide) = slides.first() {
                for child in &slide.children {
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
        ViewNode::Code { props } => {
            let view = next_dev_view(counter);
            let texts = java_string_array(props.tokens.iter().map(|token| token.text.as_str()));
            let colors = java_int_array(props.tokens.iter().map(|token| {
                dev_code_token_color(token.kind, dev_card_variant_content(&props.style))
            }));
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweCode(\"{}\", \"{}\", {texts}, {colors}, \"{}\", \"{}\", {}, {}, {});\n",
                                        escape_java(&props.source),
                                        props.language.as_str(),
                                        escape_java(&props.copy_label),
                                        escape_java(&props.copied_label),
                                        dev_card_variant_container(&props.style),
                                        dev_card_variant_content(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Video { props } => {
            let view = next_dev_view(counter);
            let poster = props
                .poster
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        FrameLayout {view} = doweVideo(\"{}\", {poster}, {}, \"{}\", {}, {});\n",
                escape_java(&props.src),
                props.autoplay,
                props.aspect.as_str(),
                dev_card_variant_container(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Candlestick { props } => {
            let view = next_dev_view(counter);
            let stream = props
                .stream
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                                        "        DoweCandlestickView {view} = doweCandlestick(\"{}\", {stream}, {}, {}, \"{}\", {}, {}, {}, {});\n",
                                        escape_java(&context.signal_path(&props.data)),
                                        java_color(props.up_color),
                                        java_color(props.down_color),
                                        escape_java(&props.empty_label),
                                        props.max_points,
                                        dev_card_variant_container(&props.style),
                                        dev_card_variant_content(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Table { props } => {
            let view = next_dev_view(counter);
            render_dev_android_table(props, &view, &context.signal_path(&props.data), output);
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}
