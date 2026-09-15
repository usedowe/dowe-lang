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
        "{pad}DoweBadge(text: {}, position: {}, backgroundColor: {}, contentColor: {}, fontSize: CGFloat({}), height: CGFloat({}), horizontalPadding: CGFloat({}), verticalPadding: CGFloat({})) {{\n",
        swift_string_literal(&props.text),
        swift_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        dowe_components::BadgeVisualContract::standard().font_size,
        dowe_components::BadgeVisualContract::standard().height,
        dowe_components::BadgeVisualContract::standard().horizontal_padding,
        dowe_components::BadgeVisualContract::standard().vertical_padding,
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
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
    let visual = ChipVisualContract::for_size(size);
    let radius = swift_control_radius(&props.style.style);
    let base_border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            (
                format!("Optional({})", variant_content(&props.style)),
                "CGFloat(1)".to_string(),
            )
        } else {
            ("nil".to_string(), "CGFloat(0)".to_string())
        };
    let (border, border_width) =
        swift_style_border(&props.style.style, &base_border.0, &base_border.1);
    let shadow = swift_shadow_spec(&props.style.style)
        .map(|value| format!("Optional({value})"))
        .unwrap_or_else(|| "nil".to_string());
    output.push_str(&format!(
        "{pad}DoweChip(text: {}, size: {}, height: CGFloat({}), horizontalPadding: CGFloat({}), textSize: CGFloat({}), contentGap: CGFloat({}), closeAlpha: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, borderWidth: {border_width}, radius: {radius}, shadow: {shadow}, action: {}, hasStart: {}, hasEnd: {}) {{\n",
        swift_string_literal(value),
        swift_string_literal(size.as_str()),
        visual.height,
        visual.horizontal_padding,
        visual.text_size,
        visual.content_gap,
        visual.close_alpha,
        variant_container(&props.style),
        variant_content(&props.style),
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
    let mut chip_style = props.style.style.clone();
    chip_style.shadow = None;
    chip_style.shadow_color = None;
    chip_style.rounded = None;
    chip_style.border = None;
    chip_style.border_color = None;
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&chip_style));
    if props.style.element.on_click.is_some() {
        append_swift_modifiers(
            output,
            indent,
            &[format!(
                ".onTapGesture(perform: {})",
                swift_component_action(props.style.element.on_click.as_deref(), None, context)
            )],
        );
    }
}

fn render_swift_skeleton(
    props: &SkeletonProps,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
) {
    let pad = " ".repeat(indent);
    let visual = dowe_components::SkeletonVisualContract::standard();
    output.push_str(&format!(
        "{pad}DoweSkeleton(variant: {}, animation: {}, textHeight: CGFloat({}), defaultRadius: CGFloat({}), pulseAlpha: CGFloat({}), pulseDurationMs: Double({}))\n",
        swift_string_literal(props.variant.as_str()),
        swift_string_literal(props.animation.as_str())
        , visual.text_height, visual.default_radius, visual.pulse_alpha, visual.pulse_duration_ms
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style, flow),
    );
}

