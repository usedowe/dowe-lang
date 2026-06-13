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
