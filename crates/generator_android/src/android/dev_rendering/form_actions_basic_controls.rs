#[allow(unused_variables)]
fn render_dev_android_form_actions_slider(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) {
    let ViewNode::Slider { props } = node else { return; };
            let view = next_dev_view(counter);
            let bar = next_dev_view(counter);
            let value_view = next_dev_view(counter);
            let min = props.min.parse::<i32>().unwrap_or(0);
            let max = props.max.parse::<i32>().unwrap_or(100);
            let value = props.value.parse::<i32>().unwrap_or(min).clamp(min, max);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(false);\n        TextView {value_view} = doweControlLabel(String.valueOf({value}), {}, {});\n",
                                        dev_scheme_color(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                    ));
            if !props.hide_label {
                output.push_str(&format!(
                                            "        LinearLayout {view}Info = doweContainer(true);\n        TextView {view}Label = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}Info, {view}Label);\n        doweAdd({view}Info, {value_view}, 8, true);\n        doweAdd({view}, {view}Info);\n",
                                            escape_java(props.style.label.as_deref().unwrap_or_default()),
                                            dev_scheme_color(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        SeekBar {bar} = new SeekBar(this);\n        {bar}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {bar}.setMax({});\n        {bar}.setProgress({});\n        {bar}.setProgressTintList(ColorStateList.valueOf({}));\n        {bar}.setThumbTintList(ColorStateList.valueOf({}));\n        {bar}.setProgressBackgroundTintList(ColorStateList.valueOf({}));\n",
                                        (max - min).max(1),
                                        (value - min).max(0),
                                        dev_scheme_color(&props.style),
                                        dev_scheme_color(&props.style),
                                        java_color(ColorToken::Muted)
                                    ));
            if let Some(path) = props.style.element.bind.as_deref() {
                let path = escape_java(&context.signal_path(path));
                output.push_str(&format!(
                                            "        int {view}BoundValue = {value};\n        try {{ {view}BoundValue = (int)Math.round(Double.parseDouble(doweTextValue(\"{path}\", null))); }} catch (NumberFormatException ignored) {{}}\n        {view}BoundValue = Math.max({min}, Math.min({max}, {view}BoundValue));\n        {bar}.setProgress({view}BoundValue - {min});\n        {value_view}.setText(String.valueOf({view}BoundValue));\n        {bar}.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {{ public void onProgressChanged(SeekBar seekBar, int progress, boolean fromUser) {{ int value = progress + {min}; doweWrite(\"{path}\", value); {value_view}.setText(String.valueOf(value)); }} public void onStartTrackingTouch(SeekBar seekBar) {{}} public void onStopTrackingTouch(SeekBar seekBar) {{}} }});\n"
                                        ));
            }
            output.push_str(&format!("        doweAdd({view}, {bar}, 4, false);\n"));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_dropzone(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) {
    let ViewNode::Dropzone { props } = node else { return; };
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let key = props
                .name
                .as_deref()
                .map(|name| format!("dropzone:{name}"))
                .unwrap_or_else(|| format!("dropzone:{field}"));
            let accept = props
                .accept
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let max_size = props
                .max_size
                .map(|value| format!("{value}L"))
                .unwrap_or_else(|| "-1L".to_string());
            let placeholder = props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Drag & drop files here or click to select");
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            if let Some(label) = props.style.label.as_deref() {
                let label_view = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                                            escape_java(label),
                                            dev_inherited_content_color(&props.style.style, inherited_color.as_deref()),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        TextView {field} = doweText(doweDropzoneText(\"{}\", \"{}\"), {}, 14f, 500, 0f, 1.4f, {});\n        {field}.setGravity(Gravity.CENTER);\n        {field}.setMinHeight(doweDp({}));\n        {field}.setPadding(doweDp(24), doweDp(24), doweDp(24), doweDp(24));\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS));\n        {field}.setEnabled({});\n        {field}.setFocusable(true);\n        {field}.setOnClickListener(view -> doweOpenDropzonePicker(\"{}\", {}, {}, {}));\n        doweAdd({view}, {field}, 4, false);\n",
                                        escape_java(&key),
                                        escape_java(placeholder),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        dev_dropzone_height(props.size),
                                        dev_variant_container(&props.style),
                                        if props.error_text.is_some() {
                                            java_color(ColorToken::Danger).to_string()
                                        } else {
                                            dev_variant_content(&props.style).to_string()
                                        },
                                        !props.disabled,
                                        escape_java(&key),
                                        accept,
                                        props.multiple,
                                        max_size
                                    ));
            if let Some(text) = props.error_text.as_deref().or(props.help_text.as_deref()) {
                let help = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {help} = doweText(\"{}\", {}, 12f, 400, 0f, 1.2f, {});\n        doweAdd({view}, {help}, 4, false);\n",
                                            escape_java(text),
                                            if props.error_text.is_some() {
                                                java_color(ColorToken::Danger).to_string()
                                            } else {
                                                dev_variant_content(&props.style).to_string()
                                            },
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_checkbox(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) {
    let ViewNode::Checkbox { props } = node else { return; };
            let view = next_dev_view(counter);
            let has_validation = dev_has_validation(&props.style.element);
            let wrapper = has_validation.then(|| next_dev_view(counter));
            let checked = dev_bound_bool(&props.style, props.checked, context);
            output.push_str(&format!(
                                        "        android.widget.CheckBox {view} = new android.widget.CheckBox(this);\n        {view}.setText(\"{}\");\n        {view}.setTextColor({});\n        {view}.setButtonTintList(ColorStateList.valueOf({}));\n        {view}.setChecked({checked});\n        {view}.setEnabled({});\n",
                                        escape_java(props.style.label.as_deref().unwrap_or_default()),
                                        dev_scheme_color(&props.style),
                                        dev_scheme_color(&props.style),
                                        !props.disabled
                                    ));
            if let Some(wrapper) = wrapper.as_deref() {
                output.push_str(&format!(
                    "        LinearLayout {wrapper} = doweContainer(false);\n        doweAdd({wrapper}, {view});\n        DoweValidationBinding {view}Validation = doweValidation(\"{view}\", {wrapper}, {view}, {view}, {}, {}, {}, () -> String.valueOf({view}.isChecked()), true, {}, {});\n",
                    dev_validation_help(&props.style.element),
                    dev_validation_error(&props.style.element),
                    dev_boolean_validation_rules(&props.style.element, context),
                    dev_scheme_color(&props.style),
                    dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                ));
            }
            let touch = has_validation
                .then(|| format!("{view}Validation.touch(); "))
                .unwrap_or_default();
            if let Some(path) = props.style.element.bind.as_ref() {
                output.push_str(&format!(
                    "        {view}.setOnCheckedChangeListener((button, value) -> {{ {touch}doweWrite(\"{}\", value); renderCurrentRoute(false); }});\n",
                    escape_java(&context.signal_path(path))
                ));
            } else if has_validation {
                output.push_str(&format!(
                    "        {view}.setOnCheckedChangeListener((button, value) -> {view}Validation.touch());\n"
                ));
            }
            let outer_view = wrapper.as_deref().unwrap_or(&view);
            apply_dev_android_style(&props.style.style, outer_view, false, output);
            output.push_str(&dev_add(parent, outer_view, parent_gap, parent_horizontal));
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_color(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) {
    let ViewNode::Color { props } = node else { return; };
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let swatch = next_dev_view(counter);
            let control_height = form_control_min_height(props.size, props.style.label_floating)
                .native_units();
            let text_size = dev_text_size_expr(false, form_control_text_size(props.size));
            let swatch_size = match props.size {
                ButtonSize::Sm => 20,
                ButtonSize::Lg => 32,
                _ => 24,
            };
            let value = dev_bound_text(&props.style, &props.value, context);
            let bind = props
                .style
                .element
                .bind
                .as_deref()
                .map(|path| format!("\"{}\"", escape_java(&context.signal_path(path))))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            if let Some(label) = props.style.label.as_deref() {
                let label_view = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                                            escape_java(label),
                                            dev_inherited_content_color(&props.style.style, inherited_color.as_deref()),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        LinearLayout {field} = doweContainer(true);\n        {field}.setGravity(Gravity.CENTER_VERTICAL);\n        {field}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS));\n        View {swatch} = new View(this);\n        {swatch}.setLayoutParams(new LinearLayout.LayoutParams(doweDp(24), doweDp(24)));\n        try {{ {swatch}.setBackgroundColor(Color.parseColor({value})); }} catch (IllegalArgumentException ignored) {{ {swatch}.setBackgroundColor({}); }}\n        doweAdd({field}, {swatch});\n        TextView {field}Value = doweText({value}.toUpperCase(), {}, {}, 600, 0f, 1.2f, {});\n        {field}Value.setPadding(doweDp(10), 0, 0, 0);\n        doweAdd({field}, {field}Value);\n        final String[] {field}Selected = new String[]{{doweColorHex(doweColorRgb({value}))}};\n        doweBindColor({field}, {swatch}, {field}Value, {field}Selected, {bind}, {}, {}, {}, {}, {}, {});\n        doweAdd({view}, {field}, 4, false);\n",
                                        dev_variant_container(&props.style),
                                        java_color(ColorToken::Muted),
                                        dev_variant_container(&props.style),
                                        dev_variant_content(&props.style),
                                        text_size,
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        props.show_hex,
                                        props.show_rgb,
                                        props.show_cmyk,
                                        props.show_oklch,
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                    ));
            output.push_str(&format!(
                "        {field}.setMinimumHeight(doweDp({control_height}));\n        {swatch}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({swatch_size}), doweDp({swatch_size})));\n"
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

