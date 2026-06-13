fn grid_classes(props: &GridProps) -> Vec<String> {
    let mut classes = vec!["grid".to_string()];
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(&mut classes, "grid-cols", props.columns.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(&mut classes, "grid-rows", props.rows.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(
        &mut classes,
        "grid-justify",
        props.justify.as_ref(),
        |value| value.as_str().to_string(),
    );
    append_responsive_classes(&mut classes, "grid-align", props.align.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(&mut classes, "gap", props.gap.as_ref(), |value| {
        value.class_suffix()
    });
    classes
}

fn variant_classes(base: &str, props: &VariantProps) -> Vec<String> {
    let mut classes = vec![base.to_string()];
    if base == "button" {
        classes.push(format!(
            "button-{}",
            props.size.unwrap_or(ButtonSize::Md).as_str()
        ));
    }
    append_style_classes(&mut classes, &props.style);
    if base == "card" {
        append_container_visual_classes(&mut classes, &props.style);
    }
    classes.push(format!(
        "is-{}",
        props.variant.unwrap_or(ComponentVariant::Solid).as_str()
    ));
    classes.push(format!(
        "is-{}",
        props.color.unwrap_or(ColorFamily::Primary).as_str()
    ));
    classes
}

fn bar_classes(base: &str, props: &BarProps) -> Vec<String> {
    let mut classes = variant_classes(base, &props.style);
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    if props.blurred {
        classes.push("is-blurred".to_string());
    }
    if props.floating {
        classes.push("is-floating".to_string());
    }
    classes
}

fn bar_content_classes(base: &str, props: &BarProps) -> Vec<String> {
    let mut classes = vec![format!("{base}-content")];
    if props.boxed {
        classes.push("is-boxed".to_string());
    }
    classes
}

fn side_nav_classes(base: &str, props: &SideNavProps) -> Vec<String> {
    let mut classes = variant_classes(base, &props.style);
    classes.push(format!("{base}-{}", props.size.as_str()));
    if props.wide {
        classes.push("is-wide".to_string());
    }
    classes
}

fn nav_menu_classes(props: &NavMenuProps) -> Vec<String> {
    let mut classes = variant_classes("navmenu", &props.style);
    classes.push(format!("navmenu-{}", props.size.as_str()));
    classes
}

fn scaffold_classes(props: &ScaffoldProps) -> Vec<String> {
    let mut classes = vec!["scaffold".to_string()];
    append_style_classes(&mut classes, &props.style);
    if props.boxed {
        classes.push("is-boxed".to_string());
    }
    classes
}

fn tabs_classes(props: &TabsProps) -> Vec<String> {
    let mut classes = vec!["tabs".to_string(), format!("is-{}", props.position.as_str())];
    append_style_classes(&mut classes, &props.style);
    classes
}

fn tabs_list_classes(props: &TabsProps) -> Vec<String> {
    vec![
        "tabs-list".to_string(),
        format!("is-{}", props.variant.as_str()),
        format!("is-{}", props.color.as_str()),
    ]
}

fn drawer_panel_classes(props: &DrawerProps) -> Vec<String> {
    let mut classes = vec!["drawer-panel".to_string()];
    append_show_classes(&mut classes, props.style.element.show.as_ref());
    classes
}

fn drawer_classes(props: &DrawerProps) -> Vec<String> {
    let mut classes = variant_classes("drawer", &props.style);
    classes.push(format!("is-{}", props.position.as_str()));
    classes
}

fn avatar_classes(props: &AvatarProps) -> Vec<String> {
    let mut classes = variant_classes("avatar", &props.style);
    classes.push(format!("avatar-{}", props.size.as_str()));
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    if props.style.element.on_click.is_some() || props.style.navigation.is_some() {
        classes.push("is-clickable".to_string());
    }
    classes
}

fn avatar_group_classes(props: &AvatarGroupProps) -> Vec<String> {
    let mut classes = variant_classes("avatar-group", &props.style);
    classes.push(format!("avatar-group-{}", props.size.as_str()));
    if props.inline {
        classes.push("is-inline".to_string());
    }
    if props.auto_fit {
        classes.push("is-auto-fit".to_string());
    }
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    classes
}

fn chat_box_classes(props: &ChatBoxProps) -> Vec<String> {
    let mut classes = variant_classes("chat-box", &props.style);
    classes.push(format!("is-{}", props.mode.as_str()));
    classes
}

fn empty_classes(props: &EmptyProps) -> Vec<String> {
    let mut classes = variant_classes("empty", &props.style);
    classes.push(format!("is-{}", props.kind.as_str()));
    classes
}

fn marquee_classes(props: &MarqueeProps) -> Vec<String> {
    let mut classes = vec![
        "marquee".to_string(),
        format!("is-{}", props.orientation.as_str()),
        format!("is-{}", props.speed.as_str()),
    ];
    append_style_classes(&mut classes, &props.style);
    if props.pause_on_hover {
        classes.push("pause-on-hover".to_string());
    }
    if props.reverse {
        classes.push("is-reverse".to_string());
    }
    if props.fade {
        classes.push("has-fade".to_string());
    }
    classes
}

fn type_writer_classes(props: &TypeWriterProps) -> Vec<String> {
    let mut classes = vec!["typewriter".to_string()];
    append_style_classes(&mut classes, &props.style);
    classes
}

fn rich_text_classes(props: &TextProps) -> Vec<String> {
    let mut classes = vec!["rich-text".to_string()];
    if let Some(size) = &props.size {
        append_responsive_classes(&mut classes, "text", Some(size), |value| {
            value.as_str().to_string()
        });
    } else {
        classes.push("text-md".to_string());
    }
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(&mut classes, "weight", props.weight.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(
        &mut classes,
        "tracking",
        props.letter_spacing.as_ref(),
        |value| value.as_str().to_string(),
    );
    classes
}

fn record_classes(props: &RecordProps) -> Vec<String> {
    let mut classes = variant_classes("media", &props.style);
    classes.push("record".to_string());
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    classes
}

fn toggle_group_classes(props: &ToggleGroupProps) -> Vec<String> {
    let mut classes = variant_classes("toggle-group", &props.style);
    classes.push(format!("toggle-group-{}", props.size.as_str()));
    if props.wide {
        classes.push("is-wide".to_string());
    }
    if props.vertical {
        classes.push("is-vertical".to_string());
    }
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    classes
}

fn collapsible_classes(props: &CollapsibleProps) -> Vec<String> {
    let mut classes = variant_classes("collapsible", &props.style);
    if props.default_open {
        classes.push("is-open".to_string());
    }
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    classes
}

fn countdown_classes(props: &CountdownProps) -> Vec<String> {
    let mut classes = variant_classes("countdown", &props.style);
    classes.push(format!("countdown-{}", props.size.as_str()));
    classes
}

fn map_classes(props: &MapProps) -> Vec<String> {
    let mut classes = variant_classes("map", &props.style);
    if !props.interactive {
        classes.push("is-static".to_string());
    }
    classes
}

fn badge_classes(props: &BadgeProps) -> Vec<String> {
    let mut classes = variant_classes("badge", &props.style);
    classes.push(format!("is-{}", props.position.as_str()));
    classes
}

fn chip_classes(props: &ChipProps) -> Vec<String> {
    let mut classes = variant_classes("chip", &props.style);
    classes.push(format!(
        "chip-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.on_close.is_some() {
        classes.push("has-close".to_string());
    }
    classes
}

fn skeleton_classes(props: &SkeletonProps) -> Vec<String> {
    let mut classes = vec![
        "skeleton".to_string(),
        format!("is-{}", props.variant.as_str()),
        format!("is-{}", props.animation.as_str()),
    ];
    append_style_classes(&mut classes, &props.style);
    classes
}

fn modal_panel_classes(props: &ModalProps) -> Vec<String> {
    let mut classes = vec!["modal-dialog".to_string()];
    append_show_classes(&mut classes, props.style.element.show.as_ref());
    classes
}

fn modal_classes(props: &ModalProps) -> Vec<String> {
    variant_classes("modal", &props.style)
}

fn alert_dialog_modal_props(props: &AlertDialogProps) -> ModalProps {
    ModalProps {
        style: props.style.clone(),
        open: props.open.clone(),
        on_close: props.on_cancel.clone(),
        disable_overlay_close: true,
        hide_close_button: true,
    }
}

fn tooltip_classes(props: &TooltipProps) -> Vec<String> {
    let mut classes = vec!["tooltip".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    classes
}

fn tooltip_popover_classes(props: &TooltipProps) -> Vec<String> {
    vec![
        "tooltip-popover".to_string(),
        format!("is-{}", props.style.variant.unwrap_or(ComponentVariant::Solid).as_str()),
        format!("is-{}", props.style.color.unwrap_or(ColorFamily::Muted).as_str()),
        format!("position-{}", props.position.as_str()),
    ]
}

fn toast_classes(props: &ToastProps) -> Vec<String> {
    let mut classes = variant_classes("toast", &props.style);
    classes.push(format!("is-{}", props.position.as_str()));
    classes
}

fn dropdown_classes(props: &DropdownProps) -> Vec<String> {
    let mut classes = vec!["dropdown".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    classes
}

fn dropdown_popover_classes(props: &DropdownProps) -> Vec<String> {
    vec![
        "dropdown-popover".to_string(),
        format!("is-{}", props.style.variant.unwrap_or(ComponentVariant::Solid).as_str()),
        format!("is-{}", props.style.color.unwrap_or(ColorFamily::Primary).as_str()),
    ]
}

fn command_panel_classes(props: &CommandProps) -> Vec<String> {
    let mut classes = vec!["command-dialog".to_string()];
    append_show_classes(&mut classes, props.style.element.show.as_ref());
    classes
}

fn command_classes(props: &CommandProps) -> Vec<String> {
    variant_classes("command", &props.style)
}

fn text_classes(base: &str, props: &TextProps) -> Vec<String> {
    let mut classes = Vec::new();
    if let Some(size) = &props.size {
        append_responsive_classes(&mut classes, base, Some(size), |value| {
            value.as_str().to_string()
        });
    } else {
        classes.push(format!("{base}-md"));
    }
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(&mut classes, "weight", props.weight.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(
        &mut classes,
        "tracking",
        props.letter_spacing.as_ref(),
        |value| value.as_str().to_string(),
    );
    classes
}

fn svg_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["svg".to_string()];
    append_style_classes(&mut classes, props);
    classes
}

fn video_classes(props: &VideoProps) -> Vec<String> {
    let mut classes = variant_classes("video", &props.style);
    classes.insert(1, props.aspect.as_str().to_string());
    classes
}

fn candlestick_classes(props: &CandlestickProps) -> Vec<String> {
    variant_classes("candlestick", &props.style)
}

fn table_wrapper_classes(props: &TableProps) -> Vec<String> {
    let mut classes = vec!["table-wrapper".to_string()];
    append_style_classes(&mut classes, &props.style.style);
    classes
}

fn table_classes(props: &TableProps) -> Vec<String> {
    let mut classes = vec![
        "table".to_string(),
        format!("is-{}", props.size.as_str()),
        format!(
            "is-{}",
            props.style.variant.unwrap_or(ComponentVariant::Solid).as_str()
        ),
        format!(
            "is-{}",
            props.style.color.unwrap_or(ColorFamily::Surface).as_str()
        ),
    ];
    if props.striped {
        classes.push("is-striped".to_string());
    }
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    if props.dividers {
        classes.push("has-dividers".to_string());
    }
    classes
}

fn divider_classes(props: &DividerProps) -> Vec<String> {
    let mut classes = vec![
        "divider".to_string(),
        format!("divider-{}", props.orientation.as_str()),
        format!("is-{}", props.color.as_str()),
    ];
    append_style_classes(&mut classes, &props.style);
    classes
}

fn append_style_classes(classes: &mut Vec<String>, props: &StyleProps) {
    append_show_classes(classes, props.element.show.as_ref());
    append_responsive_classes(classes, "font", props.font.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(classes, "bg", props.bg.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(classes, "color", props.text.as_ref(), |value| {
        value.as_str().to_string()
    });
    if let Some(animation) = props.animation
        && animation != ViewAnimation::None
    {
        classes.push(format!("animate-{}", animation.class_suffix()));
    }
    append_responsive_classes(classes, "p", props.spacing.p.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "px", props.spacing.px.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "py", props.spacing.py.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pl", props.spacing.pl.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pr", props.spacing.pr.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pt", props.spacing.pt.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pb", props.spacing.pb.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "w", props.sizing.w.as_ref(), size_suffix);
    append_responsive_classes(classes, "h", props.sizing.h.as_ref(), size_suffix);
    append_responsive_classes(classes, "min-w", props.sizing.min_w.as_ref(), size_suffix);
    append_responsive_classes(classes, "min-h", props.sizing.min_h.as_ref(), size_suffix);
    append_responsive_classes(classes, "rounded", props.rounded.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(classes, "border", props.border.as_ref(), |value| {
        value.0.to_string()
    });
    append_responsive_classes(
        classes,
        "col-span",
        props.grid_item.col_span.as_ref(),
        |value| value.0.to_string(),
    );
    append_responsive_classes(
        classes,
        "row-span",
        props.grid_item.row_span.as_ref(),
        |value| value.0.to_string(),
    );
}

fn append_show_classes(classes: &mut Vec<String>, value: Option<&VisibilityCondition>) {
    if let Some(VisibilityCondition::Static(value)) = value {
        append_responsive_classes(classes, "show", Some(value), |value| value.to_string());
    }
}

fn append_container_visual_classes(classes: &mut Vec<String>, props: &StyleProps) {
    if let Some(background) = props.background.as_ref() {
        classes.push("has-background".to_string());
        append_responsive_classes(classes, "background", Some(background), |value| {
            value.as_str().to_string()
        });
    }
    if let Some(cover) = props.cover.as_ref() {
        classes.push("has-cover".to_string());
        append_responsive_classes(classes, "cover", Some(cover), |value| cover_suffix(value));
    }
    if let Some(overlay) = props.overlay.as_ref() {
        classes.push("has-overlay".to_string());
        append_responsive_classes(classes, "overlay", Some(overlay), |value| {
            overlay_suffix(value)
        });
    }
}

fn append_responsive_classes<T, F>(
    classes: &mut Vec<String>,
    prefix: &str,
    value: Option<&ResponsiveValue<T>>,
    suffix: F,
) where
    F: Fn(&T) -> String,
{
    let Some(value) = value else {
        return;
    };

    for entry in &value.entries {
        let class_name = format!("{prefix}-{}", suffix(&entry.value));
        if entry.breakpoint == Breakpoint::Xs {
            classes.push(class_name);
        } else {
            classes.push(format!("{}:{class_name}", entry.breakpoint.as_str()));
        }
    }
}

fn size_suffix(value: &SizeValue) -> String {
    match value {
        SizeValue::Scale(value) => value.class_suffix(),
        SizeValue::Full => "full".to_string(),
    }
}

fn button_tags(props: &VariantProps, context: &ReactiveRenderContext) -> (String, &'static str) {
    let classes = variant_classes("button", props);
    match props.navigation.as_ref() {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => {
            let href = internal_href(path, fragment.as_deref());
            (
                format!(
                    "<a{}>",
                    attrs(
                        classes,
                        Some(&props.element),
                        Some(&navigation_attrs(&href, *operation)),
                        context
                    )
                ),
                "</a>",
            )
        }
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => {
            let href = format!("#{fragment}");
            (
                format!(
                    "<a{}>",
                    attrs(
                        classes,
                        Some(&props.element),
                        Some(&navigation_attrs(&href, *operation)),
                        context
                    )
                ),
                "</a>",
            )
        }
        Some(NavigationAction::External {
            url,
            web_target,
            native_external_mode,
        }) => (
            format!(
                "<a{}>",
                attrs(
                    classes,
                    Some(&props.element),
                    Some(&external_attrs(url, *web_target, *native_external_mode)),
                    context
                )
            ),
            "</a>",
        ),
        Some(NavigationAction::Back) => (
            format!(
                r#"<button{}>"#,
                attrs(
                    classes,
                    Some(&props.element),
                    Some(r#" type="button" data-dowe-history="back""#),
                    context
                )
            ),
            "</button>",
        ),
        None => (
            format!(
                r#"<button{}>"#,
                attrs(
                    classes,
                    Some(&props.element),
                    Some(r#" type="button""#),
                    context,
                )
            ),
            "</button>",
        ),
    }
}

fn internal_href(path: &str, fragment: Option<&str>) -> String {
    if let Some(fragment) = fragment {
        format!("{path}#{fragment}")
    } else {
        path.to_string()
    }
}

fn render_input_html(props: &VariantProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="input"{}{}>"#,
        input_placeholder_attr(props),
        bind_attr(props.element.bind.as_deref(), context)
    );
    if props.label.is_some() && !props.label_floating {
        let control = format!(
            "<span{}>{}</span>",
            attrs(
                variant_classes("control", props),
                Some(&props.element),
                None,
                context
            ),
            input
        );
        return format!(
            r#"<label class="field"><span class="field-label">{}</span>{}</label>"#,
            escape_html(props.label.as_deref().unwrap_or_default()),
            control
        );
    }
    if props.label_floating {
        let mut classes = variant_classes("control", props);
        classes.push("is-floating".to_string());
        return format!(
            "<label{}>{}{}</label>",
            attrs(classes, Some(&props.element), None, context),
            floating_label_html(props),
            input
        );
    }
    format!(
        r#"<div{}>{}</div>"#,
        attrs(
            variant_classes("control", props),
            Some(&props.element),
            None,
            context
        ),
        input
    )
}

fn render_select_html(
    props: &VariantProps,
    options: &[SelectOption],
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("control", props);
    classes.push("select-control".to_string());
    if props.label_floating {
        classes.push("is-floating".to_string());
    }
    let placeholder = props.placeholder.as_deref().unwrap_or("Select an option");
    let extra = format!(
        r#" type="button" role="combobox" aria-haspopup="listbox" aria-expanded="false" data-dowe-select data-dowe-placeholder="{}"{}"#,
        escape_attr(placeholder),
        bind_attr(props.element.bind.as_deref(), context)
    );
    let options_html = options
        .iter()
        .map(render_select_option_html)
        .collect::<Vec<_>>()
        .join("");
    let control = format!(
        r#"<div class="select"><button{}>{}<span class="select-value">{}</span>{}</button><div class="select-popover" data-dowe-select-popover role="listbox">{}</div></div>"#,
        attrs(classes, Some(&props.element), Some(&extra), context),
        floating_label_html(props),
        escape_html(placeholder),
        select_arrow_svg(),
        options_html
    );
    if props.label.is_some() && !props.label_floating {
        format!(
            r#"<div class="field"><span class="field-label">{}</span>{}</div>"#,
            escape_html(props.label.as_deref().unwrap_or_default()),
            control
        )
    } else {
        control
    }
}

fn select_arrow_svg() -> &'static str {
    r#"<svg class="select-arrow" aria-hidden="true" focusable="false" width="1em" height="1em" viewBox="0 0 24 24"><path d="M0 0h24v24H0z" fill="none"></path><path fill="currentColor" d="M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z"></path></svg>"#
}

fn render_select_option_html(option: &SelectOption) -> String {
    let description = option
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="select-option-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<button type="button" class="select-option" role="option" data-dowe-option-value="{}" data-dowe-option-label="{}"><span class="select-option-label">{}</span>{}</button>"#,
        escape_attr(&option.value),
        escape_attr(&option.label),
        escape_html(&option.label),
        description
    )
}

fn render_code_html(props: &CodeProps, context: &ReactiveRenderContext) -> String {
    let source = props
        .tokens
        .iter()
        .map(|token| {
            format!(
                r#"<span class="code-token-{}">{}</span>"#,
                token.kind.as_str(),
                escape_html(&token.text)
            )
        })
        .collect::<String>();
    let extra = format!(
        r#" data-dowe-code data-dowe-copy-label="{}" data-dowe-copied-label="{}""#,
        escape_attr(&props.copy_label),
        escape_attr(&props.copied_label)
    );
    format!(
        r#"<div{}><div class="code-toolbar"><span class="code-language">{}</span><button class="code-copy" type="button" data-dowe-code-copy>{}</button></div><pre class="code-pre"><code>{}</code></pre></div>"#,
        attrs(
            variant_classes("code-block", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        props.language.as_str(),
        escape_html(&props.copy_label),
        source
    )
}

fn render_video_html(props: &VideoProps, context: &ReactiveRenderContext) -> String {
    let poster = props
        .poster
        .as_deref()
        .map(|value| format!(r#" poster="{}""#, escape_attr(value)))
        .unwrap_or_default();
    let autoplay = if props.autoplay { " autoplay" } else { "" };
    format!(
        r#"<div{}><video class="video-media" src="{}" data-dowe-video data-dowe-video-source="{}" controls playsinline preload="metadata"{}{}></video></div>"#,
        attrs(
            video_classes(props),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.src),
        escape_attr(&props.src),
        poster,
        autoplay
    )
}

fn render_divider_html(props: &DividerProps, context: &ReactiveRenderContext) -> String {
    format!(
        "<div{}></div>",
        attrs(
            divider_classes(props),
            Some(&props.style.element),
            None,
            context
        )
    )
}

fn input_placeholder_attr(props: &VariantProps) -> String {
    let placeholder = props
        .placeholder
        .as_deref()
        .or((props.label_floating && props.label.is_some()).then_some(" "));
    placeholder
        .map(|value| format!(r#" placeholder="{}""#, escape_attr(value)))
        .unwrap_or_default()
}

fn floating_label_html(props: &VariantProps) -> String {
    if props.label_floating {
        props
            .label
            .as_deref()
            .map(|label| {
                format!(
                    r#"<span class="control-label">{}</span>"#,
                    escape_html(label)
                )
            })
            .unwrap_or_default()
    } else {
        String::new()
    }
}

fn navigation_attrs(href: &str, operation: NavigationOperation) -> String {
    format!(
        r#" href="{}" data-dowe-nav="{}" data-dowe-href="{}""#,
        escape_attr(href),
        operation.as_str(),
        escape_attr(href)
    )
}

fn external_attrs(
    url: &str,
    web_target: WebTarget,
    native_external_mode: NativeExternalMode,
) -> String {
    let mut attrs = format!(
        r#" href="{}" data-dowe-external-mode="{}""#,
        escape_attr(url),
        native_external_mode.as_str()
    );
    if web_target == WebTarget::Blank {
        attrs.push_str(r#" target="_blank" rel="noopener""#);
    }
    attrs
}

fn attrs(
    classes: Vec<String>,
    element: Option<&ElementProps>,
    extra: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut output = class_attr(classes);
    if let Some(id) = element.and_then(|element| element.id.as_ref()) {
        output.push_str(&format!(r#" id="{}""#, escape_attr(id)));
    }
    if let Some(action) = element.and_then(|element| element.on_click.as_ref()) {
        output.push_str(&format!(
            r#" data-dowe-click="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(VisibilityCondition::Signal(show)) = element.and_then(|element| element.show.as_ref())
    {
        output.push_str(&format!(
            r#" data-dowe-show="{}""#,
            escape_attr(&context.signal_path(show))
        ));
    }
    if let Some(extra) = extra {
        output.push_str(extra);
    }
    output
}

fn class_attr(classes: Vec<String>) -> String {
    if classes.is_empty() {
        String::new()
    } else {
        format!(r#" class="{}""#, classes.join(" "))
    }
}

fn push_literal(segments: &mut Vec<JsSegment>, value: &str) {
    if value.is_empty() {
        return;
    }

    if let Some(JsSegment::Literal(existing)) = segments.last_mut() {
        existing.push_str(value);
    } else {
        segments.push(JsSegment::Literal(value.to_string()));
    }
}

enum JsSegment {
    Literal(String),
    Children,
}

fn short_id(namespace: &str, source: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;

    for byte in namespace.bytes().chain([0]).chain(source.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut value = hash;
    let mut id = String::with_capacity(8);

    for index in 0..8 {
        let digit = (value % 36) as usize;
        id.push(alphabet[digit] as char);
        value /= 36;
        if value == 0 {
            value = hash.rotate_left((index + 1) as u32);
        }
    }

    id
}

fn js_string_literal(value: &str) -> String {
    format!("\"{}\"", escape_js(value))
}

fn escape_js(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn escape_json(value: &str) -> String {
    escape_js(value)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value)
}
