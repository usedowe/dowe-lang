fn render_compose_node_in_flow(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    if let Some(show) = node_element_props(node).and_then(|props| props.show.as_ref()) {
        let pad = " ".repeat(indent);
        output.push_str(&format!(
            "{pad}if ({}) {{\n",
            compose_show_condition(show, context)
        ));
        render_compose_node_body(
            node,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}}}\n"));
    } else {
        render_compose_node_body(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn render_compose_node_body(
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
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
        }
        ViewNode::Each {
            item,
            collection,
            children,
            ..
        } => {
            output.push_str(&format!(
                "{pad}state.rows(\"{}\").forEach {{ row ->\n",
                escape_kotlin(&context.signal_path(collection))
            ));
            let context = context.with_item(item, "row.value".to_string());
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Box { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!(
                    "{pad}DoweCoverBox(modifier = {}, source = {}, overlay = {}) {{\n",
                    modifier_for_container_style(props, flow),
                    compose_cover_value(props.cover.as_ref().expect("cover")),
                    compose_optional_overlay(props.overlay.as_ref())
                ));
                output.push_str(&format!("{pad}    Column {{\n"));
                let color_scope = compose_content_color(props);
                if let Some(color) = color_scope.as_ref() {
                    output.push_str(&format!(
                        "{pad}        CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                    ));
                }
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + if color_scope.is_some() { 12 } else { 8 },
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                if color_scope.is_some() {
                    output.push_str(&format!("{pad}        }}\n"));
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}Column(modifier = {}) {{\n",
                    modifier_for_container_style(props, flow)
                ));
                let color_scope = compose_content_color(props);
                if let Some(color) = color_scope.as_ref() {
                    output.push_str(&format!(
                        "{pad}    CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                    ));
                }
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + if color_scope.is_some() { 8 } else { 4 },
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                if color_scope.is_some() {
                    output.push_str(&format!("{pad}    }}\n"));
                }
                output.push_str(&format!("{pad}}}\n"));
            }
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!(
                    "{pad}DoweCoverBox(modifier = {}, source = {}, overlay = {}) {{\n",
                    modifier_for_container_style(props, flow),
                    compose_cover_value(props.cover.as_ref().expect("cover")),
                    compose_optional_overlay(props.overlay.as_ref())
                ));
                output.push_str(&format!("{pad}    Column {{\n"));
                let color_scope = compose_content_color(props);
                if let Some(color) = color_scope.as_ref() {
                    output.push_str(&format!(
                        "{pad}        CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                    ));
                }
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + if color_scope.is_some() { 12 } else { 8 },
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                if color_scope.is_some() {
                    output.push_str(&format!("{pad}        }}\n"));
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else if let Some(background) = props.background.as_ref() {
                output.push_str(&format!(
                    "{pad}DoweSectionBackgroundBox(modifier = {}, background = {}) {{\n",
                    modifier_for_container_style(props, flow),
                    compose_section_background_value(background)
                ));
                output.push_str(&format!("{pad}    Column {{\n"));
                let color_scope = compose_content_color(props);
                if let Some(color) = color_scope.as_ref() {
                    output.push_str(&format!(
                        "{pad}        CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                    ));
                }
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + if color_scope.is_some() { 12 } else { 8 },
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                if color_scope.is_some() {
                    output.push_str(&format!("{pad}        }}\n"));
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}Column(modifier = {}) {{\n",
                    modifier_for_container_style(props, flow)
                ));
                let color_scope = compose_content_color(props);
                if let Some(color) = color_scope.as_ref() {
                    output.push_str(&format!(
                        "{pad}    CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                    ));
                }
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + if color_scope.is_some() { 8 } else { 4 },
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                if color_scope.is_some() {
                    output.push_str(&format!("{pad}    }}\n"));
                }
                output.push_str(&format!("{pad}}}\n"));
            }
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}Row(modifier = {}, horizontalArrangement = {}, verticalAlignment = {}) {{\n",
                modifier_for_layout(props, flow),
                compose_horizontal_arrangement(props.justify.as_ref(), props.gap.as_ref()),
                compose_vertical_alignment(props.align.as_ref())
            ));
            let color_scope = compose_content_color(&props.style);
            if let Some(color) = color_scope.as_ref() {
                output.push_str(&format!(
                    "{pad}    CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                ));
            }
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + if color_scope.is_some() { 8 } else { 4 },
                    output,
                    ComposeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            if color_scope.is_some() {
                output.push_str(&format!("{pad}    }}\n"));
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Grid { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}DoweGrid(modifier = {}, columns = {}, horizontalGap = {}, verticalGap = {}, horizontalAlignment = {}) {{\n",
                modifier_for_grid(props, flow),
                compose_grid_column_count(props.columns.as_ref()),
                compose_grid_horizontal_gap(props.gap.as_ref()),
                compose_grid_vertical_gap(props.gap.as_ref()),
                compose_grid_horizontal_alignment(props.justify.as_ref())
            ));
            let color_scope = compose_content_color(&props.style);
            if let Some(color) = color_scope.as_ref() {
                output.push_str(&format!(
                    "{pad}    CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                ));
            }
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + if color_scope.is_some() { 8 } else { 4 },
                    output,
                    ComposeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
            }
            if color_scope.is_some() {
                output.push_str(&format!("{pad}    }}\n"));
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Card { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}Card(modifier = {}, shape = RoundedCornerShape({}), colors = CardDefaults.cardColors(containerColor = {}, contentColor = {}), border = {}) {{\n",
                modifier_for_container_style(&props.style, flow),
                compose_card_radius(&props.style),
                card_variant_container(props),
                card_variant_content(props),
                compose_card_border(props)
            ));
            if props.style.cover.is_some() {
                output.push_str(&format!(
                    "{pad}    DoweCoverBox(modifier = Modifier.fillMaxWidth(), source = {}, overlay = {}) {{\n",
                    compose_cover_value(props.style.cover.as_ref().expect("cover")),
                    compose_optional_overlay(props.style.overlay.as_ref())
                ));
                output.push_str(&format!("{pad}        Column {{\n"));
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + 12,
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}        }}\n"));
                output.push_str(&format!("{pad}    }}\n"));
            } else {
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Button { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let action = props
                .element
                .on_click
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| {
                    let item = context
                        .active_item()
                        .map(|value| format!(", {value}"))
                        .unwrap_or_default();
                    format!(
                        "{{ actionScope.launch {{ state.run(\"{}\"{item}) }} }}",
                        escape_kotlin(id)
                    )
                })
                .unwrap_or_else(|| compose_navigation_action(props.navigation.as_ref()));
            output.push_str(&format!(
                "{pad}Button(modifier = {}.defaultMinSize(minWidth = 0.dp, minHeight = 0.dp), shape = RoundedCornerShape({}), colors = ButtonDefaults.buttonColors(containerColor = {}, contentColor = {}), border = {}, contentPadding = PaddingValues(0.dp), onClick = {}) {{\n",
                modifier_for_style(&props.style),
                compose_control_radius(&props.style),
                variant_container(props),
                variant_content(props),
                compose_button_border(props),
                action
            ));
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    ComposeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Input { props } => {
            let (value, change) = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    let path = escape_kotlin(&context.signal_path(path));
                    (
                        format!("state.text(\"{path}\")"),
                        format!("{{ state.write(\"{path}\", it) }}"),
                    )
                })
                .unwrap_or_else(|| ("\"\"".to_string(), "{}".to_string()));
            let size = compose_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    color_ref(ColorToken::Muted)
                } else {
                    "null"
                };
            let modifier = if flow == ComposeFlow::Inline && props.style.sizing.w.is_none() {
                format!("{}.weight(1f)", modifier_for_style(&props.style))
            } else {
                modifier_for_style(&props.style)
            };
            output.push_str(&format!(
                "{pad}DoweInput(value = {value}, onValueChange = {change}, modifier = {}, label = {}, placeholder = {}, floating = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                modifier,
                compose_optional_string(props.label.as_deref()),
                compose_string_literal(props.placeholder.as_deref().unwrap_or_default()),
                props.label_floating,
                compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                compose_control_radius(&props.style),
                variant_container(props),
                variant_content(props)
            ));
        }
        ViewNode::Select { props, options } => {
            let (value, change, bound) = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    let path = escape_kotlin(&context.signal_path(path));
                    (
                        format!("state.text(\"{path}\")"),
                        format!("{{ state.write(\"{path}\", it) }}"),
                        "true",
                    )
                })
                .unwrap_or_else(|| ("\"\"".to_string(), "{}".to_string(), "false"));
            let size = compose_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    color_ref(ColorToken::Muted)
                } else {
                    "null"
                };
            let modifier = if flow == ComposeFlow::Inline && props.style.sizing.w.is_none() {
                format!("{}.weight(1f)", modifier_for_style(&props.style))
            } else {
                modifier_for_style(&props.style)
            };
            output.push_str(&format!(
                "{pad}DoweSelect(value = {value}, onValueChange = {change}, bound = {bound}, modifier = {}, label = {}, placeholder = {}, floating = {}, options = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                modifier,
                compose_optional_string(props.label.as_deref()),
                compose_string_literal(props.placeholder.as_deref().unwrap_or("Select an option")),
                props.label_floating,
                compose_select_options(options),
                compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                compose_control_radius(&props.style),
                variant_container(props),
                variant_content(props)
            ));
        }
        ViewNode::Audio { props } => render_compose_audio(props, indent, output),
        ViewNode::Image { props } => render_compose_image(props, indent, output),
        ViewNode::Accordion { props, items } => render_compose_accordion(
            props,
            items,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Carousel { props, slides } => render_compose_carousel(
            props,
            slides,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Checkbox { props } => render_compose_checkbox(props, indent, output, context),
        ViewNode::Color { props } => render_compose_color(props, indent, output, context),
        ViewNode::Date { props } => render_compose_date(props, indent, output, context),
        ViewNode::DateRange { props } => render_compose_date_range(props, indent, output, context),
        ViewNode::RadioGroup { props, options } => {
            render_compose_radio_group(props, options, indent, output, context)
        }
        ViewNode::Toggle { props } => render_compose_toggle(props, indent, output, context),
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
        ViewNode::Divider { props } => {
            output.push_str(&format!(
                "{pad}Box(modifier = {}.background({}))\n",
                modifier_for_divider(props, flow),
                color_ref(family_color(props.color))
            ));
        }
        ViewNode::Title { props, value } => render_compose_text(
            true,
            props,
            value,
            props.style.font.as_ref().or(inherited_font),
            indent,
            output,
            default_family,
            context,
        ),
        ViewNode::Text { props, value } => render_compose_text(
            false,
            props,
            value,
            props.style.font.as_ref().or(inherited_font),
            indent,
            output,
            default_family,
            context,
        ),
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
                format!(".border(1.dp, {}, RoundedCornerShape({shape}))", variant_content(&props.style))
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
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }
        | ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        | ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            render_compose_bar(
                props,
                start,
                center,
                end,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::SideNav { props, items } => {
            render_compose_side_nav(
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
        ViewNode::Sidebar { props, items } => {
            render_compose_side_nav(
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
        ViewNode::NavMenu { props, items } => {
            render_compose_nav_menu(
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
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            render_compose_scaffold(
                props,
                app_bar,
                start,
                main,
                end,
                bottom_bar,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Tabs { props, tabs } => {
            render_compose_tabs(
                props,
                tabs,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Drawer { props, children } => {
            render_compose_drawer(
                props,
                children,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Avatar { props, icon } => {
            render_compose_avatar(props, icon.as_ref(), indent, output, context);
        }
        ViewNode::Badge { props, children } => {
            render_compose_badge(
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
        ViewNode::Chip {
            props,
            value,
            start,
            end,
        } => {
            render_compose_chip(
                props,
                value,
                start.as_ref(),
                end.as_ref(),
                indent,
                output,
                context,
            );
        }
        ViewNode::Skeleton { props } => render_compose_skeleton(props, indent, output, flow),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            render_compose_modal(
                props,
                header,
                body,
                footer,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::AlertDialog { props } => {
            render_compose_alert_dialog(props, indent, output, context);
        }
        ViewNode::Tooltip { props, children } => {
            render_compose_tooltip(
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
        ViewNode::Toast { props } => render_compose_toast(props, indent, output, context),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            render_compose_dropdown(
                props,
                trigger,
                header,
                entries,
                footer,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Command { props, entries } => {
            render_compose_command(
                props,
                entries,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Children => {}
    }
}

fn render_compose_drawer(
    props: &DrawerProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_kotlin(&context.signal_path(&props.open));
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            card_variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweDrawer(open = state.bool(\"{path}\"), onClose = {{ state.write(\"{path}\", false) }}, position = \"{}\", backgroundColor = {}, contentColor = {}, borderColor = {border}, radius = {}, disableOverlayClose = {}, hideCloseButton = {}) {{\n",
        props.position.as_str(),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_drawer_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button
    ));
    output.push_str(&format!(
        "{pad}    Column(modifier = {}) {{\n",
        modifier_for_container_style(&props.style.style, ComposeFlow::Block)
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 8,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}    }}\n"));
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_avatar(
    props: &AvatarProps,
    icon: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweAvatar(source = {}, name = {}, alt = {}, size = {}, status = {}, bordered = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onClick = {}, hasIcon = {}) {{\n",
        compose_optional_string(props.src.as_deref()),
        compose_optional_string(props.name.as_deref()),
        compose_string_literal(&props.alt),
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.status.map(|value| value.as_str())),
        props.bordered,
        variant_container(&props.style),
        variant_content(&props.style),
        variant_content(&props.style),
        compose_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        icon.is_some()
    ));
    if let Some(icon) = icon {
        render_compose_side_icon(icon, indent + 4, output);
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_badge(
    props: &BadgeProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweBadge(text = {}, position = {}, backgroundColor = {}, contentColor = {}, modifier = {}) {{\n",
        compose_string_literal(&props.text),
        compose_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        modifier_for_style(&props.style.style)
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_chip(
    props: &ChipProps,
    value: &str,
    start: Option<&SideNavIcon>,
    end: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    output.push_str(&format!(
        "{pad}DoweChip(text = {}, size = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {}, onClose = {}, start = ",
        compose_string_literal(value),
        compose_string_literal(size.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        modifier_for_style(&props.style.style),
        compose_optional_component_action(props.on_close.as_deref(), None, context)
    ));
    render_compose_optional_icon_lambda(start, indent, output);
    output.push_str(", end = ");
    render_compose_optional_icon_lambda(end, indent, output);
    output.push_str(")\n");
}

fn render_compose_skeleton(
    props: &SkeletonProps,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSkeleton(variant = {}, animation = {}, modifier = {})\n",
        compose_string_literal(props.variant.as_str()),
        compose_string_literal(props.animation.as_str()),
        modifier_for_container_style(&props.style, flow)
    ));
}

fn render_compose_audio(props: &AudioProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        card_variant_content(&props.style)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweAudio(source = {}, subtitle = {}, avatarSource = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_string_literal(&props.src),
        compose_optional_string(props.subtitle.as_deref()),
        compose_optional_string(props.avatar_src.as_deref()),
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}

fn render_compose_image(props: &ImageProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        card_variant_content(&props.style)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweImage(source = {}, alt = {}, aspect = {}, objectFit = {}, loading = {}, hideControls = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_string_literal(&props.src),
        compose_string_literal(&props.alt),
        compose_string_literal(props.aspect.as_str()),
        compose_string_literal(props.object_fit.as_str()),
        compose_string_literal(props.loading.as_str()),
        props.hide_controls,
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}

fn render_compose_accordion(
    props: &AccordionProps,
    items: &[AccordionItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        variant_content(&props.style)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweAccordion(multiple = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border}) {{\n",
        props.multiple,
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for item in items {
        output.push_str(&format!(
            "{pad}    DoweAccordionItem(id = {}, label = {}, disabled = {}, defaultOpen = {}) {{\n",
            compose_string_literal(&item.id),
            compose_string_literal(&item.label),
            item.disabled,
            item.default_open
        ));
        for child in &item.children {
            render_compose_node_in_flow(
                child,
                indent + 8,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_carousel(
    props: &CarouselProps,
    slides: &[CarouselSlide],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCarousel(autoplay = {}, autoplayInterval = {}, disableLoop = {}, hideControls = {}, hideIndicators = {}, showNavigation = {}, showCounter = {}, orientation = {}, size = {}, indicatorType = {}, title = {}, slideWidth = {}, slideHeight = {}, slidesPerView = {}, gap = {}, modifier = {}, accentColor = {}) {{\n",
        props.autoplay,
        props.autoplay_interval,
        props.disable_loop,
        props.hide_controls,
        props.hide_indicators,
        props.show_navigation,
        props.show_counter,
        compose_string_literal(props.orientation.as_str()),
        compose_string_literal(props.size.as_str()),
        compose_string_literal(props.indicator_type.as_str()),
        compose_optional_string(props.title.as_deref()),
        compose_optional_u16(props.slide_width),
        compose_optional_u16(props.slide_height),
        props.slides_per_view,
        props.gap,
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style),
    ));
    for slide in slides {
        output.push_str(&format!(
            "{pad}    DoweCarouselSlide(id = {}) {{\n",
            compose_string_literal(&slide.id)
        ));
        for child in &slide.children {
            render_compose_node_in_flow(
                child,
                indent + 8,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_checkbox(
    props: &CheckboxProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (checked, change) = compose_bool_value_and_change(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweCheckbox(checked = {checked}, onCheckedChange = {change}, enabled = {}, label = {}, name = {}, modifier = {}, accentColor = {})\n",
        !props.disabled,
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.name.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_color(
    props: &ColorProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change) = compose_text_value_and_change(&props.style, &props.value, context);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined) == ComponentVariant::Outlined {
        color_ref(ColorToken::Muted)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweColorField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, size = {}, name = {}, helpText = {}, errorText = {}, showHex = {}, showRgb = {}, showCmyk = {}, showOklch = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select color")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        props.show_hex,
        props.show_rgb,
        props.show_cmyk,
        props.show_oklch,
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
}

fn render_compose_date(
    props: &DateProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change) =
        compose_text_value_and_change(&props.style, props.value.as_deref().unwrap_or_default(), context);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined) == ComponentVariant::Outlined {
        color_ref(ColorToken::Muted)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweDateField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, size = {}, name = {}, helpText = {}, errorText = {}, min = {}, max = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        compose_optional_string(props.min.as_deref()),
        compose_optional_string(props.max.as_deref()),
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
}

fn render_compose_date_range(
    props: &DateRangeProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (start_value, start_change) = compose_optional_text_path_and_change(
        props.start.as_deref(),
        props.start_value.as_deref().unwrap_or_default(),
        context,
    );
    let (end_value, end_change) = compose_optional_text_path_and_change(
        props.end.as_deref(),
        props.end_value.as_deref().unwrap_or_default(),
        context,
    );
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined) == ComponentVariant::Outlined {
        color_ref(ColorToken::Muted)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweDateRangeField(startValue = {start_value}, endValue = {end_value}, onStartChange = {start_change}, onEndChange = {end_change}, label = {}, placeholder = {}, floating = {}, size = {}, name = {}, helpText = {}, errorText = {}, min = {}, max = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date range")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        compose_optional_string(props.min.as_deref()),
        compose_optional_string(props.max.as_deref()),
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
}

fn render_compose_radio_group(
    props: &RadioGroupProps,
    options: &[RadioOption],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change) = compose_text_value_and_change(&props.style, "", context);
    output.push_str(&format!(
        "{pad}DoweRadioGroup(value = {value}, onValueChange = {change}, options = {}, size = {}, name = {}, label = {}, helpText = {}, errorText = {}, modifier = {}, accentColor = {})\n",
        compose_radio_options(options),
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.info.as_deref()),
        compose_optional_string(props.error.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_toggle(
    props: &ToggleProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (checked, change) = compose_bool_value_and_change(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweToggle(checked = {checked}, onCheckedChange = {change}, enabled = {}, label = {}, labelLeft = {}, labelRight = {}, name = {}, modifier = {}, accentColor = {})\n",
        !props.disabled,
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.label_left.as_deref()),
        compose_optional_string(props.label_right.as_deref()),
        compose_optional_string(props.name.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_modal(
    props: &ModalProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_kotlin(&context.signal_path(&props.open));
    output.push_str(&format!(
        "{pad}DoweModal(open = state.bool(\"{path}\"), close = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, radius = {}, disableOverlayClose = {}, hideCloseButton = {}, header = ",
        compose_close_action(&path, props.on_close.as_deref(), context),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_card_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button,
    ));
    render_compose_optional_region_lambda(
        header,
        indent,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(", footer = ");
    render_compose_optional_region_lambda(
        footer,
        indent,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(") {\n");
    for child in body {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_alert_dialog(
    props: &AlertDialogProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_kotlin(&context.signal_path(&props.open));
    output.push_str(&format!(
        "{pad}DoweAlertDialog(open = state.bool(\"{path}\"), close = {}, title = {}, description = {}, confirmText = {}, cancelText = {}, backgroundColor = {}, contentColor = {}, dangerColor = {}, radius = {}, loading = {}, onConfirm = {}, onCancel = {})\n",
        compose_close_action(&path, props.on_cancel.as_deref(), context),
        compose_string_literal(&props.title),
        compose_string_literal(&props.description),
        compose_string_literal(&props.confirm_text),
        compose_string_literal(&props.cancel_text),
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Danger))),
        compose_card_radius(&props.style.style),
        props.loading,
        compose_optional_component_action(props.on_confirm.as_deref(), None, context),
        compose_optional_component_action(props.on_cancel.as_deref(), None, context),
    ));
}

fn render_compose_tooltip(
    props: &TooltipProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweTooltip(label = {}, position = {}, backgroundColor = {}, contentColor = {}, modifier = {}) {{\n",
        compose_string_literal(&props.label),
        compose_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        modifier_for_style(&props.style.style)
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_toast(
    props: &ToastProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (visible, title, description, close) = if let Some(source) = props.source.as_deref() {
        let path = escape_kotlin(&context.signal_path(source));
        (
            format!("state.bool(\"{path}.visible\")"),
            format!("state.text(\"{path}.title\")"),
            format!("state.text(\"{path}.message\")"),
            format!("{{ state.write(\"{path}.visible\", false) }}"),
        )
    } else {
        (
            "true".to_string(),
            props.title
                .as_deref()
                .map(compose_string_literal)
                .unwrap_or_else(|| "\"\"".to_string()),
            compose_string_literal(&props.description),
            "null".to_string(),
        )
    };
    output.push_str(&format!(
        "{pad}DoweToast(visible = {visible}, title = {title}, description = {description}, position = {}, backgroundColor = {}, contentColor = {}, showIcon = {}, kind = {}, close = {close})\n",
        compose_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        props.show_icon,
        compose_string_literal(props.kind.as_str()),
    ));
}

fn render_compose_dropdown(
    props: &DropdownProps,
    trigger: &[ViewNode],
    header: &[ViewNode],
    entries: &[OverlayEntry],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweDropdown(backgroundColor = {}, contentColor = {}, modifier = {}) {{\n",
        variant_container(&props.style),
        variant_content(&props.style),
        modifier_for_style(&props.style.style)
    ));
    for child in trigger {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}} content: {{\n"));
    for child in header {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    for entry in entries {
        render_compose_overlay_entry(entry, indent + 4, output, props, context);
    }
    for child in footer {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_command(
    props: &CommandProps,
    entries: &[CommandEntry],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (open, close) = props
        .open
        .as_deref()
        .map(|path| {
            let path = escape_kotlin(&context.signal_path(path));
            (
                format!("state.bool(\"{path}\")"),
                format!("{{ state.write(\"{path}\", false) }}"),
            )
        })
        .unwrap_or_else(|| ("false".to_string(), "{}".to_string()));
    output.push_str(&format!(
        "{pad}DoweCommand(open = {open}, close = {close}, placeholder = {}, emptyText = {}, closeText = {}, navigateText = {}, selectText = {}, toggleText = {}, shortcut = {}, showFooter = {}, backgroundColor = {}, contentColor = {}, accentColor = {}) {{\n",
        compose_string_literal(&props.placeholder),
        compose_string_literal(&props.empty_text),
        compose_string_literal(&props.close_text),
        compose_string_literal(&props.navigate_text),
        compose_string_literal(&props.select_text),
        compose_string_literal(&props.toggle_text),
        compose_string_literal(&props.shortcut),
        props.show_footer,
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Muted))),
    ));
    for entry in entries {
        render_compose_command_entry(
            entry,
            indent + 4,
            output,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_overlay_entry(
    entry: &OverlayEntry,
    indent: usize,
    output: &mut String,
    props: &DropdownProps,
    context: &ComposeReactiveContext,
) {
    match entry {
        OverlayEntry::Item(item) => render_compose_overlay_item(
            item,
            indent,
            output,
            variant_container(&props.style),
            variant_content(&props.style),
            context,
        ),
        OverlayEntry::Divider => {
            let pad = " ".repeat(indent);
            output.push_str(&format!(
                "{pad}Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(DoweDesign.muted))\n"
            ));
        }
    }
}

fn render_compose_command_entry(
    entry: &CommandEntry,
    indent: usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match entry {
        CommandEntry::Item(item) => render_compose_overlay_item(
            item,
            indent,
            output,
            "Color.Transparent",
            "DoweDesign.onBackground",
            context,
        ),
        CommandEntry::Group { label, icon, items } => {
            output.push_str(&format!(
                "{pad}Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n"
            ));
            output.push_str(&format!(
                "{pad}    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {{\n"
            ));
            if let Some(icon) = icon {
                render_compose_side_icon(icon, indent + 8, output);
            }
            output.push_str(&format!(
                "{pad}        Text(text = {}, color = DoweDesign.onMuted, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)\n",
                compose_string_literal(label)
            ));
            output.push_str(&format!("{pad}    }}\n"));
            for item in items {
                render_compose_overlay_item(
                    item,
                    indent + 4,
                    output,
                    "Color.Transparent",
                    "DoweDesign.onBackground",
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_compose_overlay_item(
    item: &OverlayItemProps,
    indent: usize,
    output: &mut String,
    background_color: &str,
    content_color: &str,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweOverlayItem(label = {}, description = {}, disabled = {}, backgroundColor = {background_color}, contentColor = {content_color}, onClick = {}) {{\n",
        compose_string_literal(&item.label),
        compose_optional_string(item.description.as_deref()),
        item.disabled,
        compose_optional_component_action(item.on_click.as_deref(), item.navigation.as_ref(), context)
    ));
    if let Some(icon) = item.icon.as_ref() {
        render_compose_side_icon(icon, indent + 4, output);
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_optional_region_lambda(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    if children.is_empty() {
        output.push_str("null");
        return;
    }
    output.push_str("{\n");
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{}}}", " ".repeat(indent)));
}

fn render_compose_optional_icon_lambda(
    icon: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
) {
    if let Some(icon) = icon {
        output.push_str("{\n");
        render_compose_side_icon(icon, indent + 4, output);
        output.push_str(&format!("{}}}", " ".repeat(indent)));
    } else {
        output.push_str("null");
    }
}

fn render_compose_side_icon(icon: &SideNavIcon, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
        compose_svg_view_box(&icon.props.view_box),
        modifier_for_style(&icon.props.style),
        compose_svg_color(&icon.props.style),
        compose_svg_paths(&icon.paths)
    ));
}

fn compose_optional_component_action(
    action: Option<&str>,
    navigation: Option<&NavigationAction>,
    context: &ComposeReactiveContext,
) -> String {
    action
        .and_then(|name| context.action_id(name))
        .map(|id| {
            let item = context
                .active_item()
                .map(|value| format!(", {value}"))
                .unwrap_or_default();
            format!(
                "{{ actionScope.launch {{ state.run(\"{}\"{item}) }} }}",
                escape_kotlin(id)
            )
        })
        .or_else(|| navigation.map(|action| compose_navigation_action(Some(action))))
        .unwrap_or_else(|| "null".to_string())
}

fn compose_close_action(
    path: &str,
    action: Option<&str>,
    context: &ComposeReactiveContext,
) -> String {
    let after_close = action
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "; actionScope.launch {{ state.run(\"{}\") }}",
                escape_kotlin(id)
            )
        })
        .unwrap_or_default();
    format!("{{ state.write(\"{path}\", false){after_close} }}")
}

fn compose_variant_border(props: &VariantProps) -> String {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        variant_content(props).to_string()
    } else {
        "null".to_string()
    }
}

fn render_compose_tabs(
    props: &TabsProps,
    tabs: &[TabItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let initial = tabs
        .first()
        .map(|tab| tab.id.as_str())
        .unwrap_or_default();
    output.push_str(&format!(
        "{pad}DoweTabs(items = {}, initialId = {}, modifier = {}, position = {}, variant = {}, backgroundColor = {}, contentColor = {}, activeBackgroundColor = {}, activeContentColor = {}, accentColor = {}, borderColor = {}, radius = {}, fontFamily = {}) {{ activeTab ->\n",
        compose_tabs_items(tabs),
        compose_string_literal(initial),
        modifier_for_container_style(&props.style, flow),
        compose_string_literal(props.position.as_str()),
        compose_string_literal(props.variant.as_str()),
        tabs_list_background(props),
        tabs_list_content(props),
        tabs_active_background(props),
        tabs_active_content(props),
        tabs_accent(props),
        tabs_border(props),
        compose_control_radius(&props.style),
        compose_font_value(current_font, default_family),
    ));
    for (index, tab) in tabs.iter().enumerate() {
        output.push_str(&format!(
            "{pad}    {} (activeTab == {}) {{\n",
            if index == 0 { "if" } else { "else if" },
            compose_string_literal(&tab.id)
        ));
        for child in &tab.children {
            render_compose_node_in_flow(
                child,
                indent + 8,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_show_condition(show: &VisibilityCondition, context: &ComposeReactiveContext) -> String {
    match show {
        VisibilityCondition::Static(value) => {
            format!("{} ?: true", compose_bool_value(value))
        }
        VisibilityCondition::Signal(path) => {
            if let Some(item) = context.item_value(path) {
                let path = context.item_path(path).unwrap_or_else(|| path.to_string());
                format!("state.bool(\"{}\", {item})", escape_kotlin(&path))
            } else {
                format!(
                    "state.bool(\"{}\")",
                    escape_kotlin(&context.signal_path(path))
                )
            }
        }
    }
}

fn render_compose_nav_menu(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        compose_side_nav_metrics(props.size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweNavMenu(modifier = {}, gap = {gap}.dp, popoverBackgroundColor = DoweDesign.background, popoverContentColor = DoweDesign.onBackground, content = {{ openIndex, toggle ->\n",
        modifier_for_container_style(&props.style.style, flow)
    ));
    for (index, item) in items.iter().enumerate() {
        render_compose_nav_menu_trigger(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}, popover = {{ openIndex ->\n"));
    for (index, item) in items.iter().enumerate() {
        render_compose_nav_menu_popover(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            description_size,
            border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}})\n"));
}

fn render_compose_nav_menu_trigger(
    index: usize,
    item: &NavMenuItem,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    match item {
        NavMenuItem::Item(props) => render_compose_nav_menu_button(
            props,
            compose_nav_menu_action(props, context),
            compose_nav_menu_active(props.navigation.as_ref()),
            false,
            indent,
            output,
            nav,
            padding_horizontal,
            padding_vertical,
            label_size,
            border,
            inherited_font,
            default_family,
        ),
        NavMenuItem::Submenu { props, .. } | NavMenuItem::Megamenu { props, .. } => {
            render_compose_nav_menu_button(
                props,
                format!("{{ toggle({index}) }}"),
                format!("openIndex == {index}"),
                true,
                indent,
                output,
                nav,
                padding_horizontal,
                padding_vertical,
                label_size,
                border,
                inherited_font,
                default_family,
            );
        }
    }
}

fn render_compose_nav_menu_popover(
    index: usize,
    item: &NavMenuItem,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    description_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match item {
        NavMenuItem::Submenu { items, .. } => {
            output.push_str(&format!("{pad}if (openIndex == {index}) {{\n"));
            output.push_str(&format!("{pad}    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n"));
            for item in items {
                render_compose_nav_menu_subitem(
                    item,
                    indent + 8,
                    output,
                    nav,
                    padding_horizontal,
                    padding_vertical,
                    label_size,
                    description_size,
                    border,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        NavMenuItem::Megamenu { content, .. } => {
            output.push_str(&format!("{pad}if (openIndex == {index}) {{\n"));
            output.push_str(&format!("{pad}    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {{\n"));
            for child in content {
                render_compose_node_in_flow(
                    child,
                    indent + 8,
                    output,
                    ComposeFlow::Block,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        NavMenuItem::Item(_) => {}
    }
}

fn render_compose_nav_menu_subitem(
    props: &NavMenuItemProps,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    description_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    render_compose_nav_menu_button(
        props,
        compose_nav_menu_action(props, context),
        compose_nav_menu_active(props.navigation.as_ref()),
        false,
        indent,
        output,
        nav,
        padding_horizontal,
        padding_vertical,
        label_size,
        border,
        inherited_font,
        default_family,
    );
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}Text(text = \"{}\", modifier = Modifier.padding(start = 12.dp), fontSize = {description_size}.sp, fontFamily = {}, color = LocalContentColor.current.copy(alpha = 0.72f))\n",
            escape_kotlin(description),
            compose_font_value(inherited_font, default_family)
        ));
    }
}

fn render_compose_nav_menu_button(
    props: &NavMenuItemProps,
    action: String,
    active: String,
    arrow: bool,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweNavMenuItem(active = {active}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, backgroundColor = {}, contentColor = {}, borderColor = {border}, onClick = {action}) {{\n",
        variant_container(&nav.style),
        nav_active_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
            compose_svg_view_box(&icon.props.view_box),
            modifier_for_style(&icon.props.style),
            compose_svg_color(&icon.props.style),
            compose_svg_paths(&icon.paths)
        ));
    }
    output.push_str(&format!(
        "{pad}    Text(text = \"{}\", fontSize = {label_size}.sp, fontFamily = {}, fontWeight = FontWeight.Normal)\n",
        escape_kotlin(&props.label),
        compose_font_value(inherited_font, default_family),
    ));
    if arrow {
        output.push_str(&format!(
            "{pad}    Text(text = \"⌄\", fontSize = {label_size}.sp, fontWeight = FontWeight.SemiBold)\n"
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_nav_menu_action(props: &NavMenuItemProps, context: &ComposeReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "{{ actionScope.launch {{ state.run(\"{}\") }} }}",
                escape_kotlin(id)
            )
        })
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| compose_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "null".to_string())
}

fn compose_nav_menu_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_kotlin(path))
        }
        _ => "false".to_string(),
    }
}

fn render_compose_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "{pad}Column(modifier = {}) {{\n",
        modifier_for_container_style(&props.style, flow)
    ));
    for child in app_bar {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}    Row(modifier = Modifier.fillMaxWidth().weight(1f)) {{\n"));
    if !start.is_empty() {
        output.push_str(&format!("{pad}        Column {{\n"));
        for child in start {
            render_compose_node_in_flow(
                child,
                indent + 12,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}        Column(modifier = Modifier.weight(1f)) {{\n"));
    for child in main {
        render_compose_node_in_flow(
            child,
            indent + 12,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}        }}\n"));
    if !end.is_empty() {
        output.push_str(&format!("{pad}        Column {{\n"));
        for child in end {
            render_compose_node_in_flow(
                child,
                indent + 12,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}    }}\n"));
    for child in bottom_bar {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_side_nav(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "{pad}Column(modifier = {}, verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n",
        modifier_for_container_style(&props.style.style, flow)
    ));
    for item in items {
        render_compose_side_nav_item(
            item,
            indent + 4,
            output,
            props,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_side_nav_item(
    item: &SideNavItem,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match item {
        SideNavItem::Header(props) => render_compose_side_nav_row(
            props,
            true,
            compose_side_nav_action(props, context),
            indent,
            output,
            nav,
            inherited_font,
            default_family,
        ),
        SideNavItem::Item(props) => render_compose_side_nav_row(
            props,
            false,
            compose_side_nav_action(props, context),
            indent,
            output,
            nav,
            inherited_font,
            default_family,
        ),
        SideNavItem::Divider => output.push_str(&format!(
            "{pad}Box(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp).height(1.dp).background(DoweDesign.muted))\n"
        )),
        SideNavItem::Submenu { props, open, items } => {
            output.push_str(&format!("{pad}DoweSideNavSubmenu(open = {open}, trigger = {{ toggle ->\n"));
            render_compose_side_nav_row(
                props,
                true,
                "toggle".to_string(),
                indent + 4,
                output,
                nav,
                inherited_font,
                default_family,
            );
            output.push_str(&format!("{pad}}}) {{\n"));
            for item in items {
                render_compose_side_nav_row(
                    item,
                    false,
                    compose_side_nav_action(item, context),
                    indent + 4,
                    output,
                    nav,
                    inherited_font,
                    default_family,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_compose_side_nav_row(
    props: &SideNavItemProps,
    header: bool,
    action: String,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        compose_side_nav_metrics(nav.size);
    let border =
        if nav.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            variant_content(&nav.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweSideNavRow(active = {}, wide = {}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, gap = {gap}.dp, backgroundColor = {}, contentColor = {}, borderColor = {border}, onClick = {action}) {{\n",
        compose_side_nav_active(props.navigation.as_ref()),
        nav.wide,
        variant_container(&nav.style),
        variant_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
            compose_svg_view_box(&icon.props.view_box),
            modifier_for_style(&icon.props.style),
            compose_svg_color(&icon.props.style),
            compose_svg_paths(&icon.paths)
        ));
    }
    output.push_str(&format!(
        "{pad}    Column(modifier = Modifier.weight(1f)) {{\n"
    ));
    output.push_str(&format!(
        "{pad}        Text(text = \"{}\", fontSize = {label_size}.sp, fontFamily = {}, fontWeight = {})\n",
        escape_kotlin(&props.label),
        compose_font_value(inherited_font, default_family),
        if header {
            "FontWeight.SemiBold"
        } else {
            "FontWeight.Normal"
        }
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text(text = \"{}\", fontSize = {description_size}.sp, fontFamily = {}, color = LocalContentColor.current.copy(alpha = 0.72f))\n",
            escape_kotlin(description),
            compose_font_value(inherited_font, default_family),
        ));
    }
    output.push_str(&format!("{pad}    }}\n"));
    if let Some(status) = props.status.as_deref() {
        output.push_str(&format!(
            "{pad}    Text(text = \"{}\", fontSize = {description_size}.sp, fontWeight = FontWeight.SemiBold)\n",
            escape_kotlin(status)
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_side_nav_action(props: &SideNavItemProps, context: &ComposeReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "{{ actionScope.launch {{ state.run(\"{}\") }} }}",
                escape_kotlin(id)
            )
        })
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| compose_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "null".to_string())
}

fn compose_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_kotlin(path))
        }
        _ => "false".to_string(),
    }
}

fn compose_side_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 6, 12, 10),
        SideNavSize::Md => (12, 8, 8, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}

fn render_compose_bar(
    props: &BarProps,
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    if props.boxed {
        output.push_str(&format!(
            "{pad}Box(modifier = {}, contentAlignment = Alignment.Center) {{\n",
            modifier_for_bar(props, flow)
        ));
        output.push_str(&format!(
            "{pad}    CompositionLocalProvider(LocalContentColor provides {}) {{\n",
            variant_content(&props.style)
        ));
        output.push_str(&format!(
            "{pad}        Row(modifier = Modifier.fillMaxWidth().widthIn(max = 1152.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {{\n"
        ));
        render_compose_bar_regions(
            start,
            center,
            end,
            indent + 12,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}        }}\n"));
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!("{pad}}}\n"));
    } else {
        output.push_str(&format!(
            "{pad}Row(modifier = {}, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {{\n",
            modifier_for_bar(props, flow)
        ));
        output.push_str(&format!(
            "{pad}    CompositionLocalProvider(LocalContentColor provides {}) {{\n",
            variant_content(&props.style)
        ));
        render_compose_bar_regions(
            start,
            center,
            end,
            indent + 8,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!("{pad}}}\n"));
    }
}

fn render_compose_bar_regions(
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    render_compose_bar_region(
        start,
        indent,
        output,
        "Arrangement.Start",
        "Modifier",
        inherited_font,
        default_family,
        context,
    );
    render_compose_bar_region(
        center,
        indent,
        output,
        "Arrangement.Center",
        "Modifier.weight(1f)",
        inherited_font,
        default_family,
        context,
    );
    render_compose_bar_region(
        end,
        indent,
        output,
        "Arrangement.End",
        "Modifier",
        inherited_font,
        default_family,
        context,
    );
}

fn render_compose_bar_region(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    arrangement: &str,
    modifier: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    if children.is_empty() {
        return;
    }
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}Row(modifier = {modifier}.padding(8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = {arrangement}) {{\n"
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Inline,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_string_literal(value: &str) -> String {
    format!("\"{}\"", escape_kotlin(value))
}

fn compose_optional_u16(value: Option<u16>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn compose_scheme_color(props: &VariantProps) -> &'static str {
    color_ref(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn compose_text_value_and_change(
    props: &VariantProps,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> (String, String) {
    compose_optional_text_path_and_change(props.element.bind.as_deref(), fallback, context)
}

fn compose_optional_text_path_and_change(
    path: Option<&str>,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> (String, String) {
    path.map(|path| {
        let path = escape_kotlin(&context.signal_path(path));
        (
            format!("state.text(\"{path}\")"),
            format!("{{ state.write(\"{path}\", it) }}"),
        )
    })
    .unwrap_or_else(|| (compose_string_literal(fallback), "{}".to_string()))
}

fn compose_bool_value_and_change(
    props: &VariantProps,
    fallback: bool,
    context: &ComposeReactiveContext,
) -> (String, String) {
    props.element
        .bind
        .as_deref()
        .map(|path| {
            let path = escape_kotlin(&context.signal_path(path));
            (
                format!("state.bool(\"{path}\")"),
                format!("{{ state.write(\"{path}\", it) }}"),
            )
        })
        .unwrap_or_else(|| (fallback.to_string(), "{}".to_string()))
}

fn compose_radio_options(options: &[RadioOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweRadioOption(value = {}, label = {}, disabled = {})",
                compose_string_literal(&option.value),
                compose_string_literal(&option.label),
                option.disabled
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_table_columns(columns: &[TableColumn]) -> String {
    let values = columns
        .iter()
        .map(|column| {
            format!(
                "DoweTableColumn(field = {}, label = {}, align = {}, width = {})",
                compose_string_literal(&column.field),
                compose_string_literal(&column.label),
                compose_table_align(column.align),
                compose_optional_string(column.width.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_table_align(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => "DoweTableColumnAlign.Start",
        TableColumnAlign::Center => "DoweTableColumnAlign.Center",
        TableColumnAlign::End => "DoweTableColumnAlign.End",
    }
}

fn compose_table_size(value: TableSize) -> &'static str {
    match value {
        TableSize::Sm => "DoweTableSize.Sm",
        TableSize::Md => "DoweTableSize.Md",
        TableSize::Lg => "DoweTableSize.Lg",
    }
}

fn compose_select_options(options: &[SelectOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweSelectOption(value = {}, label = {}, description = {})",
                compose_string_literal(&option.value),
                compose_string_literal(&option.label),
                compose_optional_string(option.description.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_tabs_items(tabs: &[TabItem]) -> String {
    let values = tabs
        .iter()
        .map(|tab| {
            format!(
                "DoweTabItem(id = {}, label = {})",
                compose_string_literal(&tab.id),
                compose_string_literal(&tab.label)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}
