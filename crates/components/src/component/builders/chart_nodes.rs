fn parse_chart_common(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    allow_series: bool,
    default_legend_position: ChartLegendPosition,
    circular: bool,
) -> ComponentResult<(ChartCommonProps, Vec<ComponentProp>)> {
    let mut data = None;
    let mut series = None;
    let mut size = ChartSize::Md;
    let mut palette = ChartPalette::Default;
    let mut legend_position = default_legend_position;
    let mut empty_label = "No data available".to_string();
    let mut loading = false;
    let mut hide_legend = false;
    let mut style_props = Vec::new();
    let mut chart_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "data" => data = Some(parse_reference_path(&prop.name, &prop.value)?),
            "series" if allow_series => {
                series = Some(parse_reference_path(&prop.name, &prop.value)?)
            }
            "size" => size = parse_chart_size(&prop.name, &prop.value)?,
            "palette" => palette = parse_chart_palette(&prop.name, &prop.value)?,
            "legendPosition" => {
                legend_position = parse_chart_legend_position(&prop.name, &prop.value)?
            }
            "emptyLabel" => empty_label = parse_required_string(&prop.name, &prop.value)?,
            "loading" => loading = parse_static_bool(&prop.name, &prop.value)?,
            "hideLegend" => hide_legend = parse_static_bool(&prop.name, &prop.value)?,
            _ if is_chart_style_prop(&prop.name) => style_props.push(prop),
            _ => chart_props.push(prop),
        }
    }

    if data.is_none() && series.is_none() {
        return Err(ComponentError::invalid_prop(
            if allow_series { "data" } else { "data" },
            if allow_series {
                "signal array path or series signal array path"
            } else {
                "signal array path"
            },
        ));
    }

    let mut style = parse_variant_props(component, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    if style.style.sizing.h.is_none() {
        let height = if circular {
            size.circular_height()
        } else {
            size.cartesian_height()
        };
        style.style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Scale(height)));
    }

    Ok((
        ChartCommonProps {
            style,
            data,
            series,
            size,
            palette,
            legend_position,
            empty_label,
            loading,
            hide_legend,
        },
        chart_props,
    ))
}

pub fn arc_chart_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let (common, chart_props) = parse_chart_common(
        BuiltinComponent::ArcChart,
        props,
        false,
        ChartLegendPosition::Right,
        true,
    )?;
    let mut center_text = None;
    let mut center_value = None;
    let mut thickness = 16;
    let mut gap = 8;
    let mut start_angle = -90;
    let mut end_angle = 270;
    let mut show_inline_labels = false;
    let mut hide_values = false;
    let mut show_glow = false;
    for prop in chart_props {
        match prop.name.as_str() {
            "centerText" => center_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "centerValue" => center_value = Some(parse_static_string(&prop.name, &prop.value)?),
            "thickness" => thickness = parse_positive_u16(&prop.name, &prop.value)?,
            "gap" => gap = parse_positive_u16(&prop.name, &prop.value)?,
            "startAngle" => start_angle = parse_chart_angle(&prop.name, &prop.value)?,
            "endAngle" => end_angle = parse_chart_angle(&prop.name, &prop.value)?,
            "showInlineLabels" => {
                show_inline_labels = parse_static_bool(&prop.name, &prop.value)?
            }
            "hideValues" => hide_values = parse_static_bool(&prop.name, &prop.value)?,
            "showGlow" => show_glow = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::ArcChart, &prop.name)),
        }
    }
    Ok(ViewNode::ArcChart {
        props: ArcChartProps {
            common,
            center_text,
            center_value,
            thickness,
            gap,
            start_angle,
            end_angle,
            show_inline_labels,
            hide_values,
            show_glow,
        },
    })
}

