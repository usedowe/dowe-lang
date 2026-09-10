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

fn empty_icon_html(kind: EmptyKind, context: &ReactiveRenderContext) -> String {
    let icon = empty_icon(kind).expect("bundled Empty icon");
    format!(
        r#"<span class="empty-icon">{}</span>"#,
        render_svg_html(&icon.props, &icon.paths, context)
    )
}