fn render_swift_audio(props: &AudioProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let play = solar_control_icon("play").expect("bundled Audio play icon");
    let pause = solar_control_icon("pause").expect("bundled Audio pause icon");
    let mut button_style = props.style.clone();
    button_style.variant = Some(
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Solid {
            ComponentVariant::Solid
        } else {
            ComponentVariant::Solid
        },
    );
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!("Optional({})", card_variant_content(&props.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweAudioView(source: {}, subtitle: {}, avatarSource: {}, playIcon: DoweVideoIcon(viewBox: {}, paths: {}), pauseIcon: DoweVideoIcon(viewBox: {}, paths: {}), backgroundColor: {}, contentColor: {}, buttonBackgroundColor: {}, buttonContentColor: {}, borderColor: {border}, radius: {})\n",
        swift_string_literal(&props.src),
        swift_optional_literal(props.subtitle.as_deref()),
        swift_optional_literal(props.avatar_src.as_deref()),
        swift_svg_view_box(&play.props.view_box),
        swift_svg_paths(&play.paths),
        swift_svg_view_box(&pause.props.view_box),
        swift_svg_paths(&pause.paths),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        variant_container(&button_style),
        variant_content(&button_style),
        swift_card_radius(&props.style.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn swift_image_source(props: &ImageProps, context: &SwiftReactiveContext) -> String {
    let Some(binding) = props.reactive_src.as_deref() else {
        return swift_string_literal(&props.src);
    };
    let Some(path) = context.dynamic_path(binding) else {
        return swift_string_literal(&props.src);
    };
    context
        .item_value(binding)
        .map(|item| format!("state.text(\"{}\", item: {item})", escape_swift(&path)))
        .unwrap_or_else(|| format!("state.text(\"{}\")", escape_swift(&path)))
}

fn render_swift_image(
    props: &ImageProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!("Optional({})", card_variant_content(&props.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweImageView(source: {}, alt: {}, aspect: {}, objectFit: {}, loading: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
        swift_image_source(props, context),
        swift_string_literal(&props.alt),
        swift_string_literal(props.aspect.as_str()),
        swift_string_literal(props.object_fit.as_str()),
        swift_string_literal(props.loading.as_str()),
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
    let mut style = props.style.clone();
    style.variant.get_or_insert(ComponentVariant::Ghost);
    let variant = style.variant.unwrap_or(ComponentVariant::Ghost);
    let arrow = side_nav_submenu_arrow_icon();
    let content_color = card_variant_content(&style);
    let default_open_ids = items
        .iter()
        .filter(|item| item.default_open)
        .map(|item| swift_string_literal(&item.id))
        .collect::<Vec<_>>()
        .join(", ");
    let border = if variant == ComponentVariant::Outlined {
        format!(
            "Optional({})",
            color_ref(family_color(style.color.unwrap_or(ColorFamily::Primary)))
        )
    } else {
        "nil".to_string()
    };
    let item_background = match variant {
        ComponentVariant::Outlined => color_ref(ColorToken::Surface),
        _ => "Color.clear",
    };
    let item_border = match variant {
        ComponentVariant::Solid => content_color.to_string(),
        ComponentVariant::Outlined => {
            color_ref(family_color(style.color.unwrap_or(ColorFamily::Primary))).to_string()
        }
        ComponentVariant::Ghost | ComponentVariant::Line => content_color.to_string(),
    };
    let item_border_opacity = match variant {
        ComponentVariant::Solid => "0.16",
        ComponentVariant::Outlined => "0.24",
        ComponentVariant::Ghost => "0.22",
        _ => "0.24",
    };
    let item_border_style = if matches!(variant, ComponentVariant::Ghost | ComponentVariant::Line) {
        "separator"
    } else {
        "full"
    };
    let item_radius = if matches!(variant, ComponentVariant::Ghost | ComponentVariant::Line) {
        "CGFloat(0)".to_string()
    } else {
        format!("({}) * CGFloat(0.85)", swift_card_radius(&style.style))
    };
    let title_color = card_variant_title(&style);
    output.push_str(&format!(
        "{pad}DoweAccordionView(multiple: {}, variant: {}, defaultOpenIds: [{default_open_ids}], backgroundColor: {}, contentColor: {content_color}, titleColor: {title_color}, borderColor: {border}, itemBackgroundColor: {}, itemBorderColor: {}, itemBorderOpacity: {}, radius: {}) {{ openIds, toggleItem in\n",
        props.multiple,
        swift_string_literal(variant.as_str()),
        card_variant_container(&style),
        item_background,
        item_border,
        item_border_opacity,
        swift_card_radius(&props.style.style),
    ));
    for item in items {
        output.push_str(&format!(
            "{pad}    DoweAccordionItemView(label: {}, disabled: {}, open: openIds.contains({}), backgroundColor: {}, borderColor: {}, borderOpacity: {}, borderStyle: {}, contentColor: {content_color}, radius: {item_radius}, action: {{ toggleItem({}) }}, arrowIcon: {{\n",
            swift_string_literal(&item.label),
            item.disabled,
            swift_string_literal(&item.id),
            item_background,
            item_border,
            item_border_opacity,
            swift_string_literal(item_border_style),
            swift_string_literal(&item.id),
        ));
        render_swift_button_icon(&arrow, content_color, indent + 8, output);
        output.push_str(&format!("{pad}    }}) {{\n"));
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}
