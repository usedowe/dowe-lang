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
            let border = if props.style.variant.unwrap_or(ComponentVariant::Soft)
                == ComponentVariant::Outlined
            {
                format!("Optional({})", card_variant_content(&props.style))
            } else {
                "nil".to_string()
            };
            output.push_str(&format!(
                "{pad}DoweCodeView(source: {}, language: {}, tokens: {}, copyLabel: {}, copiedLabel: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_string_literal(&props.source),
                swift_string_literal(props.language.as_str()),
                swift_code_tokens(&props.tokens, card_variant_content(&props.style)),
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
            output.push_str(&format!(
                "{pad}DoweVideoView(source: {}, poster: {}, autoplay: {}, aspect: {}, backgroundColor: {}, borderColor: {border}, radius: {})\n",
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
                format!("Optional({})", card_variant_content(&props.style))
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
        _ => unreachable!(),
    }
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
