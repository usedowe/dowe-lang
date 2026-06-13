fn render_badge_html(
    props: &BadgeProps,
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<div{}>",
        attrs(
            badge_classes(props),
            Some(&props.style.element),
            None,
            context
        )
    );
    for child in children {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str(&format!(
        r#"<span class="badge-content"><span class="badge-text">{}</span></span></div>"#,
        escape_html(&props.text)
    ));
    html
}

fn render_chip_html(
    props: &ChipProps,
    value: &str,
    start: Option<&SideNavIcon>,
    end: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    let start = start
        .map(|icon| render_overlay_icon_html("chip-icon", icon, context))
        .unwrap_or_default();
    let end = end
        .map(|icon| render_overlay_icon_html("chip-icon", icon, context))
        .unwrap_or_default();
    let close = props
        .on_close
        .as_deref()
        .map(|action| {
            format!(
                r#"<button class="chip-close" type="button" aria-label="Close" data-dowe-click="{}">&times;</button>"#,
                escape_attr(&context.action_id(action))
            )
        })
        .unwrap_or_default();
    format!(
        r#"<span{}>{start}<span class="chip-label">{}</span>{end}{close}</span>"#,
        attrs(
            chip_classes(props),
            Some(&props.style.element),
            None,
            context
        ),
        escape_html(value)
    )
}

fn render_skeleton_html(props: &SkeletonProps, context: &ReactiveRenderContext) -> String {
    format!(
        r#"<div{} aria-hidden="true"></div>"#,
        attrs(
            skeleton_classes(props),
            Some(&props.style.element),
            None,
            context
        )
    )
}

