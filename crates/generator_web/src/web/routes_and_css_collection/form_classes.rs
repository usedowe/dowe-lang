fn collect_reactive_form_classes(
    classes: &mut BTreeSet<String>,
    props: &VariantProps,
    size_prefix: &str,
) {
    if props.reactive.size.is_some() {
        for value in ["xs", "sm", "md", "lg", "xl"] {
            classes.insert(format!("{size_prefix}{value}"));
        }
    }
    if props.reactive.rounded.is_some() {
        for value in ["xs", "sm", "md", "lg", "xl", "full"] {
            classes.insert(format!("rounded-{value}"));
        }
    }
}

fn collect_form_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
            ViewNode::Checkbox { props } => {
                classes.extend(variant_classes("checkbox", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend(["checkbox".to_string(), "checkbox-input".to_string()]);
                classes.insert(format!(
                    "is-{}",
                    props.style.color.unwrap_or(ColorFamily::Primary).as_str()
                ));
            }
            ViewNode::Color { props } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "color-field".to_string(),
                    "color-control-shell".to_string(),
                    "color-control-trigger".to_string(),
                    "color-input".to_string(),
                    "color-field-swatch".to_string(),
                    "color-field-value".to_string(),
                    "color-picker-popover".to_string(),
                    "color-picker-canvas".to_string(),
                    "color-picker-cursor".to_string(),
                    "color-picker-hue".to_string(),
                    "color-picker-slider-thumb".to_string(),
                    "color-picker-preview".to_string(),
                    "color-picker-preview-swatch".to_string(),
                    "color-picker-preview-color".to_string(),
                    "color-picker-preview-info".to_string(),
                    "color-picker-preview-hex".to_string(),
                    "color-picker-preview-foreground".to_string(),
                    "color-picker-values".to_string(),
                    "color-picker-value-code".to_string(),
                ]);
            }
            ViewNode::Date { props } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "date-field".to_string(),
                    "date-control-shell".to_string(),
                    "date-control-trigger".to_string(),
                    "date-control-value".to_string(),
                    "date-popover".to_string(),
                    "date-picker-header".to_string(),
                    "date-picker-month".to_string(),
                    "date-picker-nav".to_string(),
                    "date-picker-weekdays".to_string(),
                    "date-picker-days".to_string(),
                    "weekday".to_string(),
                    "date-picker-day-button".to_string(),
                    "date-picker-empty-day".to_string(),
                ]);
            }
            ViewNode::DateRange { props } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "date-range-field".to_string(),
                    "date-control-shell".to_string(),
                    "date-control-trigger".to_string(),
                    "date-control-value".to_string(),
                    "date-range-popover".to_string(),
                    "date-range-calendars".to_string(),
                    "date-range-calendar".to_string(),
                    "date-picker-header".to_string(),
                    "date-picker-month".to_string(),
                    "date-picker-nav".to_string(),
                    "date-picker-weekdays".to_string(),
                    "date-picker-days".to_string(),
                    "weekday".to_string(),
                    "date-picker-day-button".to_string(),
                    "date-picker-empty-day".to_string(),
                    "date-range-spacer".to_string(),
                ]);
            }
            ViewNode::RadioGroup { props, .. } => {
                if matches!(props.presentation, RadioGroupPresentation::Card) {
                    classes.extend(variant_classes("radio-card-group", &props.style));
                    collect_reactive_form_classes(classes, &props.style, "is-");
                    classes.extend([
                        "field".to_string(),
                        "field-label".to_string(),
                        "field-help".to_string(),
                        "radio-card-group".to_string(),
                        "radio-card".to_string(),
                        "radio-card-control".to_string(),
                        "radio-card-content".to_string(),
                        "radio-card-icon".to_string(),
                        "radio-card-copy".to_string(),
                        "radio-card-title".to_string(),
                        "radio-card-description".to_string(),
                        "radio-card-indicator".to_string(),
                        format!("is-{}", props.style.color.unwrap_or(ColorFamily::Primary).as_str()),
                        format!("is-{}", props.size.as_str()),
                        format!("is-{}", props.orientation.as_str()),
                    ]);
                    return;
                }
                classes.extend(variant_classes("radio-group", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "radio-group".to_string(),
                    "radio-item".to_string(),
                    "radio".to_string(),
                    "label".to_string(),
                    format!(
                        "is-{}",
                        props.style.color.unwrap_or(ColorFamily::Primary).as_str()
                    ),
                    format!("is-{}", props.size.as_str()),
                    format!("is-{}", props.orientation.as_str()),
                ]);
            }
            ViewNode::Toggle { props } => {
                classes.extend(variant_classes("toggle", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "toggle".to_string(),
                    "toggle-input".to_string(),
                    "toggle-label-left".to_string(),
                    "toggle-label-right".to_string(),
                    "label-md".to_string(),
                    format!(
                        "is-{}",
                        props.style.color.unwrap_or(ColorFamily::Primary).as_str()
                    ),
                ]);
            }
            ViewNode::Slider { props } => {
                classes.extend(variant_classes("slider", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "slider-wrapper".to_string(),
                    "slider-info".to_string(),
                    "slider".to_string(),
                    format!("is-{}", props.size.as_str()),
                    format!(
                        "is-{}",
                        props.style.color.unwrap_or(ColorFamily::Primary).as_str()
                    ),
                ]);
            }
            ViewNode::Dropzone { props } => {
                classes.extend(variant_classes("dropzone-input", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "dropzone".to_string(),
                    "dropzone-input".to_string(),
                    "dropzone-content".to_string(),
                    "dropzone-icon".to_string(),
                    "dropzone-placeholder".to_string(),
                    "dropzone-files".to_string(),
                    "dropzone-file".to_string(),
                    "dropzone-file-preview".to_string(),
                    "dropzone-file-image".to_string(),
                    "dropzone-file-icon".to_string(),
                    "dropzone-file-info".to_string(),
                    "dropzone-file-name".to_string(),
                    "dropzone-file-size".to_string(),
                    "dropzone-file-remove".to_string(),
                    format!("is-{}", props.size.as_str()),
                    format!(
                        "is-{}",
                        props
                            .style
                            .variant
                            .unwrap_or(ComponentVariant::Solid)
                            .as_str()
                    ),
                    format!(
                        "is-{}",
                        props.style.color.unwrap_or(ColorFamily::Primary).as_str()
                    ),
                ]);
            }
            ViewNode::Input { props } => {
                classes.extend(variant_classes("control", props));
                collect_reactive_form_classes(classes, props, "is-");
                classes.insert("input".to_string());
            }
            ViewNode::SelectTheme { props } => {
                classes.extend(variant_classes("control", &props.style));
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "theme-select".to_string(),
                    "theme-select-control".to_string(),
                    "theme-select-input".to_string(),
                ]);
            }
            ViewNode::Select { props, .. } => {
                classes.extend(variant_classes("control", props));
                collect_reactive_form_classes(classes, props, "is-");
                classes.insert("select".to_string());
                classes.insert("select-control".to_string());
                classes.insert("select-popover".to_string());
                classes.insert("select-option".to_string());
            }
            ViewNode::ComboBox { props, .. } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "combo-box".to_string(),
                    "combo-box-control".to_string(),
                    "combo-box-value".to_string(),
                    "combo-box-clear".to_string(),
                    "combo-box-popover".to_string(),
                    "combo-box-search-wrap".to_string(),
                    "combo-box-search".to_string(),
                    "combo-box-search-icon".to_string(),
                    "combo-box-options".to_string(),
                    "combo-box-option".to_string(),
                    "combo-box-option-avatar".to_string(),
                    "combo-box-option-icon".to_string(),
                    "combo-box-option-copy".to_string(),
                    "combo-box-option-label".to_string(),
                    "combo-box-option-description".to_string(),
                    "combo-box-empty".to_string(),
                    "combo-box-loading".to_string(),
                ]);
            }
            ViewNode::CsvField { props, .. } => {
                classes.extend(variant_classes("button", &props.style));
                collect_reactive_form_classes(classes, &props.style, "button-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "csv-field".to_string(),
                    "csv-field-button".to_string(),
                    "csv-field-icon".to_string(),
                    "csv-field-summary".to_string(),
                    "csv-field-preview".to_string(),
                    "csv-field-preview-title".to_string(),
                    "csv-field-preview-table".to_string(),
                    "csv-field-modal".to_string(),
                    "csv-field-dialog".to_string(),
                    "csv-field-title".to_string(),
                    "csv-field-instructions".to_string(),
                    "csv-field-columns".to_string(),
                    "csv-field-column".to_string(),
                    "csv-field-select".to_string(),
                    "csv-field-error".to_string(),
                    "csv-field-actions".to_string(),
                    "csv-field-action".to_string(),
                    "is-primary".to_string(),
                    format!(
                        "button-{}",
                        props.style.size.unwrap_or(ButtonSize::Md).as_str()
                    ),
                ]);
            }
            ViewNode::DragDrop { props, .. } => {
                classes.extend(variant_classes("drag-drop", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "drag-drop-group".to_string(),
                    "drag-drop-group-title".to_string(),
                    "drag-drop-list".to_string(),
                    "drag-drop-empty".to_string(),
                    "drag-drop-item".to_string(),
                    "drag-drop-handle".to_string(),
                    "drag-drop-item-copy".to_string(),
                    "drag-drop-item-label".to_string(),
                    "drag-drop-item-description".to_string(),
                    format!("is-{}", props.direction.as_str()),
                    format!("is-{}", props.size.as_str()),
                ]);
            }
            ViewNode::Editor { props } => {
                classes.extend(variant_classes("editor", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "editor-toolbar".to_string(),
                    "editor-toolbar-button".to_string(),
                    "editor-content".to_string(),
                ]);
            }
            ViewNode::ImageCropper { props } => {
                classes.extend(variant_classes("image-cropper", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "image-cropper-trigger".to_string(),
                    "image-cropper-dialog-header".to_string(),
                    "image-cropper-dialog-close".to_string(),
                    "image-cropper-image".to_string(),
                    "image-cropper-empty-icon".to_string(),
                    "image-cropper-label".to_string(),
                    "image-cropper-actions".to_string(),
                    "image-cropper-action".to_string(),
                    "image-cropper-action-spacer".to_string(),
                    "image-cropper-modal".to_string(),
                    "image-cropper-dialog".to_string(),
                    "image-cropper-stage".to_string(),
                    "image-cropper-canvas".to_string(),
                    "image-cropper-grid".to_string(),
                    "image-cropper-box".to_string(),
                    "image-cropper-zoom".to_string(),
                    "image-cropper-runtime-error".to_string(),
                    "image-cropper-modal-actions".to_string(),
                    format!("is-{}", props.shape.as_str()),
                    "is-primary".to_string(),
                ]);
            }
            ViewNode::Password { props } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "password".to_string(),
                    "password-input".to_string(),
                    "password-toggle".to_string(),
                    "password-strength".to_string(),
                    "password-strength-bars".to_string(),
                    "password-strength-bar".to_string(),
                    "password-strength-label".to_string(),
                ]);
            }
            ViewNode::Phone { props } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "phone".to_string(),
                    "phone-country-trigger".to_string(),
                    "phone-flag".to_string(),
                    "phone-dial".to_string(),
                    "phone-input-shell".to_string(),
                    "phone-input".to_string(),
                    "phone-popover".to_string(),
                    "phone-search-wrap".to_string(),
                    "phone-search".to_string(),
                    "phone-search-icon".to_string(),
                    "phone-countries".to_string(),
                    "phone-country".to_string(),
                    "phone-country-name".to_string(),
                    "phone-empty".to_string(),
                    "phone-loading".to_string(),
                ]);
            }
            ViewNode::Pin { props } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "pin-cells".to_string(),
                    "pin-cell".to_string(),
                ]);
            }
            ViewNode::Textarea { props } => {
                classes.extend(variant_classes("control", &props.style));
                collect_reactive_form_classes(classes, &props.style, "is-");
                classes.extend([
                    "field".to_string(),
                    "field-label".to_string(),
                    "field-help".to_string(),
                    "textarea-field".to_string(),
                    "textarea-control".to_string(),
                    "is-resizable".to_string(),
                ]);
            }
        _ => {}
    }
}
