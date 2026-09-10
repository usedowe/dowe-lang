pub fn code_node(props: Vec<ComponentProp>, content: String) -> ComponentResult<ViewNode> {
    if content.is_empty() {
        return Err(ComponentError::invalid_prop(
            "content",
            "non-empty multiline string",
        ));
    }
    let mut language = CodeLanguage::Dowe;
    let mut copy_label = "Copy".to_string();
    let mut copied_label = "Copied".to_string();
    let mut template = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "language" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                language = CodeLanguage::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop(
                        "language",
                        "dowe, typescript, javascript, go, rust or python",
                    )
                })?;
            }
            "copyLabel" => copy_label = parse_required_string(&prop.name, &prop.value)?,
            "copiedLabel" => copied_label = parse_required_string(&prop.name, &prop.value)?,
            "template" => template = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Code, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    let source = content;
    let tokens = highlight_code(language, &source);
    let template_segments = if template {
        code_template_segments(language, &source)?
    } else {
        Vec::new()
    };
    Ok(ViewNode::Code {
        props: CodeProps {
            style,
            language,
            source,
            tokens,
            copy_label,
            copied_label,
            template_segments,
        },
    })
}

fn code_template_segments(
    language: CodeLanguage,
    source: &str,
) -> ComponentResult<Vec<CodeTemplateSegment>> {
    let mut segments = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find('{') {
        if start > 0 {
            let text = rest[..start].to_string();
            let tokens = highlight_code(language, &text);
            segments.push(CodeTemplateSegment::Static { text, tokens });
        }
        let tail = &rest[start + 1..];
        let end = tail.find('}').ok_or_else(|| {
            ComponentError::invalid_prop("template", "balanced {signal.path} placeholders")
        })?;
        let path = &tail[..end];
        if path.is_empty()
            || !path.split('.').all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|value| value.is_ascii_alphanumeric() || value == '_')
            })
        {
            return Err(ComponentError::invalid_prop(
                "template",
                "{signal.path} placeholders",
            ));
        }
        segments.push(CodeTemplateSegment::Binding(path.to_string()));
        rest = &tail[end + 1..];
    }
    if !rest.is_empty() {
        let text = rest.to_string();
        let tokens = highlight_code(language, &text);
        segments.push(CodeTemplateSegment::Static { text, tokens });
    }
    Ok(segments)
}

pub fn video_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut poster = None;
    let mut autoplay = false;
    let mut aspect = VideoAspect::Horizontal;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_video_src(&prop.name, &prop.value)?),
            "poster" => poster = Some(parse_video_poster(&prop.name, &prop.value)?),
            "autoplay" => autoplay = parse_static_bool(&prop.name, &prop.value)?,
            "aspect" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                aspect = VideoAspect::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("aspect", "horizontal, vertical or square")
                })?;
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Video, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Video {
        props: VideoProps {
            style,
            src: src.ok_or_else(|| ComponentError::invalid_prop("src", "https URL"))?,
            poster,
            autoplay,
            aspect,
        },
    })
}

pub fn camera_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut facing = CameraFacing::Environment;
    let mut label = "Camera".to_string();
    let mut disabled = false;
    let mut on_start = None;
    let mut on_capture = None;
    let mut on_error = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "facing" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                facing = CameraFacing::from_name(&value)
                    .ok_or_else(|| ComponentError::invalid_prop("facing", "user or environment"))?;
            }
            "label" => label = parse_required_string(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "onStart" => on_start = Some(parse_required_string(&prop.name, &prop.value)?),
            "onCapture" => on_capture = Some(parse_required_string(&prop.name, &prop.value)?),
            "onError" => on_error = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Camera, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Camera {
        props: CameraProps {
            style,
            facing,
            label,
            disabled,
            on_start,
            on_capture,
            on_error,
        },
    })
}

