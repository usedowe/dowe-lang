fn parse_layout_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<LayoutProps> {
    let mut layout = LayoutProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "direction" => layout.direction = parse_flex_direction_prop(&prop.name, &prop.value)?,
            "wrap" => layout.wrap = parse_static_bool(&prop.name, &prop.value)?,
            "justify" => layout.justify = Some(parse_justify_prop(&prop.name, &prop.value)?),
            "align" => layout.align = Some(parse_align_prop(&prop.name, &prop.value)?),
            "gap" => layout.gap = Some(parse_gap_prop(&prop.name, &prop.value, false)?),
            _ => style_props.push(prop.clone()),
        }
    }

    layout.style = parse_style_props(component, &style_props, StylePropMode::Layout)?;
    Ok(layout)
}

fn parse_grid_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<GridProps> {
    let mut grid = GridProps {
        columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
        rows: Some(ResponsiveValue::scalar(GridTracks::Auto)),
        justify: Some(ResponsiveValue::scalar(GridAlignment::Stretch)),
        align: Some(ResponsiveValue::scalar(GridAlignment::Stretch)),
        gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
            ScaleValue::from_half_steps(0),
        )))),
        ..GridProps::default()
    };
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "columns" => {
                grid.columns = Some(parse_grid_tracks_prop(
                    &prop.name,
                    &prop.value,
                    false,
                    Some(12),
                )?)
            }
            "rows" => {
                grid.rows = Some(parse_grid_tracks_prop(&prop.name, &prop.value, true, None)?)
            }
            "justify" => grid.justify = Some(parse_grid_alignment_prop(&prop.name, &prop.value)?),
            "align" => grid.align = Some(parse_grid_alignment_prop(&prop.name, &prop.value)?),
            "gap" => grid.gap = Some(parse_gap_prop(&prop.name, &prop.value, true)?),
            _ => style_props.push(prop.clone()),
        }
    }

    grid.style = parse_style_props(component, &style_props, StylePropMode::Grid)?;
    if grid.style.sizing.w.is_none() {
        grid.style.sizing.w = Some(ResponsiveValue::scalar(SizeValue::Full));
    }
    Ok(grid)
}

