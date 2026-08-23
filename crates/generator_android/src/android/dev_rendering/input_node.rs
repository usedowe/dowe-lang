fn render_dev_android_input(
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
    let props = match node {
        ViewNode::Input { props } => props,
        _ => return,
    };

            let view = next_dev_view(counter);
            let has_validation = dev_has_validation(&props.element);
            let validation_wrapper = if has_validation
                && !props
                    .label
                    .as_deref()
                    .is_some_and(|_| !props.label_floating)
            {
                Some(next_dev_view(counter))
            } else {
                None
            };
            let control_size = props.size.unwrap_or(ButtonSize::Md);
            let control_height =
                form_control_min_height(control_size, props.label_floating).native_units();
            let text_size = dev_text_size_expr(false, form_control_text_size(control_size));
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
                    format!("doweBackground({}, {radius})", dev_variant_container(props))
                };
            let font = dev_font_value(props.style.font.as_ref().or(inherited_font));
            let content = dev_inherited_content_color(&props.style, inherited_color.as_deref());
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
                                        text_size,
                                        control_height,
                                        control_height,
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
                        Some(&content),
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
                        Some(&content),
                    );
                    output.push_str(&format!(
                        "        {icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n"
                    ));
                    icon
                })
                .unwrap_or_else(|| "null".to_string());
            if let Some(label) = props.label.as_deref().filter(|_| props.label_floating) {
                output.push_str(&format!(
                                            "        FrameLayout {view} = doweFloatingInput({field}, \"{}\", \"{}\", {content}, {font}, {start_icon}, {end_icon}, {background});\n        {view}.setMinimumHeight(doweDp({control_height}));\n",
                                            escape_java(label),
                                            escape_java(placeholder)
                                        ));
            } else if has_icons {
                let frame = field_frame.as_deref().unwrap_or(&view);
                output.push_str(&format!(
                    "        FrameLayout {frame} = doweInputFrame({field}, {start_icon}, {end_icon}, {background});\n        {frame}.setMinimumHeight(doweDp({control_height}));\n"
                ));
                if props.label.is_some() {
                    output.push_str(&format!("        doweAdd({view}, {frame}, 4, false);\n"));
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
            if let Some(wrapper) = validation_wrapper.as_deref() {
                output.push_str(&format!(
                    "        LinearLayout {wrapper} = doweContainer(false);\n        doweAdd({wrapper}, {view});\n"
                ));
            }
            let outer_view = validation_wrapper.as_deref().unwrap_or(&view);
            if has_validation {
                let validation_container = validation_wrapper.as_deref().unwrap_or(&view);
                output.push_str(&format!(
                    "        DoweValidationBinding {field}Validation = doweValidation(\"{field}\", {validation_container}, {shadow_target}, {field}, {}, {}, {}, () -> {field}.getText().toString(), false, {content}, {font});\n        {field}Validation.watchText();\n",
                    dev_validation_help(&props.element),
                    dev_validation_error(&props.element),
                    dev_validation_rules(&props.element, context)
                ));
            }
            let mut outer_style = props.style.clone();
            outer_style.shadow = None;
            outer_style.shadow_color = None;
            outer_style.rounded = None;
            apply_dev_android_style(&outer_style, outer_view, false, output);
            if let Some(rounded) = props.style.rounded.as_ref() {
                output.push_str(&format!(
                    "        doweRound({shadow_target}, {});\n",
                    dev_rounded_value(rounded)
                ));
            }
            apply_dev_android_shadow_with_radius(&props.style, shadow_target, &radius, output);
            if parent_horizontal && props.style.sizing.w.is_none() {
                output.push_str(&format!(
                                            "        {outer_view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
                                        ));
            }
            output.push_str(&dev_add(parent, outer_view, parent_gap, parent_horizontal));
}
