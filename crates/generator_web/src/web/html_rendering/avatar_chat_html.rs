fn render_avatar_html(
    props: &AvatarProps,
    icon: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    let content = if let Some(src) = props.src.as_deref() {
        format!(
            r#"<img class="avatar-image" src="{}" alt="{}">"#,
            escape_attr(src),
            escape_attr(&props.alt)
        )
    } else if let Some(icon) = icon {
        format!(
            r#"<span class="avatar-icon">{}</span>"#,
            render_svg_html(&icon.props, &icon.paths, context)
        )
    } else {
        format!(
            r#"<span class="avatar-name">{}</span>"#,
            escape_html(&avatar_initial(props))
        )
    };
    let status = props
        .status
        .map(|status| {
            format!(
                r#"<span class="avatar-status"><span class="avatar-indicator is-{}"></span></span>"#,
                status.as_str()
            )
        })
        .unwrap_or_default();
    let (tag, tag_attrs, close) = avatar_tags(props, context);
    format!("<{tag}{tag_attrs}>{status}{content}</{close}>")
}

fn avatar_tags(
    props: &AvatarProps,
    context: &ReactiveRenderContext,
) -> (&'static str, String, &'static str) {
    let classes = avatar_classes(props);
    match props.style.navigation.as_ref() {
        Some(action) => (
            "a",
            attrs(
                classes,
                Some(&props.style.element),
                Some(&side_nav_navigation_attrs("avatar", action)),
                context,
            ),
            "a",
        ),
        None if props.style.element.on_click.is_some() => (
            "button",
            attrs(
                classes,
                Some(&props.style.element),
                Some(&format!(
                    r#" type="button" data-dowe-click="{}""#,
                    escape_attr(
                        &context
                            .action_id(props.style.element.on_click.as_deref().expect("onClick"))
                    )
                )),
                context,
            ),
            "button",
        ),
        None => (
            "div",
            attrs(classes, Some(&props.style.element), None, context),
            "div",
        ),
    }
}

fn avatar_initial(props: &AvatarProps) -> String {
    props
        .name
        .as_deref()
        .unwrap_or(&props.alt)
        .chars()
        .next()
        .map(|value| value.to_uppercase().collect::<String>())
        .unwrap_or_else(|| "A".to_string())
}

