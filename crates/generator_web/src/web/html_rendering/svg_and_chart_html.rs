fn render_svg_html(props: &SvgProps, paths: &[SvgPath], context: &ReactiveRenderContext) -> String {
    if let Some(data) = props.data.as_deref() {
        let extra = format!(
            r#" data-dowe-svg-data="{}""#,
            escape_attr(&context.signal_path(data))
        );
        return format!(
            r#"<svg{} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" aria-hidden="true"></svg>"#,
            attrs(
                svg_classes(&props.style),
                Some(&props.style.element),
                Some(&extra),
                context
            )
        );
    }
    if let Some(name) = props.icon_name.as_deref() {
        let mut classes = svg_classes(&props.style);
        if let Some(fill) = props.icon_fill {
            classes.push(format!("color-{}", fill.as_str()));
        }
        if let Some(stroke) = props.icon_stroke {
            classes.push(format!("stroke-color-{}", stroke.as_str()));
        }
        let mut extra = format!(
            r#" data-dowe-icon-name="{}""#,
            escape_attr(&context.signal_path(name))
        );
        if let Some(fallback) = props.icon_fallback.as_deref() {
            extra.push_str(&format!(
                r#" data-dowe-icon-fallback="{}""#,
                escape_attr(fallback)
            ));
        }
        if let Some(binding) = props.icon_fill_binding.as_deref() {
            extra.push_str(&format!(
                r#" data-dowe-icon-fill="{}""#,
                escape_attr(&context.signal_path(binding))
            ));
        }
        if let Some(binding) = props.icon_stroke_binding.as_deref() {
            extra.push_str(&format!(
                r#" data-dowe-icon-stroke="{}""#,
                escape_attr(&context.signal_path(binding))
            ));
        }
        return format!(
            r#"<svg{} xmlns="http://www.w3.org/2000/svg" viewBox="{}" aria-hidden="true"></svg>"#,
            attrs(classes, Some(&props.style.element), Some(&extra), context),
            escape_attr(&props.view_box.as_str())
        );
    }
    if let Some(motion) = &props.motion {
        if motion.animated {
            return render_svg_spinner_html(props, motion, context);
        }
        return render_bundled_svg_html(props, motion.source, context);
    }
    let mut html = format!(
        r#"<svg{} xmlns="http://www.w3.org/2000/svg" viewBox="{}" aria-hidden="true">"#,
        attrs(
            svg_classes(&props.style),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.view_box.as_str())
    );
    for path in paths {
        let transform = path
            .transform
            .as_ref()
            .map(|value| format!(r#" transform="{}""#, escape_attr(&value.as_str())))
            .unwrap_or_default();
        html.push_str(&format!(
            r#"<path d="{}"{}{}></path>"#,
            escape_attr(&path.data),
            svg_path_attributes(path.fill),
            transform
        ));
    }
    html.push_str("</svg>");
    html
}

fn render_bundled_svg_html(
    props: &SvgProps,
    source: &str,
    context: &ReactiveRenderContext,
) -> String {
    let encoded = source
        .as_bytes()
        .iter()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (*byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect::<String>();
    format!(
        r#"<svg{} xmlns="http://www.w3.org/2000/svg" viewBox="{}" aria-hidden="true"><image width="100%" height="100%" preserveAspectRatio="xMidYMid meet" href="data:image/svg+xml,{}"></image></svg>"#,
        attrs(
            svg_classes(&props.style),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.view_box.as_str()),
        encoded
    )
}

fn render_svg_spinner_html(
    props: &SvgProps,
    motion: &dowe_components::SvgMotion,
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = svg_classes(&props.style);
    classes.push("is-svg-spinner".to_string());
    let body_start = motion.source.find('>').map(|index| index + 1).unwrap_or(0);
    let body_end = motion.source.rfind("</svg>").unwrap_or(motion.source.len());
    let body = &motion.source[body_start..body_end];
    let fill = svg_spinner_color(motion.fill);
    let stroke = svg_spinner_color(motion.stroke);
    format!(
        r#"<svg{} xmlns="http://www.w3.org/2000/svg" viewBox="{}" fill="{}" stroke="{}" aria-hidden="true">{}<path class="dowe-svg-spinner-fallback" d="M12 3a9 9 0 1 1-6.364 2.636" fill="none" stroke="{}" stroke-width="2.5" stroke-linecap="round"></path><style>.dowe-svg-spinner-fallback{{display:none}}@media (prefers-reduced-motion:reduce){{.is-svg-spinner>*:not(style):not(.dowe-svg-spinner-fallback){{display:none!important}}.dowe-svg-spinner-fallback{{display:inline}}}}</style></svg>"#,
        attrs(classes, Some(&props.style.element), None, context),
        escape_attr(&props.view_box.as_str()),
        fill,
        stroke,
        body,
        stroke
    )
}

fn svg_spinner_color(color: Option<ColorToken>) -> String {
    color
        .map(|value| format!("var(--dowe-{})", value.as_str()))
        .unwrap_or_else(|| "currentColor".to_string())
}

fn render_candlestick_html(props: &CandlestickProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" role="figure" aria-label="Candlestick chart" data-dowe-candlestick data-dowe-candlestick-data="{}" data-dowe-candlestick-up="{}" data-dowe-candlestick-down="{}" data-dowe-candlestick-max="{}""#,
        escape_attr(&context.signal_path(&props.data)),
        props.up_color.as_str(),
        props.down_color.as_str(),
        props.max_points
    );
    if let Some(stream) = props.stream.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-candlestick-stream="{}""#,
            escape_attr(stream)
        ));
    }
    format!(
        r#"<figure{}><canvas class="candlestick-canvas"></canvas><figcaption class="candlestick-empty">{}</figcaption></figure>"#,
        attrs(
            candlestick_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context,
        ),
        escape_html(&props.empty_label)
    )
}

