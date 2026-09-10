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