pub fn area_chart_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let (common, chart_props) = parse_chart_common(
        BuiltinComponent::AreaChart,
        props,
        true,
        ChartLegendPosition::Bottom,
        false,
    )?;
    let mut curve = ChartCurve::Linear;
    let mut stroke_width = 2;
    let mut fill_opacity = 30;
    let mut stacked = false;
    let mut hide_line = false;
    let mut show_points = false;
    let mut hide_grid = false;
    let mut hide_x_axis = false;
    let mut hide_y_axis = false;
    let mut show_glow = false;
    for prop in chart_props {
        match prop.name.as_str() {
            "curve" => curve = parse_chart_curve(&prop.name, &prop.value)?,
            "strokeWidth" => stroke_width = parse_positive_u16(&prop.name, &prop.value)?,
            "fillOpacity" => fill_opacity = parse_chart_opacity(&prop.name, &prop.value)?,
            "stacked" => stacked = parse_static_bool(&prop.name, &prop.value)?,
            "hideLine" => hide_line = parse_static_bool(&prop.name, &prop.value)?,
            "showPoints" => show_points = parse_static_bool(&prop.name, &prop.value)?,
            "hideGrid" => hide_grid = parse_static_bool(&prop.name, &prop.value)?,
            "hideXAxis" => hide_x_axis = parse_static_bool(&prop.name, &prop.value)?,
            "hideYAxis" => hide_y_axis = parse_static_bool(&prop.name, &prop.value)?,
            "showGlow" => show_glow = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::AreaChart, &prop.name)),
        }
    }
    Ok(ViewNode::AreaChart {
        props: AreaChartProps {
            common,
            curve,
            stroke_width,
            fill_opacity,
            stacked,
            hide_line,
            show_points,
            hide_grid,
            hide_x_axis,
            hide_y_axis,
            show_glow,
        },
    })
}

pub fn bar_chart_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let (common, chart_props) = parse_chart_common(
        BuiltinComponent::BarChart,
        props,
        true,
        ChartLegendPosition::Bottom,
        false,
    )?;
    let mut grouped = false;
    let mut stacked = false;
    let mut show_values = false;
    let mut bar_radius = 4;
    let mut hide_grid = false;
    let mut show_glow = false;
    for prop in chart_props {
        match prop.name.as_str() {
            "grouped" => grouped = parse_static_bool(&prop.name, &prop.value)?,
            "stacked" => stacked = parse_static_bool(&prop.name, &prop.value)?,
            "showValues" => show_values = parse_static_bool(&prop.name, &prop.value)?,
            "barRadius" => bar_radius = parse_positive_u16(&prop.name, &prop.value)?,
            "hideGrid" => hide_grid = parse_static_bool(&prop.name, &prop.value)?,
            "showGlow" => show_glow = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::BarChart, &prop.name)),
        }
    }
    Ok(ViewNode::BarChart {
        props: BarChartProps {
            common,
            grouped,
            stacked,
            show_values,
            bar_radius,
            hide_grid,
            show_glow,
        },
    })
}

pub fn line_chart_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let (common, chart_props) = parse_chart_common(
        BuiltinComponent::LineChart,
        props,
        true,
        ChartLegendPosition::Bottom,
        false,
    )?;
    let mut curve = ChartCurve::Linear;
    let mut stroke_width = 2;
    let mut point_radius = 3;
    let mut hide_points = false;
    let mut hide_grid = false;
    let mut hide_x_axis = false;
    let mut hide_y_axis = false;
    let mut show_gradient_fill = false;
    let mut show_glow = false;
    for prop in chart_props {
        match prop.name.as_str() {
            "curve" => curve = parse_chart_curve(&prop.name, &prop.value)?,
            "strokeWidth" => stroke_width = parse_positive_u16(&prop.name, &prop.value)?,
            "pointRadius" => point_radius = parse_positive_u16(&prop.name, &prop.value)?,
            "hidePoints" => hide_points = parse_static_bool(&prop.name, &prop.value)?,
            "hideGrid" => hide_grid = parse_static_bool(&prop.name, &prop.value)?,
            "hideXAxis" => hide_x_axis = parse_static_bool(&prop.name, &prop.value)?,
            "hideYAxis" => hide_y_axis = parse_static_bool(&prop.name, &prop.value)?,
            "showGradientFill" => {
                show_gradient_fill = parse_static_bool(&prop.name, &prop.value)?
            }
            "showGlow" => show_glow = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::LineChart, &prop.name)),
        }
    }
    Ok(ViewNode::LineChart {
        props: LineChartProps {
            common,
            curve,
            stroke_width,
            point_radius,
            hide_points,
            hide_grid,
            hide_x_axis,
            hide_y_axis,
            show_gradient_fill,
            show_glow,
        },
    })
}

