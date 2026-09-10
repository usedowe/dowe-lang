fn render_compose_avatar(
    props: &AvatarProps,
    icon: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let dynamic_text = |value: &str, binding: Option<&dowe_components::PropBinding>| {
        binding
            .map(|binding| {
                if let Some(item) = context.item_value(&binding.path) {
                    let path = context
                        .item_path(&binding.path)
                        .unwrap_or_else(|| binding.path.clone());
                    format!("state.text(\"{}\", {})", escape_kotlin(&path), item)
                } else {
                    format!(
                        "state.text(\"{}\")",
                        escape_kotlin(&context.signal_path(&binding.path))
                    )
                }
            })
            .unwrap_or_else(|| compose_string_literal(value))
    };
    let name = props
        .name_binding
        .as_ref()
        .map(|binding| dynamic_text("", Some(binding)))
        .unwrap_or_else(|| compose_optional_string(props.name.as_deref()));
    let alt = dynamic_text(&props.alt, props.alt_binding.as_ref());
    let size = dynamic_text(props.size.as_str(), props.size_binding.as_ref());
    output.push_str(&format!(
        "{pad}DoweAvatar(source = {}, name = {}, alt = {}, size = {}, status = {}, bordered = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {}, onClick = {}, hasIcon = {}) {{\n",
        compose_optional_string(props.src.as_deref()),
        name,
        alt,
        size,
        compose_optional_string(props.status.map(|value| value.as_str())),
        props.bordered,
        variant_container(&props.style),
        variant_content(&props.style),
        variant_content(&props.style),
        modifier_for_avatar_style(&props.style.style),
        compose_optional_component_action(
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
                format!("state.text(\"{}\", {item})", escape_kotlin(&path))
            } else {
                format!(
                    "state.text(\"{}\")",
                    escape_kotlin(&context.signal_path(name))
                )
            };
            let color = icon
                .props
                .icon_fill
                .or(icon.props.icon_stroke)
                .map(color_ref)
                .map(str::to_string)
                .unwrap_or_else(|| compose_svg_color(&icon.props.style));
            output.push_str(&format!(
                "{pad}    DoweDynamicIcon(name = {name}, fallback = \"\", modifier = {}, color = {})\n",
                modifier_for_style(&icon.props.style),
                color
            ));
        } else {
            render_compose_side_icon(icon, indent + 4, output);
        }
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
        "{pad}DoweChatBox(state = state, messagesPath = {}, mode = {}, currentUserId = {}, userName = {}, userAvatar = {}, userStatus = {}, assistantName = {}, assistantAvatar = {}, showHeader = {}, placeholder = {}, showAttachments = {}, showVoiceNote = {}, showCamera = {}, loading = {}, sending = {}, streaming = {}, hasMore = {}, actionLabel = {}, actionVisible = {}, onAction = {}, onSend = {}, onLoadMore = {}, onStop = {}, onVoiceNote = {}, onFileAttach = {}, onCameraCapture = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {})\n",
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
        compose_string_literal(&props.action_label),
        compose_optional_bool_signal(props.action_visible.as_deref(), context),
        compose_optional_component_action(props.on_action.as_deref(), None, context),
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
    let icon = empty_icon(props.kind).expect("bundled Empty icon");
    output.push_str(&format!(
        "{pad}DoweEmpty(kind = {}, title = {}, description = {}, actionLabel = {}, action = {}, iconViewBox = {}, iconPaths = {}, backgroundColor = {}, contentColor = {}, accentColor = {}, modifier = {})\n",
        compose_string_literal(props.kind.as_str()),
        compose_optional_string(props.title.as_deref()),
        compose_optional_string(props.description.as_deref()),
        compose_string_literal(&props.action_label),
        compose_optional_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        compose_svg_view_box(&icon.props.view_box),
        compose_svg_paths(&icon.paths),
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
    render_compose_scoped_children(
        &props.style,
        children,
        indent + 4,
        output,
        flow,
        props.style.font.as_ref().or(inherited_font),
        default_family,
        context,
    );
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
        compose_svg_color(&props.style),
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
    let size = text_size(props.title, props);
    output.push_str(&format!(
        "{pad}DoweRichText(marks = {}, fontFamily = {}, fontSize = {size}, contentColor = {}, modifier = {})\n",
        compose_rich_text_marks(marks),
        compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
        text_color(props.title, props),
        modifier_for_style(&props.style),
    ));
}

