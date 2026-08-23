fn render_dev_android_select(
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
    let (props, options, option_each) = match node {
        ViewNode::Select { props, options, option_each } => (props, options, option_each),
        _ => return,
    };

            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
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
            let frame = if props.label.is_some() && !props.label_floating {
                Some(next_dev_view(counter))
            } else {
                None
            };
            let fixed_labels =
                java_string_array(options.iter().map(|option| option.label.as_str()));
            let fixed_values =
                java_string_array(options.iter().map(|option| option.value.as_str()));
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
            let content = dev_inherited_content_color(&props.style, inherited_color.as_deref());
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
                                        "        String[] {field}Labels = {labels};\n        String[] {field}Values = {values};\n        String[] {field}Descriptions = {descriptions};\n        TextView {field} = doweSelectTrigger(\"{}\", {content}, {font});\n        {field}.setTextSize({});\n        {field}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp({}), 0, doweDp(36), 0);\n        {field}.setBackgroundColor(Color.TRANSPARENT);\n        final String[] {field}Selected = new String[]{{{selected}}};\n",
                                        escape_java(placeholder),
                                        text_size,
                                        control_height,
                                        INPUT_HORIZONTAL_PADDING.native_units()
                                    ));
            if let Some(label) = props.label.as_deref().filter(|_| props.label_floating) {
                output.push_str(&format!(
                                            "        TextView {field}Label = doweControlLabel(\"{}\", {content}, {font});\n        FrameLayout {view} = doweFloatingSelect({field}, {field}Label, {content}, {background});\n        {view}.setMinimumHeight(doweDp({control_height}));\n",
                                            escape_java(label)
                                        ));
            } else if let Some(frame) = frame.as_deref() {
                output.push_str(&format!(
                                            "        FrameLayout {frame} = doweSelectFrame({field}, {content}, {background});\n        {frame}.setMinimumHeight(doweDp({control_height}));\n        doweAdd({view}, {frame}, 4, false);\n"
                                        ));
            } else {
                output.push_str(&format!(
                                            "        FrameLayout {view} = doweSelectFrame({field}, {content}, {background});\n        {view}.setMinimumHeight(doweDp({control_height}));\n"
                                        ));
            }
            let floating_label = if props.label_floating && props.label.is_some() {
                format!("{field}Label")
            } else {
                "null".to_string()
            };
            if let Some(wrapper) = validation_wrapper.as_deref() {
                output.push_str(&format!(
                    "        LinearLayout {wrapper} = doweContainer(false);\n        doweAdd({wrapper}, {view});\n"
                ));
            }
            let outer_view = validation_wrapper.as_deref().unwrap_or(&view);
            if has_validation {
                let validation_container = validation_wrapper.as_deref().unwrap_or(&view);
                let validation_surface = frame.as_deref().unwrap_or(&view);
                output.push_str(&format!(
                    "        DoweValidationBinding {field}Validation = doweValidation(\"{field}\", {validation_container}, {validation_surface}, {field}, {}, {}, {}, () -> {field}Selected[0], false, {content}, {font});\n",
                    dev_validation_help(&props.element),
                    dev_validation_error(&props.element),
                    dev_validation_rules(&props.element, context)
                ));
            }
            output.push_str(&format!(
                                        "        doweBindSelect({field}, {floating_label}, {field}Labels, {field}Values, {field}Descriptions, {field}Selected, \"{}\", {content}, {font}, {bind_path}, {}, {on_select}, {});\n",
                                        escape_java(placeholder),
                                        props.label_floating,
                                        if has_validation { format!("{field}Validation::touch") } else { "null".to_string() }
                                    ));
            apply_dev_android_style(&props.style, outer_view, false, output);
            if parent_horizontal && props.style.sizing.w.is_none() {
                output.push_str(&format!(
                                            "        {outer_view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
                                        ));
            }
            output.push_str(&dev_add(parent, outer_view, parent_gap, parent_horizontal));
}