fn render_modal_html(
    props: &ModalProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        r#"<div{} hidden><button class="modal-overlay" type="button" aria-label="Close modal" data-dowe-modal-overlay></button><div{} role="dialog" aria-modal="true">"#,
        attrs(
            modal_panel_classes(props),
            Some(&props.style.element),
            Some(&modal_attrs(props, context)),
            context,
        ),
        class_attr(modal_classes(props))
    );
    if !header.is_empty() {
        html.push_str("<div class=\"modal-header\">");
        for child in header {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("<div class=\"modal-body\">");
    for child in body {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</div>");
    if !footer.is_empty() {
        html.push_str("<div class=\"modal-footer\">");
        for child in footer {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    if !props.hide_close_button {
        html.push_str(modal_close_html());
    }
    html.push_str("</div></div>");
    html
}

fn modal_attrs(props: &ModalProps, context: &ReactiveRenderContext) -> String {
    let mut attrs = format!(
        r#" data-dowe-modal data-dowe-modal-open="{}" data-dowe-modal-disable-overlay-close="{}""#,
        escape_attr(&context.signal_path(&props.open)),
        props.disable_overlay_close
    );
    if let Some(action) = props.on_close.as_deref() {
        attrs.push_str(&format!(
            r#" data-dowe-modal-on-close="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    attrs
}

fn modal_close_html() -> &'static str {
    r#"<button class="modal-close" type="button" aria-label="Close modal" data-dowe-modal-close>&times;</button>"#
}

fn render_alert_dialog_html(props: &AlertDialogProps, context: &ReactiveRenderContext) -> String {
    let modal = alert_dialog_modal_props(props);
    let confirm_action = props
        .on_confirm
        .as_deref()
        .map(|action| {
            format!(
                r#" data-dowe-click="{}""#,
                escape_attr(&context.action_id(action))
            )
        })
        .unwrap_or_default();
    let cancel_action = props
        .on_cancel
        .as_deref()
        .map(|action| {
            format!(
                r#" data-dowe-click="{}""#,
                escape_attr(&context.action_id(action))
            )
        })
        .unwrap_or_default();
    format!(
        r#"<div{} hidden><button class="modal-overlay" type="button" aria-label="Close dialog" data-dowe-modal-overlay></button><div{} role="alertdialog" aria-modal="true"><div class="modal-header"><h3 class="alert-dialog-title">{}</h3></div><div class="modal-body"><p class="alert-dialog-description">{}</p></div><div class="modal-footer"><div class="alert-dialog-actions"><button class="button button-md is-outlined is-muted" type="button" data-dowe-modal-close{}>{}</button><button class="button button-md is-solid is-{}" type="button"{}{}>{}</button></div></div></div></div>"#,
        attrs(
            modal_panel_classes(&modal),
            Some(&modal.style.element),
            Some(&modal_attrs(&modal, context)),
            context,
        ),
        class_attr(modal_classes(&modal)),
        escape_html(&props.title),
        escape_html(&props.description),
        cancel_action,
        escape_html(&props.cancel_text),
        props.style.color.unwrap_or(ColorFamily::Danger).as_str(),
        confirm_action,
        if props.loading {
            " disabled aria-busy=\"true\""
        } else {
            ""
        },
        escape_html(&props.confirm_text)
    )
}

fn render_tooltip_html(
    props: &TooltipProps,
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<span{} data-dowe-tooltip>",
        attrs(
            tooltip_classes(props),
            Some(&props.style.element),
            None,
            context
        )
    );
    for child in children {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str(&format!(
        r#"<span{} role="tooltip"><span class="tooltip-arrow"></span>{}</span></span>"#,
        class_attr(tooltip_popover_classes(props)),
        escape_html(&props.label)
    ));
    html
}

fn render_toast_html(props: &ToastProps, context: &ReactiveRenderContext) -> String {
    let mut extra = String::from(r#" data-dowe-toast"#);
    if let Some(source) = props.source.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-toast-source="{}""#,
            escape_attr(&context.signal_path(source))
        ));
    }
    let hidden = if props.source.is_some() {
        " hidden"
    } else {
        ""
    };
    let title = props
        .title
        .as_deref()
        .map(|title| format!(r#"<div class="toast-title">{}</div>"#, escape_html(title)))
        .unwrap_or_default();
    format!(
        r#"<div{}{}><div class="toast-content">{title}<div class="toast-description">{}</div></div><button class="toast-close" type="button" aria-label="Close toast" data-dowe-toast-close>&times;</button></div>"#,
        attrs(
            toast_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        hidden,
        escape_html(&props.description)
    )
}

fn render_dropdown_html(
    props: &DropdownProps,
    trigger: &[ViewNode],
    header: &[ViewNode],
    entries: &[OverlayEntry],
    footer: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<span{} data-dowe-dropdown>",
        attrs(
            dropdown_classes(props),
            Some(&props.style.element),
            None,
            context
        )
    );
    html.push_str("<span class=\"dropdown-trigger\" data-dowe-dropdown-trigger>");
    for child in trigger {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</span><div");
    html.push_str(&class_attr(dropdown_popover_classes(props)));
    html.push_str(" role=\"menu\" hidden>");
    for child in header {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("<div class=\"dropdown-options\">");
    for entry in entries {
        html.push_str(&render_overlay_entry_html("dropdown", entry, context));
    }
    html.push_str("</div>");
    for child in footer {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</div></span>");
    html
}

fn render_command_html(
    props: &CommandProps,
    entries: &[CommandEntry],
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = format!(
        r#" data-dowe-command data-dowe-command-shortcut="{}" data-dowe-command-disable-global="{}""#,
        escape_attr(&props.shortcut),
        props.disable_global_shortcut
    );
    if let Some(open) = props.open.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-command-open="{}""#,
            escape_attr(&context.signal_path(open))
        ));
    }
    let mut html = format!(
        r#"<div{} hidden><button class="modal-overlay" type="button" aria-label="Close command" data-dowe-command-close></button><div{} role="dialog" aria-modal="true"><div class="command-header"><input class="command-input" type="search" placeholder="{}" data-dowe-command-input><span class="command-kbd"><kbd>Esc</kbd><span>{}</span></span></div><div class="command-results" data-dowe-command-results>"#,
        attrs(
            command_panel_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context,
        ),
        class_attr(command_classes(props)),
        escape_attr(&props.placeholder),
        escape_html(&props.close_text)
    );
    for entry in entries {
        html.push_str(&render_command_entry_html(entry, context));
    }
    html.push_str(&format!(
        r#"<div class="command-empty" hidden>{}</div></div>"#,
        escape_html(&props.empty_text)
    ));
    if props.show_footer {
        html.push_str(&format!(
            r#"<div class="command-shortcuts"><span><kbd>↑</kbd><kbd>↓</kbd> {}</span><span><kbd>↵</kbd> {}</span><span><kbd>Ctrl</kbd><kbd>{}</kbd> {}</span></div>"#,
            escape_html(&props.navigate_text),
            escape_html(&props.select_text),
            escape_html(&props.shortcut.to_ascii_uppercase()),
            escape_html(&props.toggle_text)
        ));
    }
    html.push_str("</div></div>");
    html
}

fn render_command_entry_html(entry: &CommandEntry, context: &ReactiveRenderContext) -> String {
    match entry {
        CommandEntry::Item(props) => render_overlay_item_html("command", props, context),
        CommandEntry::Group { label, icon, items } => {
            let icon = icon
                .as_ref()
                .map(|icon| render_overlay_icon_html("command-group-icon", icon, context))
                .unwrap_or_default();
            let mut html = format!(
                r#"<div class="command-group"><div class="command-group-label">{icon}{}</div><div class="command-group-items">"#,
                escape_html(label)
            );
            for item in items {
                html.push_str(&render_overlay_item_html("command", item, context));
            }
            html.push_str("</div></div>");
            html
        }
    }
}

fn render_overlay_entry_html(
    base: &str,
    entry: &OverlayEntry,
    context: &ReactiveRenderContext,
) -> String {
    match entry {
        OverlayEntry::Item(props) => render_overlay_item_html(base, props, context),
        OverlayEntry::Divider => {
            r#"<div class="dropdown-divider" role="separator"></div>"#.to_string()
        }
    }
}

fn render_overlay_item_html(
    base: &str,
    props: &OverlayItemProps,
    context: &ReactiveRenderContext,
) -> String {
    let icon = props
        .icon
        .as_ref()
        .map(|icon| render_overlay_icon_html(&format!("{base}-item-icon"), icon, context))
        .unwrap_or_default();
    let description = props
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="{base}-item-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    let content = format!(
        r#"{icon}<span class="{base}-item-content"><span class="{base}-item-label">{}</span>{description}</span>"#,
        escape_html(&props.label)
    );
    let attrs = overlay_item_attrs(base, props, context);
    format!(r#"<{}{}>{}</{}>"#, attrs.0, attrs.1, content, attrs.2)
}

fn overlay_item_attrs(
    base: &str,
    props: &OverlayItemProps,
    context: &ReactiveRenderContext,
) -> (&'static str, String, &'static str) {
    let class_name = if props.disabled {
        format!("{base}-item is-disabled")
    } else {
        format!("{base}-item")
    };
    let classes = class_attr(class_name.split_whitespace().map(str::to_string).collect());
    if props.disabled {
        return ("div", format!(r#"{classes} aria-disabled="true""#), "div");
    }
    match props.navigation.as_ref() {
        Some(action) => (
            "a",
            format!("{classes}{}", side_nav_navigation_attrs(base, action)),
            "a",
        ),
        None if props.on_click.is_some() => (
            "button",
            format!(
                r#"{classes} type="button" data-dowe-click="{}""#,
                escape_attr(&context.action_id(props.on_click.as_deref().expect("onClick")))
            ),
            "button",
        ),
        None => ("button", format!(r#"{classes} type="button""#), "button"),
    }
}

fn render_overlay_icon_html(
    class_name: &str,
    icon: &SideNavIcon,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"<span class="{class_name}">{}</span>"#,
        render_svg_html(&icon.props, &icon.paths, context)
    )
}
