fn render_record_html(props: &RecordProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" data-dowe-record data-dowe-record-name="{}""#,
        escape_attr(&props.name)
    );
    if let Some(url) = props.url.as_ref() {
        extra.push_str(&format!(r#" data-dowe-record-url="{}""#, escape_attr(url)));
    }
    if let Some(value) = props.max_duration {
        extra.push_str(&format!(r#" data-dowe-record-max-duration="{}""#, value));
    }
    if let Some(action) = props.on_start.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-record-on-start="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_pause.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-record-on-pause="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_resume.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-record-on-resume="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_stop.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-record-on-stop="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_discard.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-record-on-discard="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_confirm.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-record-on-confirm="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    let disabled = if props.disabled { " disabled" } else { "" };
    let file = if props.url.is_none() {
        format!(
            r#"<input class="record-file" type="file" accept="audio/*" name="{}" data-dowe-record-file{}>"#,
            escape_attr(&props.name),
            disabled
        )
    } else {
        String::new()
    };
    let bars = (0..50)
        .map(|index| {
            format!(
                r#"<span class="record-bar" style="--record-bar:{}"></span>"#,
                (index % 9) + 2
            )
        })
        .collect::<String>();
    format!(
        r#"<div{}><div class="record-main"><div class="record-wave" aria-hidden="true">{}</div><div class="record-meta"><span class="record-time" data-dowe-record-time>00:00</span><span class="record-status" data-dowe-record-status>Ready</span></div></div><div class="record-actions"><button class="record-btn record-start" type="button" data-dowe-record-action="start"{}>Record</button><button class="record-btn record-pause" type="button" data-dowe-record-action="pause" hidden{}>Pause</button><button class="record-btn record-stop" type="button" data-dowe-record-action="stop" hidden{}>Stop</button><button class="record-btn record-discard" type="button" data-dowe-record-action="discard" hidden{}>Discard</button><button class="record-btn record-confirm" type="button" data-dowe-record-action="confirm" hidden{}>Use</button></div>{}</div>"#,
        attrs(
            record_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        bars,
        disabled,
        disabled,
        disabled,
        disabled,
        disabled,
        file
    )
}

fn render_toggle_group_html(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = String::from(r#" role="radiogroup" data-dowe-toggle-group"#);
    if let Some(value) = props.value.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-toggle-group-value="{}""#,
            escape_attr(&context.signal_path(value))
        ));
    }
    if let Some(action) = props.on_change.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-toggle-group-on-change="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(label) = props.aria_label.as_ref() {
        extra.push_str(&format!(r#" aria-label="{}""#, escape_attr(label)));
    }
    let buttons = items
        .iter()
        .map(|item| {
            let active = item.id == props.selected;
            let variant = props.style.variant.unwrap_or(ComponentVariant::Solid).as_str();
            let color = props.style.color.unwrap_or(ColorFamily::Muted).as_str();
            let icon = item
                .icon
                .map(|icon| view_icon_svg(icon, "toggle-group-icon"))
                .unwrap_or_default();
            format!(
                r#"<button class="toggle-group-item is-{} is-{}{}" type="button" role="radio" aria-checked="{}" data-dowe-toggle-group-item="{}"{}>{}<span>{}</span></button>"#,
                variant,
                color,
                if active { " is-active" } else { "" },
                active,
                escape_attr(&item.id),
                if props.disabled { " disabled" } else { "" },
                icon,
                escape_html(&item.label)
            )
        })
        .collect::<String>();
    format!(
        "<div{}>{}</div>",
        attrs(
            toggle_group_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        buttons
    )
}

fn render_collapsible_html(
    props: &CollapsibleProps,
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let body = children
        .iter()
        .map(|child| render_html_with_context(child, children_html, context))
        .collect::<String>();
    let extra = format!(
        r#" data-dowe-collapsible data-dowe-collapsible-open="{}""#,
        props.default_open
    );
    format!(
        r#"<div{}><button class="collapsible-header" type="button" aria-expanded="{}" data-dowe-collapsible-trigger{}><span class="collapsible-label">{}</span><span class="collapsible-arrow" aria-hidden="true">⌄</span></button><div class="collapsible-content" data-dowe-collapsible-content{}>{}</div></div>"#,
        attrs(
            collapsible_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        props.default_open,
        if props.disabled { " disabled" } else { "" },
        escape_html(&props.label),
        if props.default_open { "" } else { " hidden" },
        body
    )
}

fn render_countdown_html(props: &CountdownProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" data-dowe-countdown data-dowe-countdown-target="{}""#,
        escape_attr(&props.target)
    );
    if let Some(action) = props.on_complete.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-countdown-on-complete="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    let mut units = Vec::new();
    if props.show_days {
        units.push(("days", props.days_label.as_str()));
    }
    if props.show_hours {
        units.push(("hours", props.hours_label.as_str()));
    }
    if props.show_minutes {
        units.push(("minutes", props.minutes_label.as_str()));
    }
    if props.show_seconds {
        units.push(("seconds", props.seconds_label.as_str()));
    }
    let content = units
        .iter()
        .enumerate()
        .map(|(index, (unit, label))| {
            let variant = props.style.variant.unwrap_or(ComponentVariant::Solid).as_str();
            let color = props.style.color.unwrap_or(ColorFamily::Primary).as_str();
            let separator = (index + 1 < units.len())
                .then_some(r#"<span class="countdown-separator" aria-hidden="true">:</span>"#)
                .unwrap_or_default();
            format!(
                r#"<span class="countdown-unit"><span class="countdown-box is-{} is-{}"><span class="countdown-digit" data-dowe-countdown-unit="{}">00</span></span><span class="countdown-label">{}</span></span>{}"#,
                variant,
                color,
                unit,
                escape_html(label),
                separator
            )
        })
        .collect::<String>();
    format!(
        r#"<time{} datetime="{}">{}</time>"#,
        attrs(
            countdown_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        escape_attr(&props.target),
        content
    )
}

fn render_map_html(
    props: &MapProps,
    markers: &[MapMarker],
    waypoints: &[MapWaypoint],
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = format!(
        r#" style="--map-height:{};--map-width:{};" data-dowe-map data-dowe-map-center-lat="{}" data-dowe-map-center-lng="{}" data-dowe-map-zoom="{}""#,
        escape_attr(&props.height),
        escape_attr(&props.width),
        escape_attr(&props.center_lat),
        escape_attr(&props.center_lng),
        props.zoom
    );
    if let Some(action) = props.on_location.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-map-on-location="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_location_error.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-map-on-location-error="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_route.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-map-on-route="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    let controls = if props.show_controls {
        r#"<div class="map-controls" aria-hidden="true"><span>+</span><span>-</span></div>"#
    } else {
        ""
    };
    let scale = if props.show_scale {
        r#"<div class="map-scale" aria-hidden="true"><span></span>1 km</div>"#
    } else {
        ""
    };
    let location = if props.show_location_control {
        r#"<button class="map-location-btn" type="button" aria-label="Use current location" data-dowe-map-location>⌖</button>"#
    } else {
        ""
    };
    let route = if props.route_start_lat.is_some() || !waypoints.is_empty() {
        r#"<div class="map-route" aria-hidden="true"></div>"#
    } else {
        ""
    };
    let marker_html = markers
        .iter()
        .enumerate()
        .map(|(index, marker)| render_map_marker_html(marker, index, markers.len(), context))
        .collect::<String>();
    let waypoint_html = waypoints
        .iter()
        .enumerate()
        .map(|(index, waypoint)| {
            let (left, top) = map_point_position(index + markers.len(), markers.len() + waypoints.len());
            format!(
                r#"<span class="map-waypoint" style="left:{}%;top:{}%;" data-dowe-map-waypoint-lat="{}" data-dowe-map-waypoint-lng="{}"></span>"#,
                left,
                top,
                escape_attr(&waypoint.lat),
                escape_attr(&waypoint.lng)
            )
        })
        .collect::<String>();
    format!(
        r#"<div{}><div class="map-container"><div class="map-grid" aria-hidden="true"></div>{route}{marker_html}{waypoint_html}{controls}{scale}{location}</div></div>"#,
        attrs(
            map_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        )
    )
}

fn render_map_marker_html(
    marker: &MapMarker,
    index: usize,
    total: usize,
    context: &ReactiveRenderContext,
) -> String {
    let (left, top) = map_point_position(index, total);
    let mut extra = format!(
        r#" style="left:{}%;top:{}%;" data-dowe-map-marker="{}" data-dowe-map-marker-lat="{}" data-dowe-map-marker-lng="{}" data-dowe-map-marker-icon="{}""#,
        left,
        top,
        escape_attr(&marker.id),
        escape_attr(&marker.lat),
        escape_attr(&marker.lng),
        marker.icon.as_str()
    );
    if let Some(action) = marker.on_click.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-click="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    let label = marker
        .label
        .as_deref()
        .or(marker.popup.as_deref())
        .map(|label| {
            format!(
                r#"<span class="map-marker-label">{}</span>"#,
                escape_html(label)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<button class="map-marker is-{}" type="button"{}><span class="map-marker-pin"></span>{}</button>"#,
        marker.icon.as_str(),
        extra,
        label
    )
}

fn map_point_position(index: usize, total: usize) -> (usize, usize) {
    if total <= 1 {
        return (50, 50);
    }
    let step = 100 / (total + 1);
    let left = ((index + 1) * step).clamp(12, 88);
    let top = (28 + ((index * 23) % 46)).clamp(16, 84);
    (left, top)
}

fn empty_default_title(kind: EmptyKind) -> &'static str {
    match kind {
        EmptyKind::Playlist => "No playlist items",
        EmptyKind::Result => "No results found",
        EmptyKind::Data => "No data",
        EmptyKind::Template => "No template selected",
    }
}

fn empty_default_description(kind: EmptyKind) -> &'static str {
    match kind {
        EmptyKind::Playlist => "Add items to start listening.",
        EmptyKind::Result => "Try a different search or filter.",
        EmptyKind::Data => "There are no records to display.",
        EmptyKind::Template => "Choose or create a template to continue.",
    }
}

fn empty_icon_html(kind: EmptyKind) -> &'static str {
    match kind {
        EmptyKind::Playlist => {
            r#"<svg class="empty-icon" viewBox="0 0 120 100" aria-hidden="true"><rect x="28" y="18" width="54" height="64" rx="10" fill="currentColor" opacity=".12"></rect><path d="M76 29v33.5a10 10 0 1 1-5-8.66V35H49v27.5a10 10 0 1 1-5-8.66V29z" fill="currentColor" opacity=".78"></path></svg>"#
        }
        EmptyKind::Result => {
            r#"<svg class="empty-icon" viewBox="0 0 120 100" aria-hidden="true"><circle cx="54" cy="45" r="24" fill="currentColor" opacity=".12"></circle><path d="M70 62l18 18M45 38h18M45 50h13" stroke="currentColor" stroke-width="7" stroke-linecap="round" opacity=".78"></path></svg>"#
        }
        EmptyKind::Data => {
            r#"<svg class="empty-icon" viewBox="0 0 120 100" aria-hidden="true"><rect x="24" y="22" width="72" height="56" rx="10" fill="currentColor" opacity=".12"></rect><path d="M38 38h44M38 52h34M38 66h22" stroke="currentColor" stroke-width="7" stroke-linecap="round" opacity=".78"></path></svg>"#
        }
        EmptyKind::Template => {
            r#"<svg class="empty-icon" viewBox="0 0 120 100" aria-hidden="true"><path d="M30 20h42l18 18v42H30z" fill="currentColor" opacity=".12"></path><path d="M72 20v20h20M43 50h34M43 64h26" stroke="currentColor" stroke-width="7" stroke-linecap="round" stroke-linejoin="round" opacity=".78"></path></svg>"#
        }
    }
}
