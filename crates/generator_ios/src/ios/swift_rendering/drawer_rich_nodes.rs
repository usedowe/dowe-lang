fn render_swift_drawer(
    props: &DrawerProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_swift(&context.signal_path(&props.open));
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!("Optional({})", card_variant_content(&props.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweDrawer(open: state.bool(\"{path}\"), close: {{ state.write(\"{path}\", value: false) }}, position: \"{}\", backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {}, disableOverlayClose: {}, hideCloseButton: {}) {{\n",
        props.position.as_str(),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_drawer_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button
    ));
    output.push_str(&format!(
        "{pad}    let doweDrawerNavigate = navigate\n{pad}    let navigate: (String, String, String?) -> Void = {{ operation, target, fragment in\n{pad}        state.write(\"{path}\", value: false)\n{pad}        doweDrawerNavigate(operation, target, fragment)\n{pad}    }}\n{pad}    let doweDrawerGoBack = goBack\n{pad}    let goBack: () -> Void = {{\n{pad}        state.write(\"{path}\", value: false)\n{pad}        doweDrawerGoBack()\n{pad}    }}\n{pad}    let doweDrawerOpenExternal = openExternal\n{pad}    let openExternal: (String, String) -> Void = {{ mode, target in\n{pad}        state.write(\"{path}\", value: false)\n{pad}        doweDrawerOpenExternal(mode, target)\n{pad}    }}\n"
    ));
    output.push_str(&format!(
        "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    if !header.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: 0) {{\n"
        ));
        for child in header {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
        output.push_str(&format!(
            "{pad}        .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    }
    output.push_str(&format!(
        "{pad}        ScrollView {{\n{pad}            VStack(alignment: .leading, spacing: 0) {{\n"
    ));
    for child in body {
        render_swift_node_in_flow(
            child,
            indent + 16,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!(
        "{pad}            }}\n{pad}            .frame(maxWidth: .infinity, alignment: .topLeading)\n{pad}        }}\n{pad}        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
    ));
    if !footer.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: 0) {{\n"
        ));
        for child in footer {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
        output.push_str(&format!(
            "{pad}        .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    }
    output.push_str(&format!("{pad}    }}\n"));
    output.push_str(&format!(
        "{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
    ));
    append_swift_modifiers(
        output,
        indent + 4,
        &swift_modifiers_for_container_style(&props.style.style, NativeFlow::Block),
    );
    output.push_str(&format!("{pad}}}\n"));
}

fn render_swift_avatar(
    props: &AvatarProps,
    icon: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweAvatar(source: {}, name: {}, alt: {}, size: {}, status: {}, bordered: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, action: {}, hasIcon: {}) {{\n",
        swift_optional_literal(props.src.as_deref()),
        swift_optional_literal(props.name.as_deref()),
        swift_string_literal(&props.alt),
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.status.map(|value| value.as_str())),
        props.bordered,
        variant_container(&props.style),
        variant_content(&props.style),
        variant_content(&props.style),
        swift_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        icon.is_some()
    ));
    if let Some(icon) = icon {
        render_swift_side_icon(icon, indent + 4, output);
    } else {
        output.push_str(&format!("{pad}    EmptyView()\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_avatar_group(
    props: &AvatarGroupProps,
    items: &[AvatarGroupItem],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweAvatarGroup(items: {}, size: {}, maxCount: {}, inline: {}, bordered: {}, backgroundColor: {}, contentColor: {}, borderColor: {})\n",
        swift_avatar_group_items_value(props, items, context),
        swift_string_literal(props.size.as_str()),
        props.max
            .map(|value| value.to_string())
            .unwrap_or_else(|| "nil".to_string()),
        props.inline,
        props.bordered,
        variant_container(&props.style),
        variant_content(&props.style),
        variant_content(&props.style),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_chat_box(
    props: &ChatBoxProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweChatBox(state: state, messagesPath: {}, mode: {}, currentUserId: {}, userName: {}, userAvatar: {}, userStatus: {}, assistantName: {}, assistantAvatar: {}, showHeader: {}, placeholder: {}, showAttachments: {}, showVoiceNote: {}, showCamera: {}, loading: {}, sending: {}, streaming: {}, hasMore: {}, onSend: {}, onLoadMore: {}, onStop: {}, onVoiceNote: {}, onFileAttach: {}, onCameraCapture: {}, backgroundColor: {}, contentColor: {}, borderColor: {})\n",
        swift_string_literal(&context.signal_path(&props.messages)),
        swift_string_literal(props.mode.as_str()),
        swift_string_literal(&props.current_user_id),
        swift_string_literal(&props.user_name),
        swift_optional_literal(props.user_avatar.as_deref()),
        swift_string_literal(&props.user_status),
        swift_string_literal(&props.assistant_name),
        swift_optional_literal(props.assistant_avatar.as_deref()),
        props.show_header,
        swift_string_literal(&props.placeholder),
        props.show_attachments,
        props.show_voice_note,
        props.show_camera,
        swift_optional_bool_signal(props.loading.as_deref(), context),
        swift_optional_bool_signal(props.sending.as_deref(), context),
        swift_optional_bool_signal(props.streaming.as_deref(), context),
        swift_optional_bool_signal(props.has_more.as_deref(), context),
        swift_chat_send_action(props.on_send.as_deref(), context),
        swift_optional_component_action(props.on_load_more.as_deref(), None, context),
        swift_optional_component_action(props.on_stop.as_deref(), None, context),
        swift_optional_component_action(props.on_voice_note.as_deref(), None, context),
        swift_optional_component_action(props.on_file_attach.as_deref(), None, context),
        swift_optional_component_action(props.on_camera_capture.as_deref(), None, context),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_empty(
    props: &EmptyProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweEmpty(kind: {}, title: {}, description: {}, actionLabel: {}, action: {}, backgroundColor: {}, contentColor: {}, accentColor: {})\n",
        swift_string_literal(props.kind.as_str()),
        swift_optional_literal(props.title.as_deref()),
        swift_optional_literal(props.description.as_deref()),
        swift_string_literal(&props.action_label),
        swift_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Primary))),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_marquee(
    props: &MarqueeProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweMarquee(speed: {}, pauseOnHover: {}, reverse: {}, orientation: {}, fade: {}, fadeColor: {}, gap: {}) {{\n",
        swift_string_literal(props.speed.as_str()),
        props.pause_on_hover,
        props.reverse,
        swift_string_literal(props.orientation.as_str()),
        props.fade,
        color_ref(props.fade_color),
        swift_scale_literal(props.gap),
    ));
    for child in children {
        render_swift_node_in_flow(
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
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
}

fn render_swift_type_writer(
    props: &TypeWriterProps,
    items: &[TypeWriterItem],
    indent: usize,
    output: &mut String,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweTypeWriter(texts: {}, typeSpeed: {}, deleteSpeed: {}, afterTyped: {}, afterDeleted: {}, repeat: {}, contentColor: {})\n",
        swift_type_writer_items(items),
        props.type_speed,
        props.delete_speed,
        props.after_typed,
        props.after_deleted,
        props.repeat,
        color_ref(ColorToken::OnBackground),
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
}

fn render_swift_rich_text(
    props: &TextProps,
    marks: &[RichTextMark],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    let size = props
        .size
        .as_ref()
        .map(|value| value.entries[0].value)
        .unwrap_or(TextSize::Md);
    let font_size = swift_text_size_expr(false, size);
    output.push_str(&format!(
        "{pad}DoweRichText(marks: {}, font: {}, fontSize: {font_size})\n",
        swift_rich_text_marks(marks),
        swift_font_token_value(props.style.font.as_ref().or(inherited_font), default_family),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_text(false, props, inherited_font, default_family),
    );
}

fn render_swift_record(
    props: &RecordProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweRecord(name: {}, url: {}, disabled: {}, maxDuration: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, onStart: {}, onPause: {}, onResume: {}, onStop: {}, onDiscard: {}, onConfirm: {})\n",
        swift_string_literal(&props.name),
        swift_optional_literal(props.url.as_deref()),
        props.disabled,
        props.max_duration.map(|value| value.to_string()).unwrap_or_else(|| "nil".to_string()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_optional_component_action(props.on_start.as_deref(), None, context),
        swift_optional_component_action(props.on_pause.as_deref(), None, context),
        swift_optional_component_action(props.on_resume.as_deref(), None, context),
        swift_optional_component_action(props.on_stop.as_deref(), None, context),
        swift_optional_component_action(props.on_discard.as_deref(), None, context),
        swift_optional_component_action(props.on_confirm.as_deref(), None, context),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_toggle_group(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "state.binding(\"{}\")",
                escape_swift(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| format!(".constant({})", swift_string_literal(&props.selected)));
    output.push_str(&format!(
        "{pad}DoweToggleGroup(value: {binding}, items: {}, size: {}, wide: {}, vertical: {}, disabled: {}, ariaLabel: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, onChange: {})\n",
        swift_toggle_group_items(items),
        swift_string_literal(props.size.as_str()),
        props.wide,
        props.vertical,
        props.disabled,
        swift_optional_literal(props.aria_label.as_deref()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_optional_component_action(props.on_change.as_deref(), None, context),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_collapsible(
    props: &CollapsibleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCollapsible(label: {}, defaultOpen: {}, disabled: {}, backgroundColor: {}, contentColor: {}, borderColor: {}) {{\n",
        swift_string_literal(&props.label),
        props.default_open,
        props.disabled,
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
    ));
    for child in children {
        render_swift_node_in_flow(
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_countdown(
    props: &CountdownProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCountdown(target: {}, showDays: {}, showHours: {}, showMinutes: {}, showSeconds: {}, size: {}, daysLabel: {}, hoursLabel: {}, minutesLabel: {}, secondsLabel: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, onComplete: {})\n",
        swift_string_literal(&props.target),
        props.show_days,
        props.show_hours,
        props.show_minutes,
        props.show_seconds,
        swift_string_literal(props.size.as_str()),
        swift_string_literal(&props.days_label),
        swift_string_literal(&props.hours_label),
        swift_string_literal(&props.minutes_label),
        swift_string_literal(&props.seconds_label),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_optional_component_action(props.on_complete.as_deref(), None, context),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_map(
    props: &MapProps,
    markers: &[MapMarker],
    waypoints: &[MapWaypoint],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweMap(centerLat: {}, centerLng: {}, zoom: {}, height: {}, width: {}, showControls: {}, showScale: {}, showLocationControl: {}, interactive: {}, markers: {}, waypoints: {}, backgroundColor: {}, contentColor: {}, onLocation: {}, onLocationError: {}, onRoute: {})\n",
        swift_string_literal(&props.center_lat),
        swift_string_literal(&props.center_lng),
        props.zoom,
        swift_string_literal(&props.height),
        swift_string_literal(&props.width),
        props.show_controls,
        props.show_scale,
        props.show_location_control,
        props.interactive,
        swift_map_markers(markers, context),
        swift_map_waypoints(waypoints),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_optional_component_action(props.on_location.as_deref(), None, context),
        swift_optional_component_action(props.on_location_error.as_deref(), None, context),
        swift_optional_component_action(props.on_route.as_deref(), None, context),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}
