fn render_compose_drawer(
    props: &DrawerProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_kotlin(&context.signal_path(&props.open));
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            card_variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweDrawer(open = state.bool(\"{path}\"), onClose = {{ state.write(\"{path}\", false) }}, position = \"{}\", backgroundColor = {}, contentColor = {}, borderColor = {border}, radius = {}, disableOverlayClose = {}, hideCloseButton = {}) {{\n",
        props.position.as_str(),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_drawer_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button
    ));
    output.push_str(&format!(
        "{pad}    val doweDrawerNavigate = navigate\n{pad}    val navigate: (String, String, String?) -> Unit = {{ operation, target, fragment ->\n{pad}        state.write(\"{path}\", false)\n{pad}        doweDrawerNavigate(operation, target, fragment)\n{pad}    }}\n{pad}    val doweDrawerGoBack = goBack\n{pad}    val goBack: () -> Unit = {{\n{pad}        state.write(\"{path}\", false)\n{pad}        doweDrawerGoBack()\n{pad}    }}\n{pad}    val doweDrawerOpenExternal = openExternal\n{pad}    val openExternal: (String, String) -> Unit = {{ mode, target ->\n{pad}        state.write(\"{path}\", false)\n{pad}        doweDrawerOpenExternal(mode, target)\n{pad}    }}\n"
    ));
    output.push_str(&format!(
        "{pad}    Column(modifier = {}) {{\n",
        modifier_for_style_with_base(
            &props.style.style,
            "Modifier.fillMaxSize().safeDrawingPadding()".to_string(),
        )
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    if !header.is_empty() {
        output.push_str(&format!(
            "{pad}        Column(modifier = Modifier.fillMaxWidth()) {{\n"
        ));
        for child in header {
            render_compose_node_in_flow(
                child,
                indent + 12,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!(
        "{pad}        Column(modifier = Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState())) {{\n"
    ));
    for child in body {
        render_compose_node_in_flow(
            child,
            indent + 12,
            output,
            ComposeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}        }}\n"));
    if !footer.is_empty() {
        output.push_str(&format!(
            "{pad}        Column(modifier = Modifier.fillMaxWidth()) {{\n"
        ));
        for child in footer {
            render_compose_node_in_flow(
                child,
                indent + 12,
                output,
                ComposeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}    }}\n"));
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_modal(
    props: &ModalProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_kotlin(&context.signal_path(&props.open));
    output.push_str(&format!(
        "{pad}DoweModal(open = state.bool(\"{path}\"), close = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, radius = {}, disableOverlayClose = {}, hideCloseButton = {}, header = ",
        compose_close_action(&path, props.on_close.as_deref(), context),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_card_radius(&props.style.style),
        props.disable_overlay_close,
        props.hide_close_button,
    ));
    render_compose_optional_region_lambda(
        header,
        indent,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(", footer = ");
    render_compose_optional_region_lambda(
        footer,
        indent,
        output,
        inherited_font,
        default_family,
        context,
    );
    output.push_str(") {\n");
    for child in body {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_alert_dialog(
    props: &AlertDialogProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let path = escape_kotlin(&context.signal_path(&props.open));
    output.push_str(&format!(
        "{pad}DoweAlertDialog(open = state.bool(\"{path}\"), close = {}, title = {}, description = {}, confirmText = {}, cancelText = {}, backgroundColor = {}, contentColor = {}, dangerColor = {}, radius = {}, loading = {}, onConfirm = {}, onCancel = {})\n",
        compose_close_action(&path, props.on_cancel.as_deref(), context),
        compose_string_literal(&props.title),
        compose_string_literal(&props.description),
        compose_string_literal(&props.confirm_text),
        compose_string_literal(&props.cancel_text),
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Danger))),
        compose_card_radius(&props.style.style),
        props.loading,
        compose_optional_component_action(props.on_confirm.as_deref(), None, context),
        compose_optional_component_action(props.on_cancel.as_deref(), None, context),
    ));
}

fn render_compose_tooltip(
    props: &TooltipProps,
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
        "{pad}DoweTooltip(label = {}, position = {}, backgroundColor = {}, contentColor = {}, modifier = {}) {{\n",
        compose_string_literal(&props.label),
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

fn render_compose_toast(
    props: &ToastProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (visible, title, description, close) = if let Some(source) = props.source.as_deref() {
        let path = escape_kotlin(&context.signal_path(source));
        (
            format!("state.bool(\"{path}.visible\")"),
            format!("state.text(\"{path}.title\")"),
            format!("state.text(\"{path}.message\")"),
            format!("{{ state.write(\"{path}.visible\", false) }}"),
        )
    } else {
        (
            "true".to_string(),
            props
                .title
                .as_deref()
                .map(compose_string_literal)
                .unwrap_or_else(|| "\"\"".to_string()),
            compose_string_literal(&props.description),
            "null".to_string(),
        )
    };
    output.push_str(&format!(
        "{pad}DoweToast(visible = {visible}, title = {title}, description = {description}, position = {}, backgroundColor = {}, contentColor = {}, showIcon = {}, kind = {}, close = {close})\n",
        compose_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        props.show_icon,
        compose_string_literal(props.kind.as_str()),
    ));
}

fn render_compose_dropdown(
    props: &DropdownProps,
    trigger: &[ViewNode],
    header: &[ViewNode],
    entries: &[OverlayEntry],
    footer: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweDropdown(backgroundColor = {}, contentColor = {}, modifier = {}) {{\n",
        variant_container(&props.style),
        variant_content(&props.style),
        modifier_for_style(&props.style.style)
    ));
    for child in trigger {
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
    output.push_str(&format!("{pad}}} content: {{\n"));
    for child in header {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    for entry in entries {
        render_compose_overlay_entry(entry, indent + 4, output, props, context);
    }
    for child in footer {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_command(
    props: &CommandProps,
    entries: &[CommandEntry],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (open, close) = props
        .open
        .as_deref()
        .map(|path| {
            let path = escape_kotlin(&context.signal_path(path));
            (
                format!("state.bool(\"{path}\")"),
                format!("{{ state.write(\"{path}\", false) }}"),
            )
        })
        .unwrap_or_else(|| ("false".to_string(), "{}".to_string()));
    output.push_str(&format!(
        "{pad}DoweCommand(open = {open}, close = {close}, placeholder = {}, emptyText = {}, closeText = {}, navigateText = {}, selectText = {}, toggleText = {}, shortcut = {}, showFooter = {}, backgroundColor = {}, contentColor = {}, accentColor = {}) {{\n",
        compose_string_literal(&props.placeholder),
        compose_string_literal(&props.empty_text),
        compose_string_literal(&props.close_text),
        compose_string_literal(&props.navigate_text),
        compose_string_literal(&props.select_text),
        compose_string_literal(&props.toggle_text),
        compose_string_literal(&props.shortcut),
        props.show_footer,
        variant_container(&props.style),
        variant_content(&props.style),
        color_ref(family_color(props.style.color.unwrap_or(ColorFamily::Muted))),
    ));
    for entry in entries {
        render_compose_command_entry(
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
