fn render_code_html(props: &CodeProps, context: &ReactiveRenderContext) -> String {
    let source = if props.template_segments.is_empty() {
        props
            .tokens
            .iter()
            .map(|token| {
                format!(
                    r#"<span class="code-token-{}">{}</span>"#,
                    token.kind.as_str(),
                    escape_html(&token.text)
                )
            })
            .collect::<String>()
    } else {
        props
            .template_segments
            .iter()
            .map(|segment| match segment {
                CodeTemplateSegment::Static { tokens, .. } => tokens
                    .iter()
                    .map(|token| {
                        format!(
                            r#"<span class="code-token-{}">{}</span>"#,
                            token.kind.as_str(),
                            escape_html(&token.text)
                        )
                    })
                    .collect::<String>(),
                CodeTemplateSegment::Binding(path) => format!(
                    r#"<span data-dowe-text="{}"></span>"#,
                    escape_attr(&context.signal_path(path))
                ),
            })
            .collect::<String>()
    };
    let extra = format!(
        r#" data-dowe-code data-dowe-copy-label="{}" data-dowe-copied-label="{}""#,
        escape_attr(&props.copy_label),
        escape_attr(&props.copied_label)
    );
    format!(
        r#"<div{}><div class="code-toolbar"><span class="code-language">{}</span><button class="code-copy" type="button" data-dowe-code-copy>{}</button></div><pre class="code-pre"><code>{}</code></pre></div>"#,
        attrs(
            variant_classes("code-block", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        props.language.as_str(),
        escape_html(&props.copy_label),
        source
    )
}

fn render_video_html(props: &VideoProps, context: &ReactiveRenderContext) -> String {
    let poster = props
        .poster
        .as_deref()
        .map(|value| format!(r#" poster="{}""#, escape_attr(value)))
        .unwrap_or_default();
    let autoplay = if props.autoplay { " autoplay" } else { "" };
    let poster_image = props
        .poster
        .as_deref()
        .map(|value| {
            format!(
                r#"<img class="video-poster" src="{}" alt="" data-dowe-video-poster>"#,
                escape_attr(value)
            )
        })
        .unwrap_or_default();
    let play = render_video_control_icon("play", "data-dowe-video-play-icon", context);
    let pause = render_video_control_icon("pause", "data-dowe-video-pause-icon", context);
    let volume = render_video_control_icon("volume-loud", "data-dowe-video-volume-icon", context);
    let muted = render_video_control_icon("volume-cross", "data-dowe-video-muted-icon", context);
    let pip = render_video_control_icon("pip", "", context);
    let fullscreen = render_video_control_icon("full-screen", "", context);
    format!(
        r#"<div{} data-dowe-video-root><video class="video-media" src="{}" data-dowe-video data-dowe-video-source="{}" playsinline preload="metadata"{}{}></video>{poster_image}<div class="video-controls" data-dowe-video-controls><div class="video-actions"><button class="video-action" type="button" aria-label="Play video" data-dowe-video-play>{play}{pause}</button><span class="video-time" data-dowe-video-time>0:00 / 0:00</span><span class="video-actions-spacer"></span><label class="video-volume"><span class="video-volume-label">Volume</span><input type="range" min="0" max="1" step="0.05" value="1" data-dowe-video-volume></label><button class="video-action" type="button" aria-label="Mute video" data-dowe-video-mute>{volume}{muted}</button><button class="video-action" type="button" aria-label="Toggle picture-in-picture" data-dowe-video-pip>{pip}</button><button class="video-action" type="button" aria-label="Toggle fullscreen" data-dowe-video-fullscreen>{fullscreen}</button></div><input class="video-progress" type="range" min="0" max="0" step="0.05" value="0" aria-label="Video progress" data-dowe-video-progress></div></div>"#,
        attrs(
            video_classes(props),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.src),
        escape_attr(&props.src),
        poster,
        autoplay
    )
}

fn render_video_control_icon(
    name: &str,
    data_attribute: &str,
    context: &ReactiveRenderContext,
) -> String {
    let icon = solar_control_icon(name).expect("bundled Video control icon");
    format!(
        r#"<span class="video-control-icon" {}>{}</span>"#,
        data_attribute,
        render_svg_html(&icon.props, &icon.paths, context)
    )
}

fn render_iframe_html(props: &IframeProps, context: &ReactiveRenderContext) -> String {
    let allow = if props.allow.is_empty() {
        String::new()
    } else {
        format!(r#" allow="{}""#, escape_attr(&props.allow.join("; ")))
    };
    let sandbox = props
        .sandbox
        .as_ref()
        .map(|tokens| {
            let value = tokens
                .iter()
                .map(|token| format!("allow-{token}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!(r#" sandbox="{}""#, escape_attr(&value))
        })
        .unwrap_or_default();
    let fullscreen = if props.allow_fullscreen {
        " allowfullscreen"
    } else {
        ""
    };
    format!(
        r#"<iframe{} src="{}" title="{}" loading="{}" referrerpolicy="strict-origin-when-cross-origin"{}{}{}></iframe>"#,
        attrs(
            iframe_classes(props),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.src),
        escape_attr(&props.title),
        props.loading.as_str(),
        allow,
        sandbox,
        fullscreen,
    )
}

fn render_device_html(
    props: &DeviceProps,
    iframe: &IframeProps,
    context: &ReactiveRenderContext,
) -> String {
    let controls = props
        .options
        .iter()
        .map(|option| {
            let selected = option.profile == props.device;
            format!(
                r#"<button type="button" class="button icon-button button-md device-toggle{}" aria-label="{}" aria-pressed="{}" data-dowe-device-option="{}"><span data-dowe-button-icon-start>{}</span></button>"#,
                if selected { " is-active" } else { "" },
                option.profile.as_str(),
                selected,
                option.profile.as_str(),
                render_svg_html(&option.icon.props, &option.icon.paths, context),
            )
        })
        .collect::<String>();
    let (width, height) = props.device.dimensions();
    let mut nested = iframe.clone();
    nested.style.sizing.w = None;
    nested.style.sizing.h = None;
    let mut device_classes = vec!["device".to_string()];
    append_style_classes(&mut device_classes, &props.style);
    format!(
        r#"<div{} data-dowe-device data-dowe-device-profile="{}"><div class="device-toolbar" role="group" aria-label="Device preview">{}</div><div class="device-stage" data-dowe-device-stage><div class="device-viewport" data-dowe-device-viewport style="width:{}px;height:{}px;">{}</div></div></div>"#,
        attrs(device_classes, Some(&props.style.element), None, context),
        props.device.as_str(),
        controls,
        width,
        height,
        render_iframe_html(&nested, context),
    )
}

fn render_divider_html(props: &DividerProps, context: &ReactiveRenderContext) -> String {
    format!(
        "<div{}></div>",
        attrs(
            divider_classes(props),
            Some(&props.style.element),
            None,
            context
        )
    )
}

