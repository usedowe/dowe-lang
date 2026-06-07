fn render_swift_node_in_flow(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if let Some(expression) = context.node_expression(node) {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}{expression}\n"));
        return;
    }
    if let Some(show) = node_element_props(node).and_then(|props| props.show.as_ref()) {
        let pad = " ".repeat(indent);
        output.push_str(&format!(
            "{pad}if {} {{\n",
            swift_show_condition(show, context)
        ));
        render_swift_node_expression(
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
        render_swift_node_expression(
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

fn render_swift_node_expression(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if should_type_erase_swift_node(node) {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}AnyView(\n"));
        render_swift_node_body(
            node,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad})\n"));
    } else {
        render_swift_node_body(
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

fn should_type_erase_swift_node(node: &ViewNode) -> bool {
    match node {
        ViewNode::Scope { .. } | ViewNode::Children => false,
        ViewNode::Alert { props } if props.visible.is_some() => false,
        _ => true,
    }
}

fn render_swift_node_body(
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
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
            for child in children {
                render_swift_node_in_flow(
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
                "{pad}ForEach(state.rows(\"{}\")) {{ row in\n",
                escape_swift(&context.signal_path(collection))
            ));
            let context = context.with_item(item, "row.value".to_string());
            for child in children {
                render_swift_node_in_flow(
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
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                    swift_cover_value(props.cover.as_ref().expect("cover"))
                ));
                if let Some(overlay) = props.overlay.as_ref() {
                    output.push_str(&format!(
                        "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                        swift_overlay_value(overlay)
                    ));
                }
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}}}\n"));
            }
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_container_style(props, flow),
            );
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                    swift_cover_value(props.cover.as_ref().expect("cover"))
                ));
                if let Some(overlay) = props.overlay.as_ref() {
                    output.push_str(&format!(
                        "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                        swift_overlay_value(overlay)
                    ));
                }
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else if let Some(background) = props.background.as_ref() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    if let background = {} {{\n{pad}        DoweSectionBackgroundView(background: background)\n{pad}    }}\n",
                    swift_section_background_value(background)
                ));
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}}}\n"));
            }
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_container_style(props, flow),
            );
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}HStack(alignment: {}, spacing: {}) {{\n",
                swift_vertical_alignment(props.align.as_ref()),
                swift_gap(props.gap.as_ref())
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    NativeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
            append_swift_modifiers(output, indent, &swift_modifiers_for_layout(props, flow));
        }
        ViewNode::Grid { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}LazyVGrid(columns: {}, alignment: {}, spacing: {}) {{\n",
                swift_grid_columns(props),
                swift_grid_horizontal_alignment(props.justify.as_ref()),
                swift_gap(props.gap.as_ref())
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    NativeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
            append_swift_modifiers(output, indent, &swift_modifiers_for_grid(props, flow));
        }
        ViewNode::Card { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            if props.style.cover.is_some() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                    swift_cover_value(props.style.cover.as_ref().expect("cover"))
                ));
                if let Some(overlay) = props.style.overlay.as_ref() {
                    output.push_str(&format!(
                        "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                        swift_overlay_value(overlay)
                    ));
                }
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}}}\n"));
            }
            let mut modifiers = swift_modifiers_for_container_style(&props.style, flow);
            modifiers.push(format!(".background({})", card_variant_container(props)));
            modifiers.push(format!(".foregroundStyle({})", card_variant_content(props)));
            let radius = swift_card_radius(&props.style);
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(props)
                ));
            }
            append_swift_modifiers(output, indent, &modifiers);
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
                        .map(|value| format!(", item: {value}"))
                        .unwrap_or_default();
                    format!("{{ state.run(\"{}\"{item}) }}", escape_swift(id))
                })
                .unwrap_or_else(|| swift_navigation_action(props.navigation.as_ref()));
            output.push_str(&format!("{pad}Button(action: {action}) {{\n"));
            output.push_str(&format!("{pad}    HStack(spacing: 0) {{\n"));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 8,
                    output,
                    NativeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
            let mut modifiers = swift_modifiers_for_style(&props.style);
            modifiers.push(format!(".background({})", variant_container(props)));
            modifiers.push(format!(".foregroundStyle({})", variant_content(props)));
            let radius = swift_control_radius(&props.style);
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(props)
                ));
            }
            modifiers.push(".buttonStyle(.plain)".to_string());
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Input { props } => {
            let binding = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    format!(
                        "state.binding(\"{}\")",
                        escape_swift(&context.signal_path(path))
                    )
                })
                .unwrap_or_else(|| "nil".to_string());
            let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!("Optional({})", color_ref(ColorToken::Muted))
                } else {
                    "nil".to_string()
                };
            output.push_str(&format!(
                "{pad}DoweInputField(value: {binding}, label: {}, placeholder: {}, floating: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_optional_literal(props.label.as_deref()),
                swift_string_literal(props.placeholder.as_deref().unwrap_or_default()),
                props.label_floating,
                swift_font_value(
                    props.style.font.as_ref().or(inherited_font),
                    &size,
                    default_family,
                ),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                variant_container(props),
                variant_content(props),
                swift_control_radius(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::Select { props, options } => {
            let binding = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    format!(
                        "state.binding(\"{}\")",
                        escape_swift(&context.signal_path(path))
                    )
                })
                .unwrap_or_else(|| "nil".to_string());
            let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!("Optional({})", color_ref(ColorToken::Muted))
                } else {
                    "nil".to_string()
                };
            output.push_str(&format!(
                "{pad}DoweSelectField(value: {binding}, label: {}, placeholder: {}, floating: {}, options: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_optional_literal(props.label.as_deref()),
                swift_string_literal(props.placeholder.as_deref().unwrap_or("Select an option")),
                props.label_floating,
                swift_select_options(options),
                swift_font_value(props.style.font.as_ref().or(inherited_font), &size, default_family),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                variant_container(props),
                variant_content(props),
                swift_control_radius(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
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
        ViewNode::Checkbox { props } => render_swift_checkbox(props, indent, output, context),
        ViewNode::Color { props } => render_swift_color(props, indent, output, context),
        ViewNode::Date { props } => render_swift_date(props, indent, output, context),
        ViewNode::DateRange { props } => render_swift_date_range(props, indent, output, context),
        ViewNode::RadioGroup { props, options } => {
            render_swift_radio_group(props, options, indent, output, context)
        }
        ViewNode::Toggle { props } => render_swift_toggle(props, indent, output, context),
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
        ViewNode::Divider { props } => {
            output.push_str(&format!("{pad}Rectangle()\n"));
            append_swift_modifiers(output, indent, &swift_modifiers_for_divider(props, flow));
        }
        ViewNode::Title { props, value } => {
            output.push_str(&format!(
                "{pad}Text({})\n",
                swift_text_expression(value, props.i18n.as_deref(), context)
            ));
            let modifiers = swift_modifiers_for_text(
                true,
                props,
                props.style.font.as_ref().or(inherited_font),
                default_family,
            );
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Text { props, value } => {
            output.push_str(&format!(
                "{pad}Text({})\n",
                swift_text_expression(value, props.i18n.as_deref(), context)
            ));
            let modifiers = swift_modifiers_for_text(
                false,
                props,
                props.style.font.as_ref().or(inherited_font),
                default_family,
            );
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Alert { props } => {
            if let Some(visible) = props.visible.as_deref() {
                output.push_str(&format!(
                    "{pad}if state.bool(\"{}\") {{\n",
                    escape_swift(&context.signal_path(visible))
                ));
            }
            let alert_pad = if props.visible.is_some() {
                format!("{pad}    ")
            } else {
                pad.clone()
            };
            let radius = swift_control_radius(&props.style.style);
            output.push_str(&format!("{alert_pad}HStack(spacing: CGFloat(12)) {{\n"));
            output.push_str(&format!(
                "{alert_pad}    Text({})\n{alert_pad}        .frame(maxWidth: .infinity, alignment: .leading)\n",
                swift_text_expression(&props.message, None, context)
            ));
            if let Some(action) = props
                .on_close
                .as_deref()
                .and_then(|name| context.action_id(name))
            {
                output.push_str(&format!(
                    "{alert_pad}    Button(action: {{ state.run(\"{}\") }}) {{ Text(\"x\") }}\n",
                    escape_swift(action)
                ));
            }
            output.push_str(&format!("{alert_pad}}}\n"));
            let mut modifiers = swift_modifiers_for_style(&props.style.style);
            modifiers.push(".padding(.horizontal, CGFloat(14))".to_string());
            modifiers.push(".padding(.vertical, CGFloat(10))".to_string());
            modifiers.push(format!(".background({})", variant_container(&props.style)));
            modifiers.push(format!(".foregroundStyle({})", variant_content(&props.style)));
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined
            {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(&props.style)
                ));
            }
            append_swift_modifiers(
                output,
                if props.visible.is_some() {
                    indent + 4
                } else {
                    indent
                },
                &modifiers,
            );
            if props.visible.is_some() {
                output.push_str(&format!("{pad}}}\n"));
            }
        }
        ViewNode::Svg { props, paths } => {
            output.push_str(&format!(
                "{pad}DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
                swift_svg_view_box(&props.view_box),
                swift_svg_color(&props.style),
                swift_svg_paths(paths)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        } => render_swift_bar(
            props,
            start,
            center,
            end,
            SwiftBarOptions {
                start_padding: 12,
                center_padding: 12,
                end_padding: 12,
            },
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Footer {
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
        } => render_swift_bar(
            props,
            start,
            center,
            end,
            SwiftBarOptions {
                start_padding: 8,
                center_padding: 8,
                end_padding: 8,
            },
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::SideNav { props, items } => {
            render_swift_side_nav(
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
            render_swift_side_nav(
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
            render_swift_nav_menu(
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
            render_swift_scaffold(
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
            render_swift_tabs(
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
            render_swift_drawer(
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
            render_swift_avatar(props, icon.as_ref(), indent, output, context);
        }
        ViewNode::Badge { props, children } => {
            render_swift_badge(
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
            render_swift_chip(
                props,
                value,
                start.as_ref(),
                end.as_ref(),
                indent,
                output,
                context,
            );
        }
        ViewNode::Skeleton { props } => render_swift_skeleton(props, indent, output, flow),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            render_swift_modal(
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
            render_swift_alert_dialog(props, indent, output, context);
        }
        ViewNode::Tooltip { props, children } => {
            render_swift_tooltip(
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
        ViewNode::Toast { props } => render_swift_toast(props, indent, output, context),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            render_swift_dropdown(
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
            render_swift_command(
                props,
                entries,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        }
        ViewNode::Children => {
            if let Some(expression) = context.children_expression.as_ref() {
                output.push_str(&format!("{pad}{expression}\n"));
            }
        }
    }
}

fn render_swift_drawer(
    props: &DrawerProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_swift(&context.signal_path(&props.open));
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!("Optional({})", card_variant_content(&props.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweDrawer(open: state.bool(\"{path}\"), close: {{ state.write(\"{path}\", value: false) }}, position: \"{}\", backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {}, disableOverlayClose: {}, hideCloseButton: {}) {{\n",
        props.position.as_str(),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_drawer_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button
    ));
    output.push_str(&format!(
        "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    for child in children {
        render_swift_node_in_flow(
            child,
            indent + 8,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}    }}\n"));
    append_swift_modifiers(
        output,
        indent + 4,
        &swift_modifiers_for_container_style(&props.style.style, NativeFlow::Block),
    );
    output.push_str(&format!("{pad}}}\n"));
}

fn render_swift_avatar(
    props: &AvatarProps,
    icon: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweAvatar(source: {}, name: {}, alt: {}, size: {}, status: {}, bordered: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, action: {}, hasIcon: {}) {{\n",
        swift_optional_literal(props.src.as_deref()),
        swift_optional_literal(props.name.as_deref()),
        swift_string_literal(&props.alt),
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.status.map(|value| value.as_str())),
        props.bordered,
        variant_container(&props.style),
        variant_content(&props.style),
        variant_content(&props.style),
        swift_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        icon.is_some()
    ));
    if let Some(icon) = icon {
        render_swift_side_icon(icon, indent + 4, output);
    } else {
        output.push_str(&format!("{pad}    EmptyView()\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_badge(
    props: &BadgeProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweBadge(text: {}, position: {}, backgroundColor: {}, contentColor: {}) {{\n",
        swift_string_literal(&props.text),
        swift_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for child in children {
        render_swift_node_in_flow(
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
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_chip(
    props: &ChipProps,
    value: &str,
    start: Option<&SideNavIcon>,
    end: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    output.push_str(&format!(
        "{pad}DoweChip(text: {}, size: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, action: {}, hasStart: {}, hasEnd: {}) {{\n",
        swift_string_literal(value),
        swift_string_literal(size.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_optional_component_action(props.on_close.as_deref(), None, context),
        start.is_some(),
        end.is_some(),
    ));
    if let Some(icon) = start {
        render_swift_side_icon(icon, indent + 4, output);
    } else {
        output.push_str(&format!("{pad}    EmptyView()\n"));
    }
    output.push_str(&format!("{pad}}} end: {{\n"));
    if let Some(icon) = end {
        render_swift_side_icon(icon, indent + 4, output);
    } else {
        output.push_str(&format!("{pad}    EmptyView()\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_skeleton(
    props: &SkeletonProps,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSkeleton(variant: {}, animation: {})\n",
        swift_string_literal(props.variant.as_str()),
        swift_string_literal(props.animation.as_str())
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style, flow),
    );
}

fn render_swift_audio(props: &AudioProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("Optional({})", card_variant_content(&props.style))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweAudioView(source: {}, subtitle: {}, avatarSource: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
        swift_string_literal(&props.src),
        swift_optional_literal(props.subtitle.as_deref()),
        swift_optional_literal(props.avatar_src.as_deref()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_card_radius(&props.style.style)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_image(props: &ImageProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("Optional({})", card_variant_content(&props.style))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweImageView(source: {}, alt: {}, aspect: {}, objectFit: {}, loading: {}, hideControls: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
        swift_string_literal(&props.src),
        swift_string_literal(&props.alt),
        swift_string_literal(props.aspect.as_str()),
        swift_string_literal(props.object_fit.as_str()),
        swift_string_literal(props.loading.as_str()),
        props.hide_controls,
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_card_radius(&props.style.style)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_accordion(
    props: &AccordionProps,
    items: &[AccordionItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("Optional({})", variant_content(&props.style))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweAccordionView(multiple: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}) {{\n",
        props.multiple,
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for item in items {
        output.push_str(&format!(
            "{pad}    DoweAccordionItemView(id: {}, label: {}, disabled: {}, defaultOpen: {}) {{\n",
            swift_string_literal(&item.id),
            swift_string_literal(&item.label),
            item.disabled,
            item.default_open
        ));
        for child in &item.children {
            render_swift_node_in_flow(
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
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_carousel(
    props: &CarouselProps,
    slides: &[CarouselSlide],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCarouselView(autoplay: {}, autoplayInterval: {}, disableLoop: {}, hideControls: {}, hideIndicators: {}, showNavigation: {}, showCounter: {}, orientation: {}, size: {}, indicatorType: {}, title: {}, slideWidth: {}, slideHeight: {}, slidesPerView: {}, gap: {}, accentColor: {}) {{\n",
        props.autoplay,
        props.autoplay_interval,
        props.disable_loop,
        props.hide_controls,
        props.hide_indicators,
        props.show_navigation,
        props.show_counter,
        swift_string_literal(props.orientation.as_str()),
        swift_string_literal(props.size.as_str()),
        swift_string_literal(props.indicator_type.as_str()),
        swift_optional_literal(props.title.as_deref()),
        swift_optional_u16(props.slide_width),
        swift_optional_u16(props.slide_height),
        props.slides_per_view,
        props.gap,
        swift_scheme_color(&props.style),
    ));
    for slide in slides {
        output.push_str(&format!(
            "{pad}    DoweCarouselSlideView(id: {}) {{\n",
            swift_string_literal(&slide.id)
        ));
        for child in &slide.children {
            render_swift_node_in_flow(
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
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_checkbox(
    props: &CheckboxProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_bool_binding(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweCheckboxView(checked: {binding}, enabled: {}, label: {}, name: {}, accentColor: {})\n",
        !props.disabled,
        swift_optional_literal(props.style.label.as_deref()),
        swift_optional_literal(props.name.as_deref()),
        swift_scheme_color(&props.style)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_color(
    props: &ColorProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_string_binding(&props.style, &props.value, context);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined) == ComponentVariant::Outlined {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweColorField(value: {binding}, label: {}, placeholder: {}, floating: {}, size: {}, name: {}, helpText: {}, errorText: {}, showHex: {}, showRgb: {}, showCmyk: {}, showOklch: {}, backgroundColor: {}, contentColor: {}, borderColor: {border})\n",
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select color")),
        props.style.label_floating,
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        props.show_hex,
        props.show_rgb,
        props.show_cmyk,
        props.show_oklch,
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_date(
    props: &DateProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_string_binding(&props.style, props.value.as_deref().unwrap_or_default(), context);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined) == ComponentVariant::Outlined {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweDateField(value: {binding}, label: {}, placeholder: {}, floating: {}, size: {}, name: {}, helpText: {}, errorText: {}, min: {}, max: {}, backgroundColor: {}, contentColor: {}, borderColor: {border})\n",
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date")),
        props.style.label_floating,
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        swift_optional_literal(props.min.as_deref()),
        swift_optional_literal(props.max.as_deref()),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_date_range(
    props: &DateRangeProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let start_binding = swift_optional_string_binding(
        props.start.as_deref(),
        props.start_value.as_deref().unwrap_or_default(),
        context,
    );
    let end_binding = swift_optional_string_binding(
        props.end.as_deref(),
        props.end_value.as_deref().unwrap_or_default(),
        context,
    );
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined) == ComponentVariant::Outlined {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweDateRangeField(startValue: {start_binding}, endValue: {end_binding}, label: {}, placeholder: {}, floating: {}, size: {}, name: {}, helpText: {}, errorText: {}, min: {}, max: {}, backgroundColor: {}, contentColor: {}, borderColor: {border})\n",
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date range")),
        props.style.label_floating,
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        swift_optional_literal(props.min.as_deref()),
        swift_optional_literal(props.max.as_deref()),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_radio_group(
    props: &RadioGroupProps,
    options: &[RadioOption],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_string_binding(&props.style, "", context);
    output.push_str(&format!(
        "{pad}DoweRadioGroupView(value: {binding}, options: {}, size: {}, name: {}, label: {}, helpText: {}, errorText: {}, accentColor: {})\n",
        swift_radio_options(options),
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.style.label.as_deref()),
        swift_optional_literal(props.info.as_deref()),
        swift_optional_literal(props.error.as_deref()),
        swift_scheme_color(&props.style)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_toggle(
    props: &ToggleProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_bool_binding(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweToggleView(checked: {binding}, enabled: {}, label: {}, labelLeft: {}, labelRight: {}, name: {}, accentColor: {})\n",
        !props.disabled,
        swift_optional_literal(props.style.label.as_deref()),
        swift_optional_literal(props.label_left.as_deref()),
        swift_optional_literal(props.label_right.as_deref()),
        swift_optional_literal(props.name.as_deref()),
        swift_scheme_color(&props.style)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_modal(
    props: &ModalProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_swift(&context.signal_path(&props.open));
    output.push_str(&format!(
        "{pad}DoweModal(open: state.bool(\"{path}\"), close: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, radius: {}, disableOverlayClose: {}, hideCloseButton: {}, hasHeader: {}, hasFooter: {}) {{\n",
        swift_close_action(&path, props.on_close.as_deref(), context),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_card_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button,
        !header.is_empty(),
        !footer.is_empty(),
    ));
    render_swift_region_children(header, indent + 4, output, inherited_font, default_family, context);
    output.push_str(&format!("{pad}}} content: {{\n"));
    render_swift_region_children(body, indent + 4, output, inherited_font, default_family, context);
    output.push_str(&format!("{pad}}} footer: {{\n"));
    render_swift_region_children(footer, indent + 4, output, inherited_font, default_family, context);
    output.push_str(&format!("{pad}}}\n"));
}

fn render_swift_alert_dialog(
    props: &AlertDialogProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_swift(&context.signal_path(&props.open));
    output.push_str(&format!(
        "{pad}DoweAlertDialog(open: state.bool(\"{path}\"), close: {}, title: {}, description: {}, confirmText: {}, cancelText: {}, backgroundColor: {}, contentColor: {}, dangerColor: {}, radius: {}, loading: {}, confirm: {}, cancel: {})\n",
        swift_close_action(&path, props.on_cancel.as_deref(), context),
        swift_string_literal(&props.title),
        swift_string_literal(&props.description),
        swift_string_literal(&props.confirm_text),
        swift_string_literal(&props.cancel_text),
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Danger))),
        swift_card_radius(&props.style.style),
        props.loading,
        swift_optional_component_action(props.on_confirm.as_deref(), None, context),
        swift_optional_component_action(props.on_cancel.as_deref(), None, context),
    ));
}

fn render_swift_tooltip(
    props: &TooltipProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweTooltip(label: {}, position: {}, backgroundColor: {}, contentColor: {}) {{\n",
        swift_string_literal(&props.label),
        swift_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for child in children {
        render_swift_node_in_flow(
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
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_toast(
    props: &ToastProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (visible, title, description, close) = if let Some(source) = props.source.as_deref() {
        let path = escape_swift(&context.signal_path(source));
        (
            format!("state.bool(\"{path}.visible\")"),
            format!("state.text(\"{path}.title\")"),
            format!("state.text(\"{path}.message\")"),
            format!("{{ state.write(\"{path}.visible\", value: false) }}"),
        )
    } else {
        (
            "true".to_string(),
            props.title
                .as_deref()
                .map(swift_string_literal)
                .unwrap_or_else(|| "\"\"".to_string()),
            swift_string_literal(&props.description),
            "nil".to_string(),
        )
    };
    output.push_str(&format!(
        "{pad}DoweToast(visible: {visible}, title: {title}, description: {description}, position: {}, backgroundColor: {}, contentColor: {}, showIcon: {}, kind: {}, close: {close})\n",
        swift_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        props.show_icon,
        swift_string_literal(props.kind.as_str()),
    ));
}

fn render_swift_dropdown(
    props: &DropdownProps,
    trigger: &[ViewNode],
    header: &[ViewNode],
    entries: &[OverlayEntry],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweDropdown(backgroundColor: {}, contentColor: {}) {{\n",
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for child in trigger {
        render_swift_node_in_flow(
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
    render_swift_region_children(header, indent + 4, output, inherited_font, default_family, context);
    for entry in entries {
        render_swift_overlay_entry(entry, indent + 4, output, props, context);
    }
    render_swift_region_children(footer, indent + 4, output, inherited_font, default_family, context);
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn render_swift_command(
    props: &CommandProps,
    entries: &[CommandEntry],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (open, close) = props
        .open
        .as_deref()
        .map(|path| {
            let path = escape_swift(&context.signal_path(path));
            (
                format!("state.bool(\"{path}\")"),
                format!("{{ state.write(\"{path}\", value: false) }}"),
            )
        })
        .unwrap_or_else(|| ("false".to_string(), "{}".to_string()));
    output.push_str(&format!(
        "{pad}DoweCommand(open: {open}, close: {close}, placeholder: {}, emptyText: {}, closeText: {}, navigateText: {}, selectText: {}, toggleText: {}, shortcut: {}, showFooter: {}, backgroundColor: {}, contentColor: {}, accentColor: {}) {{\n",
        swift_string_literal(&props.placeholder),
        swift_string_literal(&props.empty_text),
        swift_string_literal(&props.close_text),
        swift_string_literal(&props.navigate_text),
        swift_string_literal(&props.select_text),
        swift_string_literal(&props.toggle_text),
        swift_string_literal(&props.shortcut),
        props.show_footer,
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Muted))),
    ));
    for entry in entries {
        render_swift_command_entry(
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

fn render_swift_overlay_entry(
    entry: &OverlayEntry,
    indent: usize,
    output: &mut String,
    props: &DropdownProps,
    context: &SwiftReactiveContext,
) {
    match entry {
        OverlayEntry::Item(item) => render_swift_overlay_item(
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
                "{pad}Divider().background(DoweDesign.muted)\n"
            ));
        }
    }
}

fn render_swift_command_entry(
    entry: &CommandEntry,
    indent: usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match entry {
        CommandEntry::Item(item) => render_swift_overlay_item(
            item,
            indent,
            output,
            "Color.clear",
            "DoweDesign.onBackground",
            context,
        ),
        CommandEntry::Group { label, icon, items } => {
            output.push_str(&format!(
                "{pad}VStack(alignment: .leading, spacing: CGFloat(2)) {{\n"
            ));
            output.push_str(&format!(
                "{pad}    HStack(spacing: CGFloat(6)) {{\n"
            ));
            if let Some(icon) = icon {
                render_swift_side_icon(icon, indent + 8, output);
            }
            output.push_str(&format!(
                "{pad}        Text({})\n{pad}            .font(.caption)\n{pad}            .fontWeight(.semibold)\n{pad}            .foregroundStyle(DoweDesign.onMuted)\n",
                swift_string_literal(label)
            ));
            output.push_str(&format!("{pad}    }}\n"));
            for item in items {
                render_swift_overlay_item(
                    item,
                    indent + 4,
                    output,
                    "Color.clear",
                    "DoweDesign.onBackground",
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_swift_overlay_item(
    item: &OverlayItemProps,
    indent: usize,
    output: &mut String,
    background_color: &str,
    content_color: &str,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweOverlayItem(label: {}, description: {}, disabled: {}, backgroundColor: {background_color}, contentColor: {content_color}, action: {}) {{\n",
        swift_string_literal(&item.label),
        swift_optional_literal(item.description.as_deref()),
        item.disabled,
        swift_optional_component_action(item.on_click.as_deref(), item.navigation.as_ref(), context)
    ));
    if let Some(icon) = item.icon.as_ref() {
        render_swift_side_icon(icon, indent + 4, output);
    } else {
        output.push_str(&format!("{pad}    EmptyView()\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_swift_region_children(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if children.is_empty() {
        output.push_str(&format!("{}EmptyView()\n", " ".repeat(indent)));
        return;
    }
    for child in children {
        render_swift_node_in_flow(
            child,
            indent,
            output,
            NativeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn render_swift_side_icon(icon: &SideNavIcon, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
        swift_svg_view_box(&icon.props.view_box),
        swift_svg_color(&icon.props.style),
        swift_svg_paths(&icon.paths)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&icon.props.style));
}

fn swift_optional_component_action(
    action: Option<&str>,
    navigation: Option<&NavigationAction>,
    context: &SwiftReactiveContext,
) -> String {
    action
        .and_then(|name| context.action_id(name))
        .map(|id| {
            let item = context
                .active_item()
                .map(|value| format!(", item: {value}"))
                .unwrap_or_default();
            format!("{{ state.run(\"{}\"{item}) }}", escape_swift(id))
        })
        .or_else(|| navigation.map(|action| swift_navigation_action(Some(action))))
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_close_action(
    path: &str,
    action: Option<&str>,
    context: &SwiftReactiveContext,
) -> String {
    let after_close = action
        .and_then(|name| context.action_id(name))
        .map(|id| format!("; state.run(\"{}\")", escape_swift(id)))
        .unwrap_or_default();
    format!("{{ state.write(\"{path}\", value: false){after_close} }}")
}

fn swift_variant_border(props: &VariantProps) -> String {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("Optional({})", variant_content(props))
    } else {
        "nil".to_string()
    }
}

fn render_swift_tabs(
    props: &TabsProps,
    tabs: &[TabItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let initial = tabs
        .first()
        .map(|tab| tab.id.as_str())
        .unwrap_or_default();
    output.push_str(&format!(
        "{pad}DoweTabs(items: {}, initialId: {}, position: {}, variant: {}, backgroundColor: {}, contentColor: {}, activeBackgroundColor: {}, activeContentColor: {}, accentColor: {}, borderColor: {}, radius: {}, font: {}) {{ activeTab in\n",
        swift_tabs_items(tabs),
        swift_string_literal(initial),
        swift_string_literal(props.position.as_str()),
        swift_string_literal(props.variant.as_str()),
        tabs_list_background(props),
        tabs_list_content(props),
        tabs_active_background(props),
        tabs_active_content(props),
        tabs_accent(props),
        tabs_border(props),
        swift_control_radius(&props.style),
        swift_font_value(current_font, "CGFloat(16)", default_family),
    ));
    for (index, tab) in tabs.iter().enumerate() {
        output.push_str(&format!(
            "{pad}    {} activeTab == {} {{\n",
            if index == 0 { "if" } else { "else if" },
            swift_string_literal(&tab.id)
        ));
        for child in &tab.children {
            render_swift_node_in_flow(
                child,
                indent + 8,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style, flow),
    );
}

fn swift_show_condition(show: &VisibilityCondition, context: &SwiftReactiveContext) -> String {
    match show {
        VisibilityCondition::Static(value) => {
            format!("{} ?? true", swift_bool_value(value))
        }
        VisibilityCondition::Signal(path) => {
            if let Some(item) = context.item_value(path) {
                let path = context.item_path(path).unwrap_or_else(|| path.to_string());
                format!("state.bool(\"{}\", item: {item})", escape_swift(&path))
            } else {
                format!(
                    "state.bool(\"{}\")",
                    escape_swift(&context.signal_path(path))
                )
            }
        }
    }
}

#[derive(Clone, Copy)]
struct SwiftBarOptions {
    start_padding: usize,
    center_padding: usize,
    end_padding: usize,
}

fn render_swift_nav_menu(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        swift_side_nav_metrics(props.size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            format!("Optional({})", variant_content(&props.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweNavMenu(gap: CGFloat({gap}), popoverBackgroundColor: DoweDesign.background, popoverContentColor: DoweDesign.onBackground) {{ openIndex, toggle in\n"
    ));
    for (index, item) in items.iter().enumerate() {
        render_swift_nav_menu_trigger(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            &border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}} popover: {{ openIndex in\n"));
    for (index, item) in items.iter().enumerate() {
        render_swift_nav_menu_popover(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            description_size,
            &border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style.style, flow),
    );
}

fn render_swift_nav_menu_trigger(
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
    context: &SwiftReactiveContext,
) {
    match item {
        NavMenuItem::Item(props) => render_swift_nav_menu_button(
            props,
            swift_nav_menu_action(props, context),
            swift_nav_menu_active(props.navigation.as_ref()),
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
            render_swift_nav_menu_button(
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

fn render_swift_nav_menu_popover(
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
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match item {
        NavMenuItem::Submenu { items, .. } => {
            output.push_str(&format!("{pad}if openIndex == {index} {{\n"));
            output.push_str(&format!("{pad}    VStack(alignment: .leading, spacing: CGFloat(2)) {{\n"));
            for item in items {
                render_swift_nav_menu_subitem(
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
            output.push_str(&format!("{pad}if openIndex == {index} {{\n"));
            output.push_str(&format!("{pad}    VStack(alignment: .leading, spacing: CGFloat(8)) {{\n"));
            for child in content {
                render_swift_node_in_flow(
                    child,
                    indent + 8,
                    output,
                    NativeFlow::Block,
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

fn render_swift_nav_menu_subitem(
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
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    render_swift_nav_menu_button(
        props,
        swift_nav_menu_action(props, context),
        swift_nav_menu_active(props.navigation.as_ref()),
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
            "{pad}Text(\"{}\")\n{pad}    .font({})\n{pad}    .opacity(0.72)\n{pad}    .padding(.leading, CGFloat(12))\n",
            escape_swift(description),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
}

fn render_swift_nav_menu_button(
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
        "{pad}DoweNavMenuItem(active: {active}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), backgroundColor: {}, contentColor: {}, borderColor: {border}, action: {action}) {{\n",
        variant_container(&nav.style),
        nav_active_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
            swift_svg_view_box(&icon.props.view_box),
            swift_svg_color(&icon.props.style),
            swift_svg_paths(&icon.paths)
        ));
        append_swift_modifiers(
            output,
            indent + 4,
            &swift_modifiers_for_style(&icon.props.style),
        );
    }
    output.push_str(&format!(
        "{pad}    Text(\"{}\")\n{pad}        .font({})\n{pad}        .fontWeight(.regular)\n",
        escape_swift(&props.label),
        swift_font_value(
            inherited_font,
            &format!("CGFloat({label_size})"),
            default_family
        )
    ));
    if arrow {
        output.push_str(&format!(
            "{pad}    Text(\"⌄\")\n{pad}        .fontWeight(.semibold)\n"
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn swift_nav_menu_action(props: &NavMenuItemProps, context: &SwiftReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("{{ state.run(\"{}\") }}", escape_swift(id)))
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| swift_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_nav_menu_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_swift(path))
        }
        _ => "false".to_string(),
    }
}

fn render_swift_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    output.push_str(&format!("{pad}VStack(spacing: CGFloat(0)) {{\n"));
    for child in app_bar {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}    HStack(alignment: .top, spacing: CGFloat(0)) {{\n"));
    if !start.is_empty() {
        output.push_str(&format!("{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"));
        for child in start {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"));
    for child in main {
        render_swift_node_in_flow(
            child,
            indent + 12,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}        }}\n{pad}        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"));
    if !end.is_empty() {
        output.push_str(&format!("{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"));
        for child in end {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}    }}\n{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"));
    for child in bottom_bar {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style, flow),
    );
}

fn render_swift_side_nav(
    props: &SideNavProps,
    items: &[SideNavItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "{pad}VStack(alignment: .leading, spacing: CGFloat(2)) {{\n"
    ));
    for item in items {
        render_swift_side_nav_item(
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style.style, flow),
    );
}

fn render_swift_side_nav_item(
    item: &SideNavItem,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match item {
        SideNavItem::Header(props) => render_swift_side_nav_row(
            props,
            true,
            swift_side_nav_action(props, context),
            indent,
            output,
            nav,
            inherited_font,
            default_family,
        ),
        SideNavItem::Item(props) => render_swift_side_nav_row(
            props,
            false,
            swift_side_nav_action(props, context),
            indent,
            output,
            nav,
            inherited_font,
            default_family,
        ),
        SideNavItem::Divider => {
            output.push_str(&format!(
                "{pad}Divider()\n{pad}    .padding(.vertical, CGFloat(8))\n"
            ));
        }
        SideNavItem::Submenu { props, open, items } => {
            output.push_str(&format!("{pad}DoweSideNavSubmenu(open: {open}) {{\n"));
            for item in items {
                render_swift_side_nav_row(
                    item,
                    false,
                    swift_side_nav_action(item, context),
                    indent + 4,
                    output,
                    nav,
                    inherited_font,
                    default_family,
                );
            }
            output.push_str(&format!("{pad}}} label: {{\n"));
            render_swift_side_nav_row(
                props,
                true,
                "nil".to_string(),
                indent + 4,
                output,
                nav,
                inherited_font,
                default_family,
            );
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_swift_side_nav_row(
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
        swift_side_nav_metrics(nav.size);
    let border =
        if nav.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            format!("Optional({})", variant_content(&nav.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweSideNavRow(active: {}, wide: {}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), gap: CGFloat({gap}), backgroundColor: {}, contentColor: {}, borderColor: {border}, action: {action}) {{\n",
        swift_side_nav_active(props.navigation.as_ref()),
        nav.wide,
        variant_container(&nav.style),
        variant_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
            swift_svg_view_box(&icon.props.view_box),
            swift_side_nav_icon_color(icon, nav),
            swift_svg_paths(&icon.paths)
        ));
        append_swift_modifiers(
            output,
            indent + 4,
            &swift_modifiers_for_style(&icon.props.style),
        );
    }
    output.push_str(&format!(
        "{pad}    VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    output.push_str(&format!(
        "{pad}        Text(\"{}\")\n{pad}            .font({})\n{pad}            .fontWeight({})\n",
        escape_swift(&props.label),
        swift_font_value(
            inherited_font,
            &format!("CGFloat({label_size})"),
            default_family
        ),
        if header { ".semibold" } else { ".regular" }
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text(\"{}\")\n{pad}            .font({})\n{pad}            .opacity(0.72)\n",
            escape_swift(description),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
    output.push_str(&format!(
        "{pad}    }}\n{pad}    .frame(maxWidth: .infinity, alignment: .leading)\n"
    ));
    if let Some(status) = props.status.as_deref() {
        output.push_str(&format!(
            "{pad}    Text(\"{}\")\n{pad}        .font({})\n{pad}        .fontWeight(.semibold)\n",
            escape_swift(status),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn swift_side_nav_action(props: &SideNavItemProps, context: &SwiftReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("{{ state.run(\"{}\") }}", escape_swift(id)))
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| swift_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_swift(path))
        }
        _ => "false".to_string(),
    }
}

fn swift_side_nav_icon_color(icon: &SideNavIcon, nav: &SideNavProps) -> String {
    if icon.props.style.text.is_some() {
        swift_svg_color(&icon.props.style)
    } else {
        nav_active_content(&nav.style).to_string()
    }
}

fn swift_side_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 6, 12, 10),
        SideNavSize::Md => (12, 8, 8, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}

fn render_swift_bar(
    props: &BarProps,
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    options: SwiftBarOptions,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let content_width = if props.boxed {
        "CGFloat(1152)"
    } else {
        ".infinity"
    };
    output.push_str(&format!("{pad}ZStack {{\n"));
    output.push_str(&format!(
        "{pad}    HStack(alignment: .center, spacing: 0) {{\n"
    ));
    render_swift_bar_region(
        start,
        indent + 8,
        output,
        ".leading",
        false,
        options.start_padding,
        current_font,
        default_family,
        context,
    );
    if center.is_empty() && !start.is_empty() && !end.is_empty() {
        output.push_str(&format!("{pad}        Spacer(minLength: CGFloat(0))\n"));
    }
    render_swift_bar_region(
        center,
        indent + 8,
        output,
        ".center",
        true,
        options.center_padding,
        current_font,
        default_family,
        context,
    );
    render_swift_bar_region(
        end,
        indent + 8,
        output,
        ".trailing",
        false,
        options.end_padding,
        current_font,
        default_family,
        context,
    );
    output.push_str(&format!("{pad}    }}\n"));
    output.push_str(&format!(
        "{pad}    .frame(maxWidth: {content_width}, alignment: .center)\n"
    ));
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(output, indent, &swift_modifiers_for_bar(props, flow));
}

fn render_swift_bar_region(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    alignment: &str,
    fill: bool,
    padding: usize,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if children.is_empty() {
        return;
    }
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}HStack(alignment: .center, spacing: CGFloat({padding})) {{\n"
    ));
    for child in children {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Inline,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    output.push_str(&format!(
        "{pad}    .padding(.horizontal, CGFloat({padding}))\n"
    ));
    output.push_str(&format!(
        "{pad}    .padding(.vertical, CGFloat({padding}))\n"
    ));
    if fill {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, alignment: {alignment})\n"
        ));
    } else {
        output.push_str(&format!("{pad}    .frame(alignment: {alignment})\n"));
    }
}

fn swift_text_expression(
    value: &str,
    i18n: Option<&str>,
    context: &SwiftReactiveContext,
) -> String {
    if let Some(key) = i18n {
        return format!("String(localized: \"{}\")", escape_swift(key));
    }
    match context.dynamic_path(value) {
        Some(path) => context
            .item_value(value)
            .map(|item| format!("state.text(\"{}\", item: {item})", escape_swift(&path)))
            .unwrap_or_else(|| format!("state.text(\"{}\")", escape_swift(&path))),
        None => format!("\"{}\"", escape_swift(value)),
    }
}

fn swift_string_literal(value: &str) -> String {
    format!("\"{}\"", escape_swift(value))
}

fn swift_optional_u16(value: Option<u16>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_scheme_color(props: &VariantProps) -> &'static str {
    color_ref(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn swift_string_binding(
    props: &VariantProps,
    fallback: &str,
    context: &SwiftReactiveContext,
) -> String {
    swift_optional_string_binding(props.element.bind.as_deref(), fallback, context)
}

fn swift_optional_string_binding(
    path: Option<&str>,
    fallback: &str,
    context: &SwiftReactiveContext,
) -> String {
    path.map(|path| {
        format!(
            "state.binding(\"{}\")",
            escape_swift(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| format!(".constant({})", swift_string_literal(fallback)))
}

fn swift_bool_binding(
    props: &VariantProps,
    fallback: bool,
    context: &SwiftReactiveContext,
) -> String {
    props
        .element
        .bind
        .as_deref()
        .map(|path| {
            format!(
                "state.boolBinding(\"{}\")",
                escape_swift(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| format!(".constant({fallback})"))
}

fn swift_radio_options(options: &[RadioOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweRadioOption(value: {}, label: {}, disabled: {})",
                swift_string_literal(&option.value),
                swift_string_literal(&option.label),
                option.disabled
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_table_columns(columns: &[TableColumn]) -> String {
    let values = columns
        .iter()
        .map(|column| {
            format!(
                "DoweTableColumn(field: {}, label: {}, align: {}, width: {})",
                swift_string_literal(&column.field),
                swift_string_literal(&column.label),
                swift_table_align(column.align),
                swift_optional_literal(column.width.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_table_align(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => ".start",
        TableColumnAlign::Center => ".center",
        TableColumnAlign::End => ".end",
    }
}

fn swift_table_size(value: TableSize) -> &'static str {
    match value {
        TableSize::Sm => ".sm",
        TableSize::Md => ".md",
        TableSize::Lg => ".lg",
    }
}

fn swift_optional_literal(value: Option<&str>) -> String {
    value
        .map(swift_string_literal)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_select_options(options: &[SelectOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweSelectOption(value: {}, label: {}, description: {})",
                swift_string_literal(&option.value),
                swift_string_literal(&option.label),
                swift_optional_literal(option.description.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_tabs_items(tabs: &[TabItem]) -> String {
    let values = tabs
        .iter()
        .map(|tab| {
            format!(
                "DoweTabItem(id: {}, label: {})",
                swift_string_literal(&tab.id),
                swift_string_literal(&tab.label)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}
