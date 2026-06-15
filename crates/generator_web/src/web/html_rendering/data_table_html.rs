fn render_svg_html(props: &SvgProps, paths: &[SvgPath], context: &ReactiveRenderContext) -> String {
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
        html.push_str(&format!(
            r#"<path d="{}" fill="{}"></path>"#,
            escape_attr(&path.data),
            escape_attr(&svg_path_fill(path.fill))
        ));
    }
    html.push_str("</svg>");
    html
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
    render_chart_html("arc-chart-container", "Arc chart", &props.common, extra, context)
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
    render_chart_html("area-chart-container", "Area chart", &props.common, extra, context)
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
    render_chart_html("bar-chart-container", "Bar chart", &props.common, extra, context)
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
    render_chart_html("line-chart-container", "Line chart", &props.common, extra, context)
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
    render_chart_html("pie-chart-container", "Pie chart", &props.common, extra, context)
}

fn render_chart_html(
    class_base: &str,
    label: &str,
    props: &ChartCommonProps,
    extra: String,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"<figure{}><div class="dowe-chart-viewport"><svg class="dowe-chart-svg" viewBox="0 0 600 300" preserveAspectRatio="none" aria-hidden="true"></svg><div class="dowe-chart-loading">Loading</div><figcaption class="dowe-chart-empty">{}</figcaption></div><div class="dowe-chart-legend" data-dowe-chart-legend></div></figure>"#,
        attrs(
            chart_classes(class_base, props),
            Some(&props.style.element),
            Some(&format!(r#" role="figure" aria-label="{label}"{extra}"#)),
            context,
        ),
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

fn render_table_html(props: &TableProps, context: &ReactiveRenderContext) -> String {
    let mut html = format!(
        "<div{}><div class=\"table-container\"><table{}{}><thead class=\"table-header\"><tr>",
        attrs(
            table_wrapper_classes(props),
            Some(&props.style.element),
            None,
            context,
        ),
        class_attr(table_classes(props)),
        table_attrs(props, context),
    );
    for column in &props.columns {
        html.push_str(&format!(
            r#"<th class="table-head" data-dowe-table-field="{}" data-dowe-table-align="{}"{}><div class="table-head-content"><span class="table-head-label">{}</span></div></th>"#,
            escape_attr(&column.field),
            column.align.as_str(),
            table_column_style(column),
            escape_html(&column.label)
        ));
    }
    html.push_str("</tr></thead><tbody class=\"table-body\">");
    html.push_str(&render_table_empty_row(props));
    html.push_str("</tbody></table></div></div>");
    html
}

fn render_tabs_html(
    props: &TabsProps,
    tabs: &[TabItem],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<div{} data-dowe-tabs>",
        attrs(
            tabs_classes(props),
            Some(&props.style.element),
            None,
            context
        )
    );
    html.push_str(&format!(
        r#"<div{} role="tablist">"#,
        class_attr(tabs_list_classes(props))
    ));
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tab".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        html.push_str(&format!(
            r#"<button{} type="button" role="tab" id="{}" aria-selected="{}" aria-controls="{}" tabindex="{}" data-dowe-tab="{}"><span class="tabs-label">{}</span></button>"#,
            class_attr(classes),
            escape_attr(&tab_button_id(tab)),
            if active { "true" } else { "false" },
            escape_attr(&tab_panel_id(tab)),
            if active { "0" } else { "-1" },
            escape_attr(&tab.id),
            escape_html(&tab.label)
        ));
    }
    html.push_str("</div><div class=\"tabs-wrapper\">");
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tabs-content".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        html.push_str(&format!(
            r#"<div{} id="{}" role="tabpanel" aria-labelledby="{}" data-dowe-tab-panel="{}"{}>"#,
            class_attr(classes),
            escape_attr(&tab_panel_id(tab)),
            escape_attr(&tab_button_id(tab)),
            escape_attr(&tab.id),
            if active { "" } else { " hidden" }
        ));
        for child in &tab.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("</div></div>");
    html
}

fn tab_button_id(tab: &TabItem) -> String {
    format!("tab-{}-button", tab.id)
}

fn tab_panel_id(tab: &TabItem) -> String {
    format!("tab-{}-panel", tab.id)
}

fn table_attrs(props: &TableProps, context: &ReactiveRenderContext) -> String {
    format!(
        r#" data-dowe-table data-dowe-table-data="{}" data-dowe-table-empty-title="{}" data-dowe-table-empty-description="{}""#,
        escape_attr(&context.signal_path(&props.data)),
        escape_attr(&props.empty_title),
        escape_attr(&props.empty_description)
    )
}

fn render_table_empty_row(props: &TableProps) -> String {
    format!(
        r#"<tr class="table-empty-row"><td class="table-empty-cell" colspan="{}"><div class="empty-state"><div class="empty-content"><h3 class="empty-title">{}</h3><p class="empty-description">{}</p></div></div></td></tr>"#,
        props.columns.len(),
        escape_html(&props.empty_title),
        escape_html(&props.empty_description)
    )
}

fn table_column_style(column: &TableColumn) -> String {
    let mut styles = vec![format!("text-align:{}", table_align_css(column.align))];
    if let Some(width) = column.width.as_deref() {
        styles.push(format!("width:{}", escape_attr(width)));
    }
    format!(r#" style="{}""#, styles.join(";"))
}

fn table_align_css(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => "start",
        TableColumnAlign::Center => "center",
        TableColumnAlign::End => "end",
    }
}
