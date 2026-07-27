fn render_swift_media_data_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match node {
        ViewNode::Audio { props } => render_swift_audio(props, indent, output),
        ViewNode::Image { props } => render_swift_image(props, indent, output),
        ViewNode::Accordion { props, items } => render_swift_accordion(
            props,
            items,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Carousel { props, slides } => render_swift_carousel(
            props,
            slides,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Code { props } => {
            let source = if props.template_segments.is_empty() {
                swift_string_literal(&props.source)
            } else {
                props.template_segments.iter().map(|segment| match segment {
                    CodeTemplateSegment::Static { text, .. } => swift_string_literal(text),
                    CodeTemplateSegment::Binding(path) => format!("state.text(\"{}\", fallback: \"\")", escape_swift(&context.signal_path(path))),
                }).collect::<Vec<_>>().join(" + ")
            };
            let tokens = if props.template_segments.is_empty() {
                swift_code_tokens(&props.tokens, card_variant_content(&props.style))
            } else {
                props
                    .template_segments
                    .iter()
                    .map(|segment| match segment {
                        CodeTemplateSegment::Static { tokens, .. } => {
                            swift_code_tokens(tokens, card_variant_content(&props.style))
                        }
                        CodeTemplateSegment::Binding(path) => format!(
                            "[DoweCodeToken(text: state.text(\"{}\", fallback: \"\"), color: {})]",
                            escape_swift(&context.signal_path(path)),
                            card_variant_content(&props.style)
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            let border = if props.style.variant.unwrap_or(ComponentVariant::Soft)
                == ComponentVariant::Outlined
            {
                format!("Optional({})", card_variant_content(&props.style))
            } else {
                "nil".to_string()
            };
            output.push_str(&format!(
                "{pad}DoweCodeView(source: {}, language: {}, tokens: {}, copyLabel: {}, copiedLabel: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                source,
                swift_string_literal(props.language.as_str()),
                tokens,
                swift_string_literal(&props.copy_label),
                swift_string_literal(&props.copied_label),
                card_variant_container(&props.style),
                card_variant_content(&props.style),
                swift_card_radius(&props.style.style)
            ));
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_style(&props.style.style),
            );
        }
        ViewNode::Video { props } => {
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                format!("Optional({})", card_variant_content(&props.style))
            } else {
                "nil".to_string()
            };
            let icons = swift_video_icons();
            output.push_str(&format!(
                "{pad}DoweVideoView(source: {}, poster: {}, autoplay: {}, aspect: {}, backgroundColor: {}, borderColor: {border}, radius: {}, icons: {icons})\n",
                swift_string_literal(&props.src),
                swift_optional_literal(props.poster.as_deref()),
                props.autoplay,
                swift_string_literal(props.aspect.as_str()),
                card_variant_container(&props.style),
                swift_card_radius(&props.style.style)
            ));
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_style(&props.style.style),
            );
        }
        ViewNode::Iframe { props } => {
            let sandbox = props.sandbox.as_ref().map(|tokens| {
                format!("Optional([{}])", tokens.iter().map(|token| swift_string_literal(token)).collect::<Vec<_>>().join(", "))
            }).unwrap_or_else(|| "nil".to_string());
            output.push_str(&format!(
                "{pad}DoweIframeView(source: {}, title: {}, sandbox: {sandbox}, autoplay: {})\n{pad}    .frame(maxWidth: .infinity, minHeight: CGFloat(192))\n",
                swift_string_literal(&props.src),
                swift_string_literal(&props.title),
                props.allow.iter().any(|token| token == "autoplay"),
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::Device { props, iframe } => {
            let sandbox = iframe.sandbox.as_ref().map(|tokens| {
                format!("Optional([{}])", tokens.iter().map(|token| swift_string_literal(token)).collect::<Vec<_>>().join(", "))
            }).unwrap_or_else(|| "nil".to_string());
            let icons = props.options.iter().map(|option| {
                format!(
                    "DoweDeviceIcon(profile: {}, viewBox: {}, paths: {})",
                    swift_string_literal(option.profile.as_str()),
                    swift_svg_view_box(&option.icon.props.view_box),
                    swift_svg_paths(&option.icon.paths),
                )
            }).collect::<Vec<_>>().join(", ");
            output.push_str(&format!(
                "{pad}DoweDevicePreview(initialProfile: {}, source: {}, title: {}, sandbox: {sandbox}, autoplay: {}, icons: [{icons}])\n",
                swift_string_literal(props.device.as_str()),
                swift_string_literal(&iframe.src),
                swift_string_literal(&iframe.title),
                iframe.allow.iter().any(|token| token == "autoplay"),
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::Canvas { props } => {
            let background = match props.background {
                CanvasBackground::Transparent => "Color.clear".to_string(),
                CanvasBackground::Color(color) => color_ref(color).to_string(),
            };
            output.push_str(&format!(
                "{pad}DoweCanvasView(state: state, scenePath: {}, viewWidth: CGFloat({}), viewHeight: CGFloat({}), fit: {}, fps: {}, autoplay: {}, pixelated: {}, backgroundColor: {}, label: {}, onPointer: {}, onKey: {}, onMotion: {}, motionRate: {})\n",
                swift_string_literal(&context.signal_path(&props.scene)),
                props.view_width,
                props.view_height,
                swift_string_literal(props.fit.as_str()),
                props.fps,
                props.autoplay,
                props.pixelated,
                background,
                swift_string_literal(&props.label),
                swift_optional_literal(props.on_pointer.as_deref().and_then(|value| context.action_id(value))),
                swift_optional_literal(props.on_key.as_deref().and_then(|value| context.action_id(value))),
                swift_optional_literal(props.on_motion.as_deref().and_then(|value| context.action_id(value))),
                props.motion_rate,
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::Candlestick { props } => {
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                format!("Optional({})", card_variant_content(&props.style))
            } else {
                "nil".to_string()
            };
            output.push_str(&format!(
                "{pad}DoweCandlestickView(state: state, dataPath: {}, stream: {}, upColor: {}, downColor: {}, emptyLabel: {}, maxPoints: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_string_literal(&context.signal_path(&props.data)),
                swift_optional_literal(props.stream.as_deref()),
                color_ref(props.up_color),
                color_ref(props.down_color),
                swift_string_literal(&props.empty_label),
                props.max_points,
                card_variant_container(&props.style),
                card_variant_content(&props.style),
                swift_card_radius(&props.style.style)
            ));
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_style(&props.style.style),
            );
        }
        ViewNode::ArcChart { props } => {
            render_swift_chart("arc", &props.common, indent, output, context);
        }
        ViewNode::AreaChart { props } => {
            render_swift_chart("area", &props.common, indent, output, context);
        }
        ViewNode::BarChart { props } => {
            render_swift_chart("bar", &props.common, indent, output, context);
        }
        ViewNode::LineChart { props } => {
            render_swift_chart("line", &props.common, indent, output, context);
        }
        ViewNode::PieChart { props } => {
            render_swift_chart("pie", &props.common, indent, output, context);
        }
        ViewNode::Table { props } => {
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                format!("Optional({})", table_variant_content(&props.style))
            } else {
                "nil".to_string()
            };
            output.push_str(&format!(
                "{pad}DoweTableView(state: state, dataPath: {}, columns: {}, size: {}, striped: {}, bordered: {}, dividers: {}, emptyTitle: {}, emptyDescription: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_string_literal(&context.signal_path(&props.data)),
                swift_table_columns(&props.columns),
                swift_table_size(props.size),
                props.striped,
                props.bordered,
                props.dividers,
                swift_string_literal(&props.empty_title),
                swift_string_literal(&props.empty_description),
                table_variant_container(&props.style),
                table_variant_content(&props.style),
                swift_card_radius(&props.style.style)
            ));
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_style(&props.style.style),
            );
        }
        _ => unreachable!(),
    }
}

fn swift_video_icons() -> String {
    let icon = |name: &str| {
        let icon = solar_control_icon(name).expect("bundled Video control icon");
        format!(
            "DoweVideoIcon(viewBox: {}, paths: {})",
            swift_svg_view_box(&icon.props.view_box),
            swift_svg_paths(&icon.paths)
        )
    };
    format!(
        "DoweVideoIcons(play: {}, pause: {}, volume: {}, muted: {}, pictureInPicture: {}, fullscreen: {})",
        icon("play"),
        icon("pause"),
        icon("volume-loud"),
        icon("volume-cross"),
        icon("pip"),
        icon("full-screen")
    )
}

fn render_swift_chart(
    chart_type: &str,
    props: &ChartCommonProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
        == ComponentVariant::Outlined
    {
        format!("Optional({})", card_variant_content(&props.style))
    } else {
        "nil".to_string()
    };
    let data_path = if let Some(value) = props.data.as_deref() {
        let path = context.signal_path(value);
        swift_optional_literal(Some(path.as_str()))
    } else {
        "nil".to_string()
    };
    let series_path = if let Some(value) = props.series.as_deref() {
        let path = context.signal_path(value);
        swift_optional_literal(Some(path.as_str()))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweChartView(state: state, chartType: {}, dataPath: {}, seriesPath: {}, palette: {}, legendPosition: {}, emptyLabel: {}, loading: {}, hideLegend: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
        swift_string_literal(chart_type),
        data_path,
        series_path,
        swift_string_literal(props.palette.as_str()),
        swift_string_literal(props.legend_position.as_str()),
        swift_string_literal(&props.empty_label),
        props.loading,
        props.hide_legend,
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_card_radius(&props.style.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}