pub fn microphone_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut label = "Microphone".to_string();
    let mut max_duration = None;
    let mut disabled = false;
    let mut on_start = None;
    let mut on_stop = None;
    let mut on_error = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "label" => label = parse_required_string(&prop.name, &prop.value)?,
            "maxDuration" => max_duration = Some(parse_positive_u16(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "onStart" => on_start = Some(parse_required_string(&prop.name, &prop.value)?),
            "onStop" => on_stop = Some(parse_required_string(&prop.name, &prop.value)?),
            "onError" => on_error = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Microphone, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Microphone {
        props: MicrophoneProps {
            style,
            label,
            max_duration,
            disabled,
            on_start,
            on_stop,
            on_error,
        },
    })
}

pub fn iframe_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut reactive_src = None;
    let mut title = None;
    let mut loading = IframeLoading::Lazy;
    let mut allow = Vec::new();
    let mut sandbox = None;
    let mut allow_fullscreen = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => {
                if let Some(path) = reactive_reference(&prop.value) {
                    reactive_src = Some(path);
                } else {
                    src = Some(parse_iframe_src(&prop.name, &prop.value)?);
                }
            }
            "title" => title = Some(parse_required_string(&prop.name, &prop.value)?),
            "loading" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                loading = IframeLoading::from_name(&value)
                    .ok_or_else(|| ComponentError::invalid_prop("loading", "lazy or eager"))?;
            }
            "allow" => {
                allow = parse_iframe_tokens(&prop.name, &prop.value, IFRAME_ALLOW_TOKENS, true)?
            }
            "sandbox" => {
                sandbox = Some(parse_iframe_tokens(
                    &prop.name,
                    &prop.value,
                    IFRAME_SANDBOX_TOKENS,
                    false,
                )?)
            }
            "allowFullscreen" => allow_fullscreen = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::Iframe {
        props: IframeProps {
            style: parse_style_props(BuiltinComponent::Iframe, &style_props, StylePropMode::Box)?,
            src: match (src, reactive_src.as_ref()) {
                (Some(src), _) => src,
                (None, Some(_)) => String::new(),
                (None, None) => {
                    return Err(ComponentError::invalid_prop(
                        "src",
                        "https URL, internal route or Signal path",
                    ));
                }
            },
            reactive_src,
            title: title
                .ok_or_else(|| ComponentError::invalid_prop("title", "non-empty string"))?,
            loading,
            allow,
            sandbox,
            allow_fullscreen,
        },
    })
}

pub fn device_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
) -> ComponentResult<ViewNode> {
    let mut device = DeviceProfile::Mobile;
    let mut bind = None;
    let mut hide_controls = false;
    let mut studio_inspector = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "device" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                device = DeviceProfile::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("device", "mobile, tablet, laptop or monitor")
                })?;
            }
            "bind" => {
                bind = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal string path",
                )?)
            }
            "hideControls" | "hideButtons" => {
                hide_controls = parse_static_bool(&prop.name, &prop.value)?
            }
            "studioInspector" => studio_inspector = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    if children.len() != 1 {
        return Err(ComponentError::invalid_prop_combination(
            "Device requires exactly one Iframe child",
        ));
    }
    let ViewNode::Iframe { props: iframe } = children.into_iter().next().expect("child") else {
        return Err(ComponentError::invalid_prop_combination(
            "Device can only contain one Iframe child",
        ));
    };
    Ok(ViewNode::Device {
        props: DeviceProps {
            style: parse_style_props(BuiltinComponent::Device, &style_props, StylePropMode::Box)?,
            device,
            bind,
            hide_controls,
            studio_inspector,
            options: [
                (DeviceProfile::Mobile, "i-phone"),
                (DeviceProfile::Tablet, "tablet"),
                (DeviceProfile::Laptop, "laptop-minimalistic"),
                (DeviceProfile::Monitor, "monitor"),
            ]
            .into_iter()
            .map(|(profile, name)| {
                let node = icon_component_node(vec![ComponentProp {
                    name: "name".to_string(),
                    value: PropValue::String(format!("{name}-outline")),
                }])?;
                let ViewNode::Svg { props, paths } = node else {
                    unreachable!()
                };
                Ok(DeviceOption {
                    profile,
                    icon: SideNavIcon { props, paths },
                })
            })
            .collect::<ComponentResult<Vec<_>>>()?,
        },
        iframe,
    })
}

