fn render_audio_html(props: &AudioProps, context: &ReactiveRenderContext) -> String {
    let bars = (0..50)
        .map(|index| {
            let height = 28 + ((index * 17) % 58);
            format!(
                r#"<span class="media-bar" style="height:{}%"></span>"#,
                height
            )
        })
        .collect::<String>();
    let subtitle = props
        .subtitle
        .as_deref()
        .map(|value| {
            format!(
                r#"<span class="media-subtitle">{}</span>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    let avatar = props
        .avatar_src
        .as_deref()
        .map(|src| {
            format!(
                r#"<img class="media-avatar" src="{}" alt="">"#,
                escape_attr(src)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<div{} data-dowe-audio><audio src="{}" preload="metadata" data-dowe-audio-el></audio><button class="media-button" type="button" aria-label="Play audio" data-dowe-audio-toggle><span data-dowe-audio-icon>▶</span></button><div class="media-content"><div class="media-waveform" role="slider" tabindex="0" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0" data-dowe-audio-waveform><div class="media-bars">{}</div></div><div class="media-footer"><span class="media-time" data-dowe-audio-time>0:00</span>{}</div></div>{}</div>"#,
        attrs(
            variant_classes("media", &props.style),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.src),
        bars,
        subtitle,
        avatar
    )
}

fn render_image_html(props: &ImageProps, context: &ReactiveRenderContext) -> String {
    let controls = if props.hide_controls {
        String::new()
    } else {
        r#"<div class="image-controls"><div class="image-actions"><button class="image-action" type="button" aria-label="Download image" data-dowe-image-download>↓</button><button class="image-action" type="button" aria-label="Toggle fullscreen" data-dowe-image-fullscreen>⛶</button></div></div>"#.to_string()
    };
    let mut classes = variant_classes("image", &props.style);
    classes.push(props.aspect.as_str().to_string());
    classes.push(format!("fit-{}", props.object_fit.as_str()));
    format!(
        r#"<figure{} data-dowe-image><img class="image-element" src="{}" alt="{}" loading="{}">{}</figure>"#,
        attrs(classes, Some(&props.style.element), None, context),
        escape_attr(&props.src),
        escape_attr(&props.alt),
        props.loading.as_str(),
        controls
    )
}

fn render_accordion_html(
    props: &AccordionProps,
    items: &[AccordionItem],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = format!(
        r#" data-dowe-accordion data-dowe-accordion-multiple="{}""#,
        props.multiple
    );
    if props.multiple {
        extra.push_str(r#" aria-multiselectable="true""#);
    }
    let mut html = format!(
        "<div{}>",
        attrs(
            variant_classes("accordion", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        )
    );
    for item in items {
        let mut item_classes = vec!["accordion-item".to_string()];
        if item.disabled {
            item_classes.push("is-disabled".to_string());
        }
        if item.default_open {
            item_classes.push("is-open".to_string());
        }
        let hidden = if item.default_open { "" } else { " hidden" };
        let expanded = if item.default_open { "true" } else { "false" };
        html.push_str(&format!(
            r#"<div{} data-dowe-accordion-item><button class="accordion-header{}" type="button" aria-expanded="{}" data-dowe-accordion-trigger{}><span class="accordion-start"><span class="accordion-label">{}</span></span><span class="accordion-end"><span class="accordion-arrow">⌄</span></span></button><div class="accordion-content" data-dowe-accordion-content{}>"#,
            class_attr(item_classes),
            if item.default_open { " is-open" } else { "" },
            expanded,
            if item.disabled { " disabled" } else { "" },
            escape_html(&item.label),
            hidden
        ));
        for child in &item.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div></div>");
    }
    html.push_str("</div>");
    html
}

fn render_carousel_html(
    props: &CarouselProps,
    slides: &[CarouselSlide],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("carousel", &props.style);
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
    let extra = format!(
        r#" data-dowe-carousel data-dowe-carousel-index="0" data-dowe-carousel-loop="{}" data-dowe-carousel-autoplay="{}" data-dowe-carousel-interval="{}" data-dowe-carousel-orientation="{}""#,
        !props.disable_loop,
        props.autoplay,
        props.autoplay_interval,
        props.orientation.as_str()
    );
    let mut html = format!(
        "<div{}>{}<div class=\"carousel-viewport\"><div class=\"carousel-container\" data-dowe-carousel-track style=\"gap:{}px;\">",
        attrs(classes, Some(&props.style.element), Some(&extra), context),
        title,
        props.gap
    );
    for slide in slides {
        let mut style = String::new();
        if let Some(width) = props.slide_width {
            style.push_str(&format!("width:{width}px;"));
        }
        if let Some(height) = props.slide_height {
            style.push_str(&format!("height:{height}px;"));
        }
        html.push_str(&format!(
            r#"<div class="carousel-slide" data-dowe-carousel-slide="{}"{}>"#,
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
    if props.show_navigation {
        html.push_str(r#"<button class="carousel-nav is-prev" type="button" aria-label="Previous slide" data-dowe-carousel-prev>‹</button><button class="carousel-nav is-next" type="button" aria-label="Next slide" data-dowe-carousel-next>›</button>"#);
    }
    html.push_str("</div>");
    if !props.hide_controls || !props.hide_indicators || props.show_counter {
        html.push_str("<div class=\"carousel-controls\">");
        if !props.hide_controls {
            html.push_str(r#"<button class="carousel-control" type="button" aria-label="Previous slide" data-dowe-carousel-prev>‹</button>"#);
        }
        if !props.hide_indicators {
            html.push_str("<div class=\"carousel-indicators\">");
            for (index, _slide) in slides.iter().enumerate() {
                let mut classes = vec![
                    "carousel-indicator".to_string(),
                    format!("is-{}", props.size.as_str()),
                    format!(
                        "is-{}",
                        props.style.color.unwrap_or(ColorFamily::Primary).as_str()
                    ),
                ];
                if index == 0 {
                    classes.push("is-active".to_string());
                }
                if props.indicator_type == CarouselIndicatorType::Dot {
                    classes.push("is-dot".to_string());
                }
                html.push_str(&format!(
                    r#"<button{} type="button" aria-label="Go to slide {}" data-dowe-carousel-indicator="{}"></button>"#,
                    class_attr(classes),
                    index + 1,
                    index
                ));
            }
            html.push_str("</div>");
        }
        if props.show_counter {
            html.push_str(&format!(
                r#"<div class="carousel-counter" data-dowe-carousel-counter>1 / {}</div>"#,
                slides.len()
            ));
        }
        if !props.hide_controls {
            html.push_str(r#"<button class="carousel-control" type="button" aria-label="Next slide" data-dowe-carousel-next>›</button>"#);
        }
        html.push_str("</div>");
    }
    html.push_str("</div>");
    html
}
