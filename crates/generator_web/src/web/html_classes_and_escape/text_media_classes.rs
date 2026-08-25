fn text_classes(base: &str, props: &TextProps) -> Vec<String> {
    let mut classes = vec![format!("dowe-{base}")];
    if let Some(size) = &props.size {
        append_responsive_classes(&mut classes, base, Some(size), |value| {
            value.as_str().to_string()
        });
    } else {
        classes.push(format!("{base}-md"));
    }
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(&mut classes, "text-align", props.align.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(&mut classes, "weight", props.weight.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(
        &mut classes,
        "tracking",
        props.letter_spacing.as_ref(),
        |value| value.as_str().to_string(),
    );
    classes
}

fn svg_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["svg".to_string()];
    append_style_classes(&mut classes, props);
    classes
}

fn video_classes(props: &VideoProps) -> Vec<String> {
    let mut classes = variant_classes("video", &props.style);
    classes.insert(1, props.aspect.as_str().to_string());
    classes
}

fn iframe_classes(props: &IframeProps) -> Vec<String> {
    let mut classes = vec!["iframe".to_string()];
    append_style_classes(&mut classes, &props.style);
    classes
}

fn canvas_classes(props: &CanvasProps) -> Vec<String> {
    let mut classes = vec!["canvas".to_string()];
    append_style_classes(&mut classes, &props.style);
    if props.pixelated {
        classes.push("is-pixelated".to_string());
    }
    classes
}

fn candlestick_classes(props: &CandlestickProps) -> Vec<String> {
    variant_classes("candlestick", &props.style)
}

fn diagram_classes(props: &DiagramProps) -> Vec<String> {
    let mut classes = vec!["diagram".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    if !props.controls {
        classes.push("hide-controls".to_string());
    }
    if !props.minimap {
        classes.push("hide-minimap".to_string());
    }
    if !props.show_grid {
        classes.push("hide-grid".to_string());
    }
    classes
}

fn chart_classes(base: &str, props: &ChartCommonProps) -> Vec<String> {
    let mut classes = variant_classes(base, &props.style);
    classes.push(format!("is-{}", props.size.as_str()));
    classes.push(format!("legend-{}", props.legend_position.as_str()));
    classes.push(format!("palette-{}", props.palette.as_str()));
    if props.loading {
        classes.push("is-loading".to_string());
    }
    if props.hide_legend {
        classes.push("hide-legend".to_string());
    }
    classes
}

fn table_wrapper_classes(props: &TableProps) -> Vec<String> {
    let mut classes = vec!["table-wrapper".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    classes
}

fn table_classes(props: &TableProps) -> Vec<String> {
    let mut classes = vec![
        "table".to_string(),
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
            props.style.color.unwrap_or(ColorFamily::Surface).as_str()
        ),
    ];
    if props.striped {
        classes.push("is-striped".to_string());
    }
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    if props.dividers {
        classes.push("has-dividers".to_string());
    }
    classes
}

fn divider_classes(props: &DividerProps) -> Vec<String> {
    let mut classes = vec![
        "divider".to_string(),
        format!("divider-{}", props.orientation.as_str()),
        format!("is-{}", props.color.as_str()),
    ];
    append_style_classes(&mut classes, &props.style);
    classes
}

