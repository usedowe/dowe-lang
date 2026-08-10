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
                                        "        Button {view} = new Button(this);\n        final boolean[] {view}Dark = new boolean[]{{\"dark\".equals(getSharedPreferences(\"dowe\", 0).getString(\"theme-preference\", \"light\"))}};\n        {view}.setText({view}Dark[0] ? \"sun\" : \"moon\");\n        {view}.setAllCaps(false);\n        {view}.setTextColor({});\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n        {view}.setOnClickListener(v -> {{ {view}Dark[0] = !{view}Dark[0]; doweSetTheme({view}Dark[0] ? \"dark\" : \"light\"); }});\n",
                                        dev_variant_content(&props.style),
                                        dev_variant_container(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::SelectTheme { props } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let frame = next_dev_view(counter);
            let labels = props
                .themes
                .iter()
                .map(|theme| theme_display_label(theme))
                .collect::<Vec<_>>();
            let labels = java_string_array(labels.iter().map(String::as_str));
            let values = java_string_array(props.themes.iter().map(String::as_str));
            let descriptions = java_string_array(props.themes.iter().map(|_| ""));
            let content = dev_card_variant_content(&props.style);
            let background = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
                == ComponentVariant::Outlined
            {
                format!(
                    "doweInputBackground({}, {}, DOWE_RADIUS)",
                    dev_card_variant_container(&props.style),
                    content
                )
            } else {
                format!(
                    "doweBackground({}, DOWE_RADIUS)",
                    dev_card_variant_container(&props.style)
                )
            };
            let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n        String[] {field}Labels = {labels};\n        String[] {field}Values = {values};\n        String[] {field}Descriptions = {descriptions};\n        TextView {field} = doweSelectTrigger(\"{}\", {content}, {font});\n        {field}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp({}), 0, doweDp(36), 0);\n        {field}.setBackgroundColor(Color.TRANSPARENT);\n        final String[] {field}Selected = new String[]{{getSharedPreferences(\"dowe\", 0).getString(\"theme-preference\", \"{}\")}};\n        FrameLayout {frame} = doweSelectFrame({field}, {content}, {background});\n        doweAdd({view}, {frame}, 4, false);\n        doweBindSelect({field}, null, {field}Labels, {field}Values, {field}Descriptions, {field}Selected, \"{}\", {content}, {font}, null, false, value -> doweSetTheme(value));\n",
                escape_java(&props.label),
                escape_java(&props.placeholder),
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                escape_java(&props.default_theme),
                escape_java(&props.placeholder)
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            if parent_horizontal && props.style.style.sizing.w.is_none() {
                output.push_str(&format!(
                    "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
                ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Fab { props, actions } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        {view}.setGravity({});\n",
                dev_fab_content_gravity(props.position)
            ));
            let mut action_views = Vec::new();
            for action in actions {
                let item = next_dev_view(counter);
                let label = next_dev_view(counter);
                let action_props = VariantProps {
                    color: Some(action.color),
                    variant: props.style.variant,
                    ..VariantProps::default()
                };
                output.push_str(&format!(
                    "        LinearLayout {item} = doweContainer(true);\n        {item}.setGravity(Gravity.CENTER_VERTICAL);\n        {item}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {item}.setBackground(doweBackground({}, 999f));\n        {item}.setClickable(true);\n        {item}.setFocusable(true);\n        TextView {label} = doweText(\"{}\", {}, 14f, 600, 0f, 1.2f, null);\n        doweAdd({item}, {label});\n",
                    dev_variant_container(&action_props),
                    escape_java(&action.label),
                    dev_variant_content(&action_props)
                ));
                let icon = view_icon(action.icon);
                let icon_view = render_dev_android_icon_view(
                    &icon,
                    counter,
                    output,
                    Some(&dev_variant_content(&action_props)),
                );
                output.push_str(&format!(
                    "        doweAdd({item}, {icon_view}, 8, true);\n        doweWrapContentWidth({item});\n        {item}.setVisibility(View.INVISIBLE);\n"
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
                action_views.push(item);
            }
            let trigger = next_dev_view(counter);
            let trigger_size = dev_fab_size(props.style.size.unwrap_or(ButtonSize::Lg));
            output.push_str(&format!(
                "        FrameLayout {trigger} = new FrameLayout(this);\n        {trigger}.setContentDescription(\"{}\");\n        {trigger}.setClickable(true);\n        {trigger}.setFocusable(true);\n        {trigger}.setBackground(doweBackground({}, 999f));\n",
                escape_java(&props.label),
                dev_variant_container(&props.style)
            ));
            let trigger_icon = view_icon(props.icon);
            let trigger_icon_view = render_dev_android_icon_view(
                &trigger_icon,
                counter,
                output,
                Some(&dev_variant_content(&props.style)),
            );
            output.push_str(&format!(
                "        {trigger}.addView({trigger_icon_view}, new FrameLayout.LayoutParams(doweDp({}), doweDp({}), Gravity.CENTER));\n        {trigger}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({trigger_size}), doweDp({trigger_size})));\n",
                trigger_size / 2,
                trigger_size / 2
            ));
            if action_views.is_empty() {
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
            } else {
                let visibility_updates = action_views
                    .iter()
                    .map(|item| {
                        format!("{item}.setVisibility(open ? View.VISIBLE : View.INVISIBLE);")
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                output.push_str(&format!(
                    "        {trigger}.setOnClickListener(v -> {{ boolean open = {}.getVisibility() != View.VISIBLE; {visibility_updates} {trigger_icon_view}.setRotation(open ? 45f : 0f); }});\n",
                    action_views[0],
                ));
            }
            let action_additions = action_views
                .iter()
                .map(|item| format!("        doweAdd({view}, {item}, 12, false);\n"))
                .collect::<String>();
            let trigger_addition = format!("        doweAdd({view}, {trigger}, 12, false);\n");
            if matches!(
                props.position,
                OverlayCornerPosition::TopLeft | OverlayCornerPosition::TopRight
            ) {
                output.push_str(&trigger_addition);
                output.push_str(&action_additions);
            } else {
                output.push_str(&action_additions);
                output.push_str(&trigger_addition);
            }
            if props.fixed {
                let overlay = next_dev_view(counter);
                output.push_str(&format!(
                    "        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setTag(\"dowe-fixed-fab\");\n        {overlay}.setClipChildren(false);\n        {overlay}.setClipToPadding(false);\n        FrameLayout.LayoutParams {view}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT, {});\n        {view}Params.setMargins(doweDp({}), doweDp({}), doweDp({}), doweDp({}));\n        {overlay}.addView({view}, {view}Params);\n        ((ViewGroup) scrollView.getParent()).addView({overlay}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        doweApplySystemInsets({overlay});\n",
                    dev_fab_layout_gravity(props.position),
                    if matches!(props.position, OverlayCornerPosition::TopLeft | OverlayCornerPosition::BottomLeft) { props.offset_x.native_units() } else { 0 },
                    if matches!(props.position, OverlayCornerPosition::TopLeft | OverlayCornerPosition::TopRight) { props.offset_y.native_units() } else { 0 },
                    if matches!(props.position, OverlayCornerPosition::TopRight | OverlayCornerPosition::BottomRight) { props.offset_x.native_units() } else { 0 },
                    if matches!(props.position, OverlayCornerPosition::BottomLeft | OverlayCornerPosition::BottomRight) { props.offset_y.native_units() } else { 0 }
                ));
            } else {
                apply_dev_android_style(&props.style.style, &view, false, output);
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            }
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
        ViewNode::Dropzone { props } => {
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
                                            dev_variant_content(&props.style),
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
        ViewNode::Checkbox { props } => {
            let view = next_dev_view(counter);
            let checked = dev_bound_bool(&props.style, props.checked, context);
            output.push_str(&format!(
                                        "        android.widget.CheckBox {view} = new android.widget.CheckBox(this);\n        {view}.setText(\"{}\");\n        {view}.setTextColor({});\n        {view}.setButtonTintList(ColorStateList.valueOf({}));\n        {view}.setChecked({checked});\n        {view}.setEnabled({});\n",
                                        escape_java(props.style.label.as_deref().unwrap_or_default()),
                                        dev_scheme_color(&props.style),
                                        dev_scheme_color(&props.style),
                                        !props.disabled
                                    ));
            if let Some(path) = props.style.element.bind.as_ref() {
                output.push_str(&format!(
                    "        {view}.setOnCheckedChangeListener((button, value) -> {{ doweWrite(\"{}\", value); renderCurrentRoute(false); }});\n",
                    escape_java(&context.signal_path(path))
                ));
            }
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Color { props } => {
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
                                            dev_variant_content(&props.style),
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
        ViewNode::Date { props } => {
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
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
                                            dev_variant_content(&props.style),
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
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::DateRange { props } => {
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
                                            dev_variant_content(&props.style),
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
        ViewNode::RadioGroup { props, options } => {
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
