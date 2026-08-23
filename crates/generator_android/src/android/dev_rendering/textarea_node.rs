fn render_dev_android_textarea(
    props: &dowe_components::TextareaProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let field = next_dev_view(counter);
    let frame = next_dev_view(counter);
    let background =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!(
                "doweInputBackground({}, {}, DOWE_RADIUS)",
                dev_variant_container(&props.style),
                java_color(ColorToken::Muted)
            )
        } else {
            format!(
                "doweBackground({}, DOWE_RADIUS)",
                dev_variant_container(&props.style)
            )
        };
    let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));
    let content = dev_variant_content(&props.style);
    let placeholder = props.style.placeholder.as_deref().unwrap_or_default();
    let read_only = props.readonly || props.disabled;
    let text_size = dev_text_size_expr(
        false,
        form_control_text_size(props.style.size.unwrap_or(ButtonSize::Md)),
    );
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    if let Some(label) = props
        .style
        .label
        .as_deref()
        .filter(|_| !props.style.label_floating)
    {
        output.push_str(&format!(
            "        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
            escape_java(label)
        ));
    }
    output.push_str(&format!(
        "        EditText {field} = new EditText(this);\n        {field}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {field}.setTextSize({});\n        {field}.setIncludeFontPadding(false);\n        {field}.setGravity(Gravity.TOP | Gravity.START);\n        {field}.setTextColor({content});\n        {field}.setSingleLine(false);\n        {field}.setMinLines({});\n        {field}.setMaxLines({});\n        {field}.setMinWidth(0);\n        {field}.setMinimumWidth(0);\n        {field}.setMinHeight(doweDp({}));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp(12), doweDp({}), doweDp(12), doweDp(12));\n        {field}.setBackgroundColor(Color.TRANSPARENT);\n",
        text_size,
        props.rows,
        props.rows,
        props.rows * 28,
        props.rows * 28,
        if props.style.label_floating && props.style.label.is_some() {
            22
        } else {
            12
        }
    ));
    if let Some(max_length) = props.max_length {
        output.push_str(&format!(
            "        {field}.setFilters(new android.text.InputFilter[]{{new android.text.InputFilter.LengthFilter({max_length})}});\n"
        ));
    }
    if let Some(path) = props.style.element.bind.as_deref() {
        output.push_str(&format!(
            "        {field}.setText(doweTextValue(\"{}\", null));\n",
            escape_java(&context.signal_path(path))
        ));
    } else if let Some(value) = props.value.as_deref() {
        output.push_str(&format!(
            "        {field}.setText(\"{}\");\n",
            escape_java(value)
        ));
    }
    if read_only {
        output.push_str(&format!("        {field}.setEnabled(false);\n"));
    }
    if let Some(label) = props
        .style
        .label
        .as_deref()
        .filter(|_| props.style.label_floating)
    {
        output.push_str(&format!(
            "        FrameLayout {frame} = doweFloatingTextarea({field}, \"{}\", \"{}\", {content}, {font}, {background});\n",
            escape_java(label),
            escape_java(placeholder)
        ));
    } else {
        if !placeholder.is_empty() {
            output.push_str(&format!(
                "        {field}.setHint(\"{}\");\n        {field}.setHintTextColor(doweAlpha({content}, 0.55f));\n",
                escape_java(placeholder)
            ));
        }
        output.push_str(&format!(
            "        FrameLayout {frame} = new FrameLayout(this);\n        {frame}.setBackground({background});\n        {frame}.addView({field}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.START));\n"
        ));
    }
    if let Some(path) = props.style.element.bind.as_deref() {
        output.push_str(&format!(
            "        {field}.addTextChangedListener(new TextWatcher() {{\n            public void beforeTextChanged(CharSequence value, int start, int count, int after) {{}}\n            public void onTextChanged(CharSequence value, int start, int before, int count) {{}}\n            public void afterTextChanged(Editable value) {{ doweWrite(\"{}\", value.toString()); }}\n        }});\n",
            escape_java(&context.signal_path(path))
        ));
    }
    output.push_str(&format!("        doweAdd({view}, {frame});\n"));
    if let Some(text) = props.error_text.as_deref().or(props.help_text.as_deref()) {
        let color = if props.error_text.is_some() {
            java_color(ColorToken::Danger).to_string()
        } else {
            format!("doweAlpha({content}, 0.72f)")
        };
        let help = next_dev_view(counter);
        output.push_str(&format!(
            "        TextView {help} = doweText(\"{}\", {color}, 12f, 400, 0f, 1.2f, {font});\n        doweAdd({view}, {help}, 4, false);\n",
            escape_java(text)
        ));
    }
    apply_dev_android_style(&props.style.style, &view, false, output);
    if parent_horizontal && props.style.style.sizing.w.is_none() {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
        ));
    }
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
