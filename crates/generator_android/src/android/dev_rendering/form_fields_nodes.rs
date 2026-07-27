fn render_dev_android_form_fields_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) {
    match node {
        ViewNode::Input { props } => {
            let view = next_dev_view(counter);
            let has_icons = props.icon_start.is_some() || props.icon_end.is_some();
            let field = if props.label.is_some() || has_icons {
                next_dev_view(counter)
            } else {
                view.clone()
            };
            let field_frame = if has_icons && props.label.is_some() && !props.label_floating {
                Some(next_dev_view(counter))
            } else {
                None
            };
            let radius = dev_style_radius(&props.style);
            let background =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!(
                        "doweInputBackground({}, {}, {radius})",
                        dev_variant_container(props),
                        java_color(ColorToken::Muted)
                    )
                } else {
                    format!(
                        "doweBackground({}, {radius})",
                        dev_variant_container(props)
                    )
                };
            let font = dev_font_value(props.style.font.as_ref().or(inherited_font));
            let content = dev_variant_content(props);
            let field_background = if (props.label_floating && props.label.is_some()) || has_icons {
                "setBackgroundColor(Color.TRANSPARENT)".to_string()
            } else {
                format!("setBackground({background})")
            };
            let start_padding = if props.icon_start.is_some() {
                INPUT_HORIZONTAL_PADDING.native_units() + 32
            } else {
                INPUT_HORIZONTAL_PADDING.native_units()
            };
            let end_padding = if props.icon_end.is_some() {
                INPUT_HORIZONTAL_PADDING.native_units() + 32
            } else {
                INPUT_HORIZONTAL_PADDING.native_units()
            };
            if let Some(label) = props.label.as_deref().filter(|_| !props.label_floating) {
                output.push_str(&format!(
                                            "        LinearLayout {view} = doweContainer(false);\n        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
                                            escape_java(label)
                                        ));
            }
            output.push_str(&format!(
                                        "        EditText {field} = new EditText(this);\n        {field}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {field}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {field}.setTextSize({});\n        {field}.setIncludeFontPadding(false);\n        {field}.setGravity(Gravity.CENTER_VERTICAL);\n        {field}.setTextColor({content});\n        {field}.setSingleLine(true);\n        {field}.setMinWidth(0);\n        {field}.setMinimumWidth(0);\n        {field}.setMinHeight(doweDp({}));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp({}), {}, doweDp({}), 0);\n        {field}.{field_background};\n",
                                        dev_text_size_expr(false, INPUT_TEXT_SIZE),
                                        INPUT_MIN_HEIGHT.native_units(),
                                        INPUT_MIN_HEIGHT.native_units(),
                                        start_padding,
                                        if props.label_floating && props.label.is_some() {
                                            "doweDp(10)"
                                        } else {
                                            "0"
                                        },
                                        end_padding
                                    ));
            let placeholder = props.placeholder.as_deref().unwrap_or_default();
            if !placeholder.is_empty() && !(props.label_floating && props.label.is_some()) {
                output.push_str(&format!(
                                            "        {field}.setHint(\"{}\");\n        {field}.setHintTextColor(doweAlpha({content}, 0.55f));\n",
                                            escape_java(placeholder)
                                        ));
            }
            if let Some(path) = props.element.bind.as_deref() {
                let path = escape_java(&context.signal_path(path));
                output.push_str(&format!(
                                            "        {field}.setText(doweTextValue(\"{path}\", null));\n        {field}.addTextChangedListener(new TextWatcher() {{\n            public void beforeTextChanged(CharSequence value, int start, int count, int after) {{}}\n            public void onTextChanged(CharSequence value, int start, int before, int count) {{}}\n            public void afterTextChanged(Editable value) {{ doweWrite(\"{path}\", value.toString()); }}\n        }});\n"
                                        ));
            }
            let start_icon = props
                .icon_start
                .as_ref()
                .map(|icon| {
                    let icon = render_dev_android_icon_view(
                        icon,
                        counter,
                        output,
                        Some(content),
                    );
                    output.push_str(&format!(
                        "        {icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n"
                    ));
                    icon
                })
                .unwrap_or_else(|| "null".to_string());
            let end_icon = props
                .icon_end
                .as_ref()
                .map(|icon| {
                    let icon = render_dev_android_icon_view(
                        icon,
                        counter,
                        output,
                        Some(content),
                    );
                    output.push_str(&format!(
                        "        {icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n"
                    ));
                    icon
                })
                .unwrap_or_else(|| "null".to_string());
            if let Some(label) = props.label.as_deref().filter(|_| props.label_floating) {
                output.push_str(&format!(
                                            "        FrameLayout {view} = doweFloatingInput({field}, \"{}\", \"{}\", {content}, {font}, {start_icon}, {end_icon}, {background});\n",
                                            escape_java(label),
                                            escape_java(placeholder)
                                        ));
            } else if has_icons {
                let frame = field_frame.as_deref().unwrap_or(&view);
                output.push_str(&format!(
                    "        FrameLayout {frame} = doweInputFrame({field}, {start_icon}, {end_icon}, {background});\n"
                ));
                if props.label.is_some() {
                    output.push_str(&format!(
                        "        doweAdd({view}, {frame}, 4, false);\n"
                    ));
                }
            } else if props.label.is_some() {
                output.push_str(&format!("        doweAdd({view}, {field}, 4, false);\n"));
            }
            let shadow_target = field_frame.as_deref().unwrap_or_else(|| {
                if props.label.is_some() && !props.label_floating && !has_icons {
                    field.as_str()
                } else {
                    view.as_str()
                }
            });
            let mut outer_style = props.style.clone();
            outer_style.shadow = None;
            outer_style.shadow_color = None;
            outer_style.rounded = None;
            apply_dev_android_style(&outer_style, &view, false, output);
            if let Some(rounded) = props.style.rounded.as_ref() {
                output.push_str(&format!(
                    "        doweRound({shadow_target}, {});\n",
                    dev_rounded_value(rounded)
                ));
            }
            apply_dev_android_shadow_with_radius(
                &props.style,
                shadow_target,
                &radius,
                output,
            );
            if parent_horizontal && props.style.sizing.w.is_none() {
                output.push_str(&format!(
                                            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
                                        ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Select {
            props,
            options,
            option_each,
        } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let frame = if props.label.is_some() && !props.label_floating {
                Some(next_dev_view(counter))
            } else {
                None
            };
            let fixed_labels = java_string_array(options.iter().map(|option| option.label.as_str()));
            let fixed_values = java_string_array(options.iter().map(|option| option.value.as_str()));
            let fixed_descriptions = java_string_array(
                options
                    .iter()
                    .map(|option| option.description.as_deref().unwrap_or("")),
            );
            let (labels, values, descriptions) = option_each
                .as_ref()
                .map(|each| {
                    let item_path = |path: &str| {
                        path.strip_prefix(&format!("{}.", each.item))
                            .map(|suffix| format!("item.{suffix}"))
                            .unwrap_or_else(|| path.to_string())
                    };
                    let collection = escape_java(&context.signal_path(&each.collection));
                    let dynamic_labels = format!(
                        "doweRowTextValues(\"{collection}\", \"{}\")",
                        escape_java(&item_path(&each.label))
                    );
                    let dynamic_values = format!(
                        "doweRowTextValues(\"{collection}\", \"{}\")",
                        escape_java(&item_path(&each.value))
                    );
                    let dynamic_descriptions = format!(
                        "doweRowTextValues(\"{collection}\", \"{}\")",
                        escape_java(
                            &each
                                .description
                                .as_deref()
                                .map(item_path)
                                .unwrap_or_default(),
                        )
                    );
                    (
                        format!("doweConcat({fixed_labels}, {dynamic_labels})"),
                        format!("doweConcat({fixed_values}, {dynamic_values})"),
                        format!("doweConcat({fixed_descriptions}, {dynamic_descriptions})"),
                    )
                })
                .unwrap_or((fixed_labels, fixed_values, fixed_descriptions));
            let background =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!(
                        "doweInputBackground({}, {}, DOWE_RADIUS)",
                        dev_variant_container(props),
                        java_color(ColorToken::Muted)
                    )
                } else {
                    format!(
                        "doweBackground({}, DOWE_RADIUS)",
                        dev_variant_container(props)
                    )
                };
            let font = dev_font_value(props.style.font.as_ref().or(inherited_font));
            let content = dev_variant_content(props);
            let placeholder = props.placeholder.as_deref().unwrap_or("Select an option");
            let bind_path = props
                .element
                .bind
                .as_deref()
                .map(|path| format!("\"{}\"", escape_java(&context.signal_path(path))))
                .unwrap_or_else(|| "null".to_string());
            let selected = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    format!(
                        "doweTextValue(\"{}\", null)",
                        escape_java(&context.signal_path(path))
                    )
                })
                .unwrap_or_else(|| "\"\"".to_string());
            let on_select = if props.element.bind.is_some() {
                "value -> renderCurrentRoute(false)"
            } else {
                "null"
            };
            if let Some(label) = props.label.as_deref().filter(|_| !props.label_floating) {
                output.push_str(&format!(
                                            "        LinearLayout {view} = doweContainer(false);\n        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
                                            escape_java(label)
                                        ));
            }
            output.push_str(&format!(
                                        "        String[] {field}Labels = {labels};\n        String[] {field}Values = {values};\n        String[] {field}Descriptions = {descriptions};\n        TextView {field} = doweSelectTrigger(\"{}\", {content}, {font});\n        {field}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp({}), 0, doweDp(36), 0);\n        {field}.setBackgroundColor(Color.TRANSPARENT);\n        final String[] {field}Selected = new String[]{{{selected}}};\n",
                                        escape_java(placeholder),
                                        INPUT_MIN_HEIGHT.native_units(),
                                        INPUT_HORIZONTAL_PADDING.native_units()
                                    ));
            if let Some(label) = props.label.as_deref().filter(|_| props.label_floating) {
                output.push_str(&format!(
                                            "        TextView {field}Label = doweControlLabel(\"{}\", {content}, {font});\n        FrameLayout {view} = doweFloatingSelect({field}, {field}Label, {content}, {background});\n",
                                            escape_java(label)
                                        ));
            } else if let Some(frame) = frame.as_deref() {
                output.push_str(&format!(
                                            "        FrameLayout {frame} = doweSelectFrame({field}, {content}, {background});\n        doweAdd({view}, {frame}, 4, false);\n"
                                        ));
            } else {
                output.push_str(&format!(
                                            "        FrameLayout {view} = doweSelectFrame({field}, {content}, {background});\n"
                                        ));
            }
            let floating_label = if props.label_floating && props.label.is_some() {
                format!("{field}Label")
            } else {
                "null".to_string()
            };
            output.push_str(&format!(
                                        "        doweBindSelect({field}, {floating_label}, {field}Labels, {field}Values, {field}Descriptions, {field}Selected, \"{}\", {content}, {font}, {bind_path}, {}, {on_select});\n",
                                        escape_java(placeholder),
                                        props.label_floating
                                    ));
            apply_dev_android_style(&props.style, &view, false, output);
            if parent_horizontal && props.style.sizing.w.is_none() {
                output.push_str(&format!(
                                            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
                                        ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}

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
        dev_text_size_expr(false, INPUT_TEXT_SIZE),
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

fn render_dev_android_phone_field(
    props: &PhoneFieldProps,
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
    let radius = dev_style_radius(&props.style.style);
    let background = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("doweInputBackground({}, {}, {radius})", dev_variant_container(&props.style), java_color(ColorToken::Muted))
    } else {
        format!("doweBackground({}, {radius})", dev_variant_container(&props.style))
    };
    let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));
    let content = dev_variant_content(&props.style);
    let selected = props.country.as_deref().unwrap_or("US");
    let selected_country = phone_country(Some(selected)).unwrap_or_else(|| phone_countries()[0]);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    if let Some(label) = props.style.label.as_deref().filter(|_| !props.style.label_floating) {
        output.push_str(&format!(
            "        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
            escape_java(label)
        ));
    }
    output.push_str(&format!(
        "        FrameLayout {field} = doweFloatingControl({background});\n        {field}.setMinimumHeight(doweDp(48));\n        LinearLayout {trigger} = doweContainer(true);\n        {trigger}.setGravity(Gravity.CENTER_VERTICAL);\n        {trigger}.setPadding(doweDp(8), 0, doweDp(4), 0);\n        {trigger}.setClickable(true);\n        {trigger}.setFocusable(true);\n        DoweSvgView {trigger_flag} = dowePhoneFlag(\"{}\", {content});\n        {trigger}.addView({trigger_flag}, new LinearLayout.LayoutParams(doweDp(24), doweDp(24)));\n        TextView {dial} = doweText(\"+{}\", {content}, 15f, 700, 0f, 1.2f, {font});\n        {trigger}.addView({dial}, new LinearLayout.LayoutParams(doweDp(48), ViewGroup.LayoutParams.MATCH_PARENT));\n        {field}.addView({trigger}, new FrameLayout.LayoutParams(doweDp(92), ViewGroup.LayoutParams.MATCH_PARENT, Gravity.START | Gravity.CENTER_VERTICAL));\n        EditText {input} = new EditText(this);\n        {input}.setTextSize(16f);\n        {input}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {input}.setTextColor({content});\n        {input}.setSingleLine(true);\n        {input}.setInputType(android.text.InputType.TYPE_CLASS_NUMBER);\n        {input}.setBackgroundColor(Color.TRANSPARENT);\n        {input}.setPadding(doweDp(4), 0, doweDp(12), 0);\n        {input}.setMinHeight(doweDp(48));\n        FrameLayout.LayoutParams {input}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER_VERTICAL);\n        {input}Params.leftMargin = doweDp(92);\n        {field}.addView({input}, {input}Params);\n",
        escape_java(selected_country.code),
        escape_java(selected_country.dial),
    ));
    output.push_str(&format!(
        "        {input}.setKeyListener(android.text.method.DigitsKeyListener.getInstance(\"0123456789\"));\n        {input}.setRawInputType(android.text.InputType.TYPE_CLASS_NUMBER);\n"
    ));
    output.push_str(&format!(
        "        DoweSvgView {trigger_arrow} = doweSelectArrow({content});\n        {trigger}.addView({trigger_arrow}, new LinearLayout.LayoutParams(doweDp(16), doweDp(16)));\n"
    ));
    if let Some(placeholder) = props.style.placeholder.as_deref() {
        output.push_str(&format!(
            "        {input}.setHint(\"{}\");\n        {input}.setHintTextColor(doweAlpha({content}, 0.55f));\n",
            escape_java(placeholder)
        ));
    }
    if let Some(label) = props.style.label.as_deref().filter(|_| props.style.label_floating) {
        let label_view = next_dev_view(counter);
        output.push_str(&format!(
            "        TextView {label_view} = doweControlLabel(\"{}\", {content}, {font});\n        FrameLayout.LayoutParams {label_view}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.START);\n        {label_view}Params.leftMargin = doweDp(96);\n        {label_view}Params.topMargin = doweDp(2);\n        {field}.addView({label_view}, {label_view}Params);\n",
            escape_java(label)
        ));
        output.push_str(&format!("        {input}.setPadding(doweDp(96), doweDp(10), doweDp(12), 0);\n"));
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
        "        String[] {selected_holder} = new String[] {{\"{}\"}};\n        {trigger}.setOnClickListener(target -> dowePhonePopup({trigger}, {dial}, {trigger_flag}, dowePhoneCodes(), dowePhoneNames(), dowePhoneDials(), {selected_holder}, \"{}\", \"{}\", \"{}\", {content}, {font}));\n",
        escape_java(selected_country.code),
        escape_java(&props.search_placeholder),
        escape_java(&props.empty_text),
        escape_java(&props.loading_text)
    ));
    if props.disabled {
        output.push_str(&format!("        {input}.setEnabled(false);\n        {trigger}.setEnabled(false);\n"));
    }
    output.push_str(&format!("        doweAdd({view}, {field}, 4, false);\n"));
    apply_dev_android_style(&props.style.style, &view, false, output);
    if parent_horizontal && props.style.style.sizing.w.is_none() {
        output.push_str(&format!("        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"));
    }
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

fn render_dev_android_pin_field(
    props: &dowe_components::PinFieldProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let row = next_dev_view(counter);
    let pin_cells = format!("{view}PinCells");
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    let (width, height) = match size {
        ButtonSize::Sm => (40, 34),
        ButtonSize::Lg => (52, 48),
        _ => (44, 40),
    };
    let radius = dev_style_radius(&props.style.style);
    let background = if props.style.variant.unwrap_or(ComponentVariant::Solid)
        == ComponentVariant::Outlined
    {
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
    let content = dev_variant_content(&props.style);
    let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));
    let input_type = match props.kind {
        dowe_components::PinFieldKind::Number => "android.text.InputType.TYPE_CLASS_NUMBER",
        dowe_components::PinFieldKind::Password => "android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD",
        dowe_components::PinFieldKind::Text => "android.text.InputType.TYPE_CLASS_TEXT",
    };
    let sanitize = if props.kind == dowe_components::PinFieldKind::Number {
        "next = next.replaceAll(\"[^0-9]\", \"\");"
    } else {
        ""
    };
    let bind_path = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| context.signal_path(path));
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n        LinearLayout {row} = doweContainer(true);\n        {row}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        EditText[] {pin_cells} = new EditText[{}];\n",
        props.length
    ));
    if let Some(label) = props.style.label.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
            escape_java(label)
        ));
    }
    for index in 0..props.length {
        let cell = next_dev_view(counter);
        let initial = if let Some(path) = bind_path.as_deref() {
            format!(
                "doweTextValue(\"{}\", null).length() > {} ? String.valueOf(doweTextValue(\"{}\", null).charAt({})) : \"\"",
                escape_java(path),
                index,
                escape_java(path),
                index
            )
        } else {
            props
                .value
                .as_deref()
                .and_then(|value| value.chars().nth(index as usize))
                .map(|value| format!("\"{}\"", escape_java(&value.to_string())))
                .unwrap_or_else(|| "\"\"".to_string())
        };
        output.push_str(&format!(
            "        EditText {cell} = new EditText(this);\n        {cell}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({width}), doweDp({height})));\n        {cell}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {cell}.setTextSize({});\n        {cell}.setIncludeFontPadding(false);\n        {cell}.setGravity(Gravity.CENTER);\n        {cell}.setTextColor({content});\n        {cell}.setSingleLine(true);\n        {cell}.setInputType({input_type});\n        {cell}.setMaxLines(1);\n        {cell}.setPadding(doweDp(8), 0, doweDp(8), 0);\n        {cell}.setBackground({background});\n        {cell}.setText({initial});\n        {pin_cells}[{index}] = {cell};\n",
            dev_text_size_expr(false, INPUT_TEXT_SIZE)
        ));
        let write = bind_path
            .as_deref()
            .map(|path| format!("doweWrite(\"{}\", pinValue.toString());", escape_java(path)))
            .unwrap_or_default();
        output.push_str(&format!(
            "        {cell}.addTextChangedListener(new TextWatcher() {{\n            public void beforeTextChanged(CharSequence value, int start, int count, int after) {{}}\n            public void onTextChanged(CharSequence value, int start, int before, int count) {{}}\n            public void afterTextChanged(Editable value) {{ String next = value.toString(); {sanitize} if (next.length() > 1) {{ next = next.substring(0, 1); value.replace(0, value.length(), next); return; }} if (!next.equals(value.toString())) {{ value.replace(0, value.length(), next); return; }} StringBuilder pinValue = new StringBuilder(); for (EditText pinCell : {pin_cells}) pinValue.append(pinCell.getText().toString()); {write} if (!next.isEmpty() && {} + 1 < {pin_cells}.length) {pin_cells}[{} + 1].requestFocus(); }}\n        }});\n        {cell}.setOnKeyListener((focused, keyCode, event) -> {{ if (keyCode == KeyEvent.KEYCODE_DEL && event.getAction() == KeyEvent.ACTION_DOWN && {cell}.getText().length() == 0 && {} > 0) {{ {pin_cells}[{} - 1].requestFocus(); return true; }} return false; }});\n        doweAdd({row}, {cell}, 8, true);\n",
            index,
            index,
            index,
            index
        ));
    }
    output.push_str(&format!("        doweAdd({view}, {row});\n"));
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