fn render_canvas_html(props: &CanvasProps, context: &ReactiveRenderContext) -> String {
    let background = match props.background {
        CanvasBackground::Transparent => "transparent",
        CanvasBackground::Color(color) => color.as_str(),
    };
    let mut extra = format!(
        r#" role="img" aria-label="{}" data-dowe-canvas data-dowe-canvas-scene="{}" data-dowe-canvas-view-width="{}" data-dowe-canvas-view-height="{}" data-dowe-canvas-fit="{}" data-dowe-canvas-fps="{}" data-dowe-canvas-autoplay="{}" data-dowe-canvas-background="{}""#,
        escape_attr(&props.label),
        escape_attr(&context.signal_path(&props.scene)),
        props.view_width,
        props.view_height,
        props.fit.as_str(),
        props.fps,
        props.autoplay,
        background,
    );
    if props.draw {
        let mode = if props.draw_mode_binding {
            context.signal_path(&props.draw_mode)
        } else {
            props.draw_mode.clone()
        };
        extra.push_str(&format!(r#" data-dowe-canvas-draw="true" data-dowe-canvas-draw-mode="{}"{}"#, escape_attr(&mode), if props.draw_mode_binding { " data-dowe-canvas-draw-mode-binding=\"true\"" } else { "" }));
    }
    if let Some(path) = props.layer_bind.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-canvas-layer-bind="{}""#,
            escape_attr(&context.signal_path(path))
        ));
    }
    if let Some(path) = props.selected_layer.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-canvas-selected="{}""#,
            escape_attr(&context.signal_path(path))
        ));
    }
    for (attribute, action) in [
        ("data-dowe-canvas-on-layer-add", props.on_layer_add.as_deref()),
        ("data-dowe-canvas-on-layer-change", props.on_layer_change.as_deref()),
        ("data-dowe-canvas-on-layer-remove", props.on_layer_remove.as_deref()),
        ("data-dowe-canvas-on-layer-select", props.on_layer_select.as_deref()),
    ] {
        if let Some(action) = action {
            extra.push_str(&format!(
                r#" {attribute}="{}""#,
                escape_attr(&context.action_id(action))
            ));
        }
    }
    if let Some(action) = props.on_pointer.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-canvas-on-pointer="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if props.on_key.is_some() || props.layer_bind.is_some() {
        if let Some(action) = props.on_key.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-canvas-on-key="{}" tabindex="0""#,
            escape_attr(&context.action_id(action))
        ));
        } else {
            extra.push_str(r#" tabindex="0""#);
        }
    }
    if let Some(action) = props.on_motion.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-canvas-on-motion="{}" data-dowe-canvas-motion-rate="{}""#,
            escape_attr(&context.action_id(action)),
            props.motion_rate
        ));
    }
    format!(
        r#"<canvas{} width="{}" height="{}"></canvas>"#,
        attrs(
            canvas_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context,
        ),
        props.view_width,
        props.view_height,
    )
}