fn render_avatar_group_html(
    props: &AvatarGroupProps,
    items: &[AvatarGroupItem],
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = format!(
        r#" data-dowe-avatar-group data-dowe-avatar-group-size="{}" data-dowe-avatar-group-variant="{}" data-dowe-avatar-group-scheme="{}" data-dowe-avatar-group-bordered="{}" data-dowe-avatar-group-inline="{}""#,
        props.size.as_str(),
        props
            .style
            .variant
            .unwrap_or(ComponentVariant::Solid)
            .as_str(),
        props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
        props.bordered,
        props.inline
    );
    if let Some(source) = props.items.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-avatar-group-items="{}""#,
            escape_attr(&context.signal_path(source))
        ));
    }
    if let Some(max) = props.max {
        extra.push_str(&format!(r#" data-dowe-avatar-group-max="{max}""#));
    }
    let visible_count = props
        .max
        .map(|max| usize::from(max).min(items.len()))
        .unwrap_or(items.len());
    let mut html = format!(
        "<div{}>",
        attrs(
            avatar_group_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        )
    );
    html.push_str(r#"<div class="avatar-group-list" data-dowe-avatar-group-list>"#);
    for item in items.iter().take(visible_count) {
        html.push_str(&render_avatar_group_item_html(props, item, context));
    }
    if visible_count < items.len() {
        html.push_str(&format!(
            r#"<span class="avatar-group-counter avatar-{} is-{} is-{}">+{}</span>"#,
            props.size.as_str(),
            props
                .style
                .variant
                .unwrap_or(ComponentVariant::Solid)
                .as_str(),
            props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
            items.len() - visible_count
        ));
    }
    html.push_str("</div></div>");
    html
}

fn render_avatar_group_item_html(
    group: &AvatarGroupProps,
    item: &AvatarGroupItem,
    context: &ReactiveRenderContext,
) -> String {
    let mut style = group.style.clone();
    style.element = ElementProps {
        on_click: item.on_click.clone(),
        ..ElementProps::default()
    };
    style.navigation = item.navigation.clone();
    let avatar = AvatarProps {
        style,
        src: item.src.clone(),
        name: item.name.clone(),
        alt: item
            .alt
            .clone()
            .or_else(|| item.name.clone())
            .unwrap_or_default(),
        size: group.size,
        status: None,
        bordered: group.bordered,
    };
    render_avatar_html(&avatar, None, context)
}

fn render_chat_box_html(props: &ChatBoxProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" data-dowe-chatbox data-dowe-chatbox-messages="{}" data-dowe-chatbox-current-user="{}" data-dowe-chatbox-mode="{}" data-dowe-chatbox-placeholder="{}""#,
        escape_attr(&context.signal_path(&props.messages)),
        escape_attr(&props.current_user_id),
        props.mode.as_str(),
        escape_attr(&props.placeholder)
    );
    for (name, value) in [
        ("loading", props.loading.as_deref()),
        ("sending", props.sending.as_deref()),
        ("streaming", props.streaming.as_deref()),
        ("has-more", props.has_more.as_deref()),
    ] {
        if let Some(value) = value {
            extra.push_str(&format!(
                r#" data-dowe-chatbox-{name}="{}""#,
                escape_attr(&context.signal_path(value))
            ));
        }
    }
    for (name, value) in [
        ("send", props.on_send.as_deref()),
        ("load-more", props.on_load_more.as_deref()),
        ("stop", props.on_stop.as_deref()),
        ("voice-note", props.on_voice_note.as_deref()),
        ("file-attach", props.on_file_attach.as_deref()),
        ("camera-capture", props.on_camera_capture.as_deref()),
    ] {
        if let Some(value) = value {
            extra.push_str(&format!(
                r#" data-dowe-chatbox-on-{name}="{}""#,
                escape_attr(&context.action_id(value))
            ));
        }
    }
    let header = if props.show_header {
        render_chat_box_header_html(props)
    } else {
        String::new()
    };
    let footer = render_chat_box_footer_html(props);
    format!(
        r#"<section{}>{}<div class="chat-box-body" data-dowe-chatbox-body><div class="chat-box-messages" data-dowe-chatbox-list></div><div class="chat-box-typing" data-dowe-chatbox-typing hidden><span></span><span></span><span></span></div></div>{}</section>"#,
        attrs(
            chat_box_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        header,
        footer
    )
}

fn render_chat_box_header_html(props: &ChatBoxProps) -> String {
    let avatar = props
        .assistant_avatar
        .as_deref()
        .map(|src| {
            format!(
                r#"<img class="chat-box-avatar" src="{}" alt="">"#,
                escape_attr(src)
            )
        })
        .unwrap_or_else(|| {
            format!(
                r#"<span class="chat-box-avatar">{}</span>"#,
                escape_html(
                    &props
                        .assistant_name
                        .chars()
                        .next()
                        .map(|value| value.to_uppercase().collect::<String>())
                        .unwrap_or_else(|| "A".to_string())
                )
            )
        });
    format!(
        r#"<header class="chat-box-header"><div class="chat-box-user">{avatar}<div class="chat-box-user-copy"><strong>{}</strong><span>{}</span></div></div><div class="chat-box-header-actions"><button type="button" class="chat-box-icon" aria-label="Search">⌕</button><button type="button" class="chat-box-icon" aria-label="More">⋯</button></div></header>"#,
        escape_html(&props.assistant_name),
        escape_html(&props.user_status)
    )
}

fn render_chat_box_footer_html(props: &ChatBoxProps) -> String {
    let mut actions = String::new();
    if props.show_voice_note {
        actions.push_str(r#"<button type="button" class="chat-box-tool" data-dowe-chatbox-voice aria-label="Voice note">◉</button>"#);
    }
    if props.show_attachments {
        actions.push_str(r#"<button type="button" class="chat-box-tool" data-dowe-chatbox-file aria-label="Attach file">＋</button>"#);
    }
    if props.show_camera {
        actions.push_str(r#"<button type="button" class="chat-box-tool" data-dowe-chatbox-camera aria-label="Camera">▣</button>"#);
    }
    format!(
        r#"<footer class="chat-box-footer"><div class="chat-box-input-wrap">{actions}<textarea class="chat-box-input" rows="1" placeholder="{}" data-dowe-chatbox-input></textarea><button type="button" class="chat-box-send" data-dowe-chatbox-send aria-label="Send">➤</button><button type="button" class="chat-box-stop" data-dowe-chatbox-stop aria-label="Stop" hidden>■</button></div></footer>"#,
        escape_attr(&props.placeholder)
    )
}

fn render_empty_html(props: &EmptyProps, context: &ReactiveRenderContext) -> String {
    let mut root_element = props.style.element.clone();
    root_element.on_click = None;
    let title = props
        .title
        .as_deref()
        .unwrap_or_else(|| empty_default_title(props.kind));
    let description = props
        .description
        .as_deref()
        .unwrap_or_else(|| empty_default_description(props.kind));
    let action = if props.style.navigation.is_some() || props.style.element.on_click.is_some() {
        let mut action_props = props.style.clone();
        action_props.element = ElementProps {
            on_click: props.style.element.on_click.clone(),
            ..ElementProps::default()
        };
        let (open, close) = button_tags(&action_props, context);
        format!(
            r#"<div class="empty-actions">{}{}</div>"#,
            open + &escape_html(&props.action_label),
            close
        )
    } else {
        String::new()
    };
    format!(
        r#"<div{}>{}<div class="empty-content"><h3 class="empty-title">{}</h3><p class="empty-description">{}</p></div>{}</div>"#,
        attrs(empty_classes(props), Some(&root_element), None, context),
        empty_icon_html(props.kind),
        escape_html(title),
        escape_html(description),
        action
    )
}

fn render_marquee_html(
    props: &MarqueeProps,
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let content = children
        .iter()
        .map(|child| render_html_with_context(child, children_html, context))
        .collect::<String>();
    let extra = format!(
        r#" style="--dowe-marquee-gap:{};--dowe-marquee-fade:var(--dowe-{});""#,
        scale_rem(props.gap),
        props.fade_color.as_str()
    );
    format!(
        r#"<div{}><div class="marquee-track"><div class="marquee-content">{}</div><div class="marquee-content" aria-hidden="true">{}</div></div></div>"#,
        attrs(
            marquee_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        content,
        content
    )
}

fn render_type_writer_html(
    props: &TypeWriterProps,
    items: &[TypeWriterItem],
    context: &ReactiveRenderContext,
) -> String {
    let texts = items
        .iter()
        .map(|item| js_string_literal(&item.text))
        .collect::<Vec<_>>()
        .join(",");
    let first = items
        .first()
        .map(|item| item.text.as_str())
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-typewriter data-dowe-typewriter-texts="[{}]" data-dowe-typewriter-type-speed="{}" data-dowe-typewriter-delete-speed="{}" data-dowe-typewriter-after-typed="{}" data-dowe-typewriter-after-deleted="{}" data-dowe-typewriter-repeat="{}""#,
        escape_attr(&texts),
        props.type_speed,
        props.delete_speed,
        props.after_typed,
        props.after_deleted,
        props.repeat
    );
    format!(
        r#"<span{}><span class="typewriter-text" data-dowe-typewriter-text>{}</span><span class="typewriter-caret" aria-hidden="true"></span></span>"#,
        attrs(
            type_writer_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        escape_html(first)
    )
}

fn render_rich_text_html(
    props: &TextProps,
    marks: &[RichTextMark],
    context: &ReactiveRenderContext,
) -> String {
    let content = marks
        .iter()
        .map(|mark| {
            format!(
                r#"<span class="rich-mark rich-mark-{} is-{}">{}</span>"#,
                mark.style.as_str(),
                mark.color.as_str(),
                escape_html(&mark.text)
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut extra = String::new();
    if let Some(key) = props.i18n.as_ref() {
        extra.push_str(&format!(r#" data-dowe-i18n="{}""#, escape_attr(key)));
    }
    format!(
        "<p{}>{}</p>",
        attrs(
            rich_text_classes(props),
            Some(&props.style.element),
            (!extra.is_empty()).then_some(extra.as_str()),
            context
        ),
        content
    )
}
