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
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> {{ {action} }});\n"
        ));
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
            dev_android_navigation_action(item.navigation.as_ref())
                .map(|action| format!("{action};"))
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
    let content = next_dev_view(counter);
    let popup_ref = format!("{overlay}PopupRef");
    let close = dev_android_modal_close(
        &path,
        props.on_close.as_deref(),
        context,
        &format!("{popup_ref}[0]"),
    );
    let border = dev_card_border(&props.style);
    output.push_str(&format!(
        "        if (doweBool(\"{path}\")) {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        FrameLayout {panel} = new FrameLayout(this);\n        {panel}.setBackground(doweInputBackground({}, {border}, DOWE_RADIUS));\n        LinearLayout {content} = doweContainer(false);\n        {content}.setPadding(doweDp(20), doweDp(20), doweDp(20), doweDp(20));\n        {panel}.addView({content}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n",
        dev_card_variant_container(&props.style)
    ));
    apply_dev_android_style(&props.style.style, &panel, false, output);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let current_color = Some(dev_card_variant_content(&props.style).to_string());
    if !header.is_empty() {
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
    }
    for child in body {
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
    if !footer.is_empty() {
        let footer_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {footer_content} = doweContainer(false);\n        {footer_content}.setPadding(0, doweDp(8), 0, doweDp(8));\n        {content}.addView({footer_content}, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
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
                current_color.clone(),
                context,
                children_method,
            );
        }
    }
    if !props.hide_close_button {
        render_dev_android_overlay_close(&panel, "Close modal", &close, "Gravity.TOP | Gravity.END", "0, doweDp(8), doweDp(8), 0", counter, output);
    }
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams(doweDp(Math.max(1, Math.min(560, Math.min(Math.max(0, viewportWidth - 32), (viewportWidth * 95) / 100)))), ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER);\n        {overlay}.addView({panel}, {panel}Params);\n        {popup_ref}[0] = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(false);\n        {popup_ref}[0].setOnDismissListener(() -> {{ if (doweActiveOverlay == {popup_ref}[0]) doweActiveOverlay = null; }});\n        {panel}.setOnClickListener(v -> {{ }});\n"
    ));
    output.push_str(&format!("        doweActiveOverlay = {popup_ref}[0];\n"));
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
    let mut panel_style = props.style.clone();
    panel_style.color = Some(ColorFamily::Surface);
    let panel_background = dev_card_variant_container(&panel_style);
    let panel_content = dev_card_variant_content(&panel_style);
    let panel_border = dev_card_border(&panel_style);
    let confirm_background = dev_variant_container(&VariantProps {
        variant: Some(ComponentVariant::Solid),
        color: props.style.color,
        ..Default::default()
    });
    let confirm_content = dev_variant_content(&VariantProps {
        variant: Some(ComponentVariant::Solid),
        color: props.style.color,
        ..Default::default()
    });
    output.push_str(&format!(
        "        if (doweBool(\"{path}\")) {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        LinearLayout {panel} = doweContainer(false);\n        {panel}.setPadding(doweDp(20), doweDp(20), doweDp(20), doweDp(20));\n        {panel}.setBackground(doweInputBackground({panel_background}, {panel_border}, DOWE_RADIUS));\n        TextView {panel}Title = doweText(\"{}\", {panel_content}, 18f, 700, 0f, 1.2f, {});\n        doweAdd({panel}, {panel}Title);\n        TextView {panel}Description = doweText(\"{}\", doweAlpha({panel_content}, 0.72f), 14f, 400, 0f, 1.3f, {});\n        doweAdd({panel}, {panel}Description, 8, false);\n        LinearLayout {actions} = doweContainer(true);\n        {actions}.setGravity(Gravity.END | Gravity.CENTER_VERTICAL);\n",
        escape_java(&props.title),
        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
        escape_java(&props.description),
        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
    ));
    let cancel = next_dev_view(counter);
    let confirm_view = next_dev_view(counter);
    output.push_str(&format!(
        "        Button {cancel} = new Button(this);\n        {cancel}.setText(\"{}\");\n        {cancel}.setAllCaps(false);\n        {cancel}.setMinHeight(doweDp(40));\n        {cancel}.setPadding(doweDp(16), doweDp(10), doweDp(16), doweDp(10));\n        {cancel}.setTextColor(DOWE_MUTED);\n        {cancel}.setBackground(doweInputBackground(Color.TRANSPARENT, DOWE_MUTED, DOWE_RADIUS));\n        {cancel}.setOnClickListener(v -> {{ {close} }});\n        doweAdd({actions}, {cancel});\n        Button {confirm_view} = new Button(this);\n        {confirm_view}.setText(\"{}\");\n        {confirm_view}.setAllCaps(false);\n        {confirm_view}.setMinHeight(doweDp(40));\n        {confirm_view}.setPadding(doweDp(16), doweDp(10), doweDp(16), doweDp(10));\n        {confirm_view}.setTextColor({confirm_content});\n        {confirm_view}.setEnabled({});\n        {confirm_view}.setBackground(doweInputBackground({confirm_background}, null, DOWE_RADIUS));\n        {confirm_view}.setOnClickListener(v -> {{ {confirm} }});\n        doweAdd({actions}, {confirm_view}, 12, true);\n        doweAdd({panel}, {actions}, 16, false);\n        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams(doweDp(Math.max(1, Math.min(560, Math.min(Math.max(0, viewportWidth - 32), (viewportWidth * 95) / 100)))), ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER);\n        {overlay}.addView({panel}, {panel}Params);\n        {popup_ref}[0] = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(false);\n        {popup_ref}[0].setOnDismissListener(() -> {{ if (doweActiveOverlay == {popup_ref}[0]) doweActiveOverlay = null; }});\n        {panel}.setOnClickListener(v -> {{ }});\n        doweActiveOverlay = {popup_ref}[0];\n        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup_ref}[0].showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n",
        escape_java(&props.cancel_text),
        escape_java(&props.confirm_text),
        if props.loading { "false" } else { "true" },
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
    let content = next_dev_view(counter);
    let popup_ref = format!("{overlay}PopupRef");
    let gravity = match props.position {
        OverlayCornerPosition::TopLeft => "Gravity.TOP | Gravity.START",
        OverlayCornerPosition::TopRight => "Gravity.TOP | Gravity.END",
        OverlayCornerPosition::BottomRight => "Gravity.BOTTOM | Gravity.END",
        OverlayCornerPosition::BottomLeft => "Gravity.BOTTOM | Gravity.START",
    };
    output.push_str(&format!(
        "        if ({visible}) {{\n        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        FrameLayout {overlay} = new FrameLayout(this);\n        FrameLayout {panel} = new FrameLayout(this);\n        {panel}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS));\n        LinearLayout {content} = doweContainer(true);\n        {content}.setGravity(Gravity.CENTER_VERTICAL);\n        {content}.setPadding(doweDp(16), doweDp(12), doweDp(12), doweDp(12));\n        TextView {panel}Text = doweText(({title}.isEmpty() ? {description} : {title} + \"\\n\" + {description}), {}, 14f, 500, 0f, 1.25f, {});\n        {panel}Text.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({content}, {panel}Text);\n",
        dev_card_variant_container(&props.style),
        dev_card_border(&props.style),
        dev_card_variant_content(&props.style),
        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
    ));
    let close_action = format!(
        "if ({popup_ref}[0] != null) {{ {popup_ref}[0].dismiss(); }} {}{}",
        close.unwrap_or_default(),
        if props.source.is_some() {
            " renderCurrentRoute(false);"
        } else {
            ""
        }
    );
    let close = next_dev_view(counter);
    let close_icon = next_dev_view(counter);
    let close_paths = format!("{close}Paths");
    output.push_str(&format!(
        "        DoweSvgView {close_icon} = new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_MUTED_TEXT, {close_paths});\n        {close_icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n        FrameLayout {close} = new FrameLayout(this);\n        {close}.setBackground(doweBackground(DOWE_MUTED, 999f));\n        {close}.setContentDescription(\"Close toast\");\n        {close}.setFocusable(true);\n        ArrayList<DoweSvgPathEntry> {close_paths} = new ArrayList<>();\n        {close_paths}.add(new DoweSvgPathEntry(\"M0 0h24v24H0z\", false, null));\n        {close_paths}.add(new DoweSvgPathEntry(\"m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z\", true, null));\n        {close}.addView({close_icon}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        {close}.setLayoutParams(new LinearLayout.LayoutParams(doweDp(28), doweDp(28)));\n        {close}.setOnClickListener(v -> {{ {close_action} }});\n        doweAdd({content}, {close}, 8, true);\n        {panel}.addView({content}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n",
        close = close.as_str(),
        close_icon = close_icon.as_str(),
        close_paths = close_paths.as_str(),
        close_action = close_action.as_str()
    ));
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams(Math.min(doweDp(420), Math.max(doweDp(1), Math.max(0, viewportWidth - doweDp(32)))), ViewGroup.LayoutParams.WRAP_CONTENT, {gravity});\n        {panel}Params.setMargins(doweDp(16), doweDp(16), doweDp(16), doweDp(16));\n        {overlay}.addView({panel}, {panel}Params);\n        {popup_ref}[0] = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, false);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(false);\n        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup_ref}[0].showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n"
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
        .map(|path| format!("doweBool(\"{}\")", escape_java(&context.signal_path(path))))
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

