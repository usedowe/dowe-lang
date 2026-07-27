fn render_dev_android_overlay_entry(
    entry: &OverlayEntry,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    dismiss: Option<&str>,
) {
    match entry {
        OverlayEntry::Item(item) => render_dev_android_overlay_item(
            item,
            props,
            parent,
            counter,
            output,
            inherited_font,
            context,
            dismiss,
        ),
        OverlayEntry::Divider => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setBackgroundColor({});\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));\n",
                java_color(ColorToken::Muted)
            ));
            output.push_str(&dev_add(parent, &view, None, false));
        }
    }
}

fn render_dev_android_command_entry(
    entry: &CommandEntry,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    dismiss: Option<&str>,
) {
    match entry {
        CommandEntry::Item(item) => render_dev_android_overlay_item(
            item,
            props,
            parent,
            counter,
            output,
            inherited_font,
            context,
            dismiss,
        ),
        CommandEntry::Group { label, items, .. } => {
            render_dev_android_variant_label(
                label,
                props,
                parent,
                None,
                false,
                counter,
                output,
                inherited_font,
                context,
            );
            for item in items {
                render_dev_android_overlay_item(
                    item,
                    props,
                    parent,
                    counter,
                    output,
                    inherited_font,
                    context,
                    dismiss,
                );
            }
        }
    }
}

fn render_dev_android_overlay_item(
    item: &OverlayItemProps,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    dismiss: Option<&str>,
) {
    let view = next_dev_view(counter);
    let action = if item.disabled {
        None
    } else {
        dev_android_overlay_item_action(item, context, dismiss)
    };
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setPadding(doweDp(16), doweDp(10), doweDp(16), doweDp(10));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n        TextView {view}Label = doweText(\"{}\", {}, 14f, 700, 0f, 1.2f, {});\n        doweAdd({view}, {view}Label);\n",
        if action.is_some() {
            dev_variant_container(props)
        } else {
            "Color.TRANSPARENT"
        },
        escape_java(&item.label),
        dev_variant_content(props),
        dev_font_value(props.style.font.as_ref().or(inherited_font))
    ));
    if let Some(description) = item.description.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Description = doweText(\"{}\", doweAlpha({}, 0.68f), 12f, 400, 0f, 1.2f, {});\n        doweAdd({view}, {view}Description, 4, false);\n",
            escape_java(description),
            dev_variant_content(props),
            dev_font_value(props.style.font.as_ref().or(inherited_font))
        ));
    }
    if item.disabled {
        output.push_str(&format!("        {view}.setAlpha(0.48f);\n"));
    }
    if let Some(action) = action {
        output.push_str(&format!("        {view}.setOnClickListener(v -> {{ {action} }});\n"));
    }
    output.push_str(&dev_add(parent, &view, None, false));
}

fn dev_android_overlay_item_action(
    item: &OverlayItemProps,
    context: &ComposeReactiveContext,
    dismiss: Option<&str>,
) -> Option<String> {
    let action = item
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null);", escape_java(id)))
        .or_else(|| {
            dev_android_navigation_action(item.navigation.as_ref()).map(|action| format!("{action};"))
        })?;
    let close = dismiss
        .map(|value| format!("{value}; "))
        .unwrap_or_default();
    Some(format!("{close}{action}"))
}

