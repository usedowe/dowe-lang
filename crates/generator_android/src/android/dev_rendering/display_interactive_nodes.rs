fn render_dev_android_interactive_display_node(
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
) -> bool {
    if !matches!(node, ViewNode::Accordion { .. } | ViewNode::Carousel { .. }) {
        return false;
    }
    let output_start = output.len();
    match node {
        ViewNode::Accordion { props, items } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let mut style = props.style.clone();
            style.variant.get_or_insert(ComponentVariant::Ghost);
            let variant = style.variant.unwrap_or(ComponentVariant::Ghost);
            let content_color = dev_card_variant_content(&style);
            let current_color = Some(dev_content_colors(content_color, dev_card_variant_title(&style)));
            let radius = dev_style_radius(&props.style.style);
            let item_background = match variant {
                ComponentVariant::Outlined => {
                    java_color(ColorToken::Surface)
                }
                _ => "Color.TRANSPARENT",
            };
            let outer_border = if variant == ComponentVariant::Outlined {
                java_color(family_color(style.color.unwrap_or(ColorFamily::Primary)))
            } else {
                "null"
            };
            let item_border = match variant {
                ComponentVariant::Solid => "null".to_string(),
                ComponentVariant::Outlined => {
                    format!(
                        "doweAlpha({}, 0.24f)",
                        java_color(family_color(style.color.unwrap_or(ColorFamily::Primary)))
                    )
                }
                ComponentVariant::Ghost | ComponentVariant::Line => {
                    let alpha = if variant == ComponentVariant::Ghost {
                        "0.22f"
                    } else {
                        "0.24f"
                    };
                    format!("doweAlpha({content_color}, {alpha})")
                }
            };
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweAccordion({}, \"{}\", {}, {}, {}, {}, {}, {}, {radius});\n",
                                        props.multiple,
                                        variant.as_str(),
                                        dev_card_variant_container(&style),
                                        content_color,
                                        outer_border,
                                        item_background,
                                        item_border,
                                        variant == ComponentVariant::Solid,
                                    ));
            apply_dev_android_style(&style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for item in items {
                let arrow = side_nav_submenu_arrow_icon();
                let arrow_view =
                    render_dev_android_icon_view(&arrow, counter, output, Some(content_color));
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
            render_dev_android_carousel_display_node(
                props,
                slides,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        _ => {}
    }
    let calibrated = output[output_start..]
        .replace(
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, 22f);",
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, doweNativeTextSize(22f));",
        )
        .replace(
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, 18f);",
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, doweNativeTextSize(18f));",
        )
        .replace(
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, 14f);",
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, doweNativeTextSize(14f));",
        )
        .replace(
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, 12f);",
            "setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, doweNativeTextSize(12f));",
        );
    output.truncate(output_start);
    output.push_str(&calibrated);
    true
}
