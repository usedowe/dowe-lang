#[derive(Debug, Clone, Copy)]
enum ChartDataShape {
    Category,
    Point,
    CategorySeries,
    PointSeries,
}

fn validate_chart_data(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    source: &str,
    prop: &str,
    component: &str,
    shape: ChartDataShape,
) -> DoweResult<()> {
    let root = path_root(source);
    let Some(collection_type) = signals.get(root) else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal `{root}` in `{prop}`"),
        ));
    };
    let ViewSignalValue::Array(items) = collection_type else {
        return Err(DoweError::at_path(
            path,
            format!("signal `{root}` in `{prop}` must be an array"),
        ));
    };
    for item in items {
        match shape {
            ChartDataShape::Category => validate_chart_category_item(path, item, component)?,
            ChartDataShape::Point => validate_chart_point_item(path, item, component)?,
            ChartDataShape::CategorySeries => {
                validate_chart_series_item(path, item, component, ChartDataShape::Category)?
            }
            ChartDataShape::PointSeries => {
                validate_chart_series_item(path, item, component, ChartDataShape::Point)?
            }
        }
    }
    Ok(())
}

fn validate_chart_series_item(
    path: &Path,
    item: &ViewSignalValue,
    component: &str,
    shape: ChartDataShape,
) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            format!("{component} series items must be objects"),
        ));
    };
    let name = chart_field(path, fields, component, "series", "name")?;
    if !matches!(
        name,
        ViewSignalValue::String(_) | ViewSignalValue::Number(_)
    ) {
        return Err(DoweError::at_path(
            path,
            format!("{component} series field `name` must be string or number"),
        ));
    }
    if let Some((_, color)) = fields.iter().find(|(field, _)| field == "color") {
        validate_chart_color(path, color, component)?;
    }
    let data = chart_field(path, fields, component, "series", "data")?;
    let ViewSignalValue::Array(items) = data else {
        return Err(DoweError::at_path(
            path,
            format!("{component} series field `data` must be an array"),
        ));
    };
    for item in items {
        match shape {
            ChartDataShape::Category => validate_chart_category_item(path, item, component)?,
            ChartDataShape::Point => validate_chart_point_item(path, item, component)?,
            ChartDataShape::CategorySeries | ChartDataShape::PointSeries => {}
        }
    }
    Ok(())
}

fn validate_chart_category_item(
    path: &Path,
    item: &ViewSignalValue,
    component: &str,
) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            format!("{component} data items must be objects"),
        ));
    };
    let label = chart_field(path, fields, component, "data item", "label")?;
    if !matches!(
        label,
        ViewSignalValue::String(_) | ViewSignalValue::Number(_)
    ) {
        return Err(DoweError::at_path(
            path,
            format!("{component} data item field `label` must be string or number"),
        ));
    }
    let value = chart_number_field(path, fields, component, "data item", "value")?;
    if value < 0.0 {
        return Err(DoweError::at_path(
            path,
            format!("{component} data item field `value` must be non-negative"),
        ));
    }
    if let Some((_, max)) = fields.iter().find(|(field, _)| field == "max") {
        let ViewSignalValue::Number(value) = max else {
            return Err(DoweError::at_path(
                path,
                format!("{component} data item field `max` must be number"),
            ));
        };
        let parsed = value.parse::<f64>().map_err(|_| {
            DoweError::at_path(
                path,
                format!("{component} data item field `max` must be number"),
            )
        })?;
        if parsed <= 0.0 || !parsed.is_finite() {
            return Err(DoweError::at_path(
                path,
                format!("{component} data item field `max` must be positive"),
            ));
        }
    }
    if let Some((_, color)) = fields.iter().find(|(field, _)| field == "color") {
        validate_chart_color(path, color, component)?;
    }
    Ok(())
}

fn validate_chart_point_item(
    path: &Path,
    item: &ViewSignalValue,
    component: &str,
) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            format!("{component} data items must be objects"),
        ));
    };
    chart_number_field(path, fields, component, "data item", "x")?;
    chart_number_field(path, fields, component, "data item", "y")?;
    Ok(())
}

fn validate_chart_color(path: &Path, value: &ViewSignalValue, component: &str) -> DoweResult<()> {
    let ViewSignalValue::String(value) = value else {
        return Err(DoweError::at_path(
            path,
            format!("{component} color fields must be strings"),
        ));
    };
    if dowe_components::ColorToken::from_name(value).is_some() {
        Ok(())
    } else {
        Err(DoweError::at_path(
            path,
            format!("{component} color fields must be Dowe color tokens"),
        ))
    }
}

