fn render_dev_android_combo(
    props: &ComboBoxProps,
    options: &[ComboOption],
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
    let input = next_dev_view(counter);
    let floating_label = props.style.label_floating.then(|| next_dev_view(counter));
    let clear = props.clearable.then(|| next_dev_view(counter));
    let has_validation = dev_has_validation(&props.style.element)
        || props.help_text.is_some()
        || props.error_text.is_some();
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    let control_height = form_control_min_height(size, props.style.label_floating).native_units();
    let text_size = dev_text_size_expr(false, form_control_text_size(size));
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
    let labels = options
        .iter()
        .map(|option| format!("\"{}\"", escape_java(&option.label)))
        .collect::<Vec<_>>()
        .join(", ");
    let values = options
        .iter()
        .map(|option| format!("\"{}\"", escape_java(&option.value)))
        .collect::<Vec<_>>()
        .join(", ");
    let descriptions = options
        .iter()
        .map(|option| {
            format!(
                "\"{}\"",
                escape_java(option.description.as_deref().unwrap_or_default())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let disabled = options
        .iter()
        .map(|option| option.disabled.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let icons = options
        .iter()
        .map(|option| {
            option
                .icon
                .map(|icon| {
                    let view = render_dev_android_icon_view(
                        &view_icon(icon),
                        counter,
                        output,
                        Some(content),
                    );
                    view
                })
                .map(|view| view.to_string())
                .unwrap_or_else(|| "null".to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let selected = next_dev_view(counter);
    let initial_value = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| {
            format!(
                "doweTextValue(\"{}\", null)",
                escape_java(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| {
            format!(
                "\"{}\"",
                escape_java(props.value.as_deref().unwrap_or_default())
            )
        });
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    if let Some(label) = props
        .style
        .label
        .as_deref()
        .filter(|_| !props.style.label_floating)
    {
        output.push_str(&format!("        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n", escape_java(label)));
    }
    output.push_str(&format!("        TextView {input} = doweSelectTrigger(\"{}\", {content}, {font});\n        {input}.setTextSize({text_size});\n        {input}.setMinHeight(doweDp({control_height}));\n        FrameLayout {field} = doweFloatingControl({background});\n        {field}.setMinimumHeight(doweDp({control_height}));\n        {field}.addView({input}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, Gravity.CENTER_VERTICAL));\n        DoweSvgView {field}Arrow = doweSelectArrow({content});\n        FrameLayout.LayoutParams {field}ArrowParams = new FrameLayout.LayoutParams(doweDp(16), doweDp(16), Gravity.END | Gravity.CENTER_VERTICAL);\n        {field}ArrowParams.rightMargin = doweDp(12);\n        {field}.addView({field}Arrow, {field}ArrowParams);\n",
        escape_java(props.style.placeholder.as_deref().unwrap_or("Select an option"))
    ));
    if let Some(label) = props
        .style
        .label
        .as_deref()
        .filter(|_| props.style.label_floating)
    {
        let label_view = floating_label.as_deref().unwrap_or(input.as_str());
        output.push_str(&format!("        TextView {label_view} = doweControlLabel(\"{}\", {content}, {font});\n        FrameLayout.LayoutParams {label_view}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.START);\n        {label_view}Params.leftMargin = doweDp(12);\n        {label_view}Params.rightMargin = doweDp(36);\n        {label_view}Params.topMargin = doweDp(2);\n        {field}.addView({label_view}, {label_view}Params);\n", escape_java(label)));
    }
    output.push_str(&format!("        String[] {selected} = new String[] {{{initial_value}}};\n        doweUpdateSelectTrigger({input}, {}, new String[] {{{labels}}}, new String[] {{{values}}}, {selected}[0], \"{}\", {}, false);\n", floating_label.as_deref().unwrap_or("null"), escape_java(props.style.placeholder.as_deref().unwrap_or("Select an option")), props.style.label_floating));
    if let Some(clear_view) = clear.as_deref() {
        output.push_str(&format!("        TextView {clear_view} = doweText(\"×\", {content}, 18f, 400, 0f, 1.2f, {font});\n        {clear_view}.setGravity(Gravity.CENTER);\n        {clear_view}.setOnClickListener(target -> {{ {selected}[0] = \"\"; doweUpdateSelectTrigger({input}, {}, new String[] {{{labels}}}, new String[] {{{values}}}, {selected}[0], \"{}\", {}, false); {} }});\n        FrameLayout.LayoutParams {clear_view}Params = new FrameLayout.LayoutParams(doweDp(28), ViewGroup.LayoutParams.MATCH_PARENT, Gravity.END | Gravity.CENTER_VERTICAL);\n        {clear_view}Params.rightMargin = doweDp(28);\n        {field}.addView({clear_view}, {clear_view}Params);\n", floating_label.as_deref().unwrap_or("null"), escape_java(props.style.placeholder.as_deref().unwrap_or("Select an option")), props.style.label_floating, props.style.element.bind.as_deref().map(|path| format!("doweWrite(\"{}\", \"\");", escape_java(&context.signal_path(path)))).unwrap_or_default()));
    }
    if props.disabled {
        output.push_str(&format!(
            "        {input}.setEnabled(false);\n        {field}.setAlpha(0.56f);\n"
        ));
    }
    if let Some(path) = props.style.element.bind.as_deref() {
        output.push_str(&format!("        {input}.setOnClickListener(target -> doweComboPopup({input}, {}, new String[] {{{labels}}}, new String[] {{{values}}}, new String[] {{{descriptions}}}, new boolean[] {{{disabled}}}, new DoweSvgView[] {{{icons}}}, {selected}, \"{}\", \"{}\", \"{}\", \"{}\", {content}, {font}, {}, \"{}\", {}));\n", floating_label.as_deref().unwrap_or("null"), escape_java(props.style.placeholder.as_deref().unwrap_or("Select an option")), escape_java(&props.search_placeholder), escape_java(&props.empty_text), escape_java(&props.loading_text), props.style.label_floating, escape_java(&context.signal_path(path)), if has_validation { format!("() -> doweTouchValidation({input})") } else { "null".to_string() }));
    } else {
        output.push_str(&format!("        {input}.setOnClickListener(target -> doweComboPopup({input}, {}, new String[] {{{labels}}}, new String[] {{{values}}}, new String[] {{{descriptions}}}, new boolean[] {{{disabled}}}, new DoweSvgView[] {{{icons}}}, {selected}, \"{}\", \"{}\", \"{}\", \"{}\", {content}, {font}, {}, null, {}));\n", floating_label.as_deref().unwrap_or("null"), escape_java(props.style.placeholder.as_deref().unwrap_or("Select an option")), escape_java(&props.search_placeholder), escape_java(&props.empty_text), escape_java(&props.loading_text), props.style.label_floating, if has_validation { format!("() -> doweTouchValidation({input})") } else { "null".to_string() }));
    }
    if has_validation {
        output.push_str(&format!("        DoweValidationBinding {input}Validation = doweValidation(\"{input}\", {view}, {field}, {input}, {}, {}, {}, () -> {selected}[0], false, {content}, {font});\n        {input}Validation.watchText();\n", dev_nullable_string(props.help_text.as_deref()), dev_nullable_string(props.error_text.as_deref()), dev_validation_rules(&props.style.element, context)));
    }
    apply_dev_android_style(&props.style.style, &view, false, output);
    if parent_horizontal && props.style.style.sizing.w.is_none() {
        output.push_str(&format!("        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"));
    }
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