fn render_dev_android_modal(
    props: &ModalProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let path = escape_java(&context.signal_path(&props.open));
    let overlay = next_dev_view(counter);
    let panel = next_dev_view(counter);
    let popup_ref = format!("{overlay}PopupRef");
    let close = dev_android_modal_close(
        &path,
        props.on_close.as_deref(),
        context,
        &format!("{popup_ref}[0]"),
    );
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
        == ComponentVariant::Outlined
    {
        dev_variant_content(&props.style)
    } else {
        "null"
    };
    output.push_str(&format!(
        "        if (doweBool(\"{path}\")) {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        LinearLayout {panel} = doweContainer(false);\n        {panel}.setPadding(doweDp(20), doweDp(20), doweDp(20), doweDp(20));\n        {panel}.setBackground(doweInputBackground({}, {border}, DOWE_RADIUS));\n",
        dev_variant_container(&props.style)
    ));
    apply_dev_android_style(&props.style.style, &panel, false, output);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let current_color = Some(dev_variant_content(&props.style).to_string());
    if !header.is_empty() || !props.hide_close_button {
        let header_row = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {header_row} = doweContainer(true);\n        {header_row}.setGravity(Gravity.CENTER_VERTICAL);\n        doweAdd({panel}, {header_row});\n"
        ));
        for child in header {
            render_dev_android_node(
                child,
                &header_row,
                None,
                true,
                counter,
                output,
                current_font,
                current_color.clone(),
                context,
                children_method,
            );
        }
        if !props.hide_close_button {
            let close_view = next_dev_view(counter);
            output.push_str(&format!(
                "        Button {close_view} = new Button(this);\n        {close_view}.setText(\"x\");\n        {close_view}.setAllCaps(false);\n        {close_view}.setTextColor({});\n        {close_view}.setBackground(doweBackground(DOWE_SOFT_MUTED, DOWE_RADIUS));\n        {close_view}.setOnClickListener(v -> {{ {close} }});\n        doweAdd({header_row}, {close_view}, 8, true);\n",
                dev_variant_content(&props.style)
            ));
        }
    }
    for child in body {
        render_dev_android_node(
            child,
            &panel,
            None,
            false,
            counter,
            output,
            current_font,
            current_color.clone(),
            context,
            children_method,
        );
    }
    for child in footer {
        render_dev_android_node(
            child,
            &panel,
            None,
            false,
            counter,
            output,
            current_font,
            current_color.clone(),
            context,
            children_method,
        );
    }
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER);\n        {panel}Params.setMargins(doweDp(16), 0, doweDp(16), 0);\n        {overlay}.addView({panel}, {panel}Params);\n        {popup_ref}[0] = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(false);\n        {panel}.setOnClickListener(v -> {{ }});\n"
    ));
    if !props.disable_overlay_close {
        output.push_str(&format!(
            "        {overlay}.setOnClickListener(v -> {{ {close} }});\n"
        ));
    }
    output.push_str(&format!(
        "        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup_ref}[0].showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n"
    ));
}

fn render_dev_android_alert_dialog(
    props: &AlertDialogProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let path = escape_java(&context.signal_path(&props.open));
    let overlay = next_dev_view(counter);
    let panel = next_dev_view(counter);
    let actions = next_dev_view(counter);
    let popup_ref = format!("{overlay}PopupRef");
    let close = dev_android_modal_close(
        &path,
        props.on_cancel.as_deref(),
        context,
        &format!("{popup_ref}[0]"),
    );
    let confirm = props
        .on_confirm
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null);", escape_java(id)))
        .unwrap_or_default();
    output.push_str(&format!(
        "        if (doweBool(\"{path}\")) {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        LinearLayout {panel} = doweContainer(false);\n        {panel}.setPadding(doweDp(20), doweDp(20), doweDp(20), doweDp(20));\n        {panel}.setBackground(doweInputBackground(DOWE_SURFACE, null, DOWE_RADIUS));\n        TextView {panel}Title = doweText(\"{}\", DOWE_ON_SURFACE, 18f, 700, 0f, 1.2f, {});\n        doweAdd({panel}, {panel}Title);\n        TextView {panel}Description = doweText(\"{}\", doweAlpha(DOWE_ON_SURFACE, 0.72f), 14f, 400, 0f, 1.3f, {});\n        doweAdd({panel}, {panel}Description, 8, false);\n        LinearLayout {actions} = doweContainer(true);\n        {actions}.setGravity(Gravity.END | Gravity.CENTER_VERTICAL);\n",
        escape_java(&props.title),
        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
        escape_java(&props.description),
        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
    ));
    let cancel = next_dev_view(counter);
    let confirm_view = next_dev_view(counter);
    output.push_str(&format!(
        "        Button {cancel} = new Button(this);\n        {cancel}.setText(\"{}\");\n        {cancel}.setAllCaps(false);\n        {cancel}.setTextColor(DOWE_ON_SURFACE);\n        {cancel}.setBackground(doweInputBackground(Color.TRANSPARENT, DOWE_MUTED, DOWE_RADIUS));\n        {cancel}.setOnClickListener(v -> {{ {close} }});\n        doweAdd({actions}, {cancel});\n        Button {confirm_view} = new Button(this);\n        {confirm_view}.setText(\"{}\");\n        {confirm_view}.setAllCaps(false);\n        {confirm_view}.setTextColor(DOWE_ON_DANGER);\n        {confirm_view}.setEnabled({});\n        {confirm_view}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS));\n        {confirm_view}.setOnClickListener(v -> {{ {confirm} }});\n        doweAdd({actions}, {confirm_view}, 8, true);\n        doweAdd({panel}, {actions}, 16, false);\n        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER);\n        {panel}Params.setMargins(doweDp(16), 0, doweDp(16), 0);\n        {overlay}.addView({panel}, {panel}Params);\n        {popup_ref}[0] = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(false);\n        {panel}.setOnClickListener(v -> {{ }});\n        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup_ref}[0].showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n",
        escape_java(&props.cancel_text),
        escape_java(&props.confirm_text),
        if props.loading { "false" } else { "true" },
        dev_variant_content(&props.style),
        dev_variant_content(&props.style)
    ));
}