fn render_dev_android_overlay_close(
    panel: &str,
    label: &str,
    action: &str,
    gravity: &str,
    margins: &str,
    counter: &mut usize,
    output: &mut String,
) -> String {
    let close = next_dev_view(counter);
    let close_icon = next_dev_view(counter);
    let close_paths = format!("{close}Paths");
    output.push_str(&format!(
        "        FrameLayout {close} = new FrameLayout(this);\n        {close}.setBackground(doweBackground(DOWE_MUTED, 999f));\n        {close}.setContentDescription(\"{}\");\n        {close}.setFocusable(true);\n        {close}.setOnClickListener(v -> {{ {action} }});\n        ArrayList<DoweSvgPathEntry> {close_paths} = new ArrayList<>();\n        {close_paths}.add(new DoweSvgPathEntry(\"M0 0h24v24H0z\", false, null));\n        {close_paths}.add(new DoweSvgPathEntry(\"m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z\", true, null));\n        DoweSvgView {close_icon} = new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_MUTED_TEXT, {close_paths});\n        {close_icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n        {close}.addView({close_icon}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        FrameLayout.LayoutParams {close}Params = new FrameLayout.LayoutParams(doweDp(28), doweDp(28), {gravity});\n        {close}Params.setMargins({margins});\n        {panel}.addView({close}, {close}Params);\n",
        escape_java(label)
    ));
    close
}