fn chart_field<'a>(
    path: &Path,
    fields: &'a [(String, ViewSignalValue)],
    component: &str,
    item_name: &str,
    name: &str,
) -> DoweResult<&'a ViewSignalValue> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            DoweError::at_path(
                path,
                format!("{component} {item_name} must include `{name}`"),
            )
        })
}

fn chart_number_field(
    path: &Path,
    fields: &[(String, ViewSignalValue)],
    component: &str,
    item_name: &str,
    name: &str,
) -> DoweResult<f64> {
    let ViewSignalValue::Number(value) = chart_field(path, fields, component, item_name, name)?
    else {
        return Err(DoweError::at_path(
            path,
            format!("{component} {item_name} field `{name}` must be number"),
        ));
    };
    let parsed = value.parse::<f64>().map_err(|_| {
        DoweError::at_path(
            path,
            format!("{component} {item_name} field `{name}` must be number"),
        )
    })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(DoweError::at_path(
            path,
            format!("{component} {item_name} field `{name}` must be number"),
        ))
    }
}

fn validate_table_data(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    data: &str,
    columns: &[dowe_components::TableColumn],
) -> DoweResult<()> {
    let root = path_root(data);
    let Some(collection_type) = signals.get(root) else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal `{root}` in `data`"),
        ));
    };
    let ViewSignalValue::Array(items) = collection_type else {
        return Err(DoweError::at_path(
            path,
            format!("signal `{root}` in `data` must be an array"),
        ));
    };
    for item in items {
        validate_table_item(path, item, columns)?;
    }
    Ok(())
}

fn validate_table_item(
    path: &Path,
    item: &ViewSignalValue,
    columns: &[dowe_components::TableColumn],
) -> DoweResult<()> {
    for column in columns {
        let value = table_field_value(path, item, &column.field)?;
        if matches!(
            value,
            ViewSignalValue::Array(_) | ViewSignalValue::Object(_)
        ) {
            return Err(DoweError::at_path(
                path,
                format!(
                    "Table column field `{}` must resolve to string, number or bool",
                    column.field
                ),
            ));
        }
    }
    Ok(())
}

fn table_field_value<'a>(
    path: &Path,
    item: &'a ViewSignalValue,
    field: &str,
) -> DoweResult<&'a ViewSignalValue> {
    let mut current = item;
    for segment in field.split('.') {
        let ViewSignalValue::Object(fields) = current else {
            return Err(DoweError::at_path(
                path,
                format!("unknown Table column field `{field}`"),
            ));
        };
        let Some((_, next)) = fields.iter().find(|(name, _)| name == segment) else {
            return Err(DoweError::at_path(
                path,
                format!("unknown Table column field `{field}`"),
            ));
        };
        current = next;
    }
    Ok(current)
}

fn validate_candlestick_item(path: &Path, item: &ViewSignalValue) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            "Candlestick data items must be objects",
        ));
    };
    let time = candlestick_field(path, fields, "time")?;
    if !matches!(
        time,
        ViewSignalValue::String(_) | ViewSignalValue::Number(_)
    ) {
        return Err(DoweError::at_path(
            path,
            "Candlestick data item field `time` must be string or number",
        ));
    }
    let open = candlestick_number_field(path, fields, "open")?;
    let high = candlestick_number_field(path, fields, "high")?;
    let low = candlestick_number_field(path, fields, "low")?;
    let close = candlestick_number_field(path, fields, "close")?;
    if high < open.max(close) || low > open.min(close) {
        return Err(DoweError::at_path(
            path,
            "Candlestick data item violates OHLC bounds",
        ));
    }
    Ok(())
}

fn candlestick_field<'a>(
    path: &Path,
    fields: &'a [(String, ViewSignalValue)],
    name: &str,
) -> DoweResult<&'a ViewSignalValue> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            DoweError::at_path(path, format!("Candlestick data item must include `{name}`"))
        })
}

fn candlestick_number_field(
    path: &Path,
    fields: &[(String, ViewSignalValue)],
    name: &str,
) -> DoweResult<f64> {
    let ViewSignalValue::Number(value) = candlestick_field(path, fields, name)? else {
        return Err(DoweError::at_path(
            path,
            format!("Candlestick data item field `{name}` must be number"),
        ));
    };
    value.parse::<f64>().map_err(|_| {
        DoweError::at_path(
            path,
            format!("Candlestick data item field `{name}` must be number"),
        )
    })
}