fn render_arc_chart_html(props: &ArcChartProps, context: &ReactiveRenderContext) -> String {
    let extra = format!(
        r#"{} data-dowe-chart-thickness="{}" data-dowe-chart-gap="{}" data-dowe-chart-start-angle="{}" data-dowe-chart-end-angle="{}" data-dowe-chart-show-inline-labels="{}" data-dowe-chart-hide-values="{}" data-dowe-chart-show-glow="{}"{}{}"#,
        chart_common_attrs("arc", &props.common, context),
        props.thickness,
        props.gap,
        props.start_angle,
        props.end_angle,
        props.show_inline_labels,
        props.hide_values,
        props.show_glow,
        optional_chart_attr("center-text", props.center_text.as_deref()),
        optional_chart_attr("center-value", props.center_value.as_deref()),
    );
    render_chart_html(
        "arc-chart-container",
        "Arc chart",
        &props.common,
        extra,
        context,
    )
}

fn render_area_chart_html(props: &AreaChartProps, context: &ReactiveRenderContext) -> String {
    let extra = format!(
        r#"{} data-dowe-chart-curve="{}" data-dowe-chart-stroke-width="{}" data-dowe-chart-fill-opacity="{}" data-dowe-chart-stacked="{}" data-dowe-chart-hide-line="{}" data-dowe-chart-show-points="{}" data-dowe-chart-hide-grid="{}" data-dowe-chart-hide-x-axis="{}" data-dowe-chart-hide-y-axis="{}" data-dowe-chart-show-glow="{}""#,
        chart_common_attrs("area", &props.common, context),
        props.curve.as_str(),
        props.stroke_width,
        props.fill_opacity,
        props.stacked,
        props.hide_line,
        props.show_points,
        props.hide_grid,
        props.hide_x_axis,
        props.hide_y_axis,
        props.show_glow,
    );
    render_chart_html(
        "area-chart-container",
        "Area chart",
        &props.common,
        extra,
        context,
    )
}

fn render_bar_chart_html(props: &BarChartProps, context: &ReactiveRenderContext) -> String {
    let extra = format!(
        r#"{} data-dowe-chart-grouped="{}" data-dowe-chart-stacked="{}" data-dowe-chart-show-values="{}" data-dowe-chart-bar-radius="{}" data-dowe-chart-hide-grid="{}" data-dowe-chart-show-glow="{}""#,
        chart_common_attrs("bar", &props.common, context),
        props.grouped,
        props.stacked,
        props.show_values,
        props.bar_radius,
        props.hide_grid,
        props.show_glow,
    );
    render_chart_html(
        "bar-chart-container",
        "Bar chart",
        &props.common,
        extra,
        context,
    )
}

