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
    let control = props.control_contract();
    let geometry = props.geometry_contract();
    let geometry_literal = format!(
        "DoweCarouselGeometry(contentGap: CGFloat({}), viewportPadding: CGFloat({}), verticalViewportHeight: CGFloat({}), slideFraction: CGFloat({}) / 100, slideMaxWidth: {})",
        geometry.content_gap,
        geometry.viewport_padding,
        geometry.vertical_viewport_height,
        geometry.slide_fraction_percent,
        swift_optional_u16(geometry.slide_max_width),
    );
    let control_literal = format!(
        "DoweCarouselControlContract(navigationSize: CGFloat({}), navigationInset: CGFloat({}), controlSize: CGFloat({}), controlGap: CGFloat({}), indicatorGap: CGFloat({}), indicatorHeight: CGFloat({}), indicatorInactiveWidth: CGFloat({}), indicatorActiveWidth: CGFloat({}), indicatorDotSize: CGFloat({}), indicatorDotActiveScale: CGFloat({:.2}))",
        control.navigation_size,
        control.navigation_inset,
        control.control_size,
        control.control_gap,
        control.indicator_gap,
        control.indicator_height,
        control.indicator_inactive_width,
        control.indicator_active_width,
        control.indicator_dot_size,
        f32::from(control.indicator_dot_active_scale_percent) / 100.0,
    );
    let slide_ids = format!(
        "[{}]",
        slides
            .iter()
            .map(|slide| swift_string_literal(&slide.id))
            .collect::<Vec<_>>()
            .join(", ")
    );
    output.push_str(&format!(
        "{pad}DoweCarouselView(variant: {}, snap: {}, control: {}, geometry: {}, slideIds: {}, autoplay: {}, autoplayInterval: {}, disableLoop: {}, hideControls: {}, hideIndicators: {}, showNavigation: {}, showCounter: {}, orientation: {}, size: {}, indicatorType: {}, title: {}, slideWidth: {}, slideHeight: {}, slidesPerView: {}, gap: {}, accentColor: {}) {{\n",
        swift_string_literal(props.variant.as_str()),
        props.variant.uses_snap(),
        control_literal,
        geometry_literal,
        slide_ids,
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
    for (index, slide) in slides.iter().enumerate() {
        output.push_str(&format!(
            "{pad}    DoweCarouselSlideView(id: {}, variant: {}, index: {}, geometry: {}, orientation: {}, slideWidth: {}, slideHeight: {}, slidesPerView: {}, gap: {}) {{\n",
            swift_string_literal(&slide.id),
            swift_string_literal(props.variant.as_str()),
            index,
            geometry_literal,
            swift_string_literal(props.orientation.as_str()),
            swift_optional_u16(props.slide_width),
            swift_optional_u16(props.slide_height),
            props.slides_per_view,
            props.gap,
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
