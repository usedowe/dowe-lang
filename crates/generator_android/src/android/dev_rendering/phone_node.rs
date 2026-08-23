fn render_dev_android_phone(
    props: &PhoneProps,
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
    let trigger = next_dev_view(counter);
    let dial = next_dev_view(counter);
    let trigger_arrow = next_dev_view(counter);
    let trigger_flag = next_dev_view(counter);
    let input = next_dev_view(counter);
    let has_validation = dev_has_validation(&props.style.element)
        || props.help_text.is_some()
        || props.error_text.is_some();
    let control_height = form_control_min_height(
        props.style.size.unwrap_or(ButtonSize::Md),
        props.style.label_floating,
    )
    .native_units();
    let text_size = dev_text_size_expr(
        false,
        form_control_text_size(props.style.size.unwrap_or(ButtonSize::Md)),
    );
    let radius = dev_style_radius(&props.style.style);
    let background =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!(
                "doweInputBackground({}, {}, {radius})",
                dev_variant_container(&props.style),
                java_color(ColorToken::Muted)
            )
        } else {
            format!(
                "doweBackground({}, {radius})",
                dev_variant_container(&props.style)
            )
        };
    let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));
    let content = dev_variant_content(&props.style);
    let selected = props.country.as_deref().unwrap_or("US");
    let selected_country = phone_country(Some(selected)).unwrap_or_else(|| phone_countries()[0]);
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
        "        FrameLayout {field} = doweFloatingControl({background});\n        {field}.setMinimumHeight(doweDp({control_height}));\n        LinearLayout {trigger} = doweContainer(true);\n        {trigger}.setGravity(Gravity.CENTER_VERTICAL);\n        {trigger}.setPadding(doweDp(8), 0, doweDp(4), 0);\n        {trigger}.setClickable(true);\n        {trigger}.setFocusable(true);\n        DoweSvgView {trigger_flag} = dowePhoneFlag(\"{}\", {content});\n        LinearLayout.LayoutParams {trigger_flag}Params = new LinearLayout.LayoutParams(doweDp(24), doweDp(24));\n        {trigger_flag}Params.gravity = Gravity.CENTER_VERTICAL;\n        {trigger}.addView({trigger_flag}, {trigger_flag}Params);\n        TextView {dial} = doweText(\"+{}\", {content}, {}, 700, 0f, 1.2f, {font});\n        {dial}.setGravity(Gravity.CENTER_VERTICAL);\n        {dial}.setIncludeFontPadding(false);\n        LinearLayout.LayoutParams {dial}Params = new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);\n        {dial}Params.gravity = Gravity.CENTER_VERTICAL;\n        {dial}Params.leftMargin = doweDp(6);\n        {dial}Params.rightMargin = doweDp(4);\n        {trigger}.addView({dial}, {dial}Params);\n        {field}.addView({trigger}, new FrameLayout.LayoutParams(doweDp(92), ViewGroup.LayoutParams.MATCH_PARENT, Gravity.START | Gravity.CENTER_VERTICAL));\n        EditText {input} = new EditText(this);\n        {input}.setTextSize({});\n        {input}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {input}.setTextColor({content});\n        {input}.setSingleLine(true);\n        {input}.setInputType(android.text.InputType.TYPE_CLASS_NUMBER);\n        {input}.setBackgroundColor(Color.TRANSPARENT);\n        {input}.setPadding(doweDp(4), 0, doweDp(12), 0);\n        {input}.setMinHeight(doweDp({control_height}));\n        FrameLayout.LayoutParams {input}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER_VERTICAL);\n        {input}Params.leftMargin = doweDp(92);\n        {field}.addView({input}, {input}Params);\n",
        escape_java(selected_country.code),
        escape_java(selected_country.dial),
        &text_size,
        &text_size,
    ));
    output.push_str(&format!(
        "        {input}.setKeyListener(android.text.method.DigitsKeyListener.getInstance(\"0123456789\"));\n        {input}.setRawInputType(android.text.InputType.TYPE_CLASS_NUMBER);\n"
    ));
    output.push_str(&format!(
        "        DoweSvgView {trigger_arrow} = doweSelectArrow({content});\n        LinearLayout.LayoutParams {trigger_arrow}Params = new LinearLayout.LayoutParams(doweDp(16), doweDp(16));\n        {trigger_arrow}Params.gravity = Gravity.CENTER_VERTICAL;\n        {trigger}.addView({trigger_arrow}, {trigger_arrow}Params);\n"
    ));
    if let Some(placeholder) = props.style.placeholder.as_deref() {
        output.push_str(&format!(
            "        {input}.setHint(\"{}\");\n        {input}.setHintTextColor(doweAlpha({content}, 0.55f));\n",
            escape_java(placeholder)
        ));
    }
    if let Some(label) = props
        .style
        .label
        .as_deref()
        .filter(|_| props.style.label_floating)
    {
        let label_view = next_dev_view(counter);
        output.push_str(&format!(
            "        TextView {label_view} = doweControlLabel(\"{}\", {content}, {font});\n        FrameLayout.LayoutParams {label_view}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.START);\n        {label_view}Params.leftMargin = doweDp(96);\n        {label_view}Params.topMargin = doweDp(2);\n        {field}.addView({label_view}, {label_view}Params);\n",
            escape_java(label)
        ));
        output.push_str(&format!(
            "        {input}.setPadding(doweDp(96), doweDp(10), doweDp(12), 0);\n"
        ));
    }
    if let Some(path) = props.style.element.bind.as_deref() {
        let path = escape_java(&context.signal_path(path));
        output.push_str(&format!(
            "        {input}.setText(doweTextValue(\"{path}\", null));\n        {input}.addTextChangedListener(new TextWatcher() {{ public void beforeTextChanged(CharSequence value, int start, int count, int after) {{}} public void onTextChanged(CharSequence value, int start, int before, int count) {{}} public void afterTextChanged(Editable value) {{ doweWrite(\"{path}\", value.toString()); }} }});\n"
        ));
    } else if let Some(value) = props.value.as_deref() {
        let digits = value
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();
        output.push_str(&format!(
            "        {input}.setText(\"{}\");\n",
            escape_java(&digits)
        ));
    }
    let selected_holder = next_dev_view(counter);
    output.push_str(&format!(
        "        String[] {selected_holder} = new String[] {{\"{}\"}};\n",
        escape_java(selected_country.code),
    ));
    if props.disabled {
        output.push_str(&format!(
            "        {input}.setEnabled(false);\n        {trigger}.setEnabled(false);\n"
        ));
    }
    output.push_str(&format!("        doweAdd({view}, {field}, 4, false);\n"));
    if has_validation {
        output.push_str(&format!(
            "        DoweValidationBinding {input}Validation = doweValidation(\"{input}\", {view}, {field}, {input}, {}, {}, {}, () -> {input}.getText().toString(), false, {content}, {font});\n        {input}Validation.watchText();\n",
            dev_nullable_string(props.help_text.as_deref()),
            dev_nullable_string(props.error_text.as_deref()),
            dev_validation_rules(&props.style.element, context)
        ));
    }
    output.push_str(&format!(
        "        {trigger}.setOnClickListener(target -> dowePhonePopup({trigger}, {dial}, {trigger_flag}, dowePhoneCodes(), dowePhoneNames(), dowePhoneDials(), {selected_holder}, \"{}\", \"{}\", \"{}\", {content}, {font}, {}));\n",
        escape_java(&props.search_placeholder),
        escape_java(&props.empty_text),
        escape_java(&props.loading_text),
        if has_validation { format!("{input}Validation::touch") } else { "null".to_string() }
    ));
    apply_dev_android_style(&props.style.style, &view, false, output);
    if parent_horizontal && props.style.style.sizing.w.is_none() {
        output.push_str(&format!("        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"));
    }
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
