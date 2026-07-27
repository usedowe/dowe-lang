fn render_compose_display_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match node {
        ViewNode::Audio { props } => {
            render_compose_audio(props, indent, output);
        }
        ViewNode::Image { props } => {
            render_compose_image(props, indent, output);
        }
        ViewNode::Accordion { props, items } => {
            render_compose_accordion(
                props,
                items,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Carousel { props, slides } => {
            render_compose_carousel(
                props,
                slides,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Code { props } => {
            let source = if props.template_segments.is_empty() {
                compose_string_literal(&props.source)
            } else {
                props.template_segments.iter().map(|segment| match segment {
                    CodeTemplateSegment::Static { text, .. } => compose_string_literal(text),
                    CodeTemplateSegment::Binding(path) => format!("state.text(\"{}\", \"\")", escape_kotlin(&context.signal_path(path))),
                }).collect::<Vec<_>>().join(" + ")
            };
            let tokens = if props.template_segments.is_empty() {
                compose_code_tokens(&props.tokens, card_variant_content(&props.style))
            } else {
                props
                    .template_segments
                    .iter()
                    .map(|segment| match segment {
                        CodeTemplateSegment::Static { tokens, .. } => {
                            compose_code_tokens(tokens, card_variant_content(&props.style))
                        }
                        CodeTemplateSegment::Binding(path) => format!(
                            "listOf(DoweCodeToken(text = state.text(\"{}\", \"\"), color = {}))",
                            escape_kotlin(&context.signal_path(path)),
                            card_variant_content(&props.style)
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            let border = if props.style.variant.unwrap_or(ComponentVariant::Soft)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                        "{pad}DoweCode(source = {}, language = {}, tokens = {}, copyLabel = {}, copiedLabel = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        source,
                        compose_string_literal(props.language.as_str()),
                        tokens,
                        compose_string_literal(&props.copy_label),
                        compose_string_literal(&props.copied_label),
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        card_variant_container(&props.style),
                        card_variant_content(&props.style),
                    ));
        }
        ViewNode::Video { props } => {
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
            } else {
                "null"
            };
            let icons = compose_video_icons();
            output.push_str(&format!(
                        "{pad}DoweVideo(source = {}, poster = {}, autoplay = {}, aspect = {}, icons = {icons}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, borderColor = {border})\n",
                        compose_string_literal(&props.src),
                        compose_optional_string(props.poster.as_deref()),
                        props.autoplay,
                        compose_string_literal(props.aspect.as_str()),
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        card_variant_container(&props.style),
                    ));
        }
        ViewNode::Iframe { props } => {
            let sandbox = props.sandbox.as_ref().map(|tokens| {
                format!("listOf({})", tokens.iter().map(|token| compose_string_literal(token)).collect::<Vec<_>>().join(", "))
            }).unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "{pad}DoweIframe(source = {}, title = {}, sandbox = {sandbox}, autoplay = {}, modifier = {}, shape = RoundedCornerShape({}))\n",
                compose_string_literal(&props.src),
                compose_string_literal(&props.title),
                props.allow.iter().any(|token| token == "autoplay"),
                modifier_for_style(&props.style),
                compose_card_radius(&props.style),
            ));
        }
        ViewNode::Device { props, iframe } => {
            let sandbox = iframe.sandbox.as_ref().map(|tokens| {
                format!("listOf({})", tokens.iter().map(|token| compose_string_literal(token)).collect::<Vec<_>>().join(", "))
            }).unwrap_or_else(|| "null".to_string());
            let icons = props.options.iter().map(|option| {
                format!(
                    "DoweDeviceIcon(profile = {}, viewBox = {}, paths = {})",
                    compose_string_literal(option.profile.as_str()),
                    compose_svg_view_box(&option.icon.props.view_box),
                    compose_svg_paths(&option.icon.paths),
                )
            }).collect::<Vec<_>>().join(", ");
            output.push_str(&format!(
                "{pad}DoweDevicePreview(initialProfile = {}, source = {}, title = {}, sandbox = {sandbox}, autoplay = {}, icons = listOf({icons}), modifier = {})\n",
                compose_string_literal(props.device.as_str()),
                compose_string_literal(&iframe.src),
                compose_string_literal(&iframe.title),
                iframe.allow.iter().any(|token| token == "autoplay"),
                modifier_for_style(&props.style),
            ));
        }
        ViewNode::Canvas { props } => {
            let background = match props.background {
                CanvasBackground::Transparent => "Color.Transparent".to_string(),
                CanvasBackground::Color(color) => color_ref(color).to_string(),
            };
            output.push_str(&format!(
                "{pad}DoweCanvas(state = state, scenePath = {}, viewWidth = {}f, viewHeight = {}f, fit = {}, fps = {}, autoplay = {}, pixelated = {}, backgroundColor = {}, label = {}, onPointer = {}, onKey = {}, onMotion = {}, motionRate = {}, modifier = {})\n",
                compose_string_literal(&context.signal_path(&props.scene)),
                props.view_width,
                props.view_height,
                compose_string_literal(props.fit.as_str()),
                props.fps,
                props.autoplay,
                props.pixelated,
                background,
                compose_string_literal(&props.label),
                compose_optional_string(props.on_pointer.as_deref().and_then(|value| context.action_id(value))),
                compose_optional_string(props.on_key.as_deref().and_then(|value| context.action_id(value))),
                compose_optional_string(props.on_motion.as_deref().and_then(|value| context.action_id(value))),
                props.motion_rate,
                modifier_for_style(&props.style),
            ));
        }
        ViewNode::Candlestick { props } => {
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                        "{pad}DoweCandlestick(state = state, dataPath = {}, stream = {}, upColor = {}, downColor = {}, emptyLabel = {}, maxPoints = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        compose_string_literal(&context.signal_path(&props.data)),
                        compose_optional_string(props.stream.as_deref()),
                        color_ref(props.up_color),
                        color_ref(props.down_color),
                        compose_string_literal(&props.empty_label),
                        props.max_points,
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        card_variant_container(&props.style),
                        card_variant_content(&props.style),
                    ));
        }
        ViewNode::ArcChart { props } => {
            render_compose_chart("arc", &props.common, indent, output, context);
        }
        ViewNode::AreaChart { props } => {
            render_compose_chart("area", &props.common, indent, output, context);
        }
        ViewNode::BarChart { props } => {
            render_compose_chart("bar", &props.common, indent, output, context);
        }
        ViewNode::LineChart { props } => {
            render_compose_chart("line", &props.common, indent, output, context);
        }
        ViewNode::PieChart { props } => {
            render_compose_chart("pie", &props.common, indent, output, context);
        }
        ViewNode::Table { props } => {
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                table_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                        "{pad}DoweTable(state = state, dataPath = {}, columns = {}, size = {}, striped = {}, bordered = {}, dividers = {}, emptyTitle = {}, emptyDescription = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        compose_string_literal(&context.signal_path(&props.data)),
                        compose_table_columns(&props.columns),
                        compose_table_size(props.size),
                        props.striped,
                        props.bordered,
                        props.dividers,
                        compose_string_literal(&props.empty_title),
                        compose_string_literal(&props.empty_description),
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        table_variant_container(&props.style),
                        table_variant_content(&props.style),
                    ));
        }
        ViewNode::AvatarGroup { props, items } => {
            render_compose_avatar_group(props, items, indent, output, context)
        }
        ViewNode::ChatBox { props } => {
            render_compose_chat_box(props, indent, output, context);
        }
        ViewNode::Empty { props } => {
            render_compose_empty(props, indent, output, context);
        }
        ViewNode::Marquee { props, children } => {
            render_compose_marquee(
                props,
                children,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::TypeWriter { props, items } => {
            render_compose_type_writer(props, items, indent, output)
        }
        ViewNode::RichText { props, marks } => {
            render_compose_rich_text(props, marks, indent, output, inherited_font, default_family);
        }
        ViewNode::Record { props } => {
            render_compose_record(props, indent, output, context);
        }
        ViewNode::ToggleGroup { props, items } => {
            render_compose_toggle_group(props, items, indent, output, context)
        }
        ViewNode::Collapsible { props, children } => {
            render_compose_collapsible(
                props,
                children,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Countdown { props } => {
            render_compose_countdown(props, indent, output, context);
        }
        ViewNode::Map {
            props,
            markers,
            waypoints,
        } => {
            render_compose_map(props, markers, waypoints, indent, output, context);
        }
        ViewNode::Divider { props } => {
            output.push_str(&format!(
                "{pad}Box(modifier = {}.background({}))\n",
                modifier_for_divider(props, flow),
                color_ref(family_color(props.color))
            ));
        }
        ViewNode::Title { props, value } => {
            render_compose_text(
                true,
                props,
                value,
                props.style.font.as_ref().or(inherited_font),
                indent,
                output,
                default_family,
                context,
            );
        }
        ViewNode::Text { props, value } => {
            render_compose_text(
                false,
                props,
                value,
                props.style.font.as_ref().or(inherited_font),
                indent,
                output,
                default_family,
                context,
            );
        }
        ViewNode::Alert { props } => {
            if let Some(visible) = props.visible.as_deref() {
                output.push_str(&format!(
                    "{pad}if (state.bool(\"{}\")) {{\n",
                    escape_kotlin(&context.signal_path(visible))
                ));
            }
            let alert_pad = if props.visible.is_some() {
                format!("{pad}    ")
            } else {
                pad.clone()
            };
            let shape = compose_control_radius(&props.style.style);
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                format!(
                    ".border(1.dp, {}, RoundedCornerShape({shape}))",
                    variant_content(&props.style)
                )
            } else {
                String::new()
            };
            output.push_str(&format!(
                        "{alert_pad}Row(modifier = {}.clip(RoundedCornerShape({shape})).background({}){border}.padding(horizontal = 14.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {{\n",
                        modifier_for_style(&props.style.style),
                        variant_container(&props.style)
                    ));
            output.push_str(&format!(
                        "{alert_pad}    Text({}, modifier = Modifier.weight(1f), color = {}, fontFamily = {})\n",
                        compose_text_expression(&props.message, None, context),
                        variant_content(&props.style),
                        compose_font_value(
                            props.style.style.font.as_ref().or(inherited_font),
                            default_family
                        )
                    ));
            if let Some(action) = props
                .on_close
                .as_deref()
                .and_then(|name| context.action_id(name))
            {
                output.push_str(&format!(
                            "{alert_pad}    Button(onClick = {{ actionScope.launch {{ state.run(\"{}\") }} }}, contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp)) {{ Text(\"x\") }}\n",
                            escape_kotlin(action)
                        ));
            }
            output.push_str(&format!("{alert_pad}}}\n"));
            if props.visible.is_some() {
                output.push_str(&format!("{pad}}}\n"));
            }
        }
        ViewNode::Svg { props, paths } => {
            if let Some(data) = props.data.as_deref() {
                let payload = if let Some(item) = context.item_value(data) {
                    let path = context.item_path(data).unwrap_or_else(|| data.to_string());
                    format!(
                        "state.json(\"{}\", {item})",
                        escape_kotlin(&path)
                    )
                } else {
                    format!(
                        "state.json(\"{}\")",
                        escape_kotlin(&context.signal_path(data))
                    )
                };
                output.push_str(&format!(
                    "{pad}DoweRuntimeSvg(payload = {payload}, modifier = {}, color = {}, animated = {})\n",
                    modifier_for_style(&props.style),
                    compose_svg_color(&props.style),
                    props.motion.is_some()
                ));
                return;
            }
            output.push_str(&format!(
                "{pad}DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {}, animated = {})\n",
                compose_svg_view_box(&props.view_box),
                modifier_for_style(&props.style),
                compose_svg_color(&props.style),
                compose_svg_paths(paths),
                props.motion.is_some()
            ));
        }
        _ => {}
    }
}

fn compose_video_icons() -> String {
    let icon = |name: &str| {
        let icon = solar_control_icon(name).expect("bundled Video control icon");
        format!(
            "DoweVideoIcon(viewBox = {}, paths = {})",
            compose_svg_view_box(&icon.props.view_box),
            compose_svg_paths(&icon.paths)
        )
    };
    format!(
        "DoweVideoIcons(play = {}, pause = {}, volume = {}, muted = {}, pictureInPicture = {}, fullscreen = {})",
        icon("play"),
        icon("pause"),
        icon("volume-loud"),
        icon("volume-cross"),
        icon("pip"),
        icon("full-screen")
    )
}

fn render_compose_chart(
    chart_type: &str,
    props: &ChartCommonProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
        == ComponentVariant::Outlined
    {
        card_variant_content(&props.style)
    } else {
        "null"
    };
    let data_path = props
        .data
        .as_deref()
        .map(|value| compose_string_literal(&context.signal_path(value)))
        .unwrap_or_else(|| "null".to_string());
    let series_path = props
        .series
        .as_deref()
        .map(|value| compose_string_literal(&context.signal_path(value)))
        .unwrap_or_else(|| "null".to_string());
    output.push_str(&format!(
        "{pad}DoweChart(state = state, chartType = {}, dataPath = {}, seriesPath = {}, palette = {}, legendPosition = {}, emptyLabel = {}, loading = {}, hideLegend = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_string_literal(chart_type),
        data_path,
        series_path,
        compose_string_literal(props.palette.as_str()),
        compose_string_literal(props.legend_position.as_str()),
        compose_string_literal(&props.empty_label),
        props.loading,
        props.hide_legend,
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}
