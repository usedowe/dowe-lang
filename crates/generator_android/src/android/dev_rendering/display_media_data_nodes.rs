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
                                        "        TextView {view} = doweText(\"▶ {}\", {}, 14f, 500, 0f, 1.2f, {});\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {view}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS));\n",
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
            output.push_str(&format!(
                                        "        FrameLayout {view} = doweImage(\"{}\", \"{}\", \"{}\", \"{}\", {}, {});\n",
                                        escape_java(&props.src),
                                        escape_java(&props.alt),
                                        props.aspect.as_str(),
                                        props.object_fit.as_str(),
                                        dev_card_variant_container(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Accordion { props, items } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let content_color = dev_card_variant_content(&props.style);
            let current_color = Some(content_color.to_string());
            let radius = dev_style_radius(&props.style.style);
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweAccordion({}, {}, {}, {}, {radius});\n",
                                        props.multiple,
                                        dev_card_variant_container(&props.style),
                                        content_color,
                                        dev_card_border(&props.style),
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for item in items {
                let arrow = side_nav_submenu_arrow_icon();
                let arrow_view = render_dev_android_icon_view(
                    &arrow,
                    counter,
                    output,
                    Some(content_color),
                );
                let body = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {body} = doweAccordionItem({view}, \"{}\", {}, {}, {}, {arrow_view});\n",
                    escape_java(&item.label),
                    item.disabled,
                    item.default_open,
                    dev_font_value(current_font),
                ));
                for child in &item.children {
                    render_dev_android_node(
                        child,
                        &body,
                        Some("8"),
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
        ViewNode::Carousel { props, slides } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground(Color.TRANSPARENT, DOWE_RADIUS));\n"
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
            let scroll = next_dev_view(counter);
            let track = next_dev_view(counter);
            let horizontal = props.orientation == CarouselOrientation::Horizontal;
            if horizontal {
                output.push_str(&format!(
                    "        android.widget.HorizontalScrollView {scroll} = new android.widget.HorizontalScrollView(this);\n        {scroll}.setFillViewport(false);\n        {scroll}.setHorizontalScrollBarEnabled(false);\n        {scroll}.setOverScrollMode(View.OVER_SCROLL_NEVER);\n        {scroll}.setNestedScrollingEnabled(true);\n        LinearLayout {track} = doweContainer(true);\n        {track}.setGravity(Gravity.CENTER_VERTICAL);\n"
                ));
                if props.variant == CarouselVariant::Rtl {
                    output.push_str(&format!(
                        "        {track}.setLayoutDirection(View.LAYOUT_DIRECTION_RTL);\n"
                    ));
                }
                output.push_str(&format!(
                    "        {scroll}.addView({track}, new android.widget.HorizontalScrollView.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        doweAdd({view}, {scroll});\n"
                ));
            } else {
                output.push_str(&format!(
                    "        ScrollView {scroll} = new ScrollView(this);\n        {scroll}.setFillViewport(false);\n        {scroll}.setVerticalScrollBarEnabled(false);\n        {scroll}.setOverScrollMode(View.OVER_SCROLL_NEVER);\n        LinearLayout {track} = doweContainer(false);\n        {scroll}.addView({track}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        doweAdd({view}, {scroll});\n"
                ));
            }
            let slide_width = props.slide_width.unwrap_or(if matches!(
                props.variant,
                CarouselVariant::Simple
                    | CarouselVariant::Masonry
                    | CarouselVariant::Rtl
                    | CarouselVariant::Sticky
            ) {
                280
            } else {
                320
            });
            for slide in slides {
                let slide_view = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {slide_view} = doweContainer(false);\n        {slide_view}.setClipToPadding(false);\n"
                ));
                if horizontal {
                    output.push_str(&format!(
                        "        {slide_view}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({slide_width}), ViewGroup.LayoutParams.WRAP_CONTENT));\n"
                    ));
                }
                if let Some(height) = props.slide_height {
                    output.push_str(&format!(
                        "        {slide_view}.setMinimumHeight(doweDp({height}));\n"
                    ));
                }
                output.push_str(&format!(
                    "        doweAdd({track}, {slide_view}, doweDp({}), {});\n",
                    props.gap,
                    horizontal
                ));
                for child in &slide.children {
                    render_dev_android_node(
                        child,
                        &slide_view,
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
            if horizontal
                && !matches!(
                    props.variant,
                    CarouselVariant::Simple
                        | CarouselVariant::Masonry
                        | CarouselVariant::Rtl
                        | CarouselVariant::Sticky
                )
            {
                output.push_str(&format!(
                    "        {scroll}.setOnTouchListener((target, event) -> {{\n            if (event.getAction() == android.view.MotionEvent.ACTION_UP || event.getAction() == android.view.MotionEvent.ACTION_CANCEL) {{\n                int step = doweDp({});\n                int page = Math.round((float) {scroll}.getScrollX() / Math.max(1, step));\n                {scroll}.post(() -> {scroll}.smoothScrollTo(page * step, 0));\n            }}\n            return false;\n        }});\n",
                    slide_width.saturating_add(props.gap)
                ));
            }
        }
        ViewNode::Code { props } => {
            let view = next_dev_view(counter);
            let source = if props.template_segments.is_empty() {
                format!("\"{}\"", escape_java(&props.source))
            } else {
                props.template_segments.iter().map(|segment| match segment {
                    CodeTemplateSegment::Static { text, .. } => format!("\"{}\"", escape_java(text)),
                    CodeTemplateSegment::Binding(path) => format!("doweTextValue(\"{}\", null)", escape_java(&context.signal_path(path))),
                }).collect::<Vec<_>>().join(" + ")
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
            let scripts = props.sandbox.as_ref().map(|tokens| tokens.iter().any(|token| token == "scripts")).unwrap_or(true);
            output.push_str(&format!(
                "        FrameLayout {view} = doweIframe(\"{}\", \"{}\", {}, {});\n",
                escape_java(&props.src),
                escape_java(&props.title),
                scripts,
                props.allow.iter().any(|token| token == "autoplay"),
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            if props.style.border.is_some() {
                output.push_str(&format!("        {view}.setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));\n"));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Device { props, iframe } => {
            let view = next_dev_view(counter);
            let scripts = iframe.sandbox.as_ref().map(|tokens| tokens.iter().any(|token| token == "scripts")).unwrap_or(true);
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
                    "        DoweSvgView {icon_name} = new DoweSvgView(this, {}f, {}f, {}f, {}f, DOWE_ON_BACKGROUND, {paths_name});\n",
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
                "        FrameLayout {view} = doweDevice(\"{}\", \"{}\", \"{}\", {}, {}, new DoweDeviceOption[] {{{}}});\n",
                props.device.as_str(),
                escape_java(&iframe.src),
                escape_java(&iframe.title),
                scripts,
                iframe.allow.iter().any(|token| token == "autoplay"),
                options.join(", "),
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            if props.style.border.is_some() {
                output.push_str(&format!("        {view}.setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));\n"));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Canvas { props } => {
            let view = next_dev_view(counter);
            let on_pointer = props.on_pointer.as_deref().and_then(|value| context.action_id(value)).map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string());
            let on_key = props.on_key.as_deref().and_then(|value| context.action_id(value)).map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string());
            let on_motion = props.on_motion.as_deref().and_then(|value| context.action_id(value)).map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string());
            let background = match props.background {
                CanvasBackground::Transparent => "Color.TRANSPARENT".to_string(),
                CanvasBackground::Color(color) => java_color(color).to_string(),
            };
            let border_width = props
                .style
                .border
                .as_ref()
                .map(dev_border_value)
                .unwrap_or_else(|| "null".to_string());
            let border_color = props
                .style
                .border_color
                .map(family_color)
                .map(java_color)
                .unwrap_or("DOWE_ON_BACKGROUND");
            output.push_str(&format!(
                "        DoweCanvasView {view} = doweCanvas(\"{}\", {}f, {}f, \"{}\", {}, {}, {}, {}, \"{}\", {on_pointer}, {on_key}, {on_motion}, {}, {border_width}, {border_color}, {});\n",
                escape_java(&context.signal_path(&props.scene)),
                props.view_width,
                props.view_height,
                props.fit.as_str(),
                props.fps,
                props.autoplay,
                props.pixelated,
                background,
                escape_java(&props.label),
                props.motion_rate,
                dev_style_radius(&props.style),
            ));
            apply_dev_android_style(&props.style, &view, false, output);
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
        ViewNode::ArcChart { props } => {
            render_dev_android_chart(
                "arc",
                &props.common,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::AreaChart { props } => {
            render_dev_android_chart(
                "area",
                &props.common,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::BarChart { props } => {
            render_dev_android_chart(
                "bar",
                &props.common,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::LineChart { props } => {
            render_dev_android_chart(
                "line",
                &props.common,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::PieChart { props } => {
            render_dev_android_chart(
                "pie",
                &props.common,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
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

fn render_dev_android_chart(
    chart_type: &str,
    props: &ChartCommonProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    context: &ComposeReactiveContext,
    output: &mut String,
) {
    let view = next_dev_view(counter);
    let data_path = props
        .data
        .as_deref()
        .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string());
    let series_path = props
        .series
        .as_deref()
        .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string());
    output.push_str(&format!(
        "        DoweChartView {view} = doweChart(\"{}\", {data_path}, {series_path}, \"{}\", \"{}\", \"{}\", {}, {}, {}, {}, {});\n",
        escape_java(chart_type),
        props.palette.as_str(),
        props.legend_position.as_str(),
        escape_java(&props.empty_label),
        props.loading,
        props.hide_legend,
        dev_card_variant_container(&props.style),
        dev_card_variant_content(&props.style),
        dev_card_border(&props.style)
    ));
    apply_dev_android_style(&props.style.style, &view, false, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