fn render_dev_android_toast(
    props: &ToastProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let visible = props
        .source
        .as_deref()
        .map(|source| {
            format!(
                "doweBool(\"{}.visible\")",
                escape_java(&context.signal_path(source))
            )
        })
        .unwrap_or_else(|| "true".to_string());
    let (title, description, close) = if let Some(source) = props.source.as_deref() {
        let path = escape_java(&context.signal_path(source));
        (
            format!("doweTextValue(\"{path}.title\", null)"),
            format!("doweTextValue(\"{path}.message\", null)"),
            Some(format!("doweWrite(\"{path}.visible\", false);")),
        )
    } else {
        (
            props
                .title
                .as_deref()
                .map(|title| format!("\"{}\"", escape_java(title)))
                .unwrap_or_else(|| "\"\"".to_string()),
            format!("\"{}\"", escape_java(&props.description)),
            None,
        )
    };
    let overlay = next_dev_view(counter);
    let panel = next_dev_view(counter);
    let popup_ref = format!("{overlay}PopupRef");
    let gravity = match props.position {
        OverlayCornerPosition::TopLeft => "Gravity.TOP | Gravity.START",
        OverlayCornerPosition::TopRight => "Gravity.TOP | Gravity.END",
        OverlayCornerPosition::BottomRight => "Gravity.BOTTOM | Gravity.END",
        OverlayCornerPosition::BottomLeft => "Gravity.BOTTOM | Gravity.START",
    };
    output.push_str(&format!(
        "        if ({visible}) {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        FrameLayout {overlay} = new FrameLayout(this);\n        LinearLayout {panel} = doweContainer(true);\n        {panel}.setGravity(Gravity.CENTER_VERTICAL);\n        {panel}.setPadding(doweDp(16), doweDp(12), doweDp(16), doweDp(12));\n        {panel}.setBackground(doweInputBackground({}, null, DOWE_RADIUS));\n        TextView {panel}Text = doweText(({title}.isEmpty() ? {description} : {title} + \"\\n\" + {description}), {}, 14f, 500, 0f, 1.25f, {});\n        doweAdd({panel}, {panel}Text);\n",
        dev_variant_container(&props.style),
        dev_variant_content(&props.style),
        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
    ));
    if let Some(close) = close {
        let close_view = next_dev_view(counter);
        output.push_str(&format!(
            "        Button {close_view} = new Button(this);\n        {close_view}.setText(\"x\");\n        {close_view}.setAllCaps(false);\n        {close_view}.setTextColor({});\n        {close_view}.setBackgroundColor(Color.TRANSPARENT);\n        {close_view}.setOnClickListener(v -> {{ if ({popup_ref}[0] != null) {{ {popup_ref}[0].dismiss(); }} {close} renderCurrentRoute(false); }});\n        doweAdd({panel}, {close_view}, 8, true);\n",
            dev_variant_content(&props.style)
        ));
    }
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT, {gravity});\n        {panel}Params.setMargins(doweDp(16), doweDp(16), doweDp(16), doweDp(16));\n        {overlay}.addView({panel}, {panel}Params);\n        {popup_ref}[0] = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, false);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(false);\n        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup_ref}[0].showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n"
    ));
}

