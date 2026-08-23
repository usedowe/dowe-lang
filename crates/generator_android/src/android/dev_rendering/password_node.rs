fn render_dev_android_password(
    props: &dowe_components::PasswordProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let field = next_dev_view(counter);
    let frame = next_dev_view(counter);
    let toggle = next_dev_view(counter);
    let control_height = form_control_min_height(
        props.style.size.unwrap_or(ButtonSize::Md),
        props.style.label_floating,
    )
    .native_units();
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
    let content = dev_inherited_content_color(&props.style.style, inherited_color.as_deref());
    let show_icon = solar_control_icon("eye").expect("bundled Password reveal icon");
    let hide_icon = solar_control_icon("eye-closed").expect("bundled Password conceal icon");
    let show_icon_view = render_dev_android_icon_view(&show_icon, counter, output, Some(&content));
    let hide_icon_view = render_dev_android_icon_view(&hide_icon, counter, output, Some(&content));
    let placeholder = props.style.placeholder.as_deref().unwrap_or_default();
    let read_only = props.readonly || props.disabled;
    let text_size = dev_text_size_expr(
        false,
        form_control_text_size(props.style.size.unwrap_or(ButtonSize::Md)),
    );
    if let Some(label) = props
        .style
        .label
        .as_deref()
        .filter(|_| !props.style.label_floating)
    {
        output.push_str(&format!(
            "        LinearLayout {view} = doweContainer(false);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
            escape_java(label)
        ));
    } else {
        output.push_str(&format!(
            "        LinearLayout {view} = doweContainer(false);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
        ));
    }
    output.push_str(&format!(
        "        EditText {field} = new EditText(this);\n        {field}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {field}.setTextSize({});\n        {field}.setIncludeFontPadding(false);\n        {field}.setGravity(Gravity.CENTER_VERTICAL);\n        {field}.setTextColor({content});\n        {field}.setSingleLine(true);\n        {field}.setInputType(android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD);\n        {field}.setTransformationMethod(android.text.method.PasswordTransformationMethod.getInstance());\n        {field}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {field}.setMinWidth(0);\n        {field}.setMinimumWidth(0);\n        {field}.setMinHeight(doweDp({}));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp({}), {}, doweDp({}), 0);\n        {field}.setBackgroundColor(Color.TRANSPARENT);\n",
        text_size,
        control_height,
        control_height,
        INPUT_HORIZONTAL_PADDING.native_units(),
        if props.style.label_floating && props.style.label.is_some() {
            "doweDp(10)"
        } else {
            "0"
        },
        INPUT_HORIZONTAL_PADDING.native_units() + 32
    ));
    if !placeholder.is_empty() && !(props.style.label_floating && props.style.label.is_some()) {
        output.push_str(&format!(
            "        {field}.setHint(\"{}\");\n        {field}.setHintTextColor(doweAlpha({content}, 0.55f));\n",
            escape_java(placeholder)
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
            "        FrameLayout {frame} = doweFloatingInput({field}, \"{}\", \"{}\", {content}, {font}, null, null, {background});\n        {frame}.setMinimumHeight(doweDp({control_height}));\n",
            escape_java(label),
            escape_java(placeholder)
        ));
    } else {
        output.push_str(&format!(
            "        FrameLayout {frame} = new FrameLayout(this);\n        {frame}.setMinimumHeight(doweDp({control_height}));\n        {frame}.setBackground({background});\n        {frame}.addView({field}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, Gravity.CENTER_VERTICAL));\n"
        ));
    }
    output.push_str(&format!(
        "        {frame}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
    ));
    output.push_str(&format!(
        "        FrameLayout {toggle} = new FrameLayout(this);\n        {toggle}.setContentDescription(\"Show password\");\n        {toggle}.setBackground(doweBackground(Color.TRANSPARENT, {radius}));\n        {toggle}.addView({show_icon_view}, new FrameLayout.LayoutParams(doweDp(20), doweDp(20), Gravity.CENTER));\n        {hide_icon_view}.setVisibility(View.GONE);\n        {toggle}.addView({hide_icon_view}, new FrameLayout.LayoutParams(doweDp(20), doweDp(20), Gravity.CENTER));\n        FrameLayout.LayoutParams {toggle}Params = new FrameLayout.LayoutParams(doweDp(32), doweDp(32), Gravity.END | Gravity.CENTER_VERTICAL);\n        {toggle}Params.setMargins(0, 0, doweDp(4), 0);\n        {frame}.addView({toggle}, {toggle}Params);\n        final boolean[] {toggle}Visible = new boolean[]{{false}};\n        {toggle}.setOnClickListener(target -> {{\n            {toggle}Visible[0] = !{toggle}Visible[0];\n            {field}.setTransformationMethod({toggle}Visible[0] ? android.text.method.HideReturnsTransformationMethod.getInstance() : android.text.method.PasswordTransformationMethod.getInstance());\n            {show_icon_view}.setVisibility({toggle}Visible[0] ? View.GONE : View.VISIBLE);\n            {hide_icon_view}.setVisibility({toggle}Visible[0] ? View.VISIBLE : View.GONE);\n            {toggle}.setContentDescription({toggle}Visible[0] ? \"Hide password\" : \"Show password\");\n            {field}.setSelection({field}.length());\n        }});\n"
    ));
    if read_only {
        output.push_str(&format!("        {toggle}.setEnabled(false);\n"));
    }
    output.push_str(&format!("        doweAdd({view}, {frame}, 4, false);\n"));
    if dev_has_validation(&props.style.element) {
        output.push_str(&format!(
            "        DoweValidationBinding {field}Validation = doweValidation(\"{field}\", {view}, {field}, {field}, {}, {}, {}, () -> {field}.getText().toString(), false, {content}, {font});\n        {field}Validation.watchText();\n",
            dev_validation_help(&props.style.element),
            dev_validation_error(&props.style.element),
            dev_validation_rules(&props.style.element, context)
        ));
    }

    let strength_update = if props.hide_strength {
        None
    } else {
        let meter = next_dev_view(counter);
        let bars = next_dev_view(counter);
        let label = next_dev_view(counter);
        let update = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {meter} = doweContainer(false);\n        {meter}.setPadding(0, doweDp(8), 0, 0);\n        LinearLayout {bars} = doweContainer(true);\n        final View[] {bars}Segments = new View[6];\n        for (int index = 0; index < {bars}Segments.length; index += 1) {{\n            View segment = new View(this);\n            LinearLayout.LayoutParams segmentParams = new LinearLayout.LayoutParams(0, doweDp(4), 1f);\n            if (index < {bars}Segments.length - 1) segmentParams.setMargins(0, 0, doweDp(4), 0);\n            segment.setLayoutParams(segmentParams);\n            {bars}.addView(segment);\n            {bars}Segments[index] = segment;\n        }}\n        final TextView {label} = doweText(\"\", doweAlpha({content}, 0.75f), 12f, 700, 0f, 1.2f, {font});\n        doweAdd({meter}, {bars});\n        doweAdd({meter}, {label}, 4, false);\n        doweAdd({view}, {meter}, 6, false);\n        final Runnable {update} = new Runnable() {{\n            @Override public void run() {{\n                String text = {field}.getText().toString();\n                boolean digit = false;\n                boolean uppercase = false;\n                boolean lowercase = false;\n                boolean symbol = false;\n                for (int index = 0; index < text.length(); index += 1) {{\n                    char character = text.charAt(index);\n                    digit = digit || Character.isDigit(character);\n                    uppercase = uppercase || Character.isUpperCase(character);\n                    lowercase = lowercase || Character.isLowerCase(character);\n                    symbol = symbol || !Character.isLetterOrDigit(character);\n                }}\n                int score = (text.length() >= 8 ? 1 : 0) + (text.length() >= 12 ? 1 : 0) + (digit ? 1 : 0) + (uppercase ? 1 : 0) + (lowercase ? 1 : 0) + (symbol ? 1 : 0);\n                int strengthColor = score <= 2 ? DOWE_DANGER : score <= 4 ? DOWE_WARNING : DOWE_SUCCESS;\n                for (int index = 0; index < {bars}Segments.length; index += 1) {{\n                    {bars}Segments[index].setBackground(doweBackground(index < score ? strengthColor : doweAlpha({content}, 0.18f), doweDp(999)));\n                }}\n                {label}.setText(score == 0 ? \"\" : score <= 2 ? \"{}\" : score <= 4 ? \"{}\" : \"{}\");\n                {label}.setTextColor(score == 0 ? doweAlpha({content}, 0.75f) : strengthColor);\n            }}\n        }};\n",
            escape_java(&props.weak_label),
            escape_java(&props.medium_label),
            escape_java(&props.strong_label)
        ));
        Some(update)
    };
    let bind_write = props.style.element.bind.as_deref().map(|path| {
        format!(
            "doweWrite(\"{}\", value.toString());",
            escape_java(&context.signal_path(path))
        )
    });
    if bind_write.is_some() || strength_update.is_some() {
        output.push_str(&format!(
            "        {field}.addTextChangedListener(new TextWatcher() {{\n            public void beforeTextChanged(CharSequence value, int start, int count, int after) {{}}\n            public void onTextChanged(CharSequence value, int start, int before, int count) {{}}\n            public void afterTextChanged(Editable value) {{ {}{} }}\n        }});\n",
            bind_write.as_deref().unwrap_or_default(),
            strength_update
                .as_deref()
                .map(|update| format!(" {update}.run();"))
                .unwrap_or_default()
        ));
    }
    if let Some(update) = strength_update {
        output.push_str(&format!("        {update}.run();\n"));
    }
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
