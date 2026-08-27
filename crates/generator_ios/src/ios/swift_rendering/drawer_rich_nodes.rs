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
        "{pad}    let doweDrawerNavigate = navigate\n{pad}    let navigate: (String, String, String?) -> Void = {{ operation, target, fragment in\n{pad}        state.write(\"{path}\", value: false)\n{pad}        doweDrawerNavigate(operation, target, fragment)\n{pad}    }}\n{pad}    let _ = navigate\n{pad}    let doweDrawerGoBack = goBack\n{pad}    let goBack: () -> Void = {{\n{pad}        state.write(\"{path}\", value: false)\n{pad}        doweDrawerGoBack()\n{pad}    }}\n{pad}    let _ = goBack\n{pad}    let doweDrawerOpenExternal = openExternal\n{pad}    let openExternal: (String, String) -> Void = {{ mode, target in\n{pad}        state.write(\"{path}\", value: false)\n{pad}        doweDrawerOpenExternal(mode, target)\n{pad}    }}\n{pad}    let _ = openExternal\n"
    ));
    output.push_str(&format!(
        "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    if !header.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: 0) {{\n"
        ));
        render_swift_region_children(
            header,
            indent + 12,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}        }}\n"));
        output.push_str(&format!(
            "{pad}        .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    }
    output.push_str(&format!(
        "{pad}        ScrollView {{\n{pad}            VStack(alignment: .leading, spacing: 0) {{\n"
    ));
    render_swift_region_children(
        body,
        indent + 16,
        output,
        current_font,
        default_family,
        context,
    );
    output.push_str(&format!(
        "{pad}            }}\n{pad}            .frame(maxWidth: .infinity, alignment: .topLeading)\n{pad}        }}\n{pad}        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
    ));
    if !footer.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: 0) {{\n"
        ));
        render_swift_region_children(
            footer,
            indent + 12,
            output,
            current_font,
            default_family,
            context,
        );
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
    let base_border = if props.bordered {
        (
            format!("Optional({})", variant_content(&props.style)),
            "CGFloat(3)".to_string(),
        )
    } else {
        ("nil".to_string(), "CGFloat(0)".to_string())
    };
    let (border, border_width) =
        swift_style_border(&props.style.style, &base_border.0, &base_border.1);
    let shadow = swift_shadow_spec(&props.style.style)
        .map(|value| format!("Optional({value})"))
        .unwrap_or_else(|| "nil".to_string());
    let dynamic_text = |value: &str, binding: Option<&dowe_components::PropBinding>| {
        binding.map(|binding| {
            if let Some(item) = context.item_value(&binding.path) {
                let path = context.item_path(&binding.path).unwrap_or_else(|| binding.path.clone());
                format!("state.text(\"{}\", item: {item})", escape_swift(&path))
            } else {
                format!("state.text(\"{}\")", escape_swift(&context.signal_path(&binding.path)))
            }
        }).unwrap_or_else(|| swift_string_literal(value))
    };
    let name = props.name_binding.as_ref().map(|binding| dynamic_text("", Some(binding))).unwrap_or_else(|| swift_optional_literal(props.name.as_deref()));
    let alt = dynamic_text(&props.alt, props.alt_binding.as_ref());
    let size = dynamic_text(props.size.as_str(), props.size_binding.as_ref());
    output.push_str(&format!(
        "{pad}DoweAvatar(source: {}, name: {}, alt: {}, size: {}, status: {}, backgroundColor: {}, contentColor: {}, borderColor: {border}, borderWidth: {border_width}, shadow: {shadow}, action: {}, hasIcon: {}) {{\n",
        swift_optional_literal(props.src.as_deref()),
        name,
        alt,
        size,
        swift_optional_literal(props.status.map(|value| value.as_str())),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        icon.is_some()
    ));
    if let Some(icon) = icon {
        if let Some(name) = icon.props.icon_name.as_deref() {
            let name = if let Some(item) = context.item_value(name) {
                let path = context.item_path(name).unwrap_or_else(|| name.to_string());
                format!("state.text(\"{}\", item: {item})", escape_swift(&path))
            } else {
                format!("state.text(\"{}\")", escape_swift(&context.signal_path(name)))
            };
            let color = icon
                .props
                .icon_fill
                .or(icon.props.icon_stroke)
                .map(color_ref)
                .map(str::to_string)
                .unwrap_or_else(|| swift_svg_color(&icon.props.style));
            output.push_str(&format!(
                "{pad}    DoweRuntimeSvgView(payload: DoweDynamicIconCatalog[{name}] ?? \"\", color: {color}, animated: false)\n"
            ));
            append_swift_modifiers(output, indent + 4, &swift_modifiers_for_svg(&icon.props));
        } else {
            render_swift_side_icon(icon, indent + 4, output);
        }
    } else {
        output.push_str(&format!("{pad}    EmptyView()\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
    let mut avatar_style = props.style.style.clone();
    avatar_style.shadow = None;
    avatar_style.shadow_color = None;
    avatar_style.rounded = None;
    avatar_style.border = None;
    avatar_style.border_color = None;
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&avatar_style));
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
    let icon = empty_icon(props.kind).expect("bundled Empty icon");
    output.push_str(&format!(
        "{pad}DoweEmpty(kind: {}, title: {}, description: {}, actionLabel: {}, action: {}, iconViewBox: {}, iconPaths: {}, backgroundColor: {}, contentColor: {}, accentColor: {})\n",
        swift_string_literal(props.kind.as_str()),
        swift_optional_literal(props.title.as_deref()),
        swift_optional_literal(props.description.as_deref()),
        swift_string_literal(&props.action_label),
        swift_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        swift_svg_view_box(&icon.props.view_box),
        swift_svg_paths(&icon.paths),
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
        color_ref(ColorToken::BackgroundText),
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
    let font_size = swift_text_size_expr(props.title, size);
    let content_color = text_color(props).unwrap_or_else(|| "DoweDesign.backgroundText".to_string());
    output.push_str(&format!(
        "{pad}DoweRichText(marks: {}, font: {}, fontSize: {font_size}, contentColor: {content_color})\n",
        swift_rich_text_marks(marks),
        swift_font_token_value(props.style.font.as_ref().or(inherited_font), default_family),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_text(props.title, props, inherited_font, default_family),
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
    if props.kind == ToggleGroupKind::Pagination {
        render_swift_pagination(props, items, indent, output, context);
        return;
    }
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

fn render_swift_pagination(
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
        .map(|path| format!("state.binding(\"{}\")", escape_swift(&context.signal_path(path))))
        .unwrap_or_else(|| format!(".constant({})", swift_string_literal(&props.selected)));
    let page_count = props
        .pagination
        .as_ref()
        .map(|pagination| match &pagination.total {
            dowe_components::PaginationTotal::Static(total) => {
                total.div_ceil(pagination.page_size).max(1).to_string()
            }
            dowe_components::PaginationTotal::Signal(total) => {
                let path = escape_swift(&context.signal_path(total));
                let offset = pagination.page_size - 1;
                format!(
                    "max(1, min(25, (max(0, Int(state.text(\"{path}\")) ?? 0) + {offset}) / {}))",
                    pagination.page_size
                )
            }
        })
        .unwrap_or_else(|| items.len().max(1).to_string());
    let previous = solar_control_icon("arrow-left").expect("bundled Pagination previous icon");
    let next = solar_control_icon("arrow-right").expect("bundled Pagination next icon");
    output.push_str(&format!(
        "{pad}DowePagination(value: {binding}, pageCount: {page_count}, size: {}, disabled: {}, ariaLabel: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, onChange: {}, previousIcon: {{\n",
        swift_string_literal(props.size.as_str()),
        props.disabled,
        swift_optional_literal(props.aria_label.as_deref()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_optional_component_action(props.on_change.as_deref(), None, context),
    ));
    render_swift_side_icon(&previous, indent + 4, output);
    output.push_str(&format!("{pad}}}, nextIcon: {{\n"));
    render_swift_side_icon(&next, indent + 4, output);
    output.push_str(&format!("{pad}}})\n"));
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
    let arrow = solar_control_icon("alt-arrow-down").expect("bundled Collapsible arrow icon");
    let content_color = card_variant_content(&props.style);
    output.push_str(&format!(
        "{pad}DoweCollapsible(label: {}, defaultOpen: {}, disabled: {}, backgroundColor: {}, contentColor: {content_color}, borderColor: {}, radius: {}, arrowIcon: {{\n",
        swift_string_literal(&props.label),
        props.default_open,
        props.disabled,
        card_variant_container(&props.style),
        swift_variant_border(&props.style),
        swift_card_radius(&props.style.style),
    ));
    render_swift_button_icon(&arrow, content_color, indent + 4, output);
    output.push_str(&format!("{pad}}}) {{\n"));
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
