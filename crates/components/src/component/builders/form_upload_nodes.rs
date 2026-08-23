pub fn csv_field_component_node(
    props: Vec<ComponentProp>,
    columns: Vec<CsvColumn>,
) -> ComponentResult<ViewNode> {
    let mut button_text = "Select CSV file".to_string();
    let mut modal_title = "Map CSV Columns".to_string();
    let mut instructions =
        "Map each required column to the corresponding column from the CSV file:".to_string();
    let mut cancel_text = "Cancel".to_string();
    let mut confirm_text = "Confirm mapping".to_string();
    let mut clear_text = "Clear".to_string();
    let mut preview_title = "Imported Data".to_string();
    let mut multiple = false;
    let mut show_preview = true;
    let mut preview_rows = 5;
    let mut preview_page_size = 10;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "buttonText" => button_text = parse_required_string(&prop.name, &prop.value)?,
            "modalTitle" => modal_title = parse_required_string(&prop.name, &prop.value)?,
            "instructions" => instructions = parse_required_string(&prop.name, &prop.value)?,
            "cancelText" => cancel_text = parse_required_string(&prop.name, &prop.value)?,
            "confirmText" => confirm_text = parse_required_string(&prop.name, &prop.value)?,
            "clearText" => clear_text = parse_required_string(&prop.name, &prop.value)?,
            "previewTitle" => preview_title = parse_required_string(&prop.name, &prop.value)?,
            "multiple" => multiple = parse_static_bool(&prop.name, &prop.value)?,
            "showPreview" => show_preview = parse_static_bool(&prop.name, &prop.value)?,
            "previewRows" => preview_rows = parse_u16_in_range(&prop.name, &prop.value, 1, 100)?,
            "previewPageSize" => {
                preview_page_size = parse_u16_in_range(&prop.name, &prop.value, 1, 500)?
            }
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::CsvField)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::CsvField, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    Ok(ViewNode::CsvField {
        props: CsvFieldProps {
            style,
            button_text,
            modal_title,
            instructions,
            cancel_text,
            confirm_text,
            clear_text,
            preview_title,
            multiple,
            show_preview,
            preview_rows,
            preview_page_size,
            error_text,
        },
        columns,
    })
}

pub fn csv_column_component(props: Vec<ComponentProp>) -> ComponentResult<CsvColumn> {
    let mut name = None;
    let mut label = None;
    for prop in props {
        match prop.name.as_str() {
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::CsvColumn,
                    &prop.name,
                ));
            }
        }
    }
    Ok(CsvColumn {
        name: name.ok_or_else(|| ComponentError::invalid_prop("name", "non-empty string"))?,
        label,
    })
}

pub fn drag_drop_component_node(
    props: Vec<ComponentProp>,
    items: Vec<DragItem>,
    groups: Vec<DragGroup>,
) -> ComponentResult<ViewNode> {
    if !items.is_empty() && !groups.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "DragDrop cannot mix dragItem and dragGroup children",
        ));
    }
    let mut seen = BTreeSet::new();
    for item in items
        .iter()
        .chain(groups.iter().flat_map(|group| group.items.iter()))
    {
        if !seen.insert(item.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate DragDrop item id `{}`",
                item.id
            )));
        }
    }
    let mut empty_text = "Drop items here".to_string();
    let mut direction = DragDropDirection::Vertical;
    let mut allow_group_transfer = true;
    let mut disabled = false;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "emptyText" => empty_text = parse_required_string(&prop.name, &prop.value)?,
            "direction" => direction = parse_drag_drop_direction(&prop.name, &prop.value)?,
            "allowGroupTransfer" => {
                allow_group_transfer = parse_static_bool(&prop.name, &prop.value)?
            }
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::DragDrop)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::DragDrop, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::DragDrop {
        props: DragDropProps {
            style,
            empty_text,
            direction,
            allow_group_transfer,
            disabled,
            size,
        },
        items,
        groups,
    })
}

pub fn drag_group_component(
    props: Vec<ComponentProp>,
    items: Vec<DragItem>,
) -> ComponentResult<DragGroup> {
    let mut id = None;
    let mut title = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_required_string(&prop.name, &prop.value)?),
            "title" => title = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::DragGroup,
                    &prop.name,
                ));
            }
        }
    }
    Ok(DragGroup {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "non-empty string"))?,
        title,
        items,
    })
}

pub fn drag_item_component(props: Vec<ComponentProp>) -> ComponentResult<DragItem> {
    let mut id = None;
    let mut label = None;
    let mut description = None;
    let mut disabled = false;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::DragItem,
                    &prop.name,
                ));
            }
        }
    }
    Ok(DragItem {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "non-empty string"))?,
        label,
        description,
        disabled,
    })
}

