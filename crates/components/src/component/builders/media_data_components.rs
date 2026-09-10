pub fn candlestick_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut data = None;
    let mut stream = None;
    let mut up_color = ColorToken::Success;
    let mut down_color = ColorToken::Danger;
    let mut empty_label = "No candle data".to_string();
    let mut max_points = 240;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "data" => data = Some(parse_reference_path(&prop.name, &prop.value)?),
            "stream" => stream = Some(parse_candlestick_stream(&prop.name, &prop.value)?),
            "upColor" => up_color = parse_candlestick_color(&prop.name, &prop.value)?,
            "downColor" => down_color = parse_candlestick_color(&prop.name, &prop.value)?,
            "emptyLabel" => empty_label = parse_required_string(&prop.name, &prop.value)?,
            "maxPoints" => max_points = parse_positive_u16(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Candlestick, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    if style.style.sizing.h.is_none() {
        style.style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Scale(
            ScaleValue::from_half_steps(128),
        )));
    }
    Ok(ViewNode::Candlestick {
        props: CandlestickProps {
            style,
            data: data.ok_or_else(|| ComponentError::invalid_prop("data", "signal array path"))?,
            stream,
            up_color,
            down_color,
            empty_label,
            max_points,
        },
    })
}

pub fn table_node(
    props: Vec<ComponentProp>,
    columns: Vec<TableColumn>,
) -> ComponentResult<ViewNode> {
    if columns.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Table requires at least one column",
        ));
    }
    let mut data = None;
    let mut size = TableSize::Md;
    let mut striped = false;
    let mut bordered = false;
    let mut dividers = true;
    let mut empty_title = "No data".to_string();
    let mut empty_description = "There are no records to display".to_string();
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "data" => data = Some(parse_reference_path(&prop.name, &prop.value)?),
            "size" => size = parse_table_size_prop(&prop.name, &prop.value)?,
            "striped" => striped = parse_static_bool(&prop.name, &prop.value)?,
            "bordered" => bordered = parse_static_bool(&prop.name, &prop.value)?,
            "dividers" => dividers = parse_static_bool(&prop.name, &prop.value)?,
            "emptyTitle" => empty_title = parse_required_string(&prop.name, &prop.value)?,
            "emptyDescription" => {
                empty_description = parse_required_string(&prop.name, &prop.value)?
            }
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Table`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Table, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Table {
        props: TableProps {
            style,
            data: data.ok_or_else(|| ComponentError::invalid_prop("data", "signal array path"))?,
            columns,
            size,
            striped,
            bordered,
            dividers,
            empty_title,
            empty_description,
        },
    })
}

pub fn table_column_component(props: Vec<ComponentProp>) -> ComponentResult<TableColumn> {
    let mut field = None;
    let mut label = None;
    let mut align = TableColumnAlign::Start;
    let mut width = None;
    for prop in props {
        match prop.name.as_str() {
            "field" => field = Some(parse_table_field(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "align" => align = parse_table_column_align_prop(&prop.name, &prop.value)?,
            "width" => width = Some(parse_table_column_width(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Table,
                    &prop.name,
                ));
            }
        }
    }
    Ok(TableColumn {
        field: field.ok_or_else(|| ComponentError::invalid_prop("field", "relative field path"))?,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        align,
        width,
    })
}

pub fn divider_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut orientation = DividerOrientation::Horizontal;
    let mut color = ColorFamily::Muted;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "orientation" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                orientation = DividerOrientation::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("orientation", "horizontal or vertical")
                })?;
            }
            "scheme" => {
                color = parse_family_prop(BuiltinComponent::Divider, &prop.name, &prop.value)?
            }
            _ => style_props.push(prop),
        }
    }
    let style = parse_style_props(
        BuiltinComponent::Divider,
        &style_props,
        StylePropMode::Variant,
    )?;
    Ok(ViewNode::Divider {
        props: DividerProps {
            style,
            orientation,
            color,
        },
    })
}
