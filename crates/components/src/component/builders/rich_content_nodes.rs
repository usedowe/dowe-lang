pub fn avatar_group_item_component(props: Vec<ComponentProp>) -> ComponentResult<AvatarGroupItem> {
    let mut src = None;
    let mut name = None;
    let mut alt = None;
    let mut href = None;
    let mut navigate = None;
    let mut history = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_avatar_src(&prop.name, &prop.value)?),
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "alt" => alt = Some(parse_required_string(&prop.name, &prop.value)?),
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "history" => history = Some(parse_history_prop(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::AvatarGroup,
                    &prop.name,
                ));
            }
        }
    }
    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `onClick` cannot be used on the same AvatarGroup item",
        ));
    }
    if href.is_some() && history.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `history` cannot be used on the same AvatarGroup item",
        ));
    }
    let navigation = if history.is_some() {
        if navigate.is_some() || target.is_some() || external_mode.is_some() {
            return Err(ComponentError::invalid_prop_combination(
                "`navigate`, `target` and `externalMode` require `href` on AvatarGroup item",
            ));
        }
        history
    } else {
        parse_link_navigation_props("AvatarGroup item", href, navigate, target, external_mode)?
    };
    Ok(AvatarGroupItem {
        src,
        name,
        alt,
        on_click,
        navigation,
    })
}

pub fn avatar_group_component_node(
    props: Vec<ComponentProp>,
    items: Vec<AvatarGroupItem>,
) -> ComponentResult<ViewNode> {
    let mut source = None;
    let mut size = ButtonSize::Md;
    let mut max = None;
    let mut auto_fit = false;
    let mut inline = false;
    let mut bordered = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "items" => {
                source = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal array path",
                )?)
            }
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "max" => max = Some(parse_positive_u16(&prop.name, &prop.value)?),
            "autoFit" => auto_fit = parse_static_bool(&prop.name, &prop.value)?,
            "inline" => inline = parse_static_bool(&prop.name, &prop.value)?,
            "bordered" => bordered = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::AvatarGroup)),
            _ => style_props.push(prop),
        }
    }
    if source.is_none() && items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "AvatarGroup requires `items` or at least one item entry",
        ));
    }
    let mut style = parse_variant_props(BuiltinComponent::AvatarGroup, &style_props)?;
    require_solid_or_soft(BuiltinComponent::AvatarGroup, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::AvatarGroup {
        props: AvatarGroupProps {
            style,
            items: source,
            size,
            max,
            auto_fit,
            inline,
            bordered,
        },
        items,
    })
}

pub fn chat_box_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut messages = None;
    let mut mode = ChatBoxMode::Conversation;
    let mut current_user_id = String::new();
    let mut user_name = String::new();
    let mut user_avatar = None;
    let mut user_status = "Online".to_string();
    let mut assistant_name = "Assistant".to_string();
    let mut assistant_avatar = None;
    let mut show_header = true;
    let mut placeholder = "Type a message...".to_string();
    let mut show_attachments = true;
    let mut show_voice_note = true;
    let mut show_camera = false;
    let mut loading = None;
    let mut sending = None;
    let mut streaming = None;
    let mut has_more = None;
    let mut on_send = None;
    let mut on_load_more = None;
    let mut on_stop = None;
    let mut on_voice_note = None;
    let mut on_file_attach = None;
    let mut on_camera_capture = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "messages" => {
                messages = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal array path",
                )?)
            }
            "mode" => mode = parse_chat_box_mode(&prop.name, &prop.value)?,
            "currentUserId" => current_user_id = parse_static_string(&prop.name, &prop.value)?,
            "userName" => user_name = parse_static_string(&prop.name, &prop.value)?,
            "userAvatar" => user_avatar = Some(parse_avatar_src(&prop.name, &prop.value)?),
            "userStatus" => user_status = parse_static_string(&prop.name, &prop.value)?,
            "assistantName" => assistant_name = parse_static_string(&prop.name, &prop.value)?,
            "assistantAvatar" => {
                assistant_avatar = Some(parse_avatar_src(&prop.name, &prop.value)?)
            }
            "showHeader" => show_header = parse_static_bool(&prop.name, &prop.value)?,
            "placeholder" => placeholder = parse_static_string(&prop.name, &prop.value)?,
            "showAttachments" => show_attachments = parse_static_bool(&prop.name, &prop.value)?,
            "showVoiceNote" => show_voice_note = parse_static_bool(&prop.name, &prop.value)?,
            "showCamera" => show_camera = parse_static_bool(&prop.name, &prop.value)?,
            "loading" => {
                loading = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal bool path",
                )?)
            }
            "sending" => {
                sending = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal bool path",
                )?)
            }
            "streaming" => {
                streaming = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal bool path",
                )?)
            }
            "hasMore" => {
                has_more = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal bool path",
                )?)
            }
            "onSend" => on_send = Some(parse_required_string(&prop.name, &prop.value)?),
            "onLoadMore" => on_load_more = Some(parse_required_string(&prop.name, &prop.value)?),
            "onStop" => on_stop = Some(parse_required_string(&prop.name, &prop.value)?),
            "onVoiceNote" => on_voice_note = Some(parse_required_string(&prop.name, &prop.value)?),
            "onFileAttach" => {
                on_file_attach = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "onCameraCapture" => {
                on_camera_capture = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "color" => return Err(scheme_prop_error(BuiltinComponent::ChatBox)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::ChatBox, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::ChatBox {
        props: ChatBoxProps {
            style,
            messages: messages
                .ok_or_else(|| ComponentError::invalid_prop("messages", "signal array path"))?,
            mode,
            current_user_id,
            user_name,
            user_avatar,
            user_status,
            assistant_name,
            assistant_avatar,
            show_header,
            placeholder,
            show_attachments,
            show_voice_note,
            show_camera,
            loading,
            sending,
            streaming,
            has_more,
            on_send,
            on_load_more,
            on_stop,
            on_voice_note,
            on_file_attach,
            on_camera_capture,
        },
    })
}

pub fn empty_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut kind = EmptyKind::Template;
    let mut title = None;
    let mut description = None;
    let mut action_label = "View more".to_string();
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "type" => kind = parse_empty_kind(&prop.name, &prop.value)?,
            "title" => title = Some(parse_static_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_static_string(&prop.name, &prop.value)?),
            "actionLabel" => action_label = parse_static_string(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Empty)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Empty, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Soft);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Empty {
        props: EmptyProps {
            style,
            kind,
            title,
            description,
            action_label,
        },
    })
}

