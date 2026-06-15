fn collect_media_form_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::Audio { props } => {
            classes.extend(variant_classes("media", &props.style));
            classes.extend([
                "media-button".to_string(),
                "media-content".to_string(),
                "media-waveform".to_string(),
                "media-bars".to_string(),
                "media-bar".to_string(),
                "media-footer".to_string(),
                "media-time".to_string(),
                "media-subtitle".to_string(),
                "media-avatar".to_string(),
            ]);
        }
        ViewNode::Image { props } => {
            classes.extend(variant_classes("image", &props.style));
            classes.extend([
                props.aspect.as_str().to_string(),
                format!("fit-{}", props.object_fit.as_str()),
                "image-element".to_string(),
                "image-controls".to_string(),
                "image-actions".to_string(),
                "image-action".to_string(),
            ]);
        }
        ViewNode::Accordion { props, items } => {
            classes.extend(variant_classes("accordion", &props.style));
            classes.extend([
                "accordion-item".to_string(),
                "accordion-header".to_string(),
                "accordion-start".to_string(),
                "accordion-label".to_string(),
                "accordion-end".to_string(),
                "accordion-arrow".to_string(),
                "accordion-content".to_string(),
            ]);
            for item in items {
                for child in &item.children {
                    collect_classes(child, classes);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            classes.extend(variant_classes("carousel", &props.style));
            classes.extend([
                "carousel-header".to_string(),
                "carousel-title".to_string(),
                "carousel-viewport".to_string(),
                "carousel-container".to_string(),
                "carousel-slide".to_string(),
                "carousel-controls".to_string(),
                "carousel-control".to_string(),
                "carousel-indicators".to_string(),
                "carousel-indicator".to_string(),
                "carousel-counter".to_string(),
                "carousel-nav".to_string(),
            ]);
            for slide in slides {
                for child in &slide.children {
                    collect_classes(child, classes);
                }
            }
        }
        ViewNode::Checkbox { props } => {
            classes.extend(["checkbox".to_string(), "checkbox-input".to_string()]);
            classes.insert(format!(
                "is-{}",
                props.style.color.unwrap_or(ColorFamily::Primary).as_str()
            ));
        }
        ViewNode::Color { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "color-field".to_string(),
                "color-input".to_string(),
                "color-field-display".to_string(),
                "color-field-swatch".to_string(),
                "color-field-value".to_string(),
                "color-picker-values".to_string(),
                "color-picker-value-code".to_string(),
            ]);
        }
        ViewNode::Date { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "date-field".to_string(),
                "date-input".to_string(),
            ]);
        }
        ViewNode::DateRange { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "date-range-field".to_string(),
                "date-range-inputs".to_string(),
                "date-range-separator".to_string(),
                "date-input".to_string(),
            ]);
        }
        ViewNode::RadioGroup { props, .. } => {
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
            ]);
        }
        ViewNode::Toggle { props } => {
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
            classes.insert("input".to_string());
        }
        ViewNode::Select { props, .. } => {
            classes.extend(variant_classes("control", props));
            classes.insert("select".to_string());
            classes.insert("select-control".to_string());
            classes.insert("select-popover".to_string());
            classes.insert("select-option".to_string());
        }
        ViewNode::ComboBox { props, .. } => {
            classes.extend(variant_classes("control", &props.style));
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
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "image-cropper-trigger".to_string(),
                "image-cropper-image".to_string(),
                "image-cropper-empty-icon".to_string(),
                "image-cropper-label".to_string(),
                "image-cropper-actions".to_string(),
                "image-cropper-action".to_string(),
                "image-cropper-modal".to_string(),
                "image-cropper-dialog".to_string(),
                "image-cropper-stage".to_string(),
                "image-cropper-canvas".to_string(),
                "image-cropper-box".to_string(),
                "image-cropper-modal-actions".to_string(),
                format!("is-{}", props.shape.as_str()),
                "is-primary".to_string(),
            ]);
        }
        ViewNode::PasswordField { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "password-field".to_string(),
                "password-field-input".to_string(),
                "password-field-toggle".to_string(),
                "password-strength".to_string(),
                "password-strength-bars".to_string(),
                "password-strength-bar".to_string(),
                "password-strength-label".to_string(),
            ]);
        }
        ViewNode::PhoneField { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "phone-field".to_string(),
                "phone-field-country-trigger".to_string(),
                "phone-field-flag".to_string(),
                "phone-field-dial".to_string(),
                "phone-field-input".to_string(),
                "phone-field-popover".to_string(),
                "phone-field-search-wrap".to_string(),
                "phone-field-search".to_string(),
                "phone-field-search-icon".to_string(),
                "phone-field-countries".to_string(),
                "phone-field-country".to_string(),
                "phone-field-country-name".to_string(),
                "phone-field-empty".to_string(),
                "phone-field-loading".to_string(),
            ]);
        }
        ViewNode::PinField { props } => {
            classes.extend(variant_classes("pin-field", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "pin-field-cells".to_string(),
                "pin-field-cell".to_string(),
            ]);
        }
        ViewNode::Textarea { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "textarea-field".to_string(),
                "textarea-control".to_string(),
                "is-resizable".to_string(),
            ]);
        }
        ViewNode::Code { props } => {
            classes.extend(variant_classes("code-block", &props.style));
        }
        ViewNode::Video { props } => {
            classes.extend(video_classes(props));
        }
        ViewNode::Candlestick { props } => {
            classes.extend(candlestick_classes(props));
            classes.insert("candlestick-canvas".to_string());
            classes.insert("candlestick-empty".to_string());
        }
        ViewNode::ArcChart { props } => {
            collect_chart_classes("arc-chart-container", &props.common, classes);
        }
        ViewNode::AreaChart { props } => {
            collect_chart_classes("area-chart-container", &props.common, classes);
        }
        ViewNode::BarChart { props } => {
            collect_chart_classes("bar-chart-container", &props.common, classes);
        }
        ViewNode::LineChart { props } => {
            collect_chart_classes("line-chart-container", &props.common, classes);
        }
        ViewNode::PieChart { props } => {
            collect_chart_classes("pie-chart-container", &props.common, classes);
        }
        ViewNode::Table { props } => {
            classes.extend(table_wrapper_classes(props));
            classes.insert("table-container".to_string());
            classes.extend(table_classes(props));
            classes.insert("table-header".to_string());
            classes.insert("table-head".to_string());
            classes.insert("table-head-content".to_string());
            classes.insert("table-head-label".to_string());
            classes.insert("table-body".to_string());
            classes.insert("table-empty-row".to_string());
            classes.insert("table-empty-cell".to_string());
            classes.insert("empty-state".to_string());
            classes.insert("empty-content".to_string());
            classes.insert("empty-title".to_string());
            classes.insert("empty-description".to_string());
        }
        ViewNode::Divider { props } => {
            classes.extend(divider_classes(props));
        }
        ViewNode::Alert { props } => {
            classes.extend(variant_classes("alert", &props.style));
            classes.insert("alert-close".to_string());
        }
        ViewNode::Svg { props, .. } => {
            classes.extend(svg_classes(&props.style));
        }
        ViewNode::Title { props, .. } => {
            classes.extend(text_classes("title", props));
        }
        ViewNode::Text { props, .. } => {
            classes.extend(text_classes("text", props));
        }
        _ => unreachable!(),
    }
}

fn collect_chart_classes(base: &str, props: &ChartCommonProps, classes: &mut BTreeSet<String>) {
    classes.extend(chart_classes(base, props));
    classes.insert("dowe-chart-viewport".to_string());
    classes.insert("dowe-chart-svg".to_string());
    classes.insert("dowe-chart-loading".to_string());
    classes.insert("dowe-chart-empty".to_string());
    classes.insert("dowe-chart-legend".to_string());
    classes.insert("dowe-chart-legend-item".to_string());
    classes.insert("dowe-chart-legend-color".to_string());
    classes.insert("dowe-chart-axis-line".to_string());
    classes.insert("dowe-chart-axis-label".to_string());
    classes.insert("dowe-chart-grid-line".to_string());
    classes.insert("dowe-chart-line".to_string());
    classes.insert("dowe-chart-area".to_string());
    classes.insert("dowe-chart-point".to_string());
    classes.insert("dowe-chart-bar".to_string());
    classes.insert("dowe-chart-slice".to_string());
    classes.insert("dowe-chart-arc".to_string());
    classes.insert("dowe-chart-center-value".to_string());
    classes.insert("dowe-chart-center-label".to_string());
}
