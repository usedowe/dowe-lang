fn compose_optional_scale(value: Option<&ResponsiveValue<ScaleValue>>) -> String {
    value
        .map(compose_scale_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_content_padding(spacing: &dowe_components::SpacingProps) -> String {
    if spacing.p.is_none()
        && spacing.px.is_none()
        && spacing.py.is_none()
        && spacing.pl.is_none()
        && spacing.pr.is_none()
        && spacing.pt.is_none()
        && spacing.pb.is_none()
    {
        return "PaddingValues(0.dp)".to_string();
    }
    let edge = |side: Option<&ResponsiveValue<ScaleValue>>,
                axis: Option<&ResponsiveValue<ScaleValue>>| {
        [side, axis, spacing.p.as_ref()]
            .into_iter()
            .flatten()
            .map(compose_scale_value)
            .chain(std::iter::once("0.dp".to_string()))
            .collect::<Vec<_>>()
            .join(" ?: ")
    };
    format!(
        "PaddingValues(start = {}, top = {}, end = {}, bottom = {})",
        edge(spacing.pl.as_ref(), spacing.px.as_ref()),
        edge(spacing.pt.as_ref(), spacing.py.as_ref()),
        edge(spacing.pr.as_ref(), spacing.px.as_ref()),
        edge(spacing.pb.as_ref(), spacing.py.as_ref())
    )
}

fn compose_optional_rounded(value: Option<&ResponsiveValue<RoundedSize>>) -> String {
    value
        .map(compose_rounded_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_justify(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(compose_justify_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_align(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(compose_align_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_grid_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    value
        .map(compose_grid_alignment_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(compose_gap_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_scale_value(value: &ResponsiveValue<ScaleValue>) -> String {
    compose_responsive_value(value, |value| format!("{}.dp", value.native_units()))
}

fn compose_gap_value(value: &ResponsiveValue<GapValue>) -> String {
    compose_responsive_value(value, compose_gap_expr)
}

fn compose_size_value(value: &ResponsiveValue<SizeValue>) -> String {
    compose_responsive_value(value, |value| match value {
        SizeValue::Scale(value) => format!("DoweSize.Fixed({}.dp)", value.native_units()),
        SizeValue::Container(value) => {
            format!("DoweSize.Fixed({}.dp)", value.scale_value().native_units())
        }
        SizeValue::Percent(value) => {
            format!("DoweSize.Percent({}f)", f32::from(*value) / 100.0)
        }
        SizeValue::Full => "DoweSize.Full".to_string(),
        SizeValue::Auto => "DoweSize.Auto".to_string(),
        SizeValue::ViewportMinus(value) => {
            format!("DoweSize.ViewportMinus({}.dp)", value.native_units())
        }
    })
}

fn compose_color_value(value: &ResponsiveValue<ColorToken>) -> String {
    compose_responsive_value(value, |value| color_ref(*value).to_string())
}

fn compose_bool_value(value: &ResponsiveValue<bool>) -> String {
    compose_responsive_value(value, |value| value.to_string())
}

