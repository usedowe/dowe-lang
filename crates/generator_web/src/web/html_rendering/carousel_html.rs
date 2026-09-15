fn render_carousel_html(
    props: &CarouselProps,
    slides: &[CarouselSlide],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let control_contract = props.control_contract();
    let visual_contract = props.visual_contract();
    let geometry = props.geometry_contract();
    let mut classes = variant_classes("carousel", &props.style);
    classes.push(format!("is-{}", props.variant.class_name()));
    if props.orientation == CarouselOrientation::Vertical {
        classes.push("is-vertical".to_string());
    }
    let title = props
        .title
        .as_deref()
        .map(|value| {
            format!(
                r#"<div class="carousel-header"><div class="carousel-title"><h2>{}</h2></div></div>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    let contract_style = format!(
        r#" style="--dowe-carousel-content-gap:{}px;--dowe-carousel-viewport-padding:{}px;--dowe-carousel-vertical-height:{}px;--dowe-carousel-slide-fraction:{}cqw;--dowe-carousel-slide-max:{}px;--dowe-carousel-navigation-size:{}px;--dowe-carousel-navigation-inset:{}px;--dowe-carousel-control-size:{}px;--dowe-carousel-control-gap:{}px;--dowe-carousel-control-border-alpha:{};--dowe-carousel-indicator-inactive-alpha:{};--dowe-carousel-indicator-gap:{}px;--dowe-carousel-indicator-height:{}px;--dowe-carousel-indicator-inactive-width:{}px;--dowe-carousel-indicator-active-width:{}px;--dowe-carousel-indicator-dot-size:{}px;--dowe-carousel-indicator-dot-active-scale:{};""#,
        geometry.content_gap,
        geometry.viewport_padding,
        geometry.vertical_viewport_height,
        geometry.slide_fraction_percent,
        geometry.slide_max_width.unwrap_or(u16::MAX),
        control_contract.navigation_size,
        control_contract.navigation_inset,
        control_contract.control_size,
        control_contract.control_gap,
        visual_contract.control_border_alpha,
        visual_contract.indicator_inactive_alpha,
        control_contract.indicator_gap,
        control_contract.indicator_height,
        control_contract.indicator_inactive_width,
        control_contract.indicator_active_width,
        control_contract.indicator_dot_size,
        f32::from(control_contract.indicator_dot_active_scale_percent) / 100.0,
    );
    let extra = format!(
        r#" data-dowe-carousel data-dowe-carousel-index="0" data-dowe-carousel-loop="{}" data-dowe-carousel-autoplay="{}" data-dowe-carousel-interval="{}" data-dowe-carousel-orientation="{}" data-dowe-carousel-variant="{}" data-dowe-carousel-snap="{}" data-dowe-carousel-navigation-placement="overlay-stage" data-dowe-carousel-controls-placement="below-track"{} role="region" aria-roledescription="carousel" aria-label="{}"{}"#,
        !props.disable_loop,
        props.autoplay,
        props.autoplay_interval,
        props.orientation.as_str(),
        props.variant.as_str(),
        props.variant.uses_snap(),
        contract_style,
        escape_attr(props.title.as_deref().unwrap_or("Carousel")),
        if props.variant == CarouselVariant::Rtl {
            r#" dir="rtl""#
        } else {
            ""
        }
    );
    let mut html = format!(
        "<div{}>{}<div class=\"carousel-stage\"><div class=\"carousel-viewport\" role=\"group\" aria-roledescription=\"slide viewport\" aria-label=\"Carousel slides\" tabindex=\"0\"><div class=\"carousel-container\" data-dowe-carousel-track style=\"--dowe-carousel-gap:{}px;--dowe-carousel-per-view:{};gap:var(--dowe-carousel-gap);\">",
        attrs(classes, Some(&props.style.element), Some(&extra), context),
        title,
        props.gap,
        props.slides_per_view
    );
    for slide in slides {
        let mut style = String::new();
        if let Some(width) = props.slide_width {
            style.push_str(&format!("width:{width}px;flex-basis:{width}px;"));
        }
        if let Some(height) = props.slide_height {
            style.push_str(&format!("height:{height}px;"));
        }
        html.push_str(&format!(
            r#"<div class="carousel-slide" role="group" aria-roledescription="slide" aria-label="Slide {} of {}" aria-hidden="{}" data-dowe-carousel-slide="{}"{}>"#,
            slides.iter().position(|candidate| candidate.id == slide.id).unwrap_or(0) + 1,
            slides.len(),
            if slide.id == slides.first().map(|first| first.id.as_str()).unwrap_or_default() { "false" } else { "true" },
            escape_attr(&slide.id),
            if style.is_empty() {
                String::new()
            } else {
                format!(r#" style="{}""#, escape_attr(&style))
            }
        ));
        for child in &slide.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>");
    html.push_str("</div>");
    let (previous_name, next_name) = if props.orientation == CarouselOrientation::Vertical {
        ("arrow-up", "arrow-down")
    } else {
        ("arrow-left", "arrow-right")
    };
    let previous = solar_control_icon(previous_name).expect("bundled Carousel previous icon");
    let next = solar_control_icon(next_name).expect("bundled Carousel next icon");
    let previous_icon = render_svg_html(&previous.props, &previous.paths, context);
    let next_icon = render_svg_html(&next.props, &next.paths, context);
    let icon_button = |kind: &str,
                       previous: bool,
                       size: &str,
                       label: &str,
                       icon: &str,
                       disabled: bool| {
        // Carousel navigation uses the shared IconButton structure while its
        // surface and foreground are owned by the carousel accent contract.
        // Do not carry the carousel's content variant classes here: a solid
        // button variant would otherwise paint the controls with the slide
        // container color on web, diverging from native targets.
        let button_classes = vec![
            "button".to_string(),
            format!("button-{size}"),
            "icon-button".to_string(),
            "rounded-full".to_string(),
            format!("carousel-{kind}"),
            format!("is-{}", if previous { "prev" } else { "next" }),
        ];
        format!(
            r#"<button{} type="button" aria-label="{label}"{} data-dowe-carousel-{}><span data-dowe-button-icon-start>{icon}</span></button>"#,
            class_attr(button_classes),
            if disabled { " disabled" } else { "" },
            if previous { "prev" } else { "next" },
        )
    };
    if props.show_navigation {
        let previous_disabled = props.disable_loop;
        let next_disabled = props.disable_loop && slides.len() <= 1;
        html.push_str(&icon_button(
            "nav",
            true,
            "md",
            "Previous slide",
            &previous_icon,
            previous_disabled,
        ));
        html.push_str(&icon_button(
            "nav",
            false,
            "md",
            "Next slide",
            &next_icon,
            next_disabled,
        ));
    }
    html.push_str("</div>");
    if props.shows_controls()
        || props.shows_indicators()
        || props.has_variant_indicators()
        || props.show_counter
    {
        html.push_str("<div class=\"carousel-controls\">");
        if props.shows_controls() {
            let previous_disabled = props.disable_loop;
            html.push_str(&icon_button(
                "control",
                true,
                "sm",
                "Previous slide",
                &previous_icon,
                previous_disabled,
            ));
        }
        if props.shows_indicators() || props.has_variant_indicators() {
            html.push_str("<div class=\"carousel-indicators\">");
            let dot_indicators = props.indicator_type == CarouselIndicatorType::Dot
                || props.variant == CarouselVariant::Dots;
            for (index, _slide) in slides.iter().enumerate() {
                let mut classes = vec![
                    "carousel-indicator".to_string(),
                    "pagination-indicator".to_string(),
                    format!("is-{}", props.size.as_str()),
                    format!(
                        "is-{}",
                        props.style.color.unwrap_or(ColorFamily::Primary).as_str()
                    ),
                ];
                if index == 0 {
                    classes.push("is-active".to_string());
                }
                if dot_indicators {
                    classes.push("is-dot".to_string());
                } else {
                    classes.push("is-bar".to_string());
                }
                html.push_str(&format!(
                    r#"<button{} type="button" aria-label="Go to slide {}" data-dowe-carousel-indicator="{}"{}></button>"#,
                    class_attr(classes),
                    index + 1,
                    index,
                    if index == 0 { r#" aria-current="true""# } else { "" }
                ));
            }
            html.push_str("</div>");
        }
        if props.show_counter {
            html.push_str(&format!(
                r#"<div class="carousel-counter pagination-count" data-dowe-carousel-counter>1 / {}</div>"#,
                slides.len()
            ));
        }
        if props.shows_controls() {
            let next_disabled = props.disable_loop && slides.len() <= 1;
            html.push_str(&icon_button(
                "control",
                false,
                "sm",
                "Next slide",
                &next_icon,
                next_disabled,
            ));
        }
        html.push_str("</div>");
    }
    if props.variant == CarouselVariant::Thumbnails {
        html.push_str("<div class=\"carousel-thumbnails\" aria-label=\"Slide thumbnails\">");
        for (index, slide) in slides.iter().enumerate() {
            html.push_str(&format!(
                r#"<button class="carousel-thumbnail{}" type="button" aria-label="Go to slide {}" data-dowe-carousel-indicator="{}"><span>{}</span></button>"#,
                if index == 0 { " is-active" } else { "" },
                index + 1,
                index,
                escape_html(&slide.id)
            ));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>");
    html
}
