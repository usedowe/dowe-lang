fn render_color_html(props: &ColorProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="color-input" type="color" value="{}"{}{}>"#,
        escape_attr(&props.value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let preview = format!(
        r#"<span class="color-field-swatch is-{}" style="background-color:{}"></span><span class="color-field-value">{}</span>"#,
        props.size.as_str(),
        escape_attr(&props.value),
        escape_html(&props.value.to_ascii_uppercase())
    );
    let values = render_color_values(props);
    render_field_control(
        "color-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &format!("{input}<span class=\"color-field-display\">{preview}</span>{values}"),
        context,
    )
}

fn render_date_html(props: &DateProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="date-input" type="date"{}{}{}{}{}>"#,
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
        props
            .min
            .as_deref()
            .map(|value| format!(r#" min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" max="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    render_field_control(
        "date-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        context,
    )
}

fn render_date_range_html(props: &DateRangeProps, context: &ReactiveRenderContext) -> String {
    let start_bind = props
        .start
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let end_bind = props
        .end
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let common = format!(
        "{}{}",
        props
            .min
            .as_deref()
            .map(|value| format!(r#" min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" max="{}""#, escape_attr(value)))
            .unwrap_or_default()
    );
    let input = format!(
        r#"<span class="date-range-inputs"><input class="date-input" type="date"{}{}{}><span class="date-range-separator">-</span><input class="date-input" type="date"{}{}{}></span>"#,
        props
            .start_value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}Start""#, escape_attr(name)))
            .unwrap_or_default(),
        format!("{common}{start_bind}"),
        props
            .end_value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}End""#, escape_attr(name)))
            .unwrap_or_default(),
        format!("{common}{end_bind}")
    );
    render_field_control(
        "date-range-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
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
    let mut group = String::from("<div class=\"radio-group\">");
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
        .unwrap_or_else(|| props.style.placeholder.as_deref().unwrap_or("Select an option"));
    let value = selected.map(|option| option.value.as_str()).unwrap_or_default();
    let clear = if props.clearable {
        r#"<button class="combo-box-clear" type="button" aria-label="Clear selection" data-dowe-combo-clear>&times;</button>"#
            .to_string()
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
        escape_attr(props.style.placeholder.as_deref().unwrap_or("Select an option")),
        escape_attr(value),
        escape_attr(&props.empty_text),
        escape_attr(&props.loading_text),
        escape_attr(&props.loading_more_text),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled { " disabled" } else { "" }
    );
    let options_html = options.iter().map(render_combo_option_html).collect::<String>();
    let control = format!(
        r#"<div class="combo-box">{hidden}<button{}>{}<span class="combo-box-value">{}</span>{clear}{}</button><div class="combo-box-popover" role="listbox" data-dowe-combo-popover><div class="combo-box-search-wrap">{}<input class="combo-box-search" type="search" placeholder="{}" data-dowe-combo-search></div><div class="combo-box-options">{options_html}</div><div class="combo-box-empty" hidden>{}</div><div class="combo-box-loading" hidden>{}</div></div></div>"#,
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
        if props.error_text.is_some() { "" } else { " hidden" },
        escape_html(props.error_text.as_deref().unwrap_or_default()),
        escape_html(&props.cancel_text),
        escape_html(&props.confirm_text),
        escape_html(&props.clear_text)
    );
    render_field_block(&props.style, None, props.error_text.as_deref(), &field, context)
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

fn render_drag_drop_list(id: &str, title: Option<&str>, items: &[DragItem], empty_text: &str) -> String {
    let title = title
        .map(|title| format!(r#"<div class="drag-drop-group-title">{}</div>"#, escape_html(title)))
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
        if props.disabled { " data-dowe-disabled=\"true\"" } else { "" },
        if props.readonly { " data-dowe-readonly=\"true\"" } else { "" }
    );
    let body = format!(
        r#"<div{}>{hidden}<div class="editor-toolbar">{toolbar}</div><div class="editor-content" contenteditable="{}" role="textbox" aria-multiline="true" data-dowe-editor-content placeholder="{}">{}</div></div>"#,
        attrs(
            variant_classes("editor", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        if props.disabled || props.readonly { "false" } else { "true" },
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
    let image = props
        .src
        .as_deref()
        .map(|src| {
            format!(
                r#"<img class="image-cropper-image" src="{}" alt="{}">"#,
                escape_attr(src),
                escape_attr(&props.alt)
            )
        })
        .unwrap_or_else(|| view_icon_svg(ViewIcon::Upload, "image-cropper-empty-icon"));
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<input type="hidden" name="{}" value="{}" data-dowe-cropper-hidden>"#,
                escape_attr(name),
                escape_attr(props.src.as_deref().unwrap_or_default())
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-image-cropper data-dowe-shape="{}" data-dowe-min-width="{}" data-dowe-min-height="{}"{}{}{}"#,
        props.shape.as_str(),
        props.min_width,
        props.min_height,
        props.max_width
            .map(|value| format!(r#" data-dowe-max-width="{value}""#))
            .unwrap_or_default(),
        props.max_height
            .map(|value| format!(r#" data-dowe-max-height="{value}""#))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let body = format!(
        r#"<div{}>{hidden}<input id="{uid}" class="image-cropper-input" type="file" accept="{}" hidden{}><button class="image-cropper-trigger is-{}" type="button" data-dowe-cropper-trigger{}>{image}<span class="image-cropper-label">{}</span></button><div class="image-cropper-actions"><button type="button" class="image-cropper-action" data-dowe-cropper-edit>{}</button><button type="button" class="image-cropper-action" data-dowe-cropper-remove>{}</button></div><div class="image-cropper-modal" data-dowe-cropper-modal hidden><div class="image-cropper-dialog"><div class="image-cropper-stage"><canvas class="image-cropper-canvas" data-dowe-cropper-canvas></canvas><div class="image-cropper-box is-{}"><span></span><span></span><span></span><span></span></div></div><div class="image-cropper-modal-actions"><button type="button" class="image-cropper-action" data-dowe-cropper-cancel>Cancel</button><button type="button" class="image-cropper-action is-primary" data-dowe-cropper-apply>Crop</button></div></div></div></div>"#,
        attrs(
            variant_classes("image-cropper", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        escape_attr(&props.accept),
        if props.disabled { " disabled" } else { "" },
        props.shape.as_str(),
        if props.disabled { " disabled" } else { "" },
        escape_html(props.style.placeholder.as_deref().unwrap_or("Upload")),
        escape_html("Edit"),
        escape_html("Remove"),
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

fn render_password_field_html(props: &PasswordFieldProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="password-field-input input" type="password"{}{}{}{}{} data-dowe-password-input><button class="password-field-toggle" type="button" aria-label="Show password" data-dowe-password-toggle>Show</button>{}"#,
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
        if props.disabled { " disabled" } else if props.readonly { " readonly" } else { "" },
        render_password_strength(props)
    );
    render_field_control(
        "password-field",
        &props.style,
        props.style.size.unwrap_or(ButtonSize::Md),
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        context,
    )
}

fn render_password_strength(props: &PasswordFieldProps) -> String {
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

fn render_phone_field_html(props: &PhoneFieldProps, context: &ReactiveRenderContext) -> String {
    let country = phone_country(props.country.as_deref()).unwrap_or_else(|| phone_countries()[0]);
    let options = phone_countries()
        .iter()
        .map(|country| {
            format!(
                r#"<button type="button" class="phone-field-country" data-dowe-country="{}" data-dowe-dial="{}"><span class="phone-field-flag">{}</span><span class="phone-field-country-name">{}</span><span class="phone-field-dial">+{}</span></button>"#,
                escape_attr(country.code),
                escape_attr(country.dial),
                escape_html(country.flag),
                escape_html(country.name),
                escape_html(country.dial)
            )
        })
        .collect::<String>();
    let priority = props.priority_countries.join(",");
    let input = format!(
        r#"<input type="hidden" name="{}" value="{}" data-dowe-phone-dial><button class="phone-field-country-trigger" type="button" data-dowe-phone-country><span class="phone-field-flag">{}</span><span class="phone-field-dial">+{}</span>{}</button><input class="phone-field-input input" type="tel"{}{}{}{}{} data-dowe-phone-input><div class="phone-field-popover" data-dowe-phone-popover hidden><div class="phone-field-search-wrap">{}<input class="phone-field-search" type="search" placeholder="{}" data-dowe-phone-search></div><div class="phone-field-countries" data-dowe-phone-countries>{options}</div><div class="phone-field-empty" hidden>{}</div><div class="phone-field-loading" hidden>{}</div></div>"#,
        escape_attr(&props.dial_code_name),
        escape_attr(country.dial),
        escape_html(country.flag),
        escape_html(country.dial),
        select_arrow_svg(),
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
        if props.disabled { " disabled" } else { "" },
        view_icon_svg(ViewIcon::Search, "phone-field-search-icon"),
        escape_attr(&props.search_placeholder),
        escape_html(&props.empty_text),
        escape_html(&props.loading_text)
    );
    let extra = format!(
        r#" data-dowe-phone-field data-dowe-country="{}" data-dowe-priority-countries="{}""#,
        escape_attr(country.code),
        escape_attr(&priority)
    );
    let mut control_classes = variant_classes("control", &props.style);
    control_classes.push("phone-field".to_string());
    control_classes.push(format!(
        "is-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.style.label_floating {
        control_classes.push("is-floating".to_string());
    }
    let control = format!(
        "<span{}>{}{}</span>",
        attrs(
            control_classes,
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        floating_label_html(&props.style),
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

fn render_pin_field_html(props: &PinFieldProps, context: &ReactiveRenderContext) -> String {
    let value = props.value.as_deref().unwrap_or_default();
    let inputs = (0..props.length)
        .map(|index| {
            let char_value = value.chars().nth(index as usize).unwrap_or_default();
            format!(
                r#"<input class="pin-field-cell" inputmode="{}" type="{}" maxlength="1" value="{}" data-dowe-pin-cell>"#,
                if props.kind == PinFieldKind::Number { "numeric" } else { "text" },
                props.kind.as_str(),
                escape_attr(&char_value.to_string())
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
        r#" data-dowe-pin-field data-dowe-pin-length="{}" data-dowe-pin-type="{}"{}"#,
        props.length,
        props.kind.as_str(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let body = format!(
        r#"<div{}>{hidden}<div class="pin-field-cells">{inputs}</div></div>"#,
        attrs(
            variant_classes("pin-field", &props.style),
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
        context,
    );
    if props.resize {
        html = html.replace("textarea-control input", "textarea-control input is-resizable");
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
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("control", props);
    classes.push(base.to_string());
    classes.push(format!("is-{}", size.as_str()));
    if props.label_floating {
        classes.push("is-floating".to_string());
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
    let label = if props.label.is_some() && !props.label_floating {
        format!(
            r#"<span class="field-label">{}</span>"#,
            escape_html(props.label.as_deref().unwrap_or_default())
        )
    } else {
        String::new()
    };
    let help = error_text
        .or(help_text)
        .map(|value| {
            format!(
                r#"<span class="field-help{}">{}</span>"#,
                if error_text.is_some() {
                    " is-error"
                } else {
                    ""
                },
                escape_html(value)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<div{}>{}{body_html}{}</div>"#,
        attrs(vec!["field".to_string()], None, None, context),
        label,
        help
    )
}

fn render_color_values(props: &ColorProps) -> String {
    if !(props.show_hex || props.show_rgb || props.show_cmyk || props.show_oklch) {
        return String::new();
    }
    let mut html = String::from("<span class=\"color-picker-values\">");
    if props.show_hex {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">hex: {}</code>"#,
            escape_html(&props.value)
        ));
    }
    if props.show_rgb {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">rgb: {}</code>"#,
            escape_html(&hex_rgb_label(&props.value))
        ));
    }
    if props.show_cmyk {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">cmyk: {}</code>"#,
            escape_html(&hex_cmyk_label(&props.value))
        ));
    }
    if props.show_oklch {
        html.push_str(r#"<code class="color-picker-value-code">oklch: target-derived</code>"#);
    }
    html.push_str("</span>");
    html
}

fn hex_rgb_label(value: &str) -> String {
    let Some((r, g, b)) = parse_hex_rgb(value) else {
        return "rgb(0, 0, 0)".to_string();
    };
    format!("rgb({r}, {g}, {b})")
}

fn hex_cmyk_label(value: &str) -> String {
    let Some((r, g, b)) = parse_hex_rgb(value) else {
        return "cmyk(0%, 0%, 0%, 100%)".to_string();
    };
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 {
        return "cmyk(0%, 0%, 0%, 100%)".to_string();
    }
    let c = ((1.0 - r - k) / (1.0 - k) * 100.0).round() as u8;
    let m = ((1.0 - g - k) / (1.0 - k) * 100.0).round() as u8;
    let y = ((1.0 - b - k) / (1.0 - k) * 100.0).round() as u8;
    let k = (k * 100.0).round() as u8;
    format!("cmyk({c}%, {m}%, {y}%, {k}%)")
}

fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        return Some((r, g, b));
    }
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        return Some((r, g, b));
    }
    None
}

#[derive(Clone, Copy)]
struct PhoneCountry {
    code: &'static str,
    name: &'static str,
    dial: &'static str,
    flag: &'static str,
}

fn phone_country(code: Option<&str>) -> Option<PhoneCountry> {
    let code = code?;
    phone_countries()
        .iter()
        .copied()
        .find(|country| country.code == code)
}

fn phone_countries() -> &'static [PhoneCountry] {
    &[
        PhoneCountry {
            code: "US",
            name: "United States",
            dial: "1",
            flag: "US",
        },
        PhoneCountry {
            code: "CA",
            name: "Canada",
            dial: "1",
            flag: "CA",
        },
        PhoneCountry {
            code: "CO",
            name: "Colombia",
            dial: "57",
            flag: "CO",
        },
        PhoneCountry {
            code: "MX",
            name: "Mexico",
            dial: "52",
            flag: "MX",
        },
        PhoneCountry {
            code: "AR",
            name: "Argentina",
            dial: "54",
            flag: "AR",
        },
        PhoneCountry {
            code: "BR",
            name: "Brazil",
            dial: "55",
            flag: "BR",
        },
        PhoneCountry {
            code: "CL",
            name: "Chile",
            dial: "56",
            flag: "CL",
        },
        PhoneCountry {
            code: "PE",
            name: "Peru",
            dial: "51",
            flag: "PE",
        },
        PhoneCountry {
            code: "ES",
            name: "Spain",
            dial: "34",
            flag: "ES",
        },
        PhoneCountry {
            code: "GB",
            name: "United Kingdom",
            dial: "44",
            flag: "GB",
        },
    ]
}
