fn render_dev_android_form_actions_node(
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
        ViewNode::ToggleTheme { props } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        Button {view} = new Button(this);\n        final boolean[] {view}Dark = new boolean[]{{\"dark\".equals(getSharedPreferences(\"dowe\", 0).getString(\"theme-preference\", \"light\"))}};\n        {view}.setText({view}Dark[0] ? \"sun\" : \"moon\");\n        {view}.setAllCaps(false);\n        {view}.setTextColor({});\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n        {view}.setOnClickListener(v -> {{ {view}Dark[0] = !{view}Dark[0]; getSharedPreferences(\"dowe\", 0).edit().putString(\"theme-preference\", {view}Dark[0] ? \"dark\" : \"light\").apply(); {view}.setText({view}Dark[0] ? \"sun\" : \"moon\"); }});\n",
                                        dev_variant_content(&props.style),
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Fab { props, actions } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(false);\n        {view}.setGravity(Gravity.CENTER_HORIZONTAL);\n"
                                    ));
            for action in actions {
                let item = next_dev_view(counter);
                let action_props = VariantProps {
                    color: Some(action.color),
                    variant: props.style.variant,
                    ..VariantProps::default()
                };
                output.push_str(&format!(
                                            "        Button {item} = new Button(this);\n        {item}.setText(\"{}\");\n        {item}.setAllCaps(false);\n        {item}.setTextColor({});\n        {item}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
                                            escape_java(&action.label),
                                            dev_variant_content(&action_props),
                                            dev_variant_container(&action_props)
                                        ));
                if let Some(click) = action
                    .on_click
                    .as_deref()
                    .and_then(|name| context.action_id(name))
                    .map(|id| {
                        let item = context.active_item().unwrap_or("null");
                        format!("doweRunAction(\"{}\", {item})", escape_java(id))
                    })
                    .or_else(|| dev_android_navigation_action(action.navigation.as_ref()))
                {
                    output.push_str(&format!(
                        "        {item}.setOnClickListener(v -> {click});\n"
                    ));
                }
                output.push_str(&format!("        doweAdd({view}, {item}, 0, false);\n"));
            }
            let trigger = next_dev_view(counter);
            output.push_str(&format!(
                                        "        Button {trigger} = new Button(this);\n        {trigger}.setText(\"{}\");\n        {trigger}.setAllCaps(false);\n        {trigger}.setTextColor({});\n        {trigger}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
                                        escape_java(dev_view_icon_label(props.icon)),
                                        dev_variant_content(&props.style),
                                        dev_variant_container(&props.style)
                                    ));
            if let Some(click) = props
                .style
                .element
                .on_click
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| {
                    let item = context.active_item().unwrap_or("null");
                    format!("doweRunAction(\"{}\", {item})", escape_java(id))
                })
                .or_else(|| dev_android_navigation_action(props.style.navigation.as_ref()))
            {
                output.push_str(&format!(
                    "        {trigger}.setOnClickListener(v -> {click});\n"
                ));
            }
            output.push_str(&format!("        doweAdd({view}, {trigger}, 8, false);\n"));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Slider { props } => {
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
                                        "        SeekBar {bar} = new SeekBar(this);\n        {bar}.setMax({});\n        {bar}.setProgress({});\n",
                                        (max - min).max(1),
                                        (value - min).max(0)
                                    ));
            if let Some(path) = props.style.element.bind.as_deref() {
                let path = escape_java(&context.signal_path(path));
                output.push_str(&format!(
                                            "        try {{ {bar}.setProgress((int)Math.round(Double.parseDouble(doweTextValue(\"{path}\", null))) - {min}); }} catch (NumberFormatException ignored) {{}}\n        {bar}.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {{ public void onProgressChanged(SeekBar seekBar, int progress, boolean fromUser) {{ int value = progress + {min}; doweWrite(\"{path}\", value); {value_view}.setText(String.valueOf(value)); }} public void onStartTrackingTouch(SeekBar seekBar) {{}} public void onStopTrackingTouch(SeekBar seekBar) {{}} }});\n"
                                        ));
            }
            output.push_str(&format!("        doweAdd({view}, {bar}, 4, false);\n"));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Dropzone { props } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            if let Some(label) = props.style.label.as_deref() {
                let label_view = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                                            escape_java(label),
                                            dev_variant_content(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        TextView {field} = doweText(\"Upload\\n{}\", {}, 14f, 500, 0f, 1.4f, {});\n        {field}.setGravity(Gravity.CENTER);\n        {field}.setMinHeight(doweDp({}));\n        {field}.setPadding(doweDp(24), doweDp(24), doweDp(24), doweDp(24));\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_BOX));\n        doweAdd({view}, {field}, 4, false);\n",
                                        escape_java(
                                            props
                                                .style
                                                .placeholder
                                                .as_deref()
                                                .unwrap_or("Drag & drop files here or click to select")
                                        ),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        dev_dropzone_height(props.size),
                                        dev_variant_container(&props.style),
                                        if props.error_text.is_some() {
                                            java_color(ColorToken::Danger).to_string()
                                        } else {
                                            dev_variant_content(&props.style).to_string()
                                        }
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
        ViewNode::Checkbox { props } => {
            let view = next_dev_view(counter);
            let checked = dev_bound_bool(&props.style, props.checked, context);
            output.push_str(&format!(
                                        "        android.widget.CheckBox {view} = new android.widget.CheckBox(this);\n        {view}.setText(\"{}\");\n        {view}.setTextColor({});\n        {view}.setChecked({checked});\n        {view}.setEnabled({});\n",
                                        escape_java(props.style.label.as_deref().unwrap_or_default()),
                                        dev_scheme_color(&props.style),
                                        !props.disabled
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Color { props } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let swatch = next_dev_view(counter);
            let value = dev_bound_text(&props.style, &props.value, context);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            if let Some(label) = props.style.label.as_deref() {
                let label_view = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                                            escape_java(label),
                                            dev_variant_content(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        LinearLayout {field} = doweContainer(true);\n        {field}.setGravity(Gravity.CENTER_VERTICAL);\n        {field}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_UI));\n        View {swatch} = new View(this);\n        {swatch}.setLayoutParams(new LinearLayout.LayoutParams(doweDp(24), doweDp(24)));\n        try {{ {swatch}.setBackgroundColor(Color.parseColor({value})); }} catch (IllegalArgumentException ignored) {{ {swatch}.setBackgroundColor({}); }}\n        doweAdd({field}, {swatch});\n        TextView {field}Value = doweText({value}.toUpperCase(), {}, 14f, 500, 0f, 1.2f, {});\n        {field}Value.setPadding(doweDp(10), 0, 0, 0);\n        doweAdd({field}, {field}Value);\n        doweAdd({view}, {field}, 4, false);\n",
                                        dev_variant_container(&props.style),
                                        java_color(ColorToken::Muted),
                                        dev_variant_container(&props.style),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Date { props } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let value = dev_bound_text(
                &props.style,
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            if let Some(label) = props.style.label.as_deref() {
                let label_view = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                                            escape_java(label),
                                            dev_variant_content(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        TextView {field} = doweText({value}, {}, 14f, 500, 0f, 1.2f, {});\n        {field}.setHint(\"{}\");\n        {field}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_UI));\n        doweAdd({view}, {field}, 4, false);\n",
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        escape_java(props.style.placeholder.as_deref().unwrap_or("Select date")),
                                        dev_variant_container(&props.style),
                                        java_color(ColorToken::Muted)
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::DateRange { props } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
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
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            if let Some(label) = props.style.label.as_deref() {
                let label_view = next_dev_view(counter);
                output.push_str(&format!(
                                            "        TextView {label_view} = doweControlLabel(\"{}\", {}, {});\n        doweAdd({view}, {label_view});\n",
                                            escape_java(label),
                                            dev_variant_content(&props.style),
                                            dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                        ));
            }
            output.push_str(&format!(
                                        "        TextView {field} = doweText({start} + \" - \" + {end}, {}, 14f, 500, 0f, 1.2f, {});\n        {field}.setHint(\"{}\");\n        {field}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {field}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_UI));\n        doweAdd({view}, {field}, 4, false);\n",
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                                        escape_java(props.style.placeholder.as_deref().unwrap_or("Select date range")),
                                        dev_variant_container(&props.style),
                                        java_color(ColorToken::Muted)
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::RadioGroup { props, options } => {
            let view = next_dev_view(counter);
            let value = dev_bound_text(&props.style, "", context);
            output.push_str(&format!(
                                        "        android.widget.RadioGroup {view} = new android.widget.RadioGroup(this);\n        {view}.setOrientation(android.widget.RadioGroup.VERTICAL);\n"
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
                                            "        android.widget.RadioButton {item} = new android.widget.RadioButton(this);\n        {item}.setText(\"{}\");\n        {item}.setTextColor({});\n        {item}.setChecked({value}.equals(\"{}\"));\n        {item}.setEnabled({});\n        doweAdd({view}, {item});\n",
                                            escape_java(&option.label),
                                            dev_scheme_color(&props.style),
                                            escape_java(&option.value),
                                            !option.disabled
                                        ));
            }
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Toggle { props } => {
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
        _ => {}
    }
}
