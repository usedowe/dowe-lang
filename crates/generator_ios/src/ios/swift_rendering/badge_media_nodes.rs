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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
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
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_image(props: &ImageProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
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
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}