pub fn marquee_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Marquee requires at least one child",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Marquee, &children, allow_children)?;
    let mut speed = MarqueeSpeed::Normal;
    let mut pause_on_hover = false;
    let mut reverse = false;
    let mut orientation = MarqueeOrientation::Horizontal;
    let mut fade = false;
    let mut fade_color = ColorToken::Background;
    let mut gap = ScaleValue::from_half_steps(0);
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "speed" => speed = parse_marquee_speed(&prop.name, &prop.value)?,
            "pauseOnHover" => pause_on_hover = parse_static_bool(&prop.name, &prop.value)?,
            "reverse" => reverse = parse_static_bool(&prop.name, &prop.value)?,
            "orientation" => orientation = parse_marquee_orientation(&prop.name, &prop.value)?,
            "fade" => fade = parse_static_bool(&prop.name, &prop.value)?,
            "fadeColor" => fade_color = parse_single_color_token(&prop.name, &prop.value)?,
            "gap" => gap = parse_static_scale(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::Marquee {
        props: MarqueeProps {
            style: parse_style_props(
                BuiltinComponent::Marquee,
                &style_props,
                StylePropMode::Variant,
            )?,
            speed,
            pause_on_hover,
            reverse,
            orientation,
            fade,
            fade_color,
            gap,
        },
        children,
    })
}

pub fn type_writer_item_component(props: Vec<ComponentProp>) -> ComponentResult<TypeWriterItem> {
    let mut text = None;
    for prop in props {
        match prop.name.as_str() {
            "text" => text = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::TypeWriter,
                    &prop.name,
                ));
            }
        }
    }
    Ok(TypeWriterItem {
        text: text.ok_or_else(|| ComponentError::invalid_prop("text", "non-empty string"))?,
    })
}

pub fn type_writer_component_node(
    props: Vec<ComponentProp>,
    items: Vec<TypeWriterItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "TypeWriter requires at least one item",
        ));
    }
    let mut type_speed = 100;
    let mut delete_speed = 50;
    let mut after_typed = 1000;
    let mut after_deleted = 500;
    let mut repeat = true;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "typeSpeed" => type_speed = parse_positive_u64(&prop.name, &prop.value)?,
            "deleteSpeed" => delete_speed = parse_positive_u64(&prop.name, &prop.value)?,
            "afterTyped" => after_typed = parse_non_negative_u64(&prop.name, &prop.value)?,
            "afterDeleted" => after_deleted = parse_non_negative_u64(&prop.name, &prop.value)?,
            "repeat" => repeat = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::TypeWriter {
        props: TypeWriterProps {
            style: parse_style_props(
                BuiltinComponent::TypeWriter,
                &style_props,
                StylePropMode::Text,
            )?,
            type_speed,
            delete_speed,
            after_typed,
            after_deleted,
            repeat,
        },
        items,
    })
}

pub fn rich_text_mark_component(props: Vec<ComponentProp>) -> ComponentResult<RichTextMark> {
    let mut text = None;
    let mut style = RichTextMarkStyle::Mark;
    let mut color = ColorFamily::Info;
    for prop in props {
        match prop.name.as_str() {
            "text" => text = Some(parse_required_string(&prop.name, &prop.value)?),
            "style" => style = parse_rich_text_mark_style(&prop.name, &prop.value)?,
            "scheme" => {
                color = parse_family_prop(BuiltinComponent::RichText, &prop.name, &prop.value)?
            }
            "color" => return Err(scheme_prop_error(BuiltinComponent::RichText)),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::RichText,
                    &prop.name,
                ));
            }
        }
    }
    Ok(RichTextMark {
        text: text.ok_or_else(|| ComponentError::invalid_prop("text", "non-empty string"))?,
        style,
        color,
    })
}

pub fn rich_text_component_node(
    props: Vec<ComponentProp>,
    marks: Vec<RichTextMark>,
) -> ComponentResult<ViewNode> {
    if marks.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "RichText requires at least one mark entry",
        ));
    }
    Ok(ViewNode::RichText {
        props: parse_text_props(BuiltinComponent::RichText, &props)?,
        marks,
    })
}
