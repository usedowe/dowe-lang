fn render_image_cropper_html(props: &ImageCropperProps, context: &ReactiveRenderContext) -> String {
    let source = props
        .name
        .as_deref()
        .or(props.src.as_deref())
        .unwrap_or(&props.alt);
    let uid = short_id("cropper", source);
    let value = props.src.as_deref().unwrap_or_default();
    let size = props.style.size.unwrap_or(ButtonSize::Md).as_str();
    let image = if value.is_empty() {
        view_icon_svg(ViewIcon::Upload, "image-cropper-empty-icon")
    } else {
        format!(
            r#"<img class="image-cropper-image" src="{}" alt="{}">"#,
            escape_attr(value),
            escape_attr(&props.alt)
        )
    };
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<input type="hidden" name="{}" value="{}" data-dowe-cropper-hidden>"#,
                escape_attr(name),
                escape_attr(value)
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-image-cropper data-dowe-cropper-value="{}" data-dowe-shape="{}" data-dowe-size="{}" data-dowe-alt="{}" data-dowe-disabled="{}" data-dowe-min-width="{}" data-dowe-min-height="{}"{}{}{}{}"#,
        escape_attr(value),
        props.shape.as_str(),
        size,
        escape_attr(&props.alt),
        props.disabled,
        props.min_width,
        props.min_height,
        props
            .aspect_ratio
            .as_deref()
            .map(|value| format!(r#" data-dowe-aspect-ratio="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max_width
            .map(|value| format!(r#" data-dowe-max-width="{value}""#))
            .unwrap_or_default(),
        props
            .max_height
            .map(|value| format!(r#" data-dowe-max-height="{value}""#))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let body = format!(
        r#"<div{}>{hidden}<input id="{uid}" class="image-cropper-input" type="file" accept="{}" hidden{}><button class="image-cropper-trigger is-{} is-{}" type="button" aria-label="{}" data-dowe-cropper-trigger{}>{image}<span class="image-cropper-label">{}</span></button><div class="image-cropper-actions"><button type="button" class="image-cropper-action" data-dowe-cropper-change{}>{}</button><button type="button" class="image-cropper-action" data-dowe-cropper-remove{}{}>{}</button></div><span class="image-cropper-runtime-error" data-dowe-cropper-runtime-error hidden></span><div class="image-cropper-modal" data-dowe-cropper-modal hidden><div class="image-cropper-dialog" role="dialog" aria-modal="true" aria-label="Adjust image"><div class="image-cropper-dialog-header"><strong>Adjust image</strong><button type="button" class="image-cropper-dialog-close" aria-label="Cancel" data-dowe-cropper-cancel>×</button></div><div class="image-cropper-stage" data-dowe-cropper-stage><canvas class="image-cropper-canvas" data-dowe-cropper-canvas></canvas><div class="image-cropper-grid is-{}" aria-hidden="true"><span></span><span></span></div><div class="image-cropper-box is-{}" data-dowe-cropper-box aria-label="Crop frame"></div></div><div class="image-cropper-zoom"><span>Zoom</span><input type="range" min="1" max="3" step="0.01" value="1" aria-label="Zoom" data-dowe-cropper-zoom></div><div class="image-cropper-modal-actions"><button type="button" class="image-cropper-action" data-dowe-cropper-reset>Reset</button><span class="image-cropper-action-spacer"></span><button type="button" class="image-cropper-action" data-dowe-cropper-cancel>Cancel</button><button type="button" class="image-cropper-action is-primary" data-dowe-cropper-apply>Apply</button></div></div></div></div>"#,
        attrs(
            variant_classes("image-cropper", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        escape_attr(&props.accept),
        if props.disabled { " disabled" } else { "" },
        props.shape.as_str(),
        size,
        escape_attr(&props.alt),
        if props.disabled { " disabled" } else { "" },
        escape_html(props.style.placeholder.as_deref().unwrap_or("Upload")),
        if props.disabled { " disabled" } else { "" },
        escape_html("Change"),
        if value.is_empty() { " hidden" } else { "" },
        if props.disabled { " disabled" } else { "" },
        escape_html("Remove"),
        props.shape.as_str(),
        props.shape.as_str()
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &body,
        context,
    )
}