fn dev_drawer_close_gravity(position: &DrawerPosition) -> (&'static str, &'static str) {
    match position {
        DrawerPosition::End => ("Gravity.TOP | Gravity.START", "doweDp(8), doweDp(8), 0, 0"),
        DrawerPosition::Top => ("Gravity.BOTTOM | Gravity.END", "0, 0, doweDp(8), doweDp(8)"),
        _ => ("Gravity.TOP | Gravity.END", "0, doweDp(8), doweDp(8), 0"),
    }
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
    let layer = next_dev_view(counter);
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
        "        if (doweBool(\"{path}\")) {{\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        FrameLayout {layer} = new FrameLayout(this);\n        FrameLayout {panel} = new FrameLayout(this);\n        {panel}.setBackground(doweDrawerBackground({}, {}, \"{}\", {}));\n        {panel}.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        {layer}.addView({panel});\n        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams({width}, {height}, {gravity});\n        {overlay}.addView({layer}, {panel}Params);\n        LinearLayout {content} = doweContainer(false);\n",
        dev_card_variant_container(&props.style),
        dev_card_border(&props.style),
        props.position.as_str(),
        dev_drawer_radius(&props.style.style)
    ));
    apply_dev_android_style(&props.style.style, &content, false, output);
    output.push_str(&format!("        {content}.setClipChildren(true);\n        {content}.setClipToPadding(true);\n"));
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {content}Params = doweFrameLayoutParams({content}.getLayoutParams());\n        if ({content}Params.width == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.width = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        if ({content}Params.height == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.height = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        {panel}.addView({content}, {content}Params);\n        ScrollView {body_scroll} = new ScrollView(this);\n        {body_scroll}.setFillViewport(true);\n        LinearLayout {body_content} = doweContainer(false);\n        {body_scroll}.addView({body_content}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
    ));
    if !header.is_empty() {
        let header_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {header_content} = doweContainer(false);\n        {header_content}.setBackgroundColor({});\n        {header_content}.setClipChildren(true);\n        doweAdd({content}, {header_content});\n",
            dev_card_variant_container(&props.style)
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
        "        Runnable {navigation_close} = () -> {{ PopupWindow activePopup = doweActiveOverlay; if (activePopup != null && activePopup.isShowing()) {{ activePopup.dismiss(); }} doweWrite(\"{path}\", false); }};\n        doweDrawerNavigationClose = {navigation_close};\n        {popup}.setOnDismissListener(() -> {{ doweDrawerNavigationClose = null; if (doweActiveOverlay == {popup}) {{ doweActiveOverlay = null; }} }});\n"
    ));
    if !props.disable_overlay_close {
        output.push_str(&format!(
            "        {overlay}.setOnClickListener(v -> {{ {navigation_close}.run(); renderCurrentRoute(false); }});\n"
        ));
    }
    let mut drawer_close_claim = String::new();
    if !props.hide_close_button {
        let (close_gravity, close_margins) = dev_drawer_close_gravity(&props.position);
        let drawer_close = render_dev_android_overlay_close(
            &overlay,
            "Close drawer",
            &format!("{navigation_close}.run(); renderCurrentRoute(false);"),
            close_gravity,
            close_margins,
            counter,
            output,
        );
        drawer_close_claim = format!(
            "            {overlay}.removeView({drawer_close});\n            existingOverlay.addView({drawer_close}, {drawer_close}Params);\n"
        );
    }
    output.push_str(&format!(
        "        if (doweActiveOverlay != null && doweActiveOverlay.isShowing() && doweActiveOverlay.getContentView() instanceof FrameLayout) {{\n            FrameLayout existingOverlay = (FrameLayout) doweActiveOverlay.getContentView();\n            existingOverlay.removeAllViews();\n            {overlay}.removeView({layer});\n            existingOverlay.addView({layer}, {panel}Params);\n{drawer_close_claim}\n            doweOverlayClaimed = doweOverlayRender;\n        }} else {{\n            root.post(() -> {{ if (root.getWindowToken() != null) {{ doweActiveOverlay = {popup}; {popup}.showAtLocation(root, Gravity.FILL, 0, 0); doweOverlayClaimed = doweOverlayRender; }} }});\n        }}\n        }}\n"));
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