fn render_dev_android_password_field(
    props: &dowe_components::PasswordFieldProps,
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
    let toggle = next_dev_view(counter);
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
    if let Some(label) = props
        .style
        .label
        .as_deref()
        .filter(|_| !props.style.label_floating)
    {
        output.push_str(&format!(
            "        LinearLayout {view} = doweContainer(false);\n        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
            escape_java(label)
        ));
    } else {
        output.push_str(&format!(
            "        LinearLayout {view} = doweContainer(false);\n"
        ));
    }
    output.push_str(&format!(
        "        EditText {field} = new EditText(this);\n        {field}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {field}.setTextSize({});\n        {field}.setIncludeFontPadding(false);\n        {field}.setGravity(Gravity.CENTER_VERTICAL);\n        {field}.setTextColor({content});\n        {field}.setSingleLine(true);\n        {field}.setInputType(android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD);\n        {field}.setTransformationMethod(android.text.method.PasswordTransformationMethod.getInstance());\n        {field}.setMinWidth(0);\n        {field}.setMinimumWidth(0);\n        {field}.setMinHeight(doweDp({}));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp({}), {}, doweDp(68), 0);\n        {field}.setBackgroundColor(Color.TRANSPARENT);\n",
        dev_text_size_expr(false, INPUT_TEXT_SIZE),
        INPUT_MIN_HEIGHT.native_units(),
        INPUT_MIN_HEIGHT.native_units(),
        INPUT_HORIZONTAL_PADDING.native_units(),
        if props.style.label_floating && props.style.label.is_some() {
            "doweDp(10)"
        } else {
            "0"
        }
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
            "        FrameLayout {frame} = doweFloatingInput({field}, \"{}\", \"{}\", {content}, {font}, null, null, {background});\n",
            escape_java(label),
            escape_java(placeholder)
        ));
    } else {
        output.push_str(&format!(
            "        FrameLayout {frame} = new FrameLayout(this);\n        {frame}.setBackground({background});\n        {frame}.addView({field}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER_VERTICAL));\n"
        ));
    }
    output.push_str(&format!(
        "        TextView {toggle} = doweText(\"Show\", {content}, 12f, 700, 0f, 1.2f, {font});\n        {toggle}.setGravity(Gravity.CENTER);\n        {toggle}.setPadding(doweDp(8), doweDp(6), doweDp(8), doweDp(6));\n        {toggle}.setBackground(doweBackground(Color.TRANSPARENT, DOWE_RADIUS));\n        FrameLayout.LayoutParams {toggle}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.END | Gravity.CENTER_VERTICAL);\n        {toggle}Params.setMargins(0, 0, doweDp(4), 0);\n        {frame}.addView({toggle}, {toggle}Params);\n        final boolean[] {toggle}Visible = new boolean[]{{false}};\n        {toggle}.setOnClickListener(target -> {{\n            {toggle}Visible[0] = !{toggle}Visible[0];\n            {field}.setTransformationMethod({toggle}Visible[0] ? android.text.method.HideReturnsTransformationMethod.getInstance() : android.text.method.PasswordTransformationMethod.getInstance());\n            {toggle}.setText({toggle}Visible[0] ? \"Hide\" : \"Show\");\n            {field}.setSelection({field}.length());\n        }});\n"
    ));
    if read_only {
        output.push_str(&format!("        {toggle}.setEnabled(false);\n"));
    }
    output.push_str(&format!("        doweAdd({view}, {frame});\n"));

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
