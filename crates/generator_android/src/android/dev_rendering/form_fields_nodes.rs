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
            let field = if props.label.is_some() {
                next_dev_view(counter)
            } else {
                view.clone()
            };
            let background =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!(
                        "doweInputBackground({}, {}, DOWE_RADIUS_UI)",
                        dev_variant_container(props),
                        java_color(ColorToken::Muted)
                    )
                } else {
                    format!(
                        "doweBackground({}, DOWE_RADIUS_UI)",
                        dev_variant_container(props)
                    )
                };
            let font = dev_font_value(props.style.font.as_ref().or(inherited_font));
            let content = dev_variant_content(props);
            let field_background = if props.label_floating && props.label.is_some() {
                "setBackgroundColor(Color.TRANSPARENT)".to_string()
            } else {
                format!("setBackground({background})")
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
                                        INPUT_HORIZONTAL_PADDING.native_units(),
                                        if props.label_floating && props.label.is_some() {
                                            "doweDp(10)"
                                        } else {
                                            "0"
                                        },
                                        INPUT_HORIZONTAL_PADDING.native_units()
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
            if let Some(label) = props.label.as_deref().filter(|_| props.label_floating) {
                output.push_str(&format!(
                                            "        FrameLayout {view} = doweFloatingInput({field}, \"{}\", \"{}\", {content}, {font}, {background});\n",
                                            escape_java(label),
                                            escape_java(placeholder)
                                        ));
            } else if props.label.is_some() {
                output.push_str(&format!("        doweAdd({view}, {field}, 4, false);\n"));
            }
            apply_dev_android_style(&props.style, &view, false, output);
            if parent_horizontal && props.style.sizing.w.is_none() {
                output.push_str(&format!(
                                            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
                                        ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Select { props, options } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let frame = if props.label.is_some() && !props.label_floating {
                Some(next_dev_view(counter))
            } else {
                None
            };
            let labels = java_string_array(options.iter().map(|option| option.label.as_str()));
            let values = java_string_array(options.iter().map(|option| option.value.as_str()));
            let descriptions = java_string_array(
                options
                    .iter()
                    .map(|option| option.description.as_deref().unwrap_or("")),
            );
            let background =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!(
                        "doweInputBackground({}, {}, DOWE_RADIUS_UI)",
                        dev_variant_container(props),
                        java_color(ColorToken::Muted)
                    )
                } else {
                    format!(
                        "doweBackground({}, DOWE_RADIUS_UI)",
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
                                        "        doweBindSelect({field}, {floating_label}, {field}Labels, {field}Values, {field}Descriptions, {field}Selected, \"{}\", {content}, {font}, {bind_path}, {});\n",
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
