pub fn record_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut name = None;
    let mut url = None;
    let mut disabled = false;
    let mut max_duration = None;
    let mut on_start = None;
    let mut on_pause = None;
    let mut on_resume = None;
    let mut on_stop = None;
    let mut on_discard = None;
    let mut on_confirm = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "url" => url = Some(parse_media_source(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "maxDuration" => max_duration = Some(parse_positive_u16(&prop.name, &prop.value)?),
            "onStart" => on_start = Some(parse_required_string(&prop.name, &prop.value)?),
            "onPause" => on_pause = Some(parse_required_string(&prop.name, &prop.value)?),
            "onResume" => on_resume = Some(parse_required_string(&prop.name, &prop.value)?),
            "onStop" => on_stop = Some(parse_required_string(&prop.name, &prop.value)?),
            "onDiscard" => on_discard = Some(parse_required_string(&prop.name, &prop.value)?),
            "onConfirm" => on_confirm = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Record)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Record, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Record, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Record {
        props: RecordProps {
            style,
            name: name.ok_or_else(|| ComponentError::invalid_prop("name", "non-empty string"))?,
            url,
            disabled,
            max_duration,
            on_start,
            on_pause,
            on_resume,
            on_stop,
            on_discard,
            on_confirm,
        },
    })
}

pub fn toggle_group_item_component(props: Vec<ComponentProp>) -> ComponentResult<ToggleGroupItem> {
    let mut id = None;
    let mut label = None;
    let mut icon = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "icon" => icon = Some(parse_view_icon(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::ToggleGroup,
                    &prop.name,
                ));
            }
        }
    }
    Ok(ToggleGroupItem {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "static string or number"))?,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        icon,
    })
}

pub fn toggle_group_component_node(
    props: Vec<ComponentProp>,
    items: Vec<ToggleGroupItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "ToggleGroup requires at least one item",
        ));
    }
    let mut seen = BTreeSet::new();
    for item in &items {
        if !seen.insert(item.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate ToggleGroup item id `{}`",
                item.id
            )));
        }
    }
    let mut value = None;
    let mut selected = None;
    let mut size = ButtonSize::Md;
    let mut wide = false;
    let mut vertical = false;
    let mut disabled = false;
    let mut aria_label = None;
    let mut on_change = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => {
                value = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal string path",
                )?)
            }
            "selected" => selected = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "wide" => wide = parse_static_bool(&prop.name, &prop.value)?,
            "vertical" => vertical = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "ariaLabel" => aria_label = Some(parse_required_string(&prop.name, &prop.value)?),
            "onChange" => on_change = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::ToggleGroup)),
            _ => style_props.push(prop),
        }
    }
    let selected = selected.unwrap_or_else(|| items[0].id.clone());
    if !items.iter().any(|item| item.id == selected) {
        return Err(ComponentError::invalid_prop_combination(format!(
            "ToggleGroup selected value `{}` must match an item id",
            selected
        )));
    }
    let mut style = parse_variant_props(BuiltinComponent::ToggleGroup, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::ToggleGroup {
        props: ToggleGroupProps {
            style,
            value,
            selected,
            size,
            wide,
            vertical,
            disabled,
            aria_label,
            on_change,
        },
        items,
    })
}

pub fn collapsible_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    reject_children_placeholder(BuiltinComponent::Collapsible, &children, allow_children)?;
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Collapsible requires body children",
        ));
    }
    let mut label = None;
    let mut default_open = false;
    let mut disabled = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "defaultOpen" => default_open = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Collapsible)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Collapsible, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Background);
    Ok(ViewNode::Collapsible {
        props: CollapsibleProps {
            style,
            label: label
                .ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
            default_open,
            disabled,
        },
        children,
    })
}

pub fn countdown_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut target = None;
    let mut show_days = true;
    let mut show_hours = true;
    let mut show_minutes = true;
    let mut show_seconds = true;
    let mut size = CountdownSize::Md;
    let mut days_label = "Days".to_string();
    let mut hours_label = "Hours".to_string();
    let mut minutes_label = "Minutes".to_string();
    let mut seconds_label = "Seconds".to_string();
    let mut on_complete = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "target" => target = Some(parse_required_string(&prop.name, &prop.value)?),
            "showDays" => show_days = parse_static_bool(&prop.name, &prop.value)?,
            "showHours" => show_hours = parse_static_bool(&prop.name, &prop.value)?,
            "showMinutes" => show_minutes = parse_static_bool(&prop.name, &prop.value)?,
            "showSeconds" => show_seconds = parse_static_bool(&prop.name, &prop.value)?,
            "size" => size = parse_countdown_size(&prop.name, &prop.value)?,
            "daysLabel" => days_label = parse_required_string(&prop.name, &prop.value)?,
            "hoursLabel" => hours_label = parse_required_string(&prop.name, &prop.value)?,
            "minutesLabel" => minutes_label = parse_required_string(&prop.name, &prop.value)?,
            "secondsLabel" => seconds_label = parse_required_string(&prop.name, &prop.value)?,
            "onComplete" => on_complete = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Countdown)),
            _ => style_props.push(prop),
        }
    }
    if !show_days && !show_hours && !show_minutes && !show_seconds {
        return Err(ComponentError::invalid_prop_combination(
            "Countdown must show at least one unit",
        ));
    }
    let mut style = parse_variant_props(BuiltinComponent::Countdown, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Countdown {
        props: CountdownProps {
            style,
            target: target.ok_or_else(|| ComponentError::invalid_prop("target", "date string"))?,
            show_days,
            show_hours,
            show_minutes,
            show_seconds,
            size,
            days_label,
            hours_label,
            minutes_label,
            seconds_label,
            on_complete,
        },
    })
}