pub fn pie_chart_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let (common, chart_props) = parse_chart_common(
        BuiltinComponent::PieChart,
        props,
        false,
        ChartLegendPosition::Right,
        true,
    )?;
    let mut donut = false;
    let mut donut_width = 60;
    let mut center_label = Some("Total".to_string());
    let mut center_value = None;
    let mut start_angle = -90;
    let mut pad_angle = 0;
    let mut hide_labels = false;
    let mut hide_values = false;
    let mut hide_percentages = false;
    let mut show_glow = false;
    for prop in chart_props {
        match prop.name.as_str() {
            "donut" => donut = parse_static_bool(&prop.name, &prop.value)?,
            "donutWidth" => donut_width = parse_positive_u16(&prop.name, &prop.value)?,
            "centerLabel" => center_label = Some(parse_required_string(&prop.name, &prop.value)?),
            "centerValue" => center_value = Some(parse_static_string(&prop.name, &prop.value)?),
            "startAngle" => start_angle = parse_chart_angle(&prop.name, &prop.value)?,
            "padAngle" => pad_angle = parse_positive_u16(&prop.name, &prop.value)?,
            "hideLabels" => hide_labels = parse_static_bool(&prop.name, &prop.value)?,
            "hideValues" => hide_values = parse_static_bool(&prop.name, &prop.value)?,
            "hidePercentages" => hide_percentages = parse_static_bool(&prop.name, &prop.value)?,
            "showGlow" => show_glow = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::PieChart, &prop.name)),
        }
    }
    Ok(ViewNode::PieChart {
        props: PieChartProps {
            common,
            donut,
            donut_width,
            center_label,
            center_value,
            start_angle,
            pad_angle,
            hide_labels,
            hide_values,
            hide_percentages,
            show_glow,
        },
    })
}

fn parse_chart_size(name: &str, value: &PropValue) -> ComponentResult<ChartSize> {
    let value = parse_required_string(name, value)?;
    ChartSize::from_name(&value).ok_or_else(|| ComponentError::invalid_prop(name, "sm, md, lg or xl"))
}

fn parse_chart_palette(name: &str, value: &PropValue) -> ComponentResult<ChartPalette> {
    let value = parse_required_string(name, value)?;
    ChartPalette::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(name, "default, rainbow, ocean, sunset, forest or neon")
    })
}

fn parse_chart_legend_position(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ChartLegendPosition> {
    let value = parse_required_string(name, value)?;
    ChartLegendPosition::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "top, right, bottom, left or none"))
}

fn parse_chart_curve(name: &str, value: &PropValue) -> ComponentResult<ChartCurve> {
    let value = parse_required_string(name, value)?;
    ChartCurve::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "linear or smooth"))
}

fn parse_chart_angle(name: &str, value: &PropValue) -> ComponentResult<i16> {
    match value {
        PropValue::Number(value) => value
            .parse::<i16>()
            .map_err(|_| ComponentError::invalid_prop(name, "integer degrees")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "integer degrees"))
        }
    }
}

fn parse_chart_opacity(name: &str, value: &PropValue) -> ComponentResult<u16> {
    let value = match value {
        PropValue::Number(value) => value,
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            return Err(ComponentError::invalid_prop(name, "number between 0 and 1"));
        }
    };
    let opacity = value
        .parse::<f32>()
        .ok()
        .filter(|value| *value >= 0.0 && *value <= 1.0)
        .ok_or_else(|| ComponentError::invalid_prop(name, "number between 0 and 1"))?;
    Ok((opacity * 100.0).round() as u16)
}

fn is_chart_style_prop(name: &str) -> bool {
    matches!(
        name,
        "id"
            | "font"
            | "show"
            | "variant"
            | "scheme"
            | "bg"
            | "color"
            | "p"
            | "px"
            | "py"
            | "pl"
            | "pr"
            | "pt"
            | "pb"
            | "w"
            | "h"
            | "minW"
            | "minH"
            | "rounded"
            | "border"
            | "colSpan"
            | "rowSpan"
    )
}
