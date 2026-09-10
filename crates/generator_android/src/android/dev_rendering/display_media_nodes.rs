fn render_dev_android_media_display_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) -> bool {
    if !matches!(
        node,
        ViewNode::Code { .. }
            | ViewNode::Video { .. }
            | ViewNode::Iframe { .. }
            | ViewNode::Device { .. }
    ) {
        return false;
    }
    match node {
        ViewNode::Code { props } => {
            let view = next_dev_view(counter);
            let source = if props.template_segments.is_empty() {
                format!("\"{}\"", escape_java(&props.source))
            } else {
                props
                    .template_segments
                    .iter()
                    .map(|segment| match segment {
                        CodeTemplateSegment::Static { text, .. } => {
                            format!("\"{}\"", escape_java(text))
                        }
                        CodeTemplateSegment::Binding(path) => format!(
                            "doweTextValue(\"{}\", null)",
                            escape_java(&context.signal_path(path))
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            let (texts, colors) = if props.template_segments.is_empty() {
                (
                    java_string_array(props.tokens.iter().map(|token| token.text.as_str())),
                    java_int_array(props.tokens.iter().map(|token| {
                        dev_code_token_color(token.kind, dev_card_variant_content(&props.style))
                    })),
                )
            } else {
                let mut texts = Vec::new();
                let mut colors = Vec::new();
                for segment in &props.template_segments {
                    match segment {
                        CodeTemplateSegment::Static { tokens, .. } => {
                            for token in tokens {
                                texts.push(format!("\"{}\"", escape_java(&token.text)));
                                colors.push(dev_code_token_color(
                                    token.kind,
                                    dev_card_variant_content(&props.style),
                                ));
                            }
                        }
                        CodeTemplateSegment::Binding(path) => {
                            texts.push(format!(
                                "doweTextValue(\"{}\", null)",
                                escape_java(&context.signal_path(path))
                            ));
                            colors.push(dev_card_variant_content(&props.style).to_string());
                        }
                    }
                }
                (
                    format!("new String[]{{{}}}", texts.join(", ")),
                    format!("new int[]{{{}}}", colors.join(", ")),
                )
            };
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweCode({source}, \"{}\", {texts}, {colors}, \"{}\", \"{}\", {}, {}, {});\n",
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
            let play_icon = render_dev_android_icon_view(
                &solar_control_icon("play").expect("bundled Video play icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let pause_icon = render_dev_android_icon_view(
                &solar_control_icon("pause").expect("bundled Video pause icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let volume_icon = render_dev_android_icon_view(
                &solar_control_icon("volume-loud").expect("bundled Video volume icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let muted_icon = render_dev_android_icon_view(
                &solar_control_icon("volume-cross").expect("bundled Video muted icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let picture_in_picture_icon = render_dev_android_icon_view(
                &solar_control_icon("pip").expect("bundled Video picture-in-picture icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let fullscreen_icon = render_dev_android_icon_view(
                &solar_control_icon("full-screen").expect("bundled Video fullscreen icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let poster = props
                .poster
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        FrameLayout {view} = doweVideo(\"{}\", {poster}, {}, \"{}\", {}, {}, {play_icon}, {pause_icon}, {volume_icon}, {muted_icon}, {picture_in_picture_icon}, {fullscreen_icon});\n",
                escape_java(&props.src),
                props.autoplay,
                props.aspect.as_str(),
                dev_card_variant_container(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Iframe { props } => {
            let view = next_dev_view(counter);
            let scripts = props
                .sandbox
                .as_ref()
                .map(|tokens| tokens.iter().any(|token| token == "scripts"))
                .unwrap_or(true);
            output.push_str(&format!(
                "        FrameLayout {view} = doweIframe(\"{}\", \"{}\", {}, {});\n",
                escape_java(&props.src),
                escape_java(&props.title),
                scripts,
                props.allow.iter().any(|token| token == "autoplay"),
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            if props.style.border.is_some() {
                output.push_str(&format!(
                    "        {view}.setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));\n"
                ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Device { props, iframe } => {
            let view = next_dev_view(counter);
            let scripts = iframe
                .sandbox
                .as_ref()
                .map(|tokens| tokens.iter().any(|token| token == "scripts"))
                .unwrap_or(true);
            let mut options = Vec::new();
            for (index, option) in props.options.iter().enumerate() {
                let paths_name = format!("{view}DevicePaths{index}");
                let icon_name = format!("{view}DeviceIcon{index}");
                output.push_str(&format!(
                    "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
                ));
                for path in &option.icon.paths {
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
                    "        DoweSvgView {icon_name} = new DoweSvgView(this, {}f, {}f, {}f, {}f, DOWE_BACKGROUND_TEXT, {paths_name});\n",
                    option.icon.props.view_box.min_x,
                    option.icon.props.view_box.min_y,
                    option.icon.props.view_box.width,
                    option.icon.props.view_box.height,
                ));
                options.push(format!(
                    "new DoweDeviceOption(\"{}\", {icon_name})",
                    option.profile.as_str()
                ));
            }
            output.push_str(&format!(
                "        FrameLayout {view} = doweDevice(\"{}\", \"{}\", \"{}\", {}, {}, {}, new DoweDeviceOption[] {{{}}});\n",
                props.device.as_str(),
                escape_java(&iframe.src),
                escape_java(&iframe.title),
                scripts,
                iframe.allow.iter().any(|token| token == "autoplay"),
                props.hide_controls,
                options.join(", "),
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            if props.style.border.is_some() {
                output.push_str(&format!(
                    "        {view}.setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));\n"
                ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
    true
}
