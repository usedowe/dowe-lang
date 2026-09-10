#[allow(unused_variables)]
fn render_dev_android_form_actions_date(
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
    let ViewNode::Date { props } = node else { return; };
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let has_validation = dev_has_validation(&props.style.element)
                || props.help_text.is_some()
                || props.error_text.is_some();
            let control_height = form_control_min_height(props.size, props.style.label_floating)
                .native_units();
            let text_size = dev_text_size_expr(false, form_control_text_size(props.size));
            let value = dev_bound_text(
                &props.style,
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            let bind = props
                .style
                .element
                .bind
                .as_deref()
                .map(|path| format!("\"{}\"", escape_java(&context.signal_path(path))))
                .unwrap_or_else(|| "null".to_string());
            let min = props
                .min
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let max = props
                .max
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let placeholder = props.style.placeholder.as_deref().unwrap_or("Select date");
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
                                        "        final String[] {field}Selected = new String[]{{{value}}};\n        TextView {field} = doweDateTrigger(\"{}\", {}, {});\n        {field}.setTextSize({});\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp(12), 0, doweDp(36), 0);\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS));\n        doweAdd({view}, {field}, 4, false);\n        doweBindDate({field}, {field}Selected, \"{}\", {}, {}, {}, null, false, {}, {});\n",
                                        escape_java(placeholder),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        text_size,
                                        control_height,
                                        dev_variant_container(&props.style),
                                        java_color(ColorToken::Muted),
                                        escape_java(placeholder),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        bind,
                                        min,
                                        max
                                    ));
            if has_validation {
                output.push_str(&format!(
                    "        DoweValidationBinding {field}Validation = doweValidation(\"{field}\", {view}, {field}, {field}, {}, {}, {}, () -> {field}Selected[0], false, {}, {});\n",
                    dev_nullable_string(props.help_text.as_deref()),
                    dev_nullable_string(props.error_text.as_deref()),
                    dev_validation_rules(&props.style.element, context),
                    dev_variant_content(&props.style),
                    dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                ));
            }
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_date_range(
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
    let ViewNode::DateRange { props } = node else { return; };
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let control_height = form_control_min_height(props.size, props.style.label_floating)
                .native_units();
            let text_size = dev_text_size_expr(false, form_control_text_size(props.size));
            let start = dev_optional_bound_text(
                props.start.as_deref(),
                props.start_value.as_deref().unwrap_or_default(),
                context,
            );
            let end = dev_optional_bound_text(
                props.end.as_deref(),
                props.end_value.as_deref().unwrap_or_default(),
                context,
            );
            let start_bind = props
                .start
                .as_deref()
                .map(|path| format!("\"{}\"", escape_java(&context.signal_path(path))))
                .unwrap_or_else(|| "null".to_string());
            let end_bind = props
                .end
                .as_deref()
                .map(|path| format!("\"{}\"", escape_java(&context.signal_path(path))))
                .unwrap_or_else(|| "null".to_string());
            let min = props
                .min
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let max = props
                .max
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let placeholder = props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select date range");
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
                                        "        final String[] {field}Selected = new String[]{{{start}, {end}}};\n        TextView {field} = doweDateTrigger(\"{}\", {}, {});\n        {field}.setTextSize({});\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp(12), 0, doweDp(36), 0);\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS));\n        doweAdd({view}, {field}, 4, false);\n        doweBindDate({field}, {field}Selected, \"{}\", {}, {}, {}, {}, true, {}, {});\n",
                                        escape_java(placeholder),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        text_size,
                                        control_height,
                                        dev_variant_container(&props.style),
                                        java_color(ColorToken::Muted),
                                        escape_java(placeholder),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        start_bind,
                                        end_bind,
                                        min,
                                        max
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_radio_group(
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
    let ViewNode::RadioGroup { props, options } = node else { return; };
            if matches!(props.presentation, RadioGroupPresentation::Card) {
                let view = next_dev_view(counter);
                let value = dev_bound_text(&props.style, "", context);
                let orientation = if props.orientation == RadioGroupOrientation::Horizontal {
                    "HORIZONTAL"
                } else {
                    "VERTICAL"
                };
                let container = dev_card_variant_container(&props.style);
                let content = dev_card_variant_content(&props.style);
                let title_color = dev_card_variant_title(&props.style);
                output.push_str(&format!(
                    "        LinearLayout {view} = doweContainer(false);\n        {view}.setOrientation(LinearLayout.{orientation});\n"
                ));
                if let Some(label) = props.style.label.as_deref() {
                    let label_view = next_dev_view(counter);
                    output.push_str(&format!(
                        "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                        escape_java(label),
                        dev_scheme_color(&props.style),
                        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                    ));
                }
                for option in options {
                    let item = next_dev_view(counter);
                    let copy = next_dev_view(counter);
                    let title = next_dev_view(counter);
                    let description_view = option.description.as_ref().map(|_| next_dev_view(counter));
                    let indicator = next_dev_view(counter);
                    let selected = format!("{value}.equals(\"{}\")", escape_java(&option.value));
                    output.push_str(&format!(
                        "        FrameLayout {item} = new FrameLayout(this);\n        {item}.setPadding(doweDp(16), doweDp(14), doweDp(44), doweDp(14));\n        {item}.setBackground(doweInputBackground({selected} ? doweAlpha({container}, 0.08f) : {container}, {selected} ? {content} : doweAlpha({content}, 0.18f), DOWE_RADIUS));\n        {item}.setClickable(true);\n        {item}.setFocusable(true);\n        LinearLayout {copy} = doweContainer(false);\n",
                    ));
                    if let Some(icon) = option.icon.as_ref() {
                        let icon_view = render_dev_android_icon_view(icon, counter, output, Some(content));
                        output.push_str(&format!(
                            "        {copy}.setOrientation(LinearLayout.HORIZONTAL);\n        doweAdd({copy}, {icon_view}, 8, true);\n"
                        ));
                    }
                    output.push_str(&format!(
                        "        TextView {title} = doweText(\"{}\", {}, 14f, 600, 0f, 1.2f, null);\n        doweAdd({copy}, {title});\n",
                        escape_java(&option.label),
                        title_color
                    ));
                    if let Some(description) = option.description.as_deref() {
                        let description_view = description_view
                            .as_deref()
                            .expect("radio card description view");
                        output.push_str(&format!(
                            "        TextView {description_view} = doweText(\"{}\", {}, 12f, 400, 0f, 1.2f, null);\n        doweAdd({copy}, {description_view}, 4, false);\n",
                            escape_java(description),
                            content
                        ));
                    }
                    output.push_str(&format!(
                        "        {item}.addView({copy}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        TextView {indicator} = doweText({selected} ? \"●\" : \"○\", {content}, 18f, 400, 0f, 1f, null);\n        {indicator}.setGravity(Gravity.CENTER);\n        {item}.addView({indicator}, new FrameLayout.LayoutParams(doweDp(24), doweDp(24), Gravity.TOP | Gravity.RIGHT));\n        {item}.setEnabled({});\n",
                        !option.disabled
                    ));
                    if let Some(path) = props.style.element.bind.as_ref()
                        && !option.disabled
                    {
                        output.push_str(&format!(
                            "        {item}.setOnClickListener(v -> {{ doweWrite(\"{}\", \"{}\"); renderCurrentRoute(false); }});\n",
                            escape_java(&context.signal_path(path)),
                            escape_java(&option.value)
                        ));
                    }
                    output.push_str(&format!("        doweAdd({view}, {item}, 8, true);\n"));
                }
                apply_dev_android_style(&props.style.style, &view, false, output);
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            } else {
                let view = next_dev_view(counter);
                let value = dev_bound_text(&props.style, "", context);
                let orientation = if props.orientation == RadioGroupOrientation::Horizontal {
                    "HORIZONTAL"
                } else {
                    "VERTICAL"
                };
                output.push_str(&format!(
                                            "        android.widget.RadioGroup {view} = new android.widget.RadioGroup(this);\n        {view}.setOrientation(android.widget.RadioGroup.{orientation});\n"
                                        ));
                if let Some(label) = props.style.label.as_deref() {
                    let label_view = next_dev_view(counter);
                    output.push_str(&format!(
                                                "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                                                escape_java(label),
                                                dev_scheme_color(&props.style),
                                                dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                            ));
                }
                for option in options {
                    let item = next_dev_view(counter);
                    output.push_str(&format!(
                                                "        android.widget.RadioButton {item} = new android.widget.RadioButton(this);\n        {item}.setText(\"{}\");\n        {item}.setTextColor({});\n        {item}.setButtonTintList(ColorStateList.valueOf({}));\n        {item}.setChecked({value}.equals(\"{}\"));\n        {item}.setEnabled({});\n        doweAdd({view}, {item});\n",
                                                escape_java(&option.label),
                                                dev_scheme_color(&props.style),
                                                dev_scheme_color(&props.style),
                                                escape_java(&option.value),
                                                !option.disabled
                                            ));
                }
                apply_dev_android_style(&props.style.style, &view, false, output);
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            }
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_toggle(
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
    let ViewNode::Toggle { props } = node else { return; };
            let view = next_dev_view(counter);
            let switch_view = next_dev_view(counter);
            let checked = dev_bound_bool(&props.style, props.checked, context);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n"
                                    ));
            if let Some(label_left) = props.label_left.as_deref() {
                let left = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {left} = doweText(\"{}\", {}, 14f, 400, 0f, 1.2f, {});\n        doweAdd({view}, {left});\n",
                                            escape_java(label_left),
                                            dev_scheme_color(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        android.widget.Switch {switch_view} = new android.widget.Switch(this);\n        {switch_view}.setText(\"\");\n        {switch_view}.setChecked({checked});\n        {switch_view}.setEnabled({});\n        doweAdd({view}, {switch_view}, 8, true);\n",
                                        !props.disabled
                                    ));
            if let Some(label_right) = props.label_right.as_deref() {
                let right = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {right} = doweText(\"{}\", {}, 14f, 400, 0f, 1.2f, {});\n        doweAdd({view}, {right}, 8, true);\n",
                                            escape_java(label_right),
                                            dev_scheme_color(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            if let Some(label) = props.style.label.as_deref() {
                let label_view = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {label_view} = doweText(\"{}\", {}, 14f, 400, 0f, 1.2f, {});\n        doweAdd({view}, {label_view}, 8, true);\n",
                                            escape_java(label),
                                            dev_scheme_color(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

fn theme_display_label(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
