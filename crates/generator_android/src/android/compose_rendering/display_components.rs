fn render_compose_avatar(
    props: &AvatarProps,
    icon: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweAvatar(source = {}, name = {}, alt = {}, size = {}, status = {}, bordered = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onClick = {}, hasIcon = {}) {{\n",
        compose_optional_string(props.src.as_deref()),
        compose_optional_string(props.name.as_deref()),
        compose_string_literal(&props.alt),
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.status.map(|value| value.as_str())),
        props.bordered,
        variant_container(&props.style),
        variant_content(&props.style),
        variant_content(&props.style),
        compose_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        icon.is_some()
    ));
    if let Some(icon) = icon {
        render_compose_side_icon(icon, indent + 4, output);
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_avatar_group(
    props: &AvatarGroupProps,
    items: &[AvatarGroupItem],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweAvatarGroup(items = {}, size = {}, maxCount = {}, inline = {}, bordered = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {})\n",
        compose_avatar_group_items_value(props, items, context),
        compose_string_literal(props.size.as_str()),
        props.max
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_string()),
        props.inline,
        props.bordered,
        variant_container(&props.style),
        variant_content(&props.style),
        variant_content(&props.style),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_chat_box(
    props: &ChatBoxProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweChatBox(state = state, messagesPath = {}, mode = {}, currentUserId = {}, userName = {}, userAvatar = {}, userStatus = {}, assistantName = {}, assistantAvatar = {}, showHeader = {}, placeholder = {}, showAttachments = {}, showVoiceNote = {}, showCamera = {}, loading = {}, sending = {}, streaming = {}, hasMore = {}, onSend = {}, onLoadMore = {}, onStop = {}, onVoiceNote = {}, onFileAttach = {}, onCameraCapture = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {})\n",
        compose_string_literal(&context.signal_path(&props.messages)),
        compose_string_literal(props.mode.as_str()),
        compose_string_literal(&props.current_user_id),
        compose_string_literal(&props.user_name),
        compose_optional_string(props.user_avatar.as_deref()),
        compose_string_literal(&props.user_status),
        compose_string_literal(&props.assistant_name),
        compose_optional_string(props.assistant_avatar.as_deref()),
        props.show_header,
        compose_string_literal(&props.placeholder),
        props.show_attachments,
        props.show_voice_note,
        props.show_camera,
        compose_optional_bool_signal(props.loading.as_deref(), context),
        compose_optional_bool_signal(props.sending.as_deref(), context),
        compose_optional_bool_signal(props.streaming.as_deref(), context),
        compose_optional_bool_signal(props.has_more.as_deref(), context),
        compose_chat_send_action(props.on_send.as_deref(), context),
        compose_optional_component_action(props.on_load_more.as_deref(), None, context),
        compose_optional_component_action(props.on_stop.as_deref(), None, context),
        compose_optional_component_action(props.on_voice_note.as_deref(), None, context),
        compose_optional_component_action(props.on_file_attach.as_deref(), None, context),
        compose_optional_component_action(props.on_camera_capture.as_deref(), None, context),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_empty(
    props: &EmptyProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweEmpty(kind = {}, title = {}, description = {}, actionLabel = {}, action = {}, backgroundColor = {}, contentColor = {}, accentColor = {}, modifier = {})\n",
        compose_string_literal(props.kind.as_str()),
        compose_optional_string(props.title.as_deref()),
        compose_optional_string(props.description.as_deref()),
        compose_string_literal(&props.action_label),
        compose_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Primary))),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_marquee(
    props: &MarqueeProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweMarquee(speed = {}, pauseOnHover = {}, reverse = {}, orientation = {}, fade = {}, fadeColor = {}, gap = {}, modifier = {}) {{\n",
        compose_string_literal(props.speed.as_str()),
        props.pause_on_hover,
        props.reverse,
        compose_string_literal(props.orientation.as_str()),
        props.fade,
        color_ref(props.fade_color),
        compose_scale_literal(props.gap),
        modifier_for_style(&props.style),
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            props.style.font.as_ref().or(inherited_font),
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_type_writer(
    props: &TypeWriterProps,
    items: &[TypeWriterItem],
    indent: usize,
    output: &mut String,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweTypeWriter(texts = {}, typeSpeed = {}, deleteSpeed = {}, afterTyped = {}, afterDeleted = {}, repeat = {}, contentColor = {}, modifier = {})\n",
        compose_type_writer_items(items),
        props.type_speed,
        props.delete_speed,
        props.after_typed,
        props.after_deleted,
        props.repeat,
        color_ref(ColorToken::OnBackground),
        modifier_for_style(&props.style),
    ));
}

fn render_compose_rich_text(
    props: &TextProps,
    marks: &[RichTextMark],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    let size = text_size(false, props);
    output.push_str(&format!(
        "{pad}DoweRichText(marks = {}, fontFamily = {}, fontSize = {size}, contentColor = {}, modifier = {})\n",
        compose_rich_text_marks(marks),
        compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
        text_color(props),
        modifier_for_style(&props.style),
    ));
}

fn render_compose_record(
    props: &RecordProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweRecord(name = {}, url = {}, disabled = {}, maxDuration = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onStart = {}, onPause = {}, onResume = {}, onStop = {}, onDiscard = {}, onConfirm = {}, modifier = {})\n",
        compose_string_literal(&props.name),
        compose_optional_string(props.url.as_deref()),
        props.disabled,
        props.max_duration.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_optional_component_action(props.on_start.as_deref(), None, context),
        compose_optional_component_action(props.on_pause.as_deref(), None, context),
        compose_optional_component_action(props.on_resume.as_deref(), None, context),
        compose_optional_component_action(props.on_stop.as_deref(), None, context),
        compose_optional_component_action(props.on_discard.as_deref(), None, context),
        compose_optional_component_action(props.on_confirm.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_toggle_group(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let value = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "state.text(\"{}\")",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| compose_string_literal(&props.selected));
    let change = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "{{ value -> state.write(\"{}\", value) }}",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| "{ _ -> }".to_string());
    output.push_str(&format!(
        "{pad}DoweToggleGroup(value = {value}, onValueChange = {change}, items = {}, size = {}, wide = {}, vertical = {}, disabled = {}, ariaLabel = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onChange = {}, modifier = {})\n",
        compose_toggle_group_items(items),
        compose_string_literal(props.size.as_str()),
        props.wide,
        props.vertical,
        props.disabled,
        compose_optional_string(props.aria_label.as_deref()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_optional_component_action(props.on_change.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_collapsible(
    props: &CollapsibleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCollapsible(label = {}, defaultOpen = {}, disabled = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {}) {{\n",
        compose_string_literal(&props.label),
        props.default_open,
        props.disabled,
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        modifier_for_style(&props.style.style),
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            props.style.style.font.as_ref().or(inherited_font),
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_countdown(
    props: &CountdownProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCountdown(target = {}, showDays = {}, showHours = {}, showMinutes = {}, showSeconds = {}, size = {}, daysLabel = {}, hoursLabel = {}, minutesLabel = {}, secondsLabel = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onComplete = {}, modifier = {})\n",
        compose_string_literal(&props.target),
        props.show_days,
        props.show_hours,
        props.show_minutes,
        props.show_seconds,
        compose_string_literal(props.size.as_str()),
        compose_string_literal(&props.days_label),
        compose_string_literal(&props.hours_label),
        compose_string_literal(&props.minutes_label),
        compose_string_literal(&props.seconds_label),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_optional_component_action(props.on_complete.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_map(
    props: &MapProps,
    markers: &[MapMarker],
    waypoints: &[MapWaypoint],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweMap(centerLat = {}, centerLng = {}, zoom = {}, height = {}, width = {}, showControls = {}, showScale = {}, showLocationControl = {}, interactive = {}, markers = {}, waypoints = {}, backgroundColor = {}, contentColor = {}, onLocation = {}, onLocationError = {}, onRoute = {}, modifier = {})\n",
        compose_string_literal(&props.center_lat),
        compose_string_literal(&props.center_lng),
        props.zoom,
        compose_string_literal(&props.height),
        compose_string_literal(&props.width),
        props.show_controls,
        props.show_scale,
        props.show_location_control,
        props.interactive,
        compose_map_markers(markers, context),
        compose_map_waypoints(waypoints),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_optional_component_action(props.on_location.as_deref(), None, context),
        compose_optional_component_action(props.on_location_error.as_deref(), None, context),
        compose_optional_component_action(props.on_route.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_badge(
    props: &BadgeProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweBadge(text = {}, position = {}, backgroundColor = {}, contentColor = {}, modifier = {}) {{\n",
        compose_string_literal(&props.text),
        compose_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        modifier_for_style(&props.style.style)
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_chip(
    props: &ChipProps,
    value: &str,
    start: Option<&SideNavIcon>,
    end: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    output.push_str(&format!(
        "{pad}DoweChip(text = {}, size = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {}, onClose = {}, start = ",
        compose_string_literal(value),
        compose_string_literal(size.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        modifier_for_style(&props.style.style),
        compose_optional_component_action(props.on_close.as_deref(), None, context)
    ));
    render_compose_optional_icon_lambda(start, indent, output);
    output.push_str(", end = ");
    render_compose_optional_icon_lambda(end, indent, output);
    output.push_str(")\n");
}

fn render_compose_skeleton(
    props: &SkeletonProps,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSkeleton(variant = {}, animation = {}, modifier = {})\n",
        compose_string_literal(props.variant.as_str()),
        compose_string_literal(props.animation.as_str()),
        modifier_for_container_style(&props.style, flow)
    ));
}