fn render_line_chart_html(props: &LineChartProps, context: &ReactiveRenderContext) -> String {
    let extra = format!(
        r#"{} data-dowe-chart-curve="{}" data-dowe-chart-stroke-width="{}" data-dowe-chart-point-radius="{}" data-dowe-chart-hide-points="{}" data-dowe-chart-hide-grid="{}" data-dowe-chart-hide-x-axis="{}" data-dowe-chart-hide-y-axis="{}" data-dowe-chart-show-gradient-fill="{}" data-dowe-chart-show-glow="{}""#,
        chart_common_attrs("line", &props.common, context),
        props.curve.as_str(),
        props.stroke_width,
        props.point_radius,
        props.hide_points,
        props.hide_grid,
        props.hide_x_axis,
        props.hide_y_axis,
        props.show_gradient_fill,
        props.show_glow,
    );
    render_chart_html(
        "line-chart-container",
        "Line chart",
        &props.common,
        extra,
        context,
    )
}

fn render_pie_chart_html(props: &PieChartProps, context: &ReactiveRenderContext) -> String {
    let extra = format!(
        r#"{} data-dowe-chart-donut="{}" data-dowe-chart-donut-width="{}" data-dowe-chart-start-angle="{}" data-dowe-chart-pad-angle="{}" data-dowe-chart-hide-labels="{}" data-dowe-chart-hide-values="{}" data-dowe-chart-hide-percentages="{}" data-dowe-chart-show-glow="{}"{}{}"#,
        chart_common_attrs("pie", &props.common, context),
        props.donut,
        props.donut_width,
        props.start_angle,
        props.pad_angle,
        props.hide_labels,
        props.hide_values,
        props.hide_percentages,
        props.show_glow,
        optional_chart_attr("center-label", props.center_label.as_deref()),
        optional_chart_attr("center-value", props.center_value.as_deref()),
    );
    render_chart_html(
        "pie-chart-container",
        "Pie chart",
        &props.common,
        extra,
        context,
    )
}

fn render_chart_html(
    class_base: &str,
    label: &str,
    props: &ChartCommonProps,
    extra: String,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"<figure{}><div class="dowe-chart-viewport{}"><svg class="dowe-chart-svg" viewBox="0 0 600 300" preserveAspectRatio="{}" aria-hidden="true"></svg><div class="dowe-chart-tooltip" data-dowe-chart-tooltip hidden role="status"></div><div class="dowe-chart-loading">Loading</div><figcaption class="dowe-chart-empty">{}</figcaption></div><div class="dowe-chart-legend" data-dowe-chart-legend></div></figure>"#,
        attrs(
            chart_classes(class_base, props),
            Some(&props.style.element),
            Some(&format!(r#" role="figure" aria-label="{label}"{extra}"#)),
            context,
        ),
        if class_base == "arc-chart-container" {
            " dowe-chart-arc-viewport"
        } else {
            ""
        },
        if class_base == "arc-chart-container" {
            "xMidYMid meet"
        } else {
            "none"
        },
        escape_html(&props.empty_label)
    )
}

fn chart_common_attrs(
    chart_type: &str,
    props: &ChartCommonProps,
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = format!(
        r#" data-dowe-chart data-dowe-chart-type="{}" data-dowe-chart-size="{}" data-dowe-chart-palette="{}" data-dowe-chart-legend-position="{}" data-dowe-chart-empty-label="{}" data-dowe-chart-loading="{}" data-dowe-chart-hide-legend="{}""#,
        chart_type,
        props.size.as_str(),
        props.palette.as_str(),
        props.legend_position.as_str(),
        escape_attr(&props.empty_label),
        props.loading,
        props.hide_legend,
    );
    if let Some(data) = props.data.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-chart-data="{}""#,
            escape_attr(&context.signal_path(data))
        ));
    }
    if let Some(series) = props.series.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-chart-series="{}""#,
            escape_attr(&context.signal_path(series))
        ));
    }
    extra
}

fn optional_chart_attr(name: &str, value: Option<&str>) -> String {
    value
        .map(|value| format!(r#" data-dowe-chart-{name}="{}""#, escape_attr(value)))
        .unwrap_or_default()
}