pub fn map_marker_component(props: Vec<ComponentProp>) -> ComponentResult<MapMarker> {
    let mut id = None;
    let mut lat = None;
    let mut lng = None;
    let mut label = None;
    let mut popup = None;
    let mut icon = MapMarkerIcon::Default;
    let mut on_click = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "lat" => lat = Some(parse_number_literal(&prop.name, &prop.value)?),
            "lng" => lng = Some(parse_number_literal(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "popup" => popup = Some(parse_required_string(&prop.name, &prop.value)?),
            "icon" => icon = parse_map_marker_icon(&prop.name, &prop.value)?,
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Map,
                    &prop.name,
                ));
            }
        }
    }
    Ok(MapMarker {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "static string or number"))?,
        lat: lat.ok_or_else(|| ComponentError::invalid_prop("lat", "number"))?,
        lng: lng.ok_or_else(|| ComponentError::invalid_prop("lng", "number"))?,
        label,
        popup,
        icon,
        on_click,
    })
}

pub fn map_waypoint_component(props: Vec<ComponentProp>) -> ComponentResult<MapWaypoint> {
    let mut lat = None;
    let mut lng = None;
    for prop in props {
        match prop.name.as_str() {
            "lat" => lat = Some(parse_number_literal(&prop.name, &prop.value)?),
            "lng" => lng = Some(parse_number_literal(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Map,
                    &prop.name,
                ));
            }
        }
    }
    Ok(MapWaypoint {
        lat: lat.ok_or_else(|| ComponentError::invalid_prop("lat", "number"))?,
        lng: lng.ok_or_else(|| ComponentError::invalid_prop("lng", "number"))?,
    })
}

pub fn map_component_node(
    props: Vec<ComponentProp>,
    markers: Vec<MapMarker>,
    waypoints: Vec<MapWaypoint>,
) -> ComponentResult<ViewNode> {
    let mut seen = BTreeSet::new();
    for marker in &markers {
        if !seen.insert(marker.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Map marker id `{}`",
                marker.id
            )));
        }
    }
    let mut center_lat = "0".to_string();
    let mut center_lng = "0".to_string();
    let mut zoom = 13;
    let mut height = "400px".to_string();
    let mut width = "100%".to_string();
    let mut show_controls = true;
    let mut show_scale = false;
    let mut show_location_control = false;
    let mut interactive = true;
    let mut route_start_lat = None;
    let mut route_start_lng = None;
    let mut route_end_lat = None;
    let mut route_end_lng = None;
    let mut on_location = None;
    let mut on_location_error = None;
    let mut on_route = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "centerLat" => center_lat = parse_number_literal(&prop.name, &prop.value)?,
            "centerLng" => center_lng = parse_number_literal(&prop.name, &prop.value)?,
            "zoom" => zoom = parse_non_negative_u16(&prop.name, &prop.value)?,
            "height" => height = parse_required_string(&prop.name, &prop.value)?,
            "width" => width = parse_required_string(&prop.name, &prop.value)?,
            "showControls" => show_controls = parse_static_bool(&prop.name, &prop.value)?,
            "showScale" => show_scale = parse_static_bool(&prop.name, &prop.value)?,
            "showLocationControl" => {
                show_location_control = parse_static_bool(&prop.name, &prop.value)?
            }
            "interactive" => interactive = parse_static_bool(&prop.name, &prop.value)?,
            "routeStartLat" => {
                route_start_lat = Some(parse_number_literal(&prop.name, &prop.value)?)
            }
            "routeStartLng" => {
                route_start_lng = Some(parse_number_literal(&prop.name, &prop.value)?)
            }
            "routeEndLat" => route_end_lat = Some(parse_number_literal(&prop.name, &prop.value)?),
            "routeEndLng" => route_end_lng = Some(parse_number_literal(&prop.name, &prop.value)?),
            "onLocation" => on_location = Some(parse_required_string(&prop.name, &prop.value)?),
            "onLocationError" => {
                on_location_error = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "onRoute" => on_route = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Map)),
            _ => style_props.push(prop),
        }
    }
    let route_count = [
        route_start_lat.is_some(),
        route_start_lng.is_some(),
        route_end_lat.is_some(),
        route_end_lng.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count();
    if route_count != 0 && route_count != 4 {
        return Err(ComponentError::invalid_prop_combination(
            "Map route requires routeStartLat, routeStartLng, routeEndLat and routeEndLng",
        ));
    }
    let mut style = parse_variant_props(BuiltinComponent::Map, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Map {
        props: MapProps {
            style,
            center_lat,
            center_lng,
            zoom,
            height,
            width,
            show_controls,
            show_scale,
            show_location_control,
            interactive,
            route_start_lat,
            route_start_lng,
            route_end_lat,
            route_end_lng,
            on_location,
            on_location_error,
            on_route,
        },
        markers,
        waypoints,
    })
}
