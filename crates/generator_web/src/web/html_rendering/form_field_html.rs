fn render_color_html(props: &ColorProps, context: &ReactiveRenderContext) -> String {
    let name = props
        .name
        .as_deref()
        .map(|name| format!(r#" name="{}""#, escape_attr(name)))
        .unwrap_or_default();
    let bind = props
        .style
        .element
        .bind
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-color-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let values = render_color_values(props);
    let picker = format!(
        r#"<div class="color-control-shell" data-dowe-color-picker data-dowe-color-value="{}"{}><button class="color-control-trigger" data-dowe-color-trigger type="button" aria-haspopup="dialog" aria-expanded="false"><span class="color-field-swatch is-{}" data-dowe-color-swatch></span><span class="color-field-value" data-dowe-color-value-label></span></button><input class="color-input" type="hidden" value="{}"{}><div class="color-picker-popover" data-dowe-color-popover role="dialog" aria-label="Color picker"><div class="color-picker-canvas" data-dowe-color-sv role="slider" tabindex="0" aria-label="Saturation and brightness" aria-valuemin="0" aria-valuemax="100"><span class="color-picker-cursor" data-dowe-color-cursor></span></div><div class="color-picker-hue" data-dowe-color-hue role="slider" tabindex="0" aria-label="Hue" aria-valuemin="0" aria-valuemax="360"><span class="color-picker-slider-thumb" data-dowe-color-hue-thumb></span></div><div class="color-picker-preview"><span class="color-picker-preview-swatch"><span class="color-picker-preview-color" data-dowe-color-preview></span></span><span class="color-picker-preview-info"><strong class="color-picker-preview-hex" data-dowe-color-preview-hex></strong><span class="color-picker-preview-foreground" data-dowe-color-foreground></span></span></div>{values}</div></div>"#,
        escape_attr(&props.value),
        bind,
        props.size.as_str(),
        escape_attr(&props.value),
        name
    );
    render_field_control(
        "color-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &picker,
        true,
        true,
        context,
    )
}

fn render_date_html(props: &DateProps, context: &ReactiveRenderContext) -> String {
    let bind = props
        .style
        .element
        .bind
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-date-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let value = props.value.as_deref().unwrap_or_default();
    let hidden = format!(
        r#"<input class="date-hidden" type="hidden" value="{}"{}>"#,
        escape_attr(value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default()
    );
    let input = format!(
        r#"<div class="date-control-shell" data-dowe-date-field data-dowe-date-value="{}" data-dowe-date-placeholder="{}"{}{}{}><button class="date-control-trigger" data-dowe-date-trigger data-dowe-validation-control type="button" aria-haspopup="dialog" aria-expanded="false"><span class="date-control-value"></span>{}</button>{}<div class="date-popover" data-dowe-date-popover role="dialog" aria-label="Date picker"><div class="date-picker-header"><button class="date-picker-nav" type="button" data-dowe-date-prev aria-label="Previous month">‹</button><span class="date-picker-month"></span><button class="date-picker-nav" type="button" data-dowe-date-next aria-label="Next month">›</button></div><div class="date-picker-weekdays"></div><div class="date-picker-days"></div></div></div>"#,
        escape_attr(value),
        escape_attr(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select a date")
        ),
        bind,
        props
            .min
            .as_deref()
            .map(|value| format!(r#" data-dowe-date-min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" data-dowe-date-max="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        select_arrow_svg(),
        hidden
    );
    render_field_control(
        "date-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        false,
        props
            .value
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        context,
    )
}

fn render_date_range_html(props: &DateRangeProps, context: &ReactiveRenderContext) -> String {
    let start_bind = props
        .start
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-date-start-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let end_bind = props
        .end
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-date-end-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let start_value = props.start_value.as_deref().unwrap_or_default();
    let end_value = props.end_value.as_deref().unwrap_or_default();
    let input = format!(
        r#"<div class="date-control-shell" data-dowe-date-range data-dowe-date-placeholder="{}" data-dowe-date-start-value="{}" data-dowe-date-end-value="{}"{}{}{}{}><button class="date-control-trigger" data-dowe-date-trigger type="button" aria-haspopup="dialog" aria-expanded="false"><span class="date-control-value"></span>{}</button><input class="date-hidden date-hidden-start" type="hidden" value="{}"{}><input class="date-hidden date-hidden-end" type="hidden" value="{}"{}><div class="date-range-popover" data-dowe-date-popover role="dialog" aria-label="Date range picker"><div class="date-range-calendars"><div class="date-range-calendar"><div class="date-picker-header"><button class="date-picker-nav" type="button" data-dowe-date-prev aria-label="Previous month">‹</button><span class="date-picker-month" data-dowe-date-month-current></span><span class="date-range-spacer"></span></div><div class="date-picker-weekdays"></div><div class="date-picker-days" data-dowe-date-days-current></div></div><div class="date-range-calendar"><div class="date-picker-header"><span class="date-range-spacer"></span><span class="date-picker-month" data-dowe-date-month-next></span><button class="date-picker-nav" type="button" data-dowe-date-next aria-label="Next month">›</button></div><div class="date-picker-weekdays"></div><div class="date-picker-days" data-dowe-date-days-next></div></div></div></div></div>"#,
        escape_attr(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select a date range")
        ),
        escape_attr(start_value),
        escape_attr(end_value),
        start_bind,
        end_bind,
        props
            .min
            .as_deref()
            .map(|value| format!(r#" data-dowe-date-min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" data-dowe-date-max="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        select_arrow_svg(),
        escape_attr(start_value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}Start""#, escape_attr(name)))
            .unwrap_or_default(),
        escape_attr(end_value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}End""#, escape_attr(name)))
            .unwrap_or_default()
    );
    render_field_control(
        "date-range-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        false,
        props
            .start_value
            .as_deref()
            .is_some_and(|value| !value.is_empty())
            || props
                .end_value
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
        context,
    )
}

fn render_radio_group_html(
    props: &RadioGroupProps,
    options: &[RadioOption],
    context: &ReactiveRenderContext,
) -> String {
    let name = props
        .name
        .clone()
        .unwrap_or_else(|| format!("radio-{}", short_id("radio", &options[0].value)));
    let mut group = format!(
        "<div class=\"radio-group is-{}\">",
        props.orientation.as_str()
    );
    for option in options {
        group.push_str(&format!(
            r#"<label class="radio-item"><input type="radio" class="radio is-{} is-{}" name="{}" value="{}"{}{}><span class="label">{}</span></label>"#,
            props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
            props.size.as_str(),
            escape_attr(&name),
            escape_attr(&option.value),
            bind_attr(props.style.element.bind.as_deref(), context),
            if option.disabled { " disabled" } else { "" },
            escape_html(&option.label)
        ));
    }
    group.push_str("</div>");
    render_field_block(
        &props.style,
        props.info.as_deref(),
        props.error.as_deref(),
        &group,
        context,
    )
}

fn render_toggle_html(props: &ToggleProps, context: &ReactiveRenderContext) -> String {
    let left = props
        .label_left
        .as_deref()
        .map(|label| {
            format!(
                r#"<span class="toggle-label-left{}">{}</span>"#,
                if props.checked { "" } else { " is-active" },
                escape_html(label)
            )
        })
        .unwrap_or_default();
    let right = props
        .label_right
        .as_deref()
        .map(|label| {
            format!(
                r#"<span class="toggle-label-right{}">{}</span>"#,
                if props.checked { " is-active" } else { "" },
                escape_html(label)
            )
        })
        .unwrap_or_default();
    let label = props
        .style
        .label
        .as_deref()
        .map(|label| format!(r#"<span class="label-md">{}</span>"#, escape_html(label)))
        .unwrap_or_default();
    let input = format!(
        r#"<input type="checkbox" role="switch" class="toggle-input is-{}" aria-checked="{}"{}{}{}{}>"#,
        props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
        props.checked,
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.checked { " checked" } else { "" },
        if props.disabled { " disabled" } else { "" }
    );
    format!(
        "<label{}>{left}{input}{right}{label}</label>",
        attrs(
            vec!["toggle".to_string()],
            Some(&props.style.element),
            None,
            context
        )
    )
}

fn render_combo_box_html(
    props: &ComboBoxProps,
    options: &[ComboOption],
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("control", &props.style);
    classes.push("combo-box-control".to_string());
    classes.push(format!(
        "is-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.style.label_floating {
        classes.push("is-floating".to_string());
    }
    if props.error_text.is_some() {
        classes.push("is-error".to_string());
    }
    let selected = props
        .value
        .as_deref()
        .and_then(|value| options.iter().find(|option| option.value == value));
    let label = selected
        .map(|option| option.label.as_str())
        .unwrap_or_else(|| {
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select an option")
        });
    let value = selected
        .map(|option| option.value.as_str())
        .unwrap_or_default();
    let clear = if props.clearable {
        format!(
            r#"<button class="combo-box-clear" type="button" aria-label="Clear selection" data-dowe-combo-clear{}>&times;</button>"#,
            if props.disabled { " disabled" } else { "" }
        )
    } else {
        String::new()
    };
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<input type="hidden" name="{}" value="{}" data-dowe-combo-hidden>"#,
                escape_attr(name),
                escape_attr(value)
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" type="button" role="combobox" aria-haspopup="listbox" aria-expanded="false" data-dowe-combo-box data-dowe-placeholder="{}" data-dowe-value="{}" data-dowe-empty-text="{}" data-dowe-loading-text="{}" data-dowe-loading-more-text="{}"{}{}"#,
        escape_attr(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select an option")
        ),
        escape_attr(value),
        escape_attr(&props.empty_text),
        escape_attr(&props.loading_text),
        escape_attr(&props.loading_more_text),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled { " disabled" } else { "" }
    );
    let options_html = options
        .iter()
        .map(render_combo_option_html)
        .collect::<String>();
    let control = format!(
        r#"<div class="combo-box">{hidden}<button{}>{}<span class="combo-box-value">{}</span>{}</button>{clear}<div class="combo-box-popover" role="listbox" data-dowe-combo-popover hidden><div class="combo-box-search-wrap">{}<input class="combo-box-search" type="search" placeholder="{}" data-dowe-combo-search></div><div class="combo-box-options">{options_html}</div><div class="combo-box-empty" hidden>{}</div><div class="combo-box-loading" hidden>{}</div></div></div>"#,
        attrs(classes, Some(&props.style.element), Some(&extra), context),
        floating_label_html(&props.style),
        escape_html(label),
        select_arrow_svg(),
        view_icon_svg(ViewIcon::Search, "combo-box-search-icon"),
        escape_attr(&props.search_placeholder),
        escape_html(&props.empty_text),
        escape_html(&props.loading_text)
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &control,
        context,
    )
}

fn render_combo_option_html(option: &ComboOption) -> String {
    let media = option
        .src
        .as_deref()
        .map(|src| {
            format!(
                r#"<img class="combo-box-option-avatar" src="{}" alt="">"#,
                escape_attr(src)
            )
        })
        .or_else(|| {
            option
                .icon
                .map(|icon| view_icon_svg(icon, "combo-box-option-icon"))
        })
        .unwrap_or_default();
    let description = option
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="combo-box-option-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<button type="button" class="combo-box-option" role="option" data-dowe-combo-value="{}" data-dowe-combo-label="{}"{}>{media}<span class="combo-box-option-copy"><span class="combo-box-option-label">{}</span>{description}</span></button>"#,
        escape_attr(&option.value),
        escape_attr(&option.label),
        if option.disabled { " disabled" } else { "" },
        escape_html(&option.label)
    )
}

fn render_csv_field_html(
    props: &CsvFieldProps,
    columns: &[CsvColumn],
    context: &ReactiveRenderContext,
) -> String {
    let source = format!(
        "{}:{}:{}",
        props.button_text,
        props.modal_title,
        columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    let uid = short_id("csv", &source);
    let mut button_classes = variant_classes("button", &props.style);
    button_classes.push(format!(
        "button-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    button_classes.push("csv-field-button".to_string());
    let columns_html = columns
        .iter()
        .map(|column| {
            format!(
                r#"<div class="csv-field-column" data-dowe-csv-column="{}"><span>{}</span><select class="csv-field-select" data-dowe-csv-select><option value="">{}</option></select></div>"#,
                escape_attr(&column.name),
                escape_html(column.label.as_deref().unwrap_or(&column.name)),
                escape_html(column.label.as_deref().unwrap_or(&column.name))
            )
        })
        .collect::<String>();
    let preview = if props.show_preview {
        format!(
            r#"<div class="csv-field-preview" data-dowe-csv-preview hidden><div class="csv-field-preview-title">{}</div><div class="csv-field-preview-table" data-dowe-csv-table></div></div>"#,
            escape_html(&props.preview_title)
        )
    } else {
        String::new()
    };
    let extra = format!(
        r#" data-dowe-csv data-dowe-csv-preview-rows="{}" data-dowe-csv-preview-page-size="{}""#,
        props.preview_rows, props.preview_page_size
    );
    let field = format!(
        r#"<div{}><input id="{uid}" class="csv-field-input" type="file" accept=".csv,text/csv"{} hidden><button{} type="button" data-dowe-csv-trigger>{}{}</button><div class="csv-field-summary" data-dowe-csv-summary hidden></div>{preview}<div class="csv-field-modal" data-dowe-csv-modal hidden><div class="csv-field-dialog"><h2 class="csv-field-title">{}</h2><p class="csv-field-instructions">{}</p><div class="csv-field-columns">{columns_html}</div><div class="csv-field-error" data-dowe-csv-error{}>{}</div><div class="csv-field-actions"><button class="csv-field-action" type="button" data-dowe-csv-cancel>{}</button><button class="csv-field-action is-primary" type="button" data-dowe-csv-confirm>{}</button><button class="csv-field-action" type="button" data-dowe-csv-clear>{}</button></div></div></div></div>"#,
        attrs(
            vec!["csv-field".to_string()],
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        if props.multiple { " multiple" } else { "" },
        class_attr(button_classes),
        view_icon_svg(ViewIcon::Upload, "csv-field-icon"),
        escape_html(&props.button_text),
        escape_html(&props.modal_title),
        escape_html(&props.instructions),
        if props.error_text.is_some() {
            ""
        } else {
            " hidden"
        },
        escape_html(props.error_text.as_deref().unwrap_or_default()),
        escape_html(&props.cancel_text),
        escape_html(&props.confirm_text),
        escape_html(&props.clear_text)
    );
    render_field_block(
        &props.style,
        None,
        props.error_text.as_deref(),
        &field,
        context,
    )
}

fn render_drag_drop_html(
    props: &DragDropProps,
    items: &[DragItem],
    groups: &[DragGroup],
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("drag-drop", &props.style);
    classes.push(format!("is-{}", props.direction.as_str()));
    classes.push(format!("is-{}", props.size.as_str()));
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    let body = if groups.is_empty() {
        render_drag_drop_list("root", None, items, &props.empty_text)
    } else {
        groups
            .iter()
            .map(|group| {
                render_drag_drop_list(
                    &group.id,
                    group.title.as_deref(),
                    &group.items,
                    &props.empty_text,
                )
            })
            .collect::<String>()
    };
    let extra = format!(
        r#" data-dowe-drag-drop data-dowe-direction="{}" data-dowe-allow-group-transfer="{}""#,
        props.direction.as_str(),
        props.allow_group_transfer
    );
    let surface = format!(
        r#"<div{}>{}</div>"#,
        attrs(classes, Some(&props.style.element), Some(&extra), context),
        body
    );
    render_field_block(&props.style, None, None, &surface, context)
}

fn render_drag_drop_list(
    id: &str,
    title: Option<&str>,
    items: &[DragItem],
    empty_text: &str,
) -> String {
    let title = title
        .map(|title| {
            format!(
                r#"<div class="drag-drop-group-title">{}</div>"#,
                escape_html(title)
            )
        })
        .unwrap_or_default();
    let mut html = format!(
        r#"<div class="drag-drop-group" data-dowe-drag-group="{}">{title}<div class="drag-drop-list">"#,
        escape_attr(id)
    );
    for item in items {
        html.push_str(&render_drag_item_html(item));
    }
    if items.is_empty() {
        html.push_str(&format!(
            r#"<div class="drag-drop-empty">{}</div>"#,
            escape_html(empty_text)
        ));
    }
    html.push_str("</div></div>");
    html
}

fn render_drag_item_html(item: &DragItem) -> String {
    let description = item
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="drag-drop-item-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<button class="drag-drop-item" type="button" draggable="true" data-dowe-drag-item="{}"{}><span class="drag-drop-handle">::</span><span class="drag-drop-item-copy"><span class="drag-drop-item-label">{}</span>{description}</span></button>"#,
        escape_attr(&item.id),
        if item.disabled { " disabled" } else { "" },
        escape_html(item.label.as_deref().unwrap_or(&item.id))
    )
}

fn render_editor_html(props: &EditorProps, context: &ReactiveRenderContext) -> String {
    let toolbar = if props.hide_toolbar {
        String::new()
    } else {
        [
            ("bold", "B"),
            ("italic", "I"),
            ("underline", "U"),
            ("insertUnorderedList", "List"),
            ("justifyLeft", "Left"),
            ("justifyCenter", "Center"),
            ("justifyRight", "Right"),
            ("removeFormat", "Clear"),
        ]
        .into_iter()
        .map(|(command, label)| {
            format!(
                r#"<button class="editor-toolbar-button" type="button" data-dowe-editor-command="{}">{}</button>"#,
                escape_attr(command),
                escape_html(label)
            )
        })
        .collect::<String>()
    };
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<textarea name="{}" data-dowe-editor-hidden hidden>{}</textarea>"#,
                escape_attr(name),
                escape_html(props.value.as_deref().unwrap_or_default())
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-editor style="--dowe-editor-min-height:{}px"{}{}{}"#,
        props.min_height,
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled {
            " data-dowe-disabled=\"true\""
        } else {
            ""
        },
        if props.readonly {
            " data-dowe-readonly=\"true\""
        } else {
            ""
        }
    );
    let body = format!(
        r#"<div{}>{hidden}<div class="editor-toolbar">{toolbar}</div><div class="editor-content" contenteditable="{}" role="textbox" aria-multiline="true" data-dowe-editor-content placeholder="{}">{}</div></div>"#,
        attrs(
            variant_classes("editor", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        if props.disabled || props.readonly {
            "false"
        } else {
            "true"
        },
        escape_attr(props.style.placeholder.as_deref().unwrap_or_default()),
        escape_html(props.value.as_deref().unwrap_or_default())
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &body,
        context,
    )
}

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

fn render_password_html(props: &PasswordProps, context: &ReactiveRenderContext) -> String {
    let has_validation = has_form_validation_contract(&props.style.element);
    let show_icon = solar_control_icon("eye").expect("bundled Password reveal icon");
    let hide_icon = solar_control_icon("eye-closed").expect("bundled Password conceal icon");
    let toggle = format!(
        r#"<button class="password-toggle" type="button" aria-label="Show password" data-dowe-password-toggle><span data-dowe-password-show-icon>{}</span><span data-dowe-password-hide-icon hidden>{}</span></button>"#,
        render_svg_html(&show_icon.props, &show_icon.paths, context),
        render_svg_html(&hide_icon.props, &hide_icon.paths, context)
    );
    let input = format!(
        r#"<input class="password-input input" type="password"{}{}{}{}{}{} data-dowe-password-input>{toggle}"#,
        input_placeholder_attr(&props.style),
        props
            .value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled {
            " disabled"
        } else if props.readonly {
            " readonly"
        } else {
            ""
        },
        if has_validation {
            " data-dowe-validation-control"
        } else {
            ""
        }
    );
    let mut classes = variant_classes("control", &props.style);
    classes.push("password".to_string());
    classes.push(format!(
        "is-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.style.label_floating {
        classes.push("is-floating".to_string());
    }
    if props.error_text.is_some() {
        classes.push("is-error".to_string());
    }
    let control = format!(
        "<span{}>{}{input}</span>",
        attrs(classes, Some(&props.style.element), None, context),
        floating_label_html(&props.style)
    );
    let body = format!("{control}{}", render_password_strength(props));
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &body,
        context,
    )
}

fn render_password_strength(props: &PasswordProps) -> String {
    if props.hide_strength {
        return String::new();
    }
    format!(
        r#"<span class="password-strength" data-dowe-password-strength data-dowe-weak-label="{}" data-dowe-medium-label="{}" data-dowe-strong-label="{}"><span class="password-strength-bars">{}</span><span class="password-strength-label"></span></span>"#,
        escape_attr(&props.weak_label),
        escape_attr(&props.medium_label),
        escape_attr(&props.strong_label),
        (0..6)
            .map(|_| r#"<span class="password-strength-bar"></span>"#)
            .collect::<String>()
    )
}

fn render_phone_html(props: &PhoneProps, context: &ReactiveRenderContext) -> String {
    let country = phone_country(props.country.as_deref()).unwrap_or_else(|| phone_countries()[0]);
    let selected_code = country.code;
    let options = ordered_phone_countries(Some(country.code), &props.priority_countries)
        .iter()
        .map(|country| {
            format!(
                r#"<button type="button" class="phone-country" data-dowe-phone-option data-dowe-country="{}" data-dowe-dial="{}" aria-selected="{}"><span class="phone-flag">{}</span><span class="phone-country-name">{}</span><span class="phone-dial">+{}</span></button>"#,
                escape_attr(country.code),
                escape_attr(country.dial),
                country.code == selected_code,
                country_flag_html(country.code, context),
                escape_html(country.name),
                escape_html(country.dial)
            )
        })
        .collect::<String>();
    let priority = props.priority_countries.join(",");
    let country_trigger = format!(
        r#"<button class="phone-country-trigger" type="button" data-dowe-phone-country aria-expanded="false" aria-haspopup="listbox"><span class="phone-flag">{}</span><span class="phone-dial">+{}</span>{}</button>"#,
        country_flag_html(country.code, context),
        escape_html(country.dial),
        select_arrow_svg()
    );
    let number_input = format!(
        r#"<span class="phone-input-shell">{}<input class="phone-input input" type="tel" inputmode="numeric" pattern="[0-9]*"{}{}{}{}{} data-dowe-phone-input data-dowe-validation-control></span>"#,
        floating_label_html(&props.style),
        input_placeholder_attr(&props.style),
        props
            .value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled { " disabled" } else { "" }
    );
    let input = format!(
        r#"<input type="hidden" name="{}" value="{}" data-dowe-phone-dial>{}{}<div class="phone-popover" data-dowe-phone-popover hidden><div class="phone-search-wrap">{}<input class="phone-search" type="search" placeholder="{}" data-dowe-phone-search></div><div class="phone-countries" data-dowe-phone-countries role="listbox">{options}</div><div class="phone-empty" hidden>{}</div><div class="phone-loading" hidden>{}</div></div>"#,
        escape_attr(&props.dial_code_name),
        escape_attr(country.dial),
        country_trigger,
        number_input,
        view_icon_svg(ViewIcon::Search, "phone-search-icon"),
        escape_attr(&props.search_placeholder),
        escape_html(&props.empty_text),
        escape_html(&props.loading_text)
    );
    let extra = format!(
        r#" data-dowe-phone data-dowe-country="{}" data-dowe-priority-countries="{}""#,
        escape_attr(country.code),
        escape_attr(&priority)
    );
    let mut control_classes = variant_classes("control", &props.style);
    control_classes.push("phone".to_string());
    control_classes.push(format!(
        "is-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.style.label_floating {
        control_classes.push("is-floating".to_string());
    }
    if props.error_text.is_some()
        || props
            .style
            .element
            .form_validation()
            .and_then(|validation| validation.error_text.as_ref())
            .is_some()
    {
        control_classes.push("is-error".to_string());
    }
    let control = format!(
        "<span{}>{}</span>",
        attrs(
            control_classes,
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        input
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &control,
        context,
    )
}

fn country_flag_html(code: &str, context: &ReactiveRenderContext) -> String {
    phone_country_flag_icon(code)
        .map(|icon| render_svg_html(&icon.props, &icon.paths, context))
        .unwrap_or_else(|| "<span class=\"phone-flag-fallback\">--</span>".to_string())
}

fn render_pin_html(props: &PinProps, context: &ReactiveRenderContext) -> String {
    let value = props.value.as_deref().unwrap_or_default();
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    let inputs = (0..props.length)
        .map(|index| {
            let char_value = value
                .chars()
                .nth(index as usize)
                .map(|character| character.to_string())
                .unwrap_or_default();
            let input_type = match props.kind {
                PinKind::Password => "password",
                PinKind::Text | PinKind::Number => "text",
            };
            let input_mode = if props.kind == PinKind::Number {
                "numeric"
            } else {
                "text"
            };
            let mut cell_classes = variant_classes("control", &props.style);
            cell_classes.push("pin-cell".to_string());
            cell_classes.push(format!("is-{}", size.as_str()));
            if props.error_text.is_some()
                || props
                    .style
                    .element
                    .form_validation()
                    .and_then(|validation| validation.error_text.as_ref())
                    .is_some()
            {
                cell_classes.push("is-error".to_string());
            }
            format!(
                r#"<label{}><input class="pin-input" inputmode="{}" type="{}" maxlength="1" value="{}" autocomplete="one-time-code" data-dowe-pin-cell data-dowe-validation-control></label>"#,
                attrs(cell_classes, None, None, context),
                input_mode,
                input_type,
                escape_attr(&char_value),
            )
        })
        .collect::<String>();
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<input type="hidden" name="{}" value="{}" data-dowe-pin-hidden>"#,
                escape_attr(name),
                escape_attr(value)
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-pin data-dowe-pin-length="{}" data-dowe-pin-type="{}"{}"#,
        props.length,
        props.kind.as_str(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let body = format!(
        r#"<div{}>{hidden}<div class="pin-cells">{inputs}</div></div>"#,
        attrs(
            vec!["pin".to_string()],
            Some(&props.style.element),
            Some(&extra),
            context
        )
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &body,
        context,
    )
}

fn render_textarea_html(props: &TextareaProps, context: &ReactiveRenderContext) -> String {
    let control = format!(
        r#"<textarea class="textarea-control input" rows="{}"{}{}{}{}{}{}>{}</textarea>"#,
        props.rows,
        props
            .cols
            .map(|value| format!(r#" cols="{value}""#))
            .unwrap_or_default(),
        props
            .max_length
            .map(|value| format!(r#" maxlength="{value}""#))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        input_placeholder_attr(&props.style),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled {
            " disabled"
        } else if props.readonly {
            " readonly"
        } else {
            ""
        },
        escape_html(props.value.as_deref().unwrap_or_default())
    );
    let mut html = render_field_control(
        "textarea-field",
        &props.style,
        props.style.size.unwrap_or(ButtonSize::Md),
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &control,
        false,
        props
            .value
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        context,
    );
    if props.resize {
        html = html.replace(
            "textarea-control input",
            "textarea-control input is-resizable",
        );
    }
    html
}

fn render_field_control(
    base: &str,
    props: &VariantProps,
    size: ButtonSize,
    help_text: Option<&str>,
    error_text: Option<&str>,
    control_html: &str,
    has_start_adornment: bool,
    has_value: bool,
    context: &ReactiveRenderContext,
) -> String {
    let validation = props.element.form_validation();
    let help_text = validation
        .and_then(|validation| validation.help_text.as_deref())
        .or(help_text);
    let error_text = validation
        .and_then(|validation| validation.error_text.as_deref())
        .or(error_text);
    let mut classes = variant_classes("control", props);
    classes.push(base.to_string());
    classes.push(format!("is-{}", size.as_str()));
    if props.label_floating {
        classes.push("is-floating".to_string());
    }
    if has_start_adornment {
        classes.push("has-start-adornment".to_string());
    }
    if has_value {
        classes.push("has-value".to_string());
    }
    if error_text.is_some() {
        classes.push("is-error".to_string());
    }
    let control = format!(
        "<span{}>{}{}</span>",
        attrs(classes, Some(&props.element), None, context),
        floating_label_html(props),
        control_html
    );
    render_field_block(props, help_text, error_text, &control, context)
}

fn render_field_block(
    props: &VariantProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    body_html: &str,
    context: &ReactiveRenderContext,
) -> String {
    render_field_block_kind(props, help_text, error_text, body_html, "string", context)
}

fn render_field_block_kind(
    props: &VariantProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    body_html: &str,
    value_kind: &str,
    context: &ReactiveRenderContext,
) -> String {
    let validation = props.element.form_validation();
    let help_text = validation
        .and_then(|validation| validation.help_text.as_deref())
        .or(help_text);
    let error_text = validation
        .and_then(|validation| validation.error_text.as_deref())
        .or(error_text);
    let label = if props.label.is_some() && !props.label_floating {
        format!(
            r#"<span class="field-label">{}</span>"#,
            escape_html(props.label.as_deref().unwrap_or_default())
        )
    } else {
        String::new()
    };
    let message = error_text.or(help_text);
    let has_rules = validation.is_some_and(|validation| !validation.rules.is_empty());
    let help = if message.is_some() || has_rules {
        format!(
            r#"<span class="field-help{}" data-dowe-validation-feedback{}>{}</span>"#,
            if error_text.is_some() {
                " is-error"
            } else {
                ""
            },
            if message.is_none() { " hidden" } else { "" },
            escape_html(message.unwrap_or_default())
        )
    } else {
        String::new()
    };
    let validation_attrs =
        render_form_validation_attrs(&props.element, help_text, error_text, value_kind, context);
    let mut field_classes = vec!["field".to_string()];
    append_responsive_classes(
        &mut field_classes,
        "w",
        props.style.sizing.w.as_ref(),
        size_suffix,
    );
    append_responsive_classes(
        &mut field_classes,
        "min-w",
        props.style.sizing.min_w.as_ref(),
        size_suffix,
    );
    append_responsive_classes(
        &mut field_classes,
        "max-w",
        props.style.sizing.max_w.as_ref(),
        size_suffix,
    );
    format!(
        r#"<div{}>{}{body_html}{}</div>"#,
        attrs(
            field_classes,
            Some(&props.element),
            Some(&validation_attrs),
            context,
        ),
        label,
        help
    )
}

fn render_form_validation_attrs(
    element: &ElementProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    value_kind: &str,
    context: &ReactiveRenderContext,
) -> String {
    let Some(validation) = element.form_validation() else {
        if help_text.is_none() && error_text.is_none() {
            return String::new();
        }
        return format!(
            r#" data-dowe-validation-kind="{}"{}{}"#,
            escape_attr(value_kind),
            help_text
                .map(|value| format!(r#" data-dowe-validation-help="{}""#, escape_attr(value)))
                .unwrap_or_default(),
            error_text
                .map(|value| format!(r#" data-dowe-validation-error="{}""#, escape_attr(value)))
                .unwrap_or_default()
        );
    };
    let rules = validation
        .rules
        .iter()
        .map(|rule| {
            let argument = match &rule.kind {
                dowe_components::FormValidationRuleKind::Matches(path) => {
                    Some(context.signal_path(path))
                }
                _ => rule.kind.argument(),
            };
            format!(
                r#"{{"kind":"{}","argument":{},"message":"{}"}}"#,
                escape_json(rule.kind.name()),
                argument
                    .as_deref()
                    .map(|value| format!(r#""{}""#, escape_json(value)))
                    .unwrap_or_else(|| "null".to_string()),
                escape_json(&rule.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let form_attrs = element
        .bind
        .as_deref()
        .and_then(|bind| {
            let (signal, field) = bind.split_once('.')?;
            Some(format!(
                r#" data-dowe-validation-form="{}" data-dowe-validation-field="{}""#,
                escape_attr(&context.signal_path(signal)),
                escape_attr(field)
            ))
        })
        .unwrap_or_default();
    format!(
        r#" data-dowe-validation-kind="{}" data-dowe-validation="{}"{}{}{}"#,
        escape_attr(value_kind),
        escape_attr(&format!("[{rules}]")),
        form_attrs,
        help_text
            .map(|value| format!(r#" data-dowe-validation-help="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        error_text
            .map(|value| format!(r#" data-dowe-validation-error="{}""#, escape_attr(value)))
            .unwrap_or_default()
    )
}

fn render_color_values(props: &ColorProps) -> String {
    if !(props.show_hex || props.show_rgb || props.show_cmyk || props.show_oklch) {
        return String::new();
    }
    let mut html = String::from("<span class=\"color-picker-values\">");
    if props.show_hex {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="hex"></code>"#,
        );
    }
    if props.show_rgb {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="rgb"></code>"#,
        );
    }
    if props.show_cmyk {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="cmyk"></code>"#,
        );
    }
    if props.show_oklch {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="oklch"></code>"#,
        );
    }
    html.push_str("</span>");
    html
}
