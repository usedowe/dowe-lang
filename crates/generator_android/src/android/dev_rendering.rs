fn render_dev_android_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    if let Some(show) = node_element_props(node).and_then(|props| props.show.as_ref()) {
        output.push_str(&format!(
            "        if ({}) {{\n",
            dev_show_condition(show, context)
        ));
        render_dev_android_node_body(
            node,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            inherited_color,
            context,
            children_method,
        );
        output.push_str("        }\n");
    } else {
        render_dev_android_node_body(
            node,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            inherited_color,
            context,
            children_method,
        );
    }
}

fn render_dev_android_node_body(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    match node {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
            for child in children {
                render_dev_android_node(
                    child,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    inherited_font,
                    inherited_color.clone(),
                    &context,
                    children_method,
                );
            }
        }
        ViewNode::Each {
            item,
            collection,
            children,
            ..
        } => {
            let row = format!("row{}", *counter);
            *counter += 1;
            output.push_str(&format!(
                "        for (Map<String, Object> {row} : doweRows(\"{}\")) {{\n",
                escape_java(&context.signal_path(collection))
            ));
            let context = context.with_item(item, row);
            for child in children {
                render_dev_android_node(
                    child,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    inherited_font,
                    inherited_color.clone(),
                    &context,
                    children_method,
                );
            }
            output.push_str("        }\n");
        }
        ViewNode::Box { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(props, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(props, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(props, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(props, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        DoweFlexLayout {view} = doweFlex({}, {}, {});\n",
                dev_flex_justify(props.justify.as_ref()),
                dev_flex_align(props.align.as_ref()),
                dev_optional_gap(props.gap.as_ref(), true).unwrap_or_else(|| "null".to_string())
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    true,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Grid { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
            let view = next_dev_view(counter);
            let columns = dev_grid_columns(props.columns.as_ref());
            let row_gap =
                dev_optional_gap(props.gap.as_ref(), false).unwrap_or_else(|| "null".to_string());
            let column_gap =
                dev_optional_gap(props.gap.as_ref(), true).unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        DoweGridLayout {view} = doweGrid({columns}, {row_gap}, {column_gap});\n"
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Card { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_card_variant_content(props).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweCard({}, {});\n",
                dev_card_variant_container(props),
                dev_card_border(props)
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Button { props, children } => {
            let view = next_dev_view(counter);
            let text = collect_joined_text(children);
            output.push_str(&format!(
                "        Button {view} = new Button(this);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setText(\"{}\");\n        {view}.setAllCaps(false);\n        {view}.setTypeface(Typeface.create({}, android.graphics.Typeface.NORMAL));\n        {view}.setTextSize({});\n        {view}.setIncludeFontPadding(false);\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setMinWidth(0);\n        {view}.setMinimumWidth(0);\n        {view}.setMinHeight(0);\n        {view}.setMinimumHeight(0);\n        {view}.setTextColor({});\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
                escape_java(&text),
                dev_font_value(props.style.font.as_ref().or(inherited_font)),
                dev_text_size_expr(false, INPUT_TEXT_SIZE),
                dev_variant_content(props),
                dev_variant_container(props)
            ));
            let action = props
                .element
                .on_click
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| {
                    let item = context.active_item().unwrap_or("null");
                    format!("doweRunAction(\"{}\", {item})", escape_java(id))
                })
                .or_else(|| dev_android_navigation_action(props.navigation.as_ref()));
            if let Some(action) = action {
                output.push_str(&format!(
                    "        {view}.setOnClickListener(v -> {action});\n"
                ));
            }
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
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
        ViewNode::Audio { props } => {
            let view = next_dev_view(counter);
            let label = props.subtitle.as_deref().unwrap_or(&props.src);
            output.push_str(&format!(
                "        TextView {view} = doweText(\"▶ {}\", {}, 14f, 500, 0f, 1.2f, {});\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {view}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_BOX));\n",
                escape_java(label),
                dev_card_variant_content(&props.style),
                dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                dev_card_variant_container(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Image { props } => {
            let view = next_dev_view(counter);
            let label = if props.alt.is_empty() {
                &props.src
            } else {
                &props.alt
            };
            output.push_str(&format!(
                "        TextView {view} = doweText(\"Image: {}\", {}, 14f, 500, 0f, 1.2f, {});\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setMinHeight(doweDp(160));\n        {view}.setBackground(doweInputBackground({}, {}, DOWE_RADIUS_BOX));\n",
                escape_java(label),
                dev_card_variant_content(&props.style),
                dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
                dev_card_variant_container(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Accordion { props, items } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                dev_variant_container(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for item in items {
                render_dev_android_variant_label(
                    &item.label,
                    &props.style,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    context,
                );
                if item.default_open {
                    for child in &item.children {
                        render_dev_android_node(
                            child,
                            &view,
                            None,
                            false,
                            counter,
                            output,
                            current_font,
                            current_color.clone(),
                            context,
                            children_method,
                        );
                    }
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground(Color.TRANSPARENT, DOWE_RADIUS_BOX));\n"
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            if let Some(title) = props.title.as_deref() {
                render_dev_android_variant_label(
                    title,
                    &props.style,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
            if let Some(slide) = slides.first() {
                for child in &slide.children {
                    render_dev_android_node(
                        child,
                        &view,
                        None,
                        false,
                        counter,
                        output,
                        current_font,
                        current_color.clone(),
                        context,
                        children_method,
                    );
                }
            }
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
            output.push_str(&format!("        LinearLayout {view} = doweContainer(false);\n"));
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
            let value = dev_bound_text(&props.style, props.value.as_deref().unwrap_or_default(), context);
            output.push_str(&format!("        LinearLayout {view} = doweContainer(false);\n"));
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
            output.push_str(&format!("        LinearLayout {view} = doweContainer(false);\n"));
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
        ViewNode::Code { props } => {
            let view = next_dev_view(counter);
            let texts = java_string_array(props.tokens.iter().map(|token| token.text.as_str()));
            let colors = java_int_array(props.tokens.iter().map(|token| {
                dev_code_token_color(token.kind, dev_card_variant_content(&props.style))
            }));
            output.push_str(&format!(
                "        LinearLayout {view} = doweCode(\"{}\", \"{}\", {texts}, {colors}, \"{}\", \"{}\", {}, {}, {});\n",
                escape_java(&props.source),
                props.language.as_str(),
                escape_java(&props.copy_label),
                escape_java(&props.copied_label),
                dev_card_variant_container(&props.style),
                dev_card_variant_content(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Video { props } => {
            let view = next_dev_view(counter);
            let poster = props
                .poster
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        FrameLayout {view} = doweVideo(\"{}\", {poster}, {}, \"{}\", {}, {});\n",
                escape_java(&props.src),
                props.autoplay,
                props.aspect.as_str(),
                dev_card_variant_container(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Candlestick { props } => {
            let view = next_dev_view(counter);
            let stream = props
                .stream
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        DoweCandlestickView {view} = doweCandlestick(\"{}\", {stream}, {}, {}, \"{}\", {}, {}, {}, {});\n",
                escape_java(&context.signal_path(&props.data)),
                java_color(props.up_color),
                java_color(props.down_color),
                escape_java(&props.empty_label),
                props.max_points,
                dev_card_variant_container(&props.style),
                dev_card_variant_content(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Table { props } => {
            let view = next_dev_view(counter);
            render_dev_android_table(props, &view, &context.signal_path(&props.data), output);
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Divider { props } => {
            let view = next_dev_view(counter);
            let (default_width, default_height) = match props.orientation {
                DividerOrientation::Horizontal => {
                    ("ViewGroup.LayoutParams.MATCH_PARENT", "doweDp(1)")
                }
                DividerOrientation::Vertical => {
                    ("doweDp(1)", "ViewGroup.LayoutParams.MATCH_PARENT")
                }
            };
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setBackgroundColor({});\n        {view}.setLayoutParams(new LinearLayout.LayoutParams({default_width}, {default_height}));\n",
                java_color(family_color(props.color))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            if props.style.sizing.w.is_none() || props.style.sizing.h.is_none() {
                output.push_str(&format!(
                    "        LinearLayout.LayoutParams {view}Params = (LinearLayout.LayoutParams) {view}.getLayoutParams();\n"
                ));
                if props.style.sizing.w.is_none() {
                    output.push_str(&format!("        {view}Params.width = {default_width};\n"));
                }
                if props.style.sizing.h.is_none() {
                    output.push_str(&format!(
                        "        {view}Params.height = {default_height};\n"
                    ));
                }
                output.push_str(&format!("        {view}.setLayoutParams({view}Params);\n"));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Title { props, value } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {view} = doweText({}, {}, {}, {}, {}, {}, {});\n",
                dev_text_expression(value, props.i18n.as_deref(), context),
                dev_text_color(props, inherited_color.as_deref()),
                dev_text_size(true, props),
                dev_text_weight(true, props),
                dev_text_spacing(true, props),
                dev_text_line_height(true, props),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Text { props, value } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {view} = doweText({}, {}, {}, {}, {}, {}, {});\n",
                dev_text_expression(value, props.i18n.as_deref(), context),
                dev_text_color(props, inherited_color.as_deref()),
                dev_text_size(false, props),
                dev_text_weight(false, props),
                dev_text_spacing(false, props),
                dev_text_line_height(false, props),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Alert { props } => {
            let view = next_dev_view(counter);
            if let Some(visible) = props.visible.as_deref() {
                output.push_str(&format!(
                    "        if (doweBool(\"{}\")) {{\n",
                    escape_java(&context.signal_path(visible))
                ));
            }
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                dev_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp(14), doweDp(10), doweDp(14), doweDp(10));\n        {view}.setBackground(doweInputBackground({}, {border}, DOWE_RADIUS_UI));\n        TextView {view}Text = doweText({}, {}, 14f, 400, 0f, 1.2f, {});\n        {view}Text.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({view}, {view}Text);\n",
                dev_variant_container(&props.style),
                dev_text_expression(&props.message, None, context),
                dev_variant_content(&props.style),
                dev_font_value(props.style.style.font.as_ref().or(inherited_font))
            ));
            if let Some(action) = props
                .on_close
                .as_deref()
                .and_then(|name| context.action_id(name))
            {
                let close = next_dev_view(counter);
                output.push_str(&format!(
                    "        Button {close} = new Button(this);\n        {close}.setText(\"x\");\n        {close}.setOnClickListener(v -> doweRunAction(\"{}\", null));\n",
                    escape_java(action)
                ));
                output.push_str(&format!("        doweAdd({view}, {close});\n"));
            }
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            if props.visible.is_some() {
                output.push_str("        }\n");
            }
        }
        ViewNode::Svg { props, paths } => {
            let view = next_dev_view(counter);
            let paths_name = format!("{view}Paths");
            output.push_str(&format!(
                "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
            ));
            for path in paths {
                output.push_str(&format!(
                    "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}));\n",
                    escape_java(&path.data),
                    dev_svg_path_current_color(path.fill),
                    dev_svg_path_color(path.fill)
                ));
            }
            output.push_str(&format!(
                "        DoweSvgView {view} = new DoweSvgView(this, {}f, {}f, {}f, {}f, {}, {paths_name});\n",
                props.view_box.min_x,
                props.view_box.min_y,
                props.view_box.width,
                props.view_box.height,
                dev_svg_color(&props.style, inherited_color.as_deref())
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }
        | ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        | ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setMinimumHeight(doweDp(48));\n        {view}.setBackground(doweBackground({}, {}));\n",
                dev_variant_container(&props.style),
                if props.floating {
                    "DOWE_RADIUS_BOX"
                } else {
                    "0"
                }
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            render_dev_android_bar_region(
                start,
                &view,
                "Gravity.START",
                false,
                counter,
                output,
                current_font,
                current_color.clone(),
                context,
                children_method,
            );
            if center.is_empty() && !end.is_empty() {
                render_dev_android_bar_spacer(&view, counter, output);
            }
            render_dev_android_bar_region(
                center,
                &view,
                "Gravity.CENTER",
                true,
                counter,
                output,
                current_font,
                current_color.clone(),
                context,
                children_method,
            );
            render_dev_android_bar_region(
                end,
                &view,
                "Gravity.END",
                false,
                counter,
                output,
                current_font,
                current_color,
                context,
                children_method,
            );
        }
        ViewNode::SideNav { props, items } => {
            render_dev_android_side_nav(
                props,
                items,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Sidebar { props, items } => {
            render_dev_android_side_nav(
                props,
                items,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::NavMenu { props, items } => {
            render_dev_android_nav_menu(
                props,
                items,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            render_dev_android_scaffold(
                props,
                app_bar,
                start,
                main,
                end,
                bottom_bar,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                inherited_color,
                context,
                children_method,
            );
        }
        ViewNode::Tabs { props, tabs } => {
            render_dev_android_tabs(
                props,
                tabs,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::Drawer { props, children } => {
            render_dev_android_drawer(
                props,
                children,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::Avatar { props, .. } => {
            let label = props
                .name
                .as_deref()
                .or(Some(props.alt.as_str()))
                .and_then(|value| value.chars().next())
                .map(|value| value.to_uppercase().to_string())
                .unwrap_or_else(|| "A".to_string());
            render_dev_android_variant_label(
                &label,
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Badge { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
            render_dev_android_variant_label(
                &props.text,
                &props.style,
                &view,
                None,
                false,
                counter,
                output,
                current_font,
                context,
            );
        }
        ViewNode::Chip { props, value, .. } => {
            render_dev_android_variant_label(
                value,
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Skeleton { props } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setMinimumHeight(doweDp(16));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
                java_color(ColorToken::Muted)
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            let path = escape_java(&context.signal_path(&props.open));
            output.push_str(&format!("        if (doweBool(\"{path}\")) {{\n"));
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                dev_variant_container(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in header.iter().chain(body).chain(footer) {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    props.style.style.font.as_ref().or(inherited_font),
                    Some(dev_variant_content(&props.style).to_string()),
                    context,
                    children_method,
                );
            }
            output.push_str("        }\n");
        }
        ViewNode::AlertDialog { props } => {
            let path = escape_java(&context.signal_path(&props.open));
            output.push_str(&format!("        if (doweBool(\"{path}\")) {{\n"));
            render_dev_android_variant_label(
                &format!("{} {}", props.title, props.description),
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
            output.push_str("        }\n");
        }
        ViewNode::Tooltip { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    inherited_color.clone(),
                    context,
                    children_method,
                );
            }
            render_dev_android_variant_label(
                &props.label,
                &props.style,
                &view,
                None,
                false,
                counter,
                output,
                current_font,
                context,
            );
        }
        ViewNode::Toast { props } => {
            let visible = props
                .source
                .as_deref()
                .map(|source| format!("doweBool(\"{}.visible\")", escape_java(&context.signal_path(source))))
                .unwrap_or_else(|| "true".to_string());
            output.push_str(&format!("        if ({visible}) {{\n"));
            render_dev_android_variant_label(
                props.title.as_deref().unwrap_or(&props.description),
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
            output.push_str("        }\n");
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                dev_variant_container(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in trigger.iter().chain(header).chain(footer) {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    Some(dev_variant_content(&props.style).to_string()),
                    context,
                    children_method,
                );
            }
            for entry in entries {
                render_dev_android_overlay_entry(
                    entry,
                    &props.style,
                    &view,
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
        }
        ViewNode::Command { props, entries } => {
            if let Some(open) = props.open.as_deref() {
                output.push_str(&format!(
                    "        if (doweBool(\"{}\")) {{\n",
                    escape_java(&context.signal_path(open))
                ));
            }
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                dev_variant_container(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            render_dev_android_variant_label(
                &props.placeholder,
                &props.style,
                &view,
                None,
                false,
                counter,
                output,
                props.style.style.font.as_ref().or(inherited_font),
                context,
            );
            for entry in entries {
                render_dev_android_command_entry(
                    entry,
                    &props.style,
                    &view,
                    counter,
                    output,
                    props.style.style.font.as_ref().or(inherited_font),
                    context,
                );
            }
            if props.open.is_some() {
                output.push_str("        }\n");
            }
        }
        ViewNode::Children => {
            if let Some(method) = children_method {
                output.push_str(&format!("        {method}({parent});\n"));
            }
        }
    }
}

fn dev_show_condition(show: &VisibilityCondition, context: &ComposeReactiveContext) -> String {
    match show {
        VisibilityCondition::Static(value) => {
            format!("doweShow({})", dev_bool_value(value))
        }
        VisibilityCondition::Signal(path) => {
            let item = context.item_value(path).unwrap_or("null");
            let path = context
                .item_path(path)
                .unwrap_or_else(|| context.signal_path(path));
            format!("doweBool(\"{}\", {item})", escape_java(&path))
        }
    }
}

fn dev_scheme_color(props: &VariantProps) -> &'static str {
    java_color(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn dev_bound_text(
    props: &VariantProps,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> String {
    dev_optional_bound_text(props.element.bind.as_deref(), fallback, context)
}

fn dev_optional_bound_text(
    path: Option<&str>,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> String {
    path.map(|path| {
        format!(
            "doweTextValue(\"{}\", null)",
            escape_java(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| format!("\"{}\"", escape_java(fallback)))
}

fn dev_bound_bool(
    props: &VariantProps,
    fallback: bool,
    context: &ComposeReactiveContext,
) -> String {
    props
        .element
        .bind
        .as_deref()
        .map(|path| {
            format!(
                "doweBool(\"{}\")",
                escape_java(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn render_dev_android_variant_label(
    value: &str,
    props: &VariantProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        TextView {view} = doweText({}, {}, 14f, 500, 0f, 1.2f, {});\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
        dev_text_expression(value, None, context),
        dev_variant_content(props),
        dev_font_value(props.style.font.as_ref().or(inherited_font)),
        dev_variant_container(props)
    ));
    apply_dev_android_style(&props.style, &view, false, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

fn render_dev_android_overlay_entry(
    entry: &OverlayEntry,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    match entry {
        OverlayEntry::Item(item) => render_dev_android_overlay_item(
            item,
            props,
            parent,
            counter,
            output,
            inherited_font,
            context,
        ),
        OverlayEntry::Divider => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setBackgroundColor({});\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));\n",
                java_color(ColorToken::Muted)
            ));
            output.push_str(&dev_add(parent, &view, None, false));
        }
    }
}

fn render_dev_android_command_entry(
    entry: &CommandEntry,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    match entry {
        CommandEntry::Item(item) => render_dev_android_overlay_item(
            item,
            props,
            parent,
            counter,
            output,
            inherited_font,
            context,
        ),
        CommandEntry::Group { label, items, .. } => {
            render_dev_android_variant_label(
                label,
                props,
                parent,
                None,
                false,
                counter,
                output,
                inherited_font,
                context,
            );
            for item in items {
                render_dev_android_overlay_item(
                    item,
                    props,
                    parent,
                    counter,
                    output,
                    inherited_font,
                    context,
                );
            }
        }
    }
}

fn render_dev_android_overlay_item(
    item: &OverlayItemProps,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    render_dev_android_variant_label(
        &item.label,
        props,
        parent,
        None,
        false,
        counter,
        output,
        inherited_font,
        context,
    );
}

fn render_dev_android_nav_menu(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let view = next_dev_view(counter);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    let row = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {row} = doweContainer(true);\n        doweAdd({view}, {row});\n"
    ));
    for item in items {
        match item {
            NavMenuItem::Item(props) => {
                render_dev_android_nav_menu_button(
                    props,
                    &row,
                    props.navigation.as_ref(),
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
            NavMenuItem::Submenu { props, items } => {
                render_dev_android_nav_menu_button(
                    props,
                    &row,
                    None,
                    counter,
                    output,
                    current_font,
                    context,
                );
                let submenu = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {submenu} = doweContainer(false);\n        {submenu}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n        doweAdd({view}, {submenu});\n"
                ));
                for item in items {
                    render_dev_android_nav_menu_button(
                        item,
                        &submenu,
                        item.navigation.as_ref(),
                        counter,
                        output,
                        current_font,
                        context,
                    );
                }
            }
            NavMenuItem::Megamenu { props, content } => {
                render_dev_android_nav_menu_button(
                    props,
                    &row,
                    None,
                    counter,
                    output,
                    current_font,
                    context,
                );
                let panel = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {panel} = doweContainer(false);\n        {panel}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n        doweAdd({view}, {panel});\n"
                ));
                for child in content {
                    render_dev_android_node(
                        child,
                        &panel,
                        None,
                        false,
                        counter,
                        output,
                        current_font,
                        None,
                        context,
                        children_method,
                    );
                }
            }
        }
    }
}

fn render_dev_android_nav_menu_button(
    props: &NavMenuItemProps,
    parent: &str,
    navigation: Option<&NavigationAction>,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        doweAdd({parent}, {view});\n        TextView {view}Label = doweText(\"{}\", DOWE_ON_BACKGROUND, 14f, 400, 0f, 18f, {});\n        doweAdd({view}, {view}Label);\n",
        escape_java(&props.label),
        dev_font_value(inherited_font)
    ));
    if let Some(action) = props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null)", escape_java(id)))
        .or_else(|| dev_android_navigation_action(navigation))
    {
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> {action});\n"
        ));
    }
}

fn render_dev_android_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let view = next_dev_view(counter);
    let current_font = props.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    for child in app_bar {
        render_dev_android_node(
            child,
            &view,
            None,
            false,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    let body = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {body} = doweContainer(true);\n        {body}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));\n        doweAdd({view}, {body});\n"
    ));
    for child in start {
        render_dev_android_node(
            child,
            &body,
            None,
            true,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    let main_view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {main_view} = doweContainer(false);\n        {main_view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f));\n        doweAdd({body}, {main_view});\n"
    ));
    for child in main {
        render_dev_android_node(
            child,
            &main_view,
            None,
            false,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    for child in end {
        render_dev_android_node(
            child,
            &body,
            None,
            true,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
    for child in bottom_bar {
        render_dev_android_node(
            child,
            &view,
            None,
            false,
            counter,
            output,
            current_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
}

fn render_dev_android_drawer(
    props: &DrawerProps,
    children: &[ViewNode],
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let overlay = next_dev_view(counter);
    let panel = next_dev_view(counter);
    let content = next_dev_view(counter);
    let popup = format!("{overlay}Popup");
    let path = escape_java(&context.signal_path(&props.open));
    let (width, height, gravity) = match props.position {
        DrawerPosition::Start => (
            "doweDp(320)",
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "Gravity.START",
        ),
        DrawerPosition::End => (
            "doweDp(320)",
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "Gravity.END",
        ),
        DrawerPosition::Top => (
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "doweDp(320)",
            "Gravity.TOP",
        ),
        DrawerPosition::Bottom => (
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "doweDp(320)",
            "Gravity.BOTTOM",
        ),
    };
    output.push_str(&format!(
        "        if (doweBool(\"{path}\")) {{\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        FrameLayout {panel} = new FrameLayout(this);\n        {panel}.setBackground(doweDrawerBackground({}, {}, \"{}\", {}));\n        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams({width}, {height}, {gravity});\n        {overlay}.addView({panel}, {panel}Params);\n        LinearLayout {content} = doweContainer(false);\n        {panel}.addView({content}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n",
        dev_card_variant_container(&props.style),
        dev_card_border(&props.style),
        props.position.as_str(),
        dev_drawer_radius(&props.style.style)
    ));
    apply_dev_android_style(&props.style.style, &content, false, output);
    output.push_str(&format!(
        "        PopupWindow {popup} = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup}.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup}.setOutsideTouchable(false);\n        {panel}.setOnClickListener(v -> {{ }});\n"
    ));
    if !props.disable_overlay_close {
        output.push_str(&format!(
            "        {overlay}.setOnClickListener(v -> {{ {popup}.dismiss(); doweWrite(\"{path}\", false); renderCurrentRoute(false); }});\n"
        ));
    }
    if !props.hide_close_button {
        let close = next_dev_view(counter);
        output.push_str(&format!(
            "        TextView {close} = new TextView(this);\n        {close}.setText(\"x\");\n        {close}.setTextColor(DOWE_ON_SOFT_MUTED);\n        {close}.setGravity(Gravity.CENTER);\n        {close}.setIncludeFontPadding(false);\n        {close}.setBackground(doweBackground(DOWE_SOFT_MUTED, 999f));\n        {close}.setOnClickListener(v -> {{ {popup}.dismiss(); doweWrite(\"{path}\", false); renderCurrentRoute(false); }});\n        FrameLayout.LayoutParams {close}Params = new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END);\n        {close}Params.setMargins(0, doweDp(8), doweDp(8), 0);\n        {panel}.addView({close}, {close}Params);\n"
        ));
    }
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    for child in children {
        render_dev_android_node(
            child,
            &content,
            None,
            false,
            counter,
            output,
            current_font,
            Some(dev_card_variant_content(&props.style).to_string()),
            context,
            children_method,
        );
    }
    output.push_str(&format!(
        "        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup}.showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n"
    ));
}

fn render_dev_android_side_nav(
    props: &SideNavProps,
    items: &[SideNavItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    for item in items {
        render_dev_android_side_nav_item(
            item,
            &view,
            props,
            counter,
            output,
            current_font,
            context,
        );
    }
}

fn render_dev_android_side_nav_item(
    item: &SideNavItem,
    parent: &str,
    nav: &SideNavProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    match item {
        SideNavItem::Header(props) => {
            render_dev_android_side_nav_row(
                props,
                true,
                parent,
                nav,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        SideNavItem::Item(props) => {
            render_dev_android_side_nav_row(
                props,
                false,
                parent,
                nav,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        SideNavItem::Divider => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setBackgroundColor(DOWE_MUTED);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));\n        doweAdd({parent}, {view}, 8, false);\n"
            ));
        }
        SideNavItem::Submenu { props, open, items } => {
            let trigger = render_dev_android_side_nav_row(
                props,
                true,
                parent,
                nav,
                counter,
                output,
                inherited_font,
                context,
            );
            let submenu = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {submenu} = doweContainer(false);\n        {submenu}.setPadding(doweDp(16), 0, 0, 0);\n        {submenu}.setVisibility({});\n        doweAdd({parent}, {submenu});\n        {trigger}.setOnClickListener(v -> doweToggleSideNavSubmenu({submenu}));\n",
                if *open { "View.VISIBLE" } else { "View.GONE" }
            ));
            for item in items {
                render_dev_android_side_nav_row(
                    item,
                    false,
                    &submenu,
                    nav,
                    counter,
                    output,
                    inherited_font,
                    context,
                );
            }
        }
    }
}

fn render_dev_android_side_nav_row(
    props: &SideNavItemProps,
    header: bool,
    parent: &str,
    nav: &SideNavProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) -> String {
    let view = next_dev_view(counter);
    let (padding_horizontal, padding_vertical, _, label_size, description_size) =
        compose_side_nav_metrics(nav.size);
    let content = dev_nav_active_content(&nav.style);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp({padding_horizontal}), doweDp({padding_vertical}), doweDp({padding_horizontal}), doweDp({padding_vertical}));\n        if ({}) {{ {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI)); }}\n",
        dev_side_nav_active(props.navigation.as_ref()),
        dev_variant_container(&nav.style)
    ));
    output.push_str(&format!("        doweAdd({parent}, {view});\n"));
    if let Some(icon) = props.icon.as_ref() {
        render_dev_android_side_nav_icon(icon, &view, counter, output, Some(content));
    }
    let copy = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {copy} = doweContainer(false);\n        {copy}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({view}, {copy});\n        TextView {copy}Label = doweText(\"{}\", {content}, {label_size}f, {}, 0f, {label_size}f, {});\n        doweAdd({copy}, {copy}Label);\n",
        escape_java(&props.label),
        if header { "600" } else { "400" },
        dev_font_value(inherited_font)
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "        TextView {copy}Description = doweText(\"{}\", {content}, {description_size}f, 400, 0f, {description_size}f, {});\n        {copy}Description.setAlpha(0.72f);\n        doweAdd({copy}, {copy}Description);\n",
            escape_java(description),
            dev_font_value(inherited_font)
        ));
    }
    if let Some(status) = props.status.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Status = doweText(\"{}\", {content}, {description_size}f, 600, 0f, {description_size}f, {});\n        doweAdd({view}, {view}Status);\n",
            escape_java(status),
            dev_font_value(inherited_font)
        ));
    }
    if let Some(action) = dev_side_nav_action(props, context) {
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> {action});\n"
        ));
    }
    view
}

fn render_dev_android_tabs(
    props: &TabsProps,
    tabs: &[TabItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let root = next_dev_view(counter);
    let list = next_dev_view(counter);
    let panels = next_dev_view(counter);
    let vertical = matches!(
        props.position,
        dowe_components::TabsPosition::Start | dowe_components::TabsPosition::End
    );
    let root_horizontal = if vertical { "true" } else { "false" };
    let list_horizontal = if vertical { "false" } else { "true" };
    let list_radius = match props.variant {
        TabsVariant::Pills => "999f",
        TabsVariant::Solid | TabsVariant::Outlined => "DOWE_RADIUS_BOX",
        TabsVariant::Line | TabsVariant::Ghost => "0f",
    };
    let tab_radius = match props.variant {
        TabsVariant::Pills => "999f",
        _ => "DOWE_RADIUS_UI",
    };
    let active_background = dev_tab_background(props, true, tab_radius);
    let inactive_background = dev_tab_background(props, false, tab_radius);
    let active_content = dev_tabs_active_content(props);
    let inactive_content = dev_tabs_list_content(props);
    let font = dev_font_value(props.style.font.as_ref().or(inherited_font));
    output.push_str(&format!(
        "        LinearLayout {root} = doweContainer({root_horizontal});\n"
    ));
    apply_dev_android_style(&props.style, &root, true, output);
    output.push_str(&dev_add(parent, &root, parent_gap, parent_horizontal));
    output.push_str(&format!(
        "        LinearLayout {list} = doweContainer({list_horizontal});\n        {list}.setGravity(Gravity.CENTER_VERTICAL);\n        {list}.setPadding(doweDp({}), doweDp({}), doweDp({}), doweDp({}));\n",
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        },
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        },
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        },
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        }
    ));
    if !matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
        let background = if dev_tabs_border(props) == "null" {
            format!(
                "doweBackground({}, {list_radius})",
                dev_tabs_list_background(props)
            )
        } else {
            format!(
                "doweInputBackground({}, {}, {list_radius})",
                dev_tabs_list_background(props),
                dev_tabs_border(props)
            )
        };
        output.push_str(&format!("        {list}.setBackground({background});\n"));
    }
    output.push_str(&format!(
        "        FrameLayout {panels} = new FrameLayout(this);\n        {panels}.setLayoutParams(new LinearLayout.LayoutParams({}, ViewGroup.LayoutParams.WRAP_CONTENT{}));\n",
        if vertical { "0" } else { "ViewGroup.LayoutParams.MATCH_PARENT" },
        if vertical { ", 1f" } else { "" }
    ));
    match props.position {
        dowe_components::TabsPosition::Bottom | dowe_components::TabsPosition::End => {
            output.push_str(&dev_add(&root, &panels, None, vertical));
            output.push_str(&dev_add(&root, &list, Some("8"), vertical));
        }
        dowe_components::TabsPosition::Top | dowe_components::TabsPosition::Start => {
            output.push_str(&dev_add(&root, &list, None, vertical));
            output.push_str(&dev_add(&root, &panels, Some("8"), vertical));
        }
    }
    let mut button_names = Vec::new();
    let mut panel_names = Vec::new();
    let current_font = props.style.font.as_ref().or(inherited_font);
    for (index, tab) in tabs.iter().enumerate() {
        let button = next_dev_view(counter);
        let panel = next_dev_view(counter);
        button_names.push(button.clone());
        panel_names.push(panel.clone());
        output.push_str(&format!(
            "        TextView {button} = doweText(\"{}\", {}, 16f, 500, 0f, 1.25f, {font});\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setPadding(doweDp(16), doweDp(6), doweDp(16), doweDp(6));\n        {button}.setTextColor({});\n        {button}.setBackground({});\n",
            escape_java(&tab.label),
            if index == 0 { active_content } else { inactive_content },
            if index == 0 { active_content } else { inactive_content },
            if index == 0 {
                active_background.as_str()
            } else {
                inactive_background.as_str()
            }
        ));
        output.push_str(&dev_add(&list, &button, Some("8"), !vertical));
        output.push_str(&format!(
            "        LinearLayout {panel} = doweContainer(false);\n        {panel}.setVisibility({});\n        {panels}.addView({panel}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n",
            if index == 0 {
                "View.VISIBLE"
            } else {
                "View.GONE"
            }
        ));
        for child in &tab.children {
            render_dev_android_node(
                child,
                &panel,
                None,
                false,
                counter,
                output,
                current_font,
                None,
                context,
                children_method,
            );
        }
    }
    output.push_str(&format!(
        "        TextView[] {root}Tabs = new TextView[]{{{}}};\n        View[] {root}Panels = new View[]{{{}}};\n",
        button_names.join(", "),
        panel_names.join(", ")
    ));
    for (index, button) in button_names.iter().enumerate() {
        output.push_str(&format!(
            "        {button}.setOnClickListener(v -> {{ for (int index = 0; index < {root}Tabs.length; index++) {{ boolean active = index == {index}; {root}Panels[index].setVisibility(active ? View.VISIBLE : View.GONE); {root}Tabs[index].setTextColor(active ? {active_content} : {inactive_content}); {root}Tabs[index].setBackground(active ? {active_background} : {inactive_background}); }} }});\n"
        ));
    }
}

fn dev_tab_background(props: &TabsProps, active: bool, radius: &str) -> String {
    if active {
        match props.variant {
            TabsVariant::Solid | TabsVariant::Outlined | TabsVariant::Pills => {
                format!("doweBackground({}, {radius})", dev_tabs_active_background(props))
            }
            TabsVariant::Line => {
                format!("doweInputBackground(Color.TRANSPARENT, {}, {radius})", dev_tabs_accent(props))
            }
            TabsVariant::Ghost => format!("doweBackground(Color.TRANSPARENT, {radius})"),
        }
    } else {
        format!("doweBackground(Color.TRANSPARENT, {radius})")
    }
}

fn render_dev_android_side_nav_icon(
    icon: &SideNavIcon,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_color: Option<&str>,
) {
    let view = next_dev_view(counter);
    let paths_name = format!("{view}Paths");
    output.push_str(&format!(
        "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
    ));
    for path in &icon.paths {
        output.push_str(&format!(
            "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}));\n",
            escape_java(&path.data),
            dev_svg_path_current_color(path.fill),
            dev_svg_path_color(path.fill)
        ));
    }
    output.push_str(&format!(
        "        DoweSvgView {view} = new DoweSvgView(this, {}f, {}f, {}f, {}f, {}, {paths_name});\n",
        icon.props.view_box.min_x,
        icon.props.view_box.min_y,
        icon.props.view_box.width,
        icon.props.view_box.height,
        dev_svg_color(&icon.props.style, inherited_color)
    ));
    apply_dev_android_style(&icon.props.style, &view, false, output);
    output.push_str(&format!("        doweAdd({parent}, {view}, 8, true);\n"));
}

fn dev_side_nav_action(
    props: &SideNavItemProps,
    context: &ComposeReactiveContext,
) -> Option<String> {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null)", escape_java(id)))
        .or_else(|| dev_android_navigation_action(props.navigation.as_ref()))
}

fn dev_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("\"{}\".equals(currentPath)", escape_java(path))
        }
        _ => "false".to_string(),
    }
}

fn dev_add(parent: &str, view: &str, gap: Option<&str>, horizontal: bool) -> String {
    match gap {
        Some(gap) => format!(
            "        doweAdd({parent}, {view}, {gap}, {});\n",
            if horizontal { "true" } else { "false" }
        ),
        None => format!("        doweAdd({parent}, {view});\n"),
    }
}

fn render_dev_android_table(props: &TableProps, view: &str, data_path: &str, output: &mut String) {
    let fields = java_string_array(props.columns.iter().map(|column| column.field.as_str()));
    let labels = java_string_array(props.columns.iter().map(|column| column.label.as_str()));
    let alignments = java_int_array(
        props
            .columns
            .iter()
            .map(|column| dev_table_align(column.align).to_string()),
    );
    let widths =
        java_nullable_string_array(props.columns.iter().map(|column| column.width.as_deref()));
    output.push_str(&format!(
        "        LinearLayout {view} = doweTable(\"{}\", {fields}, {labels}, {alignments}, {widths}, {}, {}, {}, {}, \"{}\", \"{}\", {}, {}, {});\n",
        escape_java(data_path),
        dev_table_size(props.size),
        props.striped,
        props.bordered,
        props.dividers,
        escape_java(&props.empty_title),
        escape_java(&props.empty_description),
        dev_card_variant_container(&props.style),
        dev_card_variant_content(&props.style),
        dev_card_border(&props.style)
    ));
}

fn dev_table_size(value: TableSize) -> &'static str {
    match value {
        TableSize::Sm => "0",
        TableSize::Md => "1",
        TableSize::Lg => "2",
    }
}

fn dev_table_align(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => "Gravity.START",
        TableColumnAlign::Center => "Gravity.CENTER",
        TableColumnAlign::End => "Gravity.END",
    }
}

fn render_dev_android_bar_region(
    children: &[ViewNode],
    parent: &str,
    gravity: &str,
    weighted: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    if children.is_empty() {
        return;
    }
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL | {gravity});\n        {view}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n"
    ));
    if weighted {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
        ));
    } else {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
        ));
    }
    output.push_str(&format!("        {parent}.addView({view});\n"));
    for child in children {
        render_dev_android_node(
            child,
            &view,
            Some("8"),
            true,
            counter,
            output,
            inherited_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
}

fn render_dev_android_bar_spacer(parent: &str, counter: &mut usize, output: &mut String) {
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        View {view} = new View(this);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, 0, 1f));\n        {parent}.addView({view});\n"
    ));
}

fn dev_optional_gap(value: Option<&ResponsiveValue<GapValue>>, horizontal: bool) -> Option<String> {
    value.map(|value| dev_responsive_value(value, |value| dev_gap_expr(value, horizontal)))
}

fn dev_flex_justify(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(dev_flex_justify_value)
        .unwrap_or_else(|| "null".to_string())
}

fn dev_flex_justify_value(value: &ResponsiveValue<Justify>) -> String {
    dev_responsive_value(value, |value| match value {
        Justify::Start => "DOWE_JUSTIFY_START".to_string(),
        Justify::Center => "DOWE_JUSTIFY_CENTER".to_string(),
        Justify::End => "DOWE_JUSTIFY_END".to_string(),
        Justify::Between => "DOWE_JUSTIFY_BETWEEN".to_string(),
        Justify::Around => "DOWE_JUSTIFY_AROUND".to_string(),
        Justify::Evenly => "DOWE_JUSTIFY_EVENLY".to_string(),
    })
}

fn dev_flex_align(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(dev_flex_align_value)
        .unwrap_or_else(|| "null".to_string())
}

fn dev_flex_align_value(value: &ResponsiveValue<Align>) -> String {
    dev_responsive_value(value, |value| match value {
        Align::Start => "DOWE_ALIGN_START".to_string(),
        Align::Center => "DOWE_ALIGN_CENTER".to_string(),
        Align::End => "DOWE_ALIGN_END".to_string(),
        Align::Stretch => "DOWE_ALIGN_STRETCH".to_string(),
        Align::Baseline => "DOWE_ALIGN_BASELINE".to_string(),
    })
}

fn dev_grid_columns(value: Option<&ResponsiveValue<GridTracks>>) -> String {
    value
        .map(|value| dev_responsive_value(value, |value| value.count().unwrap_or(1).to_string()))
        .unwrap_or_else(|| "1".to_string())
}

fn dev_inherited_color(props: &StyleProps, inherited_color: Option<&str>) -> Option<String> {
    props
        .text
        .as_ref()
        .map(dev_color_value)
        .or_else(|| inherited_color.map(str::to_string))
}

fn dev_svg_color(props: &StyleProps, inherited_color: Option<&str>) -> String {
    let fallback = inherited_color.unwrap_or("DOWE_ON_BACKGROUND");
    props
        .text
        .as_ref()
        .map(dev_color_value)
        .map(|value| format!("doweColor({value}, {fallback})"))
        .unwrap_or_else(|| fallback.to_string())
}

fn dev_svg_path_current_color(fill: SvgPathFill) -> &'static str {
    match fill {
        SvgPathFill::CurrentColor => "true",
        SvgPathFill::None | SvgPathFill::Color(_) => "false",
    }
}

fn dev_svg_path_color(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None | SvgPathFill::CurrentColor => "null".to_string(),
        SvgPathFill::Color(token) => java_color(token).to_string(),
    }
}

fn dev_gap_expr(value: &GapValue, horizontal: bool) -> String {
    match value {
        GapValue::Single(value) => dev_gap_size(value),
        GapValue::Pair(row, column) => {
            if horizontal {
                dev_gap_size(column)
            } else {
                dev_gap_size(row)
            }
        }
    }
}

fn dev_gap_size(value: &GapSize) -> String {
    match value {
        GapSize::Scale(value) => value.native_units().to_string(),
        GapSize::Px(value) => value.to_string(),
    }
}

fn dev_android_navigation_action(action: Option<&NavigationAction>) -> Option<String> {
    match action {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => Some(format!(
            "doweNavigate(\"{}\", \"{}\", {})",
            operation.as_str(),
            escape_java(path),
            fragment
                .as_ref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string())
        )),
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => Some(format!(
            "doweNavigate(\"{}\", currentPath, \"{}\")",
            operation.as_str(),
            escape_java(fragment)
        )),
        Some(NavigationAction::External {
            url,
            native_external_mode,
            ..
        }) => Some(format!(
            "doweOpenExternal(\"{}\", \"{}\")",
            native_external_mode.as_str(),
            escape_java(url)
        )),
        Some(NavigationAction::Back) => Some("doweBack()".to_string()),
        None => None,
    }
}

fn dev_text_expression(
    value: &str,
    i18n: Option<&str>,
    context: &ComposeReactiveContext,
) -> String {
    if let Some(key) = i18n {
        return format!("getString(R.string.{})", translation_resource_name(key));
    }
    match context.dynamic_path(value) {
        Some(path) => context
            .item_value(value)
            .map(|item| format!("doweTextValue(\"{}\", {item})", escape_java(&path)))
            .unwrap_or_else(|| format!("doweTextValue(\"{}\", null)", escape_java(&path))),
        None => format!("\"{}\"", escape_java(value)),
    }
}

fn dev_route_method_name(route: &str) -> String {
    format!("render{}", pascal_route(route))
}

fn dev_route_page_method_name(route: &str) -> String {
    format!("{}Page", dev_route_method_name(route))
}

fn next_dev_view(counter: &mut usize) -> String {
    let value = format!("view{}", *counter);
    *counter += 1;
    value
}

fn apply_dev_android_style(
    props: &StyleProps,
    view: &str,
    include_background: bool,
    output: &mut String,
) {
    if let Some(id) = props.element.id.as_ref() {
        output.push_str(&format!(
            "        doweRegisterSection(\"{}\", {view});\n",
            escape_java(id)
        ));
    }

    if include_background && let Some(value) = props.bg.as_ref() {
        output.push_str(&format!(
            "        Integer {view}Background = {};\n        if ({view}Background != null) {{\n            {view}.setBackgroundColor({view}Background);\n        }}\n",
            dev_color_value(value)
        ));
    }

    if include_background && let Some(value) = props.background.as_ref() {
        output.push_str(&format!(
            "        String {view}SectionBackground = {};\n        if ({view}SectionBackground != null) {{\n            {view}.setBackground(doweSectionBackground({view}SectionBackground));\n        }}\n",
            dev_section_background_value(value)
        ));
    }

    if props.spacing.p.is_some()
        || props.spacing.px.is_some()
        || props.spacing.py.is_some()
        || props.spacing.pl.is_some()
        || props.spacing.pr.is_some()
        || props.spacing.pt.is_some()
        || props.spacing.pb.is_some()
    {
        output.push_str(&format!(
            "        int {view}Left = 0;\n        int {view}Top = 0;\n        int {view}Right = 0;\n        int {view}Bottom = 0;\n        Integer {view}Padding = {};\n        if ({view}Padding != null) {{\n            int value = doweDp({view}Padding);\n            {view}Left = value;\n            {view}Top = value;\n            {view}Right = value;\n            {view}Bottom = value;\n        }}\n        Integer {view}PaddingX = {};\n        if ({view}PaddingX != null) {{\n            int value = doweDp({view}PaddingX);\n            {view}Left = value;\n            {view}Right = value;\n        }}\n        Integer {view}PaddingY = {};\n        if ({view}PaddingY != null) {{\n            int value = doweDp({view}PaddingY);\n            {view}Top = value;\n            {view}Bottom = value;\n        }}\n        Integer {view}PaddingLeft = {};\n        if ({view}PaddingLeft != null) {{\n            {view}Left = doweDp({view}PaddingLeft);\n        }}\n        Integer {view}PaddingRight = {};\n        if ({view}PaddingRight != null) {{\n            {view}Right = doweDp({view}PaddingRight);\n        }}\n        Integer {view}PaddingTop = {};\n        if ({view}PaddingTop != null) {{\n            {view}Top = doweDp({view}PaddingTop);\n        }}\n        Integer {view}PaddingBottom = {};\n        if ({view}PaddingBottom != null) {{\n            {view}Bottom = doweDp({view}PaddingBottom);\n        }}\n        {view}.setPadding({view}Left, {view}Top, {view}Right, {view}Bottom);\n",
            dev_optional_scale(props.spacing.p.as_ref()),
            dev_optional_scale(props.spacing.px.as_ref()),
            dev_optional_scale(props.spacing.py.as_ref()),
            dev_optional_scale(props.spacing.pl.as_ref()),
            dev_optional_scale(props.spacing.pr.as_ref()),
            dev_optional_scale(props.spacing.pt.as_ref()),
            dev_optional_scale(props.spacing.pb.as_ref())
        ));
    }

    if props.sizing.w.is_some() || props.sizing.h.is_some() {
        output.push_str(&format!(
            "        Integer {view}Width = {};\n        Integer {view}Height = {};\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(doweDimension({view}Width), doweDimension({view}Height)));\n",
            dev_optional_size(props.sizing.w.as_ref()),
            dev_optional_size(props.sizing.h.as_ref())
        ));
    }

    if let Some(value) = props.sizing.min_w.as_ref() {
        output.push_str(&format!(
            "        Integer {view}MinWidth = {};\n        if ({view}MinWidth != null && {view}MinWidth != ViewGroup.LayoutParams.MATCH_PARENT) {{\n            {view}.setMinimumWidth(doweDp({view}MinWidth));\n        }}\n",
            dev_size_value(value)
        ));
    }
    if let Some(value) = props.sizing.min_h.as_ref() {
        output.push_str(&format!(
            "        Integer {view}MinHeight = {};\n        if ({view}MinHeight != null && {view}MinHeight != ViewGroup.LayoutParams.MATCH_PARENT) {{\n            {view}.setMinimumHeight(doweDp({view}MinHeight));\n        }}\n",
            dev_size_value(value)
        ));
    }

    if let Some(animation) = props.animation {
        output.push_str(&format!(
            "        doweAnimate({view}, \"{}\");\n",
            animation.as_str()
        ));
    }
}

fn collect_joined_text(children: &[ViewNode]) -> String {
    let mut texts = Vec::new();
    for child in children {
        collect_texts(child, &mut texts);
    }
    texts.join(" ")
}

fn java_string_array<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let values = values
        .map(|value| format!("\"{}\"", escape_java(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("new String[]{{{values}}}")
}

fn java_nullable_string_array<'a>(values: impl Iterator<Item = Option<&'a str>>) -> String {
    let values = values
        .map(|value| {
            value
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("new String[]{{{values}}}")
}

fn java_int_array(values: impl Iterator<Item = String>) -> String {
    format!("new int[]{{{}}}", values.collect::<Vec<_>>().join(", "))
}

fn dev_code_token_color(kind: CodeTokenKind, plain: &str) -> String {
    match kind {
        CodeTokenKind::Plain => plain.to_string(),
        CodeTokenKind::Keyword => "DOWE_PRIMARY".to_string(),
        CodeTokenKind::Type => "DOWE_INFO".to_string(),
        CodeTokenKind::String => "DOWE_SUCCESS".to_string(),
        CodeTokenKind::Number => "DOWE_WARNING".to_string(),
        CodeTokenKind::Attribute => "DOWE_TERTIARY".to_string(),
        CodeTokenKind::Comment => "DOWE_MUTED".to_string(),
        CodeTokenKind::Punctuation => "DOWE_DANGER".to_string(),
    }
}

fn collect_texts<'a>(node: &'a ViewNode, output: &mut Vec<&'a str>) {
    match node {
        ViewNode::Scope { children, .. }
        | ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Button { children, .. } => {
            for child in children {
                collect_texts(child, output);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_texts(child, output);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            entries,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_texts(child, output);
            }
            for entry in entries {
                match entry {
                    OverlayEntry::Item(item) => output.push(&item.label),
                    OverlayEntry::Divider => {}
                }
            }
        }
        ViewNode::Command { entries, .. } => {
            for entry in entries {
                match entry {
                    CommandEntry::Item(item) => output.push(&item.label),
                    CommandEntry::Group { label, items, .. } => {
                        output.push(label);
                        output.extend(items.iter().map(|item| item.label.as_str()));
                    }
                }
            }
        }
        ViewNode::SideNav { items, .. } | ViewNode::Sidebar { items, .. } => {
            for item in items {
                match item {
                    SideNavItem::Header(props) | SideNavItem::Item(props) => {
                        output.push(&props.label);
                    }
                    SideNavItem::Submenu { props, items, .. } => {
                        output.push(&props.label);
                        output.extend(items.iter().map(|props| props.label.as_str()));
                    }
                    SideNavItem::Divider => {}
                }
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                match item {
                    NavMenuItem::Item(props) => output.push(&props.label),
                    NavMenuItem::Submenu { props, items } => {
                        output.push(&props.label);
                        output.extend(items.iter().map(|props| props.label.as_str()));
                    }
                    NavMenuItem::Megamenu { props, content } => {
                        output.push(&props.label);
                        for child in content {
                            collect_texts(child, output);
                        }
                    }
                }
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                output.push(&tab.label);
                for child in &tab.children {
                    collect_texts(child, output);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                output.push(&item.label);
                for child in &item.children {
                    collect_texts(child, output);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            if let Some(title) = props.title.as_deref() {
                output.push(title);
            }
            for slide in slides {
                for child in &slide.children {
                    collect_texts(child, output);
                }
            }
        }
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => {
            for child in start.iter().chain(center.iter()).chain(end.iter()) {
                collect_texts(child, output);
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_texts(child, output);
            }
        }
        ViewNode::Title { value, .. } | ViewNode::Text { value, .. } => output.push(value),
        ViewNode::Alert { props } => output.push(&props.message),
        ViewNode::Avatar { props, .. } => output.push(&props.alt),
        ViewNode::Image { props } => output.push(&props.alt),
        ViewNode::Audio { props } => {
            if let Some(subtitle) = props.subtitle.as_deref() {
                output.push(subtitle);
            }
        }
        ViewNode::Chip { value, .. } => output.push(value),
        ViewNode::AlertDialog { props } => {
            output.push(&props.title);
            output.push(&props.description);
        }
        ViewNode::Toast { props } => {
            if let Some(title) = props.title.as_deref() {
                output.push(title);
            }
            output.push(&props.description);
        }
        ViewNode::Input { .. }
        | ViewNode::Select { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Children => {}
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComposeFlow {
    Block,
    Inline,
}

fn render_compose_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    default_family: FontFamily,
) {
    render_compose_node_in_flow(
        node,
        indent,
        output,
        ComposeFlow::Block,
        None,
        default_family,
        &ComposeReactiveContext::default(),
    );
}