fn render_dev_android_dropdown(
    props: &DropdownProps,
    trigger: &[ViewNode],
    header: &[ViewNode],
    entries: &[OverlayEntry],
    footer: &[ViewNode],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        FrameLayout {view} = new FrameLayout(this);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
    ));
    apply_dev_android_style(&props.style.style, &view, false, output);
    for child in trigger {
        render_dev_android_node(
            child,
            &view,
            None,
            false,
            counter,
            output,
            inherited_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    let hit = next_dev_view(counter);
    output.push_str(&format!(
        "        View {hit} = new View(this);\n        {hit}.setBackgroundColor(Color.TRANSPARENT);\n        FrameLayout.LayoutParams {hit}HitParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0);\n        {view}.addView({hit}, {hit}HitParams);\n        {view}.post(() -> {{\n            int {view}TriggerHeight = {view}.getHeight();\n            {hit}HitParams.height = {view}TriggerHeight;\n            {hit}.requestLayout();\n        }});\n"
    ));
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    let content = next_dev_view(counter);
    let scroll = next_dev_view(counter);
    let popup_ref = format!("{view}PopupRef");
    let dismiss = format!("if ({popup_ref}[0] != null) {{ {popup_ref}[0].dismiss(); }}");
    output.push_str(&format!(
        "        {hit}.setOnClickListener(anchor -> {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        LinearLayout {content} = doweContainer(false);\n        {content}.setAlpha(0f);\n        {content}.setScaleX(0.98f);\n        {content}.setScaleY(0.98f);\n        {content}.setTranslationY(-doweDp(4));\n        {content}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n        {content}.setBackground(doweInputBackground({}, null, DOWE_RADIUS));\n",
        dev_variant_container(&props.style)
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let current_color = Some(dev_variant_content(&props.style).to_string());
    for child in header {
        render_dev_android_node(
            child,
            &content,
            None,
            false,
            counter,
            output,
            current_font,
            current_color.clone(),
            context,
            children_method,
        );
    }
    for entry in entries {
        render_dev_android_overlay_entry(
            entry,
            &props.style,
            &content,
            counter,
            output,
            current_font,
            context,
            Some(&dismiss),
        );
    }
    for child in footer {
        render_dev_android_node(
            child,
            &content,
            None,
            false,
            counter,
            output,
            current_font,
            current_color.clone(),
            context,
            children_method,
        );
    }
    output.push_str(&format!(
        "        int {content}Width = Math.min(Math.max({view}.getWidth(), doweDp(220)), doweDp(360));\n        ScrollView {scroll} = new ScrollView(this);\n        {scroll}.setFillViewport(false);\n        {scroll}.addView({content}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {content}.measure(View.MeasureSpec.makeMeasureSpec({content}Width, View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));\n        {popup_ref}[0] = new PopupWindow({scroll}, {content}Width, ViewGroup.LayoutParams.WRAP_CONTENT, true);\n        {popup_ref}[0].setHeight(Math.min({content}.getMeasuredHeight(), doweDp(260)));\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(true);\n        {popup_ref}[0].showAsDropDown({view}, 0, doweDp(4));\n        {content}.animate().alpha(1f).scaleX(1f).scaleY(1f).translationY(0f).setDuration(160).start();\n        }});\n"
    ));
}

fn render_dev_android_command(
    props: &CommandProps,
    entries: &[CommandEntry],
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let open = props
        .open
        .as_deref()
        .map(|path| {
            format!(
                "doweBool(\"{}\")",
                escape_java(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| "false".to_string());
    let overlay = next_dev_view(counter);
    let panel = next_dev_view(counter);
    let popup_ref = format!("{overlay}PopupRef");
    let dismiss = format!("if ({popup_ref}[0] != null) {{ {popup_ref}[0].dismiss(); }}");
    output.push_str(&format!(
        "        if ({open}) {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        LinearLayout {panel} = doweContainer(false);\n        {panel}.setPadding(doweDp(12), doweDp(12), doweDp(12), doweDp(12));\n        {panel}.setBackground(doweInputBackground({}, null, DOWE_RADIUS));\n        TextView {panel}Search = doweText(\"{}\", {}, 15f, 500, 0f, 1.2f, {});\n        doweAdd({panel}, {panel}Search);\n",
        dev_variant_container(&props.style),
        escape_java(&props.placeholder),
        dev_variant_content(&props.style),
        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    for entry in entries {
        render_dev_android_command_entry(
            entry,
            &props.style,
            &panel,
            counter,
            output,
            current_font,
            context,
            Some(&dismiss),
        );
    }
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.CENTER_HORIZONTAL);\n        {panel}Params.setMargins(doweDp(16), doweDp(64), doweDp(16), 0);\n        {overlay}.addView({panel}, {panel}Params);\n        {popup_ref}[0] = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(false);\n        {overlay}.setOnClickListener(v -> {{ if ({popup_ref}[0] != null) {{ {popup_ref}[0].dismiss(); }} }});\n        {panel}.setOnClickListener(v -> {{ }});\n        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup_ref}[0].showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n"
    ));
}

fn dev_android_modal_close(
    path: &str,
    action: Option<&str>,
    context: &ComposeReactiveContext,
    popup: &str,
) -> String {
    let action = action
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null); ", escape_java(id)))
        .unwrap_or_default();
    format!(
        "if ({popup} != null) {{ {popup}.dismiss(); }} doweWrite(\"{path}\", false); {action}renderCurrentRoute(false);"
    )
}

fn render_dev_android_drawer(
    props: &DrawerProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let overlay = next_dev_view(counter);
    let panel = next_dev_view(counter);
    let content = next_dev_view(counter);
    let body_scroll = next_dev_view(counter);
    let body_content = next_dev_view(counter);
    let navigation_close = format!("{overlay}NavigationClose");
    let popup = format!("{overlay}Popup");
    let path = escape_java(&context.signal_path(&props.open));
    let (width, height, gravity) = match props.position {
        DrawerPosition::Start => (
            "doweDp(320)",
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "Gravity.START",
        ),
        DrawerPosition::End => (
            "doweDp(320)",
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "Gravity.END",
        ),
        DrawerPosition::Top => (
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "doweDp(320)",
            "Gravity.TOP",
        ),
        DrawerPosition::Bottom => (
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "doweDp(320)",
            "Gravity.BOTTOM",
        ),
    };
    output.push_str(&format!(
        "        if (doweBool(\"{path}\")) {{\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        FrameLayout {panel} = new FrameLayout(this);\n        {panel}.setBackground(doweDrawerBackground({}, {}, \"{}\", {}));\n        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams({width}, {height}, {gravity});\n        {overlay}.addView({panel}, {panel}Params);\n        LinearLayout {content} = doweContainer(false);\n",
        dev_card_variant_container(&props.style),
        dev_card_border(&props.style),
        props.position.as_str(),
        dev_drawer_radius(&props.style.style)
    ));
    apply_dev_android_style(&props.style.style, &content, false, output);
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {content}Params = doweFrameLayoutParams({content}.getLayoutParams());\n        if ({content}Params.width == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.width = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        if ({content}Params.height == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.height = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        {panel}.addView({content}, {content}Params);\n        ScrollView {body_scroll} = new ScrollView(this);\n        {body_scroll}.setFillViewport(true);\n        LinearLayout {body_content} = doweContainer(false);\n        {body_scroll}.addView({body_content}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
    ));
    if !header.is_empty() {
        let header_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {header_content} = doweContainer(false);\n        doweAdd({content}, {header_content});\n"
        ));
        let current_font = props.style.style.font.as_ref().or(inherited_font);
        for child in header {
            render_dev_android_node(
                child,
                &header_content,
                None,
                false,
                counter,
                output,
                current_font,
                Some(dev_card_variant_content(&props.style).to_string()),
                context,
                children_method,
            );
        }
    }
    output.push_str(&format!(
        "        {content}.addView({body_scroll}, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));\n"
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    for child in body {
        render_dev_android_node(
            child,
            &body_content,
            None,
            false,
            counter,
            output,
            current_font,
            Some(dev_card_variant_content(&props.style).to_string()),
            context,
            children_method,
        );
    }
    if !footer.is_empty() {
        let footer_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {footer_content} = doweContainer(false);\n        doweAdd({content}, {footer_content});\n"
        ));
        for child in footer {
            render_dev_android_node(
                child,
                &footer_content,
                None,
                false,
                counter,
                output,
                current_font,
                Some(dev_card_variant_content(&props.style).to_string()),
                context,
                children_method,
            );
        }
    }
    output.push_str(&format!(
        "        PopupWindow {popup} = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup}.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup}.setOutsideTouchable(false);\n        {panel}.setOnClickListener(v -> {{ }});\n"
    ));
    output.push_str(&format!(
        "        Runnable {navigation_close} = () -> {{ if ({popup}.isShowing()) {{ {popup}.dismiss(); }} doweWrite(\"{path}\", false); }};\n        doweDrawerNavigationClose = {navigation_close};\n        {popup}.setOnDismissListener(() -> {{ if (doweDrawerNavigationClose == {navigation_close}) {{ doweDrawerNavigationClose = null; }} }});\n"
    ));
    if !props.disable_overlay_close {
        output.push_str(&format!(
            "        {overlay}.setOnClickListener(v -> {{ {popup}.dismiss(); doweWrite(\"{path}\", false); renderCurrentRoute(false); }});\n"
        ));
    }
    if !props.hide_close_button {
        let close = next_dev_view(counter);
        let close_icon = next_dev_view(counter);
        let close_paths = format!("{close}Paths");
        output.push_str(&format!(
            "        FrameLayout {close} = new FrameLayout(this);\n        {close}.setBackground(doweBackground(DOWE_SOFT_MUTED, 999f));\n        {close}.setContentDescription(\"Close drawer\");\n        {close}.setFocusable(true);\n        {close}.setOnClickListener(v -> {{ {popup}.dismiss(); doweWrite(\"{path}\", false); renderCurrentRoute(false); }});\n        ArrayList<DoweSvgPathEntry> {close_paths} = new ArrayList<>();\n        {close_paths}.add(new DoweSvgPathEntry(\"M0 0h24v24H0z\", false, null));\n        {close_paths}.add(new DoweSvgPathEntry(\"m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z\", true, null));\n        DoweSvgView {close_icon} = new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_ON_SOFT_MUTED, {close_paths});\n        {close_icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n        {close}.addView({close_icon}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        FrameLayout.LayoutParams {close}Params = new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END);\n        {close}Params.setMargins(0, doweDp(8), doweDp(8), 0);\n        {panel}.addView({close}, {close}Params);\n"
        ));
    }
    output.push_str(&format!(
        "        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup}.showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n"
    ));
}

fn dev_fab_content_gravity(position: OverlayCornerPosition) -> &'static str {
    match position {
        OverlayCornerPosition::TopLeft | OverlayCornerPosition::BottomLeft => "Gravity.START",
        OverlayCornerPosition::TopRight | OverlayCornerPosition::BottomRight => "Gravity.END",
    }
}

fn dev_fab_layout_gravity(position: OverlayCornerPosition) -> &'static str {
    match position {
        OverlayCornerPosition::TopLeft => "Gravity.TOP | Gravity.START",
        OverlayCornerPosition::TopRight => "Gravity.TOP | Gravity.END",
        OverlayCornerPosition::BottomLeft => "Gravity.BOTTOM | Gravity.START",
        OverlayCornerPosition::BottomRight => "Gravity.BOTTOM | Gravity.END",
    }
}

fn dev_fab_size(size: ButtonSize) -> u16 {
    match size {
        ButtonSize::Xs => 40,
        ButtonSize::Sm => 48,
        ButtonSize::Md => 52,
        ButtonSize::Lg => 56,
        ButtonSize::Xl => 64,
    }
}

fn dev_dropzone_height(size: ButtonSize) -> u16 {
    match size {
        ButtonSize::Xs | ButtonSize::Sm => 128,
        ButtonSize::Md => 192,
        ButtonSize::Lg | ButtonSize::Xl => 256,
    }
}
