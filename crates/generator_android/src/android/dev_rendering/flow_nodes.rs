fn render_dev_android_flow_node(
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
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            output.push_str(&format!(
                "        if (doweBool(\"{}\")) {{\n",
                escape_java(&context.signal_path(binding))
            ));
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
                    context,
                    children_method,
                );
            }
            output.push_str("        } else {\n");
            for child in content {
                render_dev_android_node(
                    child,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    inherited_font,
                    inherited_color.clone(),
                    context,
                    children_method,
                );
            }
            output.push_str("        }\n");
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, actions);
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
            apply_dev_android_inline_width(props, &view, parent_horizontal, output);
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
            let body = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            let mut outer_props = props.clone();
            outer_props.spacing = Default::default();
            apply_dev_android_style(&outer_props, &view, true, output);
            apply_dev_android_inline_width(props, &view, parent_horizontal, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            let body_constructor = if props.boxed {
                "doweBoxedContainer(1536)"
            } else {
                "doweContainer(false)"
            };
            output.push_str(&format!(
                "        LinearLayout {body} = {body_constructor};\n"
            ));
            let mut body_props = StyleProps::default();
            body_props.spacing = props.spacing.with_padding_default(ResponsiveValue::ordered(vec![
                dowe_components::ResponsiveEntry { breakpoint: Breakpoint::Xs, value: ScaleValue::from_half_steps(8) },
                dowe_components::ResponsiveEntry { breakpoint: Breakpoint::Md, value: ScaleValue::from_half_steps(12) },
            ]));
            apply_dev_android_style(&body_props, &body, false, output);
            output.push_str(&dev_add(&view, &body, None, false));
            for child in children {
                render_dev_android_node(
                    child,
                    &body,
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
                "        DoweFlexLayout {view} = doweFlex({}, {}, {}, {}, {});\n",
                dev_flex_direction(&props.direction),
                props.wrap,
                dev_flex_justify(props.justify.as_ref()),
                dev_flex_align(props.align.as_ref()),
                dev_optional_gap(props.gap.as_ref(), true).unwrap_or_else(|| "null".to_string())
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    dev_flex_has_row(&props.direction),
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
            apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
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
            apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
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
        ViewNode::Brand { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n"
            ));
            if let Some(label) = props.label.as_deref() {
                output.push_str(&format!(
                    "        {view}.setContentDescription(\"{}\");\n",
                    escape_java(label)
                ));
            }
            if let Some(action) = dev_android_navigation_action(props.navigation.as_ref()) {
                output.push_str(&format!(
                    "        {view}.setOnClickListener(v -> {action});\n"
                ));
            }
            apply_dev_android_style(&props.style, &view, false, output);
            apply_dev_android_inline_width(&props.style, &view, true, output);
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
        ViewNode::Button { props, children } => {
            let view = next_dev_view(counter);
            let text = dev_visible_text_expression(&collect_joined_text(children), None, context);
            let reactive_text = |path: &str| {
                let item = context.active_item().unwrap_or("null");
                let path = context
                    .item_path(path)
                    .unwrap_or_else(|| context.signal_path(path));
                format!("doweTextValue(\"{}\", {item})", escape_java(&path))
            };
            let reactive_bool = |path: &str| {
                let item = context.active_item().unwrap_or("null");
                let path = context
                    .item_path(path)
                    .unwrap_or_else(|| context.signal_path(path));
                format!("doweBool(\"{}\", {item})", escape_java(&path))
            };
            let icon_condition = |path: &str,
                                  comparison: Option<&dowe_components::ReactiveNumberComparison>| {
                comparison
                    .map(|comparison| {
                        format!(
                            "Double.parseDouble({}) {} {}",
                            reactive_text(path),
                            comparison.operator.as_str(),
                            comparison.value
                        )
                    })
                    .unwrap_or_else(|| reactive_bool(path))
            };
            let variant = props.reactive.variant.as_ref().map(|path| reactive_text(path));
            let scheme = props.reactive.scheme.as_ref().map(|path| reactive_text(path));
            let variant_value = variant.clone().unwrap_or_else(|| {
                format!(
                    "\"{}\"",
                    props.variant.unwrap_or(ComponentVariant::Solid).as_str()
                )
            });
            let scheme_value = scheme.clone().unwrap_or_else(|| {
                format!(
                    "\"{}\"",
                    props.color.unwrap_or(ColorFamily::Primary).as_str()
                )
            });
            let reactive_visual = variant.is_some() || scheme.is_some();
            let content = if reactive_visual {
                format!("doweButtonContent({variant_value}, {scheme_value})")
            } else {
                dev_variant_content(props).to_string()
            };
            let container = if reactive_visual {
                format!("doweButtonContainer({variant_value}, {scheme_value})")
            } else {
                dev_variant_container(props).to_string()
            };
            let border = if reactive_visual {
                format!("(\"outlined\".equals({variant_value}) ? {content} : null)")
            } else {
                dev_button_border(props).to_string()
            };
            let radius = props.reactive.rounded.as_ref().map(|path| format!("doweButtonRadius({})", reactive_text(path))).unwrap_or_else(|| dev_style_radius(&props.style));
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
            if props.icon_start.is_some() || props.icon_end.is_some() {
                output.push_str(&format!("        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER);\n"));
                if let Some(size) = props.reactive.size.as_ref().map(|path| reactive_text(path)) {
                    output.push_str(&format!("        {view}.setMinimumHeight(doweButtonMinHeight({size}));\n        {view}.setPadding(doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}), doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}));\n"));
                }
                output.push_str(&format!("        {view}.setBackground(doweInputBackground({container}, {border}, {radius}));\n"));
                if props.icon_only {
                    output.push_str(&format!(
                        "        {view}.setContentDescription(\"{}\");\n",
                        escape_java(props.label.as_deref().unwrap_or_default())
                    ));
                }
                if let Some(icon) = props.icon_start.as_ref() {
                    if let Some(path) = props.reactive.icon_start_when.as_ref() {
                        output.push_str(&format!("        if ({}) {{\n", icon_condition(path, props.reactive.icon_start_comparison.as_ref())));
                        render_dev_android_side_nav_icon(icon, &view, counter, output, Some(&content));
                        output.push_str("        }\n");
                    } else {
                        render_dev_android_side_nav_icon(icon, &view, counter, output, Some(&content));
                    }
                }
                if !props.icon_only {
                    let label = next_dev_view(counter);
                    output.push_str(&format!("        TextView {label} = doweText({text}, {content}, {}, 400, 0f, 1.2f, {});\n        doweAdd({view}, {label}, 8, true);\n", dev_text_size_expr(false, INPUT_TEXT_SIZE), dev_font_value(props.style.font.as_ref().or(inherited_font))));
                    if let Some(icon) = props.icon_end.as_ref() {
                        if let Some(path) = props.reactive.icon_end_when.as_ref() {
                            output.push_str(&format!("        if ({}) {{\n", icon_condition(path, props.reactive.icon_end_comparison.as_ref())));
                            render_dev_android_side_nav_icon(icon, &view, counter, output, Some(&content));
                            output.push_str("        }\n");
                        } else {
                            render_dev_android_side_nav_icon(icon, &view, counter, output, Some(&content));
                        }
                    }
                }
                if let Some(action) = action {
                    output.push_str(&format!("        {view}.setOnClickListener(v -> {action});\n"));
                }
                let mut button_style = props.style.clone();
                button_style.shadow = None;
                button_style.shadow_color = None;
                if props.reactive.rounded.is_some() {
                    button_style.rounded = None;
                }
                apply_dev_android_style(&button_style, &view, false, output);
                apply_dev_android_shadow_with_radius(&props.style, &view, &radius, output);
                apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
                return;
            }
            output.push_str(&format!(
                            "        Button {view} = new Button(this);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setText({});\n        {view}.setAllCaps(false);\n        {view}.setTypeface(Typeface.create({}, android.graphics.Typeface.NORMAL));\n        {view}.setTextSize({});\n        {view}.setIncludeFontPadding(false);\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setMinWidth(0);\n        {view}.setMinimumWidth(0);\n        {view}.setMinHeight(0);\n        {view}.setMinimumHeight(0);\n        {view}.setTextColor({});\n        {view}.setBackgroundTintList(null);\n        {view}.setBackground(doweInputBackground({}, {}, {}));\n",
                            text,
                            dev_font_value(props.style.font.as_ref().or(inherited_font)),
                            dev_text_size_expr(false, INPUT_TEXT_SIZE),
                            content,
                            container,
                            border,
                            radius
                        ));
            if let Some(size) = props.reactive.size.as_ref().map(|path| reactive_text(path)) {
                output.push_str(&format!("        {view}.setMinHeight(doweButtonMinHeight({size}));\n        {view}.setMinimumHeight(doweButtonMinHeight({size}));\n        {view}.setPadding(doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}), doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}));\n"));
            }
            if let Some(action) = action {
                output.push_str(&format!(
                    "        {view}.setOnClickListener(v -> {action});\n"
                ));
            }
            let mut button_style = props.style.clone();
            button_style.shadow = None;
            button_style.shadow_color = None;
            if props.reactive.rounded.is_some() {
                button_style.rounded = None;
            }
            apply_dev_android_style(&button_style, &view, false, output);
            apply_dev_android_shadow_with_radius(&props.style, &view, &radius, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}
