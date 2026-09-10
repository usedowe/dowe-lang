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
        render_dev_android_overlay_close(
            &panel,
            "Close modal",
            &close,
            "Gravity.TOP | Gravity.END",
            "0, doweDp(8), doweDp(8), 0",
            counter,
            output,
        );
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
