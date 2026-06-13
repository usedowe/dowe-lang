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
            let border = if props.style.variant.unwrap_or(ComponentVariant::Soft)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                        "{pad}DoweCode(source = {}, language = {}, tokens = {}, copyLabel = {}, copiedLabel = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        compose_string_literal(&props.source),
                        compose_string_literal(props.language.as_str()),
                        compose_code_tokens(&props.tokens, card_variant_content(&props.style)),
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
            output.push_str(&format!(
                        "{pad}DoweVideo(source = {}, poster = {}, autoplay = {}, aspect = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, borderColor = {border})\n",
                        compose_string_literal(&props.src),
                        compose_optional_string(props.poster.as_deref()),
                        props.autoplay,
                        compose_string_literal(props.aspect.as_str()),
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        card_variant_container(&props.style),
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
        ViewNode::Table { props } => {
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
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
                        card_variant_container(&props.style),
                        card_variant_content(&props.style),
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
            output.push_str(&format!(
                "{pad}DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
                compose_svg_view_box(&props.view_box),
                modifier_for_style(&props.style),
                compose_svg_color(&props.style),
                compose_svg_paths(paths)
            ));
        }
        _ => {}
    }
}
