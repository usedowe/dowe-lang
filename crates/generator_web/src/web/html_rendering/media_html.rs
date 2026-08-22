fn render_audio_html(props: &AudioProps, context: &ReactiveRenderContext) -> String {
    const AUDIO_WAVEFORM_HEIGHTS: [u8; 50] = [
        48, 62, 38, 54, 76, 44, 30, 52, 68, 84, 58, 42, 65, 92, 72, 49, 35, 61, 80, 55,
        41, 71, 96, 64, 46, 32, 57, 75, 88, 60, 37, 51, 69, 83, 47, 29, 55, 73, 63, 40,
        67, 89, 58, 34, 50, 77, 68, 43, 60, 82,
    ];
    let bars = AUDIO_WAVEFORM_HEIGHTS
        .into_iter()
        .map(|height| {
            format!(
                r#"<span class="media-bar" style="height:{}%" aria-hidden="true"></span>"#,
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
        r#"<div{} data-dowe-audio><audio src="{}" preload="metadata" data-dowe-audio-el></audio><button class="media-button" type="button" aria-label="Play audio" data-dowe-audio-toggle><span class="media-icon" data-dowe-audio-play-icon><svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M5 5.274c0-1.707 1.826-2.792 3.325-1.977l12.362 6.727c1.566.852 1.566 3.1 0 3.952L8.325 20.702C6.826 21.518 5 20.432 5 18.726z"/></svg></span><span class="media-icon" data-dowe-audio-pause-icon hidden><svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M5.746 3a1.75 1.75 0 0 0-1.75 1.75v14.5c0 .966.784 1.75 1.75 1.75h3.5a1.75 1.75 0 0 0 1.75-1.75V4.75A1.75 1.75 0 0 0 9.246 3zm9 0a1.75 1.75 0 0 0-1.75 1.75v14.5c0 .966.784 1.75 1.75 1.75h3.5a1.75 1.75 0 0 0 1.75-1.75V4.75A1.75 1.75 0 0 0 18.246 3z"/></svg></span></button><div class="media-content"><div class="media-waveform" role="slider" tabindex="0" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0" aria-valuetext="0:00 remaining" aria-label="Audio progress" data-dowe-audio-waveform><div class="media-bars loaded">{}</div></div><div class="media-footer"><span class="media-time" data-dowe-audio-time>0:00</span>{}</div></div>{}</div>"#,
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

fn render_camera_html(props: &CameraProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" data-dowe-camera data-dowe-camera-facing="{}" data-dowe-camera-label="{}""#,
        props.facing.as_str(),
        escape_attr(&props.label)
    );
    for (name, action) in [
        ("start", props.on_start.as_deref()),
        ("capture", props.on_capture.as_deref()),
        ("error", props.on_error.as_deref()),
    ] {
        if let Some(action) = action {
            extra.push_str(&format!(
                r#" data-dowe-camera-on-{}="{}""#,
                name,
                escape_attr(&context.action_id(action))
            ));
        }
    }
    let disabled = if props.disabled { " disabled" } else { "" };
    format!(
        r#"<section{}><div class="camera-preview"><video autoplay muted playsinline data-dowe-camera-video hidden></video><canvas data-dowe-camera-canvas hidden></canvas><div class="camera-placeholder" data-dowe-camera-placeholder>{}</div></div><div class="camera-controls"><button class="camera-button" type="button" data-dowe-camera-start{}>{}</button><button class="camera-button" type="button" data-dowe-camera-capture{} disabled>Capture</button></div><span class="camera-status" data-dowe-camera-status role="status"></span></section>"#,
        attrs(
            variant_classes("camera", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        escape_html(&props.label),
        disabled,
        escape_html(&props.label),
        disabled
    )
}

fn render_microphone_html(props: &MicrophoneProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" data-dowe-microphone data-dowe-microphone-label="{}""#,
        escape_attr(&props.label)
    );
    if let Some(max_duration) = props.max_duration {
        extra.push_str(&format!(r#" data-dowe-microphone-max-duration="{}""#, max_duration));
    }
    for (name, action) in [
        ("start", props.on_start.as_deref()),
        ("stop", props.on_stop.as_deref()),
        ("error", props.on_error.as_deref()),
    ] {
        if let Some(action) = action {
            extra.push_str(&format!(
                r#" data-dowe-microphone-on-{}="{}""#,
                name,
                escape_attr(&context.action_id(action))
            ));
        }
    }
    let disabled = if props.disabled { " disabled" } else { "" };
    format!(
        r#"<section{}><div class="microphone-panel"><span class="microphone-label">{}</span><span class="microphone-status" data-dowe-microphone-status role="status">Ready</span><span class="microphone-time" data-dowe-microphone-time>0:00</span></div><div class="microphone-controls"><button class="microphone-button" type="button" data-dowe-microphone-start{}>{}</button><button class="microphone-button" type="button" data-dowe-microphone-stop{} disabled>Stop</button></div></section>"#,
        attrs(
            variant_classes("microphone", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        escape_html(&props.label),
        disabled,
        escape_html(&props.label),
        disabled
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
    let source = if props.reactive_src.is_some() {
        ""
    } else {
        &props.src
    };
    let reactive_source = props
        .reactive_src
        .as_deref()
        .map(|path| {
            format!(
                r#" data-dowe-image-src="{}""#,
                escape_attr(&context.signal_path(path))
            )
        })
        .unwrap_or_default();
    format!(
        r#"<figure{} data-dowe-image><img class="image-element" src="{}"{} alt="{}" loading="{}">{}</figure>"#,
        attrs(classes, Some(&props.style.element), None, context),
        escape_attr(source),
        reactive_source,
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
    let arrow = side_nav_submenu_arrow_icon();
    let arrow_html = render_svg_html(&arrow.props, &arrow.paths, context);
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
            r#"<div{} data-dowe-accordion-item><button class="accordion-header{}" type="button" aria-expanded="{}" data-dowe-accordion-trigger{}><span class="accordion-start"><span class="accordion-label">{}</span></span><span class="accordion-end"><span class="accordion-arrow" aria-hidden="true">{}</span></span></button><div class="accordion-content" data-dowe-accordion-content{}><div class="accordion-content-inner">"#,
            class_attr(item_classes),
            if item.default_open { " is-open" } else { "" },
            expanded,
            if item.disabled { " disabled" } else { "" },
            escape_html(&item.label),
            arrow_html,
            hidden
        ));
        for child in &item.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div></div></div>");
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
    let extra = format!(
        r#" data-dowe-carousel data-dowe-carousel-index="0" data-dowe-carousel-loop="{}" data-dowe-carousel-autoplay="{}" data-dowe-carousel-interval="{}" data-dowe-carousel-orientation="{}" data-dowe-carousel-variant="{}"{}"#,
        !props.disable_loop,
        props.autoplay,
        props.autoplay_interval,
        props.orientation.as_str(),
        props.variant.as_str(),
        if props.variant == CarouselVariant::Rtl {
            r#" dir="rtl""#
        } else {
            ""
        }
    );
    let mut html = format!(
        "<div{}>{}<div class=\"carousel-viewport\"><div class=\"carousel-container\" data-dowe-carousel-track style=\"--dowe-carousel-gap:{}px;--dowe-carousel-per-view:{};gap:var(--dowe-carousel-gap);\">",
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
        html.push_str(&format!(
            r#"<button class="carousel-nav is-prev" type="button" aria-label="Previous slide"{} data-dowe-carousel-prev>‹</button><button class="carousel-nav is-next" type="button" aria-label="Next slide"{} data-dowe-carousel-next>›</button>"#,
            if props.disable_loop { " disabled" } else { "" },
            if props.disable_loop && slides.len() <= 1 {
                " disabled"
            } else {
                ""
            }
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
            html.push_str(&format!(
                r#"<button class="carousel-control" type="button" aria-label="Previous slide"{} data-dowe-carousel-prev>‹</button>"#,
                if props.disable_loop { " disabled" } else { "" }
            ));
        }
        if props.shows_indicators() || props.has_variant_indicators() {
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
                if props.indicator_type == CarouselIndicatorType::Dot
                    || props.variant == CarouselVariant::Dots
                {
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
        if props.shows_controls() {
            html.push_str(&format!(
                r#"<button class="carousel-control" type="button" aria-label="Next slide"{} data-dowe-carousel-next>›</button>"#,
                if props.disable_loop && slides.len() <= 1 {
                    " disabled"
                } else {
                    ""
                }
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
