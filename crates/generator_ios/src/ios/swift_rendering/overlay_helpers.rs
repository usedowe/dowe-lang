fn render_swift_modal(
    props: &ModalProps,
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
    output.push_str(&format!(
        "{pad}DoweModal(open: state.bool(\"{path}\"), close: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, radius: {}, disableOverlayClose: {}, hideCloseButton: {}, hasHeader: {}, hasFooter: {}) {{\n",
        swift_close_action(&path, props.on_close.as_deref(), context),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_card_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button,
        !header.is_empty(),
        !footer.is_empty(),
    ));
    render_swift_region_children(
        header,
        indent + 4,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(&format!("{pad}}} content: {{\n"));
    render_swift_region_children(
        body,
        indent + 4,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(&format!("{pad}}} footer: {{\n"));
    render_swift_region_children(
        footer,
        indent + 4,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(&format!("{pad}}}\n"));
}

fn render_swift_alert_dialog(
    props: &AlertDialogProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_swift(&context.signal_path(&props.open));
    output.push_str(&format!(
        "{pad}DoweAlertDialog(open: state.bool(\"{path}\"), close: {}, title: {}, description: {}, confirmText: {}, cancelText: {}, backgroundColor: {}, contentColor: {}, dangerColor: {}, radius: {}, loading: {}, confirm: {}, cancel: {})\n",
        swift_close_action(&path, props.on_cancel.as_deref(), context),
        swift_string_literal(&props.title),
        swift_string_literal(&props.description),
        swift_string_literal(&props.confirm_text),
        swift_string_literal(&props.cancel_text),
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Danger))),
        swift_card_radius(&props.style.style),
        props.loading,
        swift_optional_component_action(props.on_confirm.as_deref(), None, context),
        swift_optional_component_action(props.on_cancel.as_deref(), None, context),
    ));
}

fn render_swift_tooltip(
    props: &TooltipProps,
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
        "{pad}DoweTooltip(label: {}, position: {}, backgroundColor: {}, contentColor: {}) {{\n",
        swift_string_literal(&props.label),
        swift_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for child in children {
        render_swift_node_in_flow(
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
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_toast(
    props: &ToastProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (visible, title, description, close) = if let Some(source) = props.source.as_deref() {
        let path = escape_swift(&context.signal_path(source));
        (
            format!("state.bool(\"{path}.visible\")"),
            format!("state.text(\"{path}.title\")"),
            format!("state.text(\"{path}.message\")"),
            format!("{{ state.write(\"{path}.visible\", value: false) }}"),
        )
    } else {
        (
            "true".to_string(),
            props
                .title
                .as_deref()
                .map(swift_string_literal)
                .unwrap_or_else(|| "\"\"".to_string()),
            swift_string_literal(&props.description),
            "nil".to_string(),
        )
    };
    output.push_str(&format!(
        "{pad}DoweToast(visible: {visible}, title: {title}, description: {description}, position: {}, backgroundColor: {}, contentColor: {}, showIcon: {}, kind: {}, close: {close})\n",
        swift_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        props.show_icon,
        swift_string_literal(props.kind.as_str()),
    ));
}

fn render_swift_dropdown(
    props: &DropdownProps,
    trigger: &[ViewNode],
    header: &[ViewNode],
    entries: &[OverlayEntry],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweDropdown(backgroundColor: {}, contentColor: {}) {{\n",
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for child in trigger {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}} content: {{\n"));
    render_swift_region_children(
        header,
        indent + 4,
        output,
        inherited_font,
        default_family,
        context,
    );
    for entry in entries {
        render_swift_overlay_entry(entry, indent + 4, output, props, context);
    }
    render_swift_region_children(
        footer,
        indent + 4,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_command(
    props: &CommandProps,
    entries: &[CommandEntry],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (open, close) = props
        .open
        .as_deref()
        .map(|path| {
            let path = escape_swift(&context.signal_path(path));
            (
                format!("state.bool(\"{path}\")"),
                format!("{{ state.write(\"{path}\", value: false) }}"),
            )
        })
        .unwrap_or_else(|| ("false".to_string(), "{}".to_string()));
    output.push_str(&format!(
        "{pad}DoweCommand(open: {open}, close: {close}, placeholder: {}, emptyText: {}, closeText: {}, navigateText: {}, selectText: {}, toggleText: {}, shortcut: {}, showFooter: {}, backgroundColor: {}, contentColor: {}, accentColor: {}) {{\n",
        swift_string_literal(&props.placeholder),
        swift_string_literal(&props.empty_text),
        swift_string_literal(&props.close_text),
        swift_string_literal(&props.navigate_text),
        swift_string_literal(&props.select_text),
        swift_string_literal(&props.toggle_text),
        swift_string_literal(&props.shortcut),
        props.show_footer,
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Muted))),
    ));
    for entry in entries {
        render_swift_command_entry(
            entry,
            indent + 4,
            output,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_swift_overlay_entry(
    entry: &OverlayEntry,
    indent: usize,
    output: &mut String,
    props: &DropdownProps,
    context: &SwiftReactiveContext,
) {
    match entry {
        OverlayEntry::Item(item) => render_swift_overlay_item(
            item,
            indent,
            output,
            variant_container(&props.style),
            variant_content(&props.style),
            context,
        ),
        OverlayEntry::Divider => {
            let pad = " ".repeat(indent);
            output.push_str(&format!("{pad}Divider().background(DoweDesign.muted)\n"));
        }
    }
}

fn render_swift_command_entry(
    entry: &CommandEntry,
    indent: usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match entry {
        CommandEntry::Item(item) => render_swift_overlay_item(
            item,
            indent,
            output,
            "Color.clear",
            "DoweDesign.onBackground",
            context,
        ),
        CommandEntry::Group { label, icon, items } => {
            output.push_str(&format!(
                "{pad}VStack(alignment: .leading, spacing: CGFloat(2)) {{\n"
            ));
            output.push_str(&format!("{pad}    HStack(spacing: CGFloat(6)) {{\n"));
            if let Some(icon) = icon {
                render_swift_side_icon(icon, indent + 8, output);
            }
            output.push_str(&format!(
                "{pad}        Text({})\n{pad}            .font(.caption)\n{pad}            .fontWeight(.semibold)\n{pad}            .foregroundStyle(DoweDesign.onMuted)\n",
                swift_string_literal(label)
            ));
            output.push_str(&format!("{pad}    }}\n"));
            for item in items {
                render_swift_overlay_item(
                    item,
                    indent + 4,
                    output,
                    "Color.clear",
                    "DoweDesign.onBackground",
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_swift_overlay_item(
    item: &OverlayItemProps,
    indent: usize,
    output: &mut String,
    background_color: &str,
    content_color: &str,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweOverlayItem(label: {}, description: {}, disabled: {}, backgroundColor: {background_color}, contentColor: {content_color}, action: {}) {{\n",
        swift_string_literal(&item.label),
        swift_optional_literal(item.description.as_deref()),
        item.disabled,
        swift_optional_component_action(item.on_click.as_deref(), item.navigation.as_ref(), context)
    ));
    if let Some(icon) = item.icon.as_ref() {
        render_swift_side_icon(icon, indent + 4, output);
    } else {
        output.push_str(&format!("{pad}    EmptyView()\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}
