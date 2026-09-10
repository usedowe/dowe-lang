fn render_dev_android_button_flow_node(
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
) -> bool {
    if !matches!(node, ViewNode::Button { .. }) {
        return false;
    }
    match node {
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
            let icon_condition =
                |path: &str, comparison: Option<&dowe_components::ReactiveNumberComparison>| {
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
            let variant = props
                .reactive
                .variant
                .as_ref()
                .map(|path| reactive_text(path));
            let scheme = props
                .reactive
                .scheme
                .as_ref()
                .map(|path| reactive_text(path));
            let disabled = props
                .reactive
                .disabled
                .as_ref()
                .map(|path| reactive_bool(path));
            let disabled_path = props.reactive.disabled.as_ref().map(|path| {
                context
                    .item_path(path)
                    .unwrap_or_else(|| context.signal_path(path))
            });
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
            let radius = props
                .reactive
                .rounded
                .as_ref()
                .map(|path| format!("doweButtonRadius({})", reactive_text(path)))
                .unwrap_or_else(|| dev_style_radius(&props.style));
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
                output.push_str(&format!("        LinearLayout {view} = doweContainer(true);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setGravity(Gravity.CENTER);\n"));
                if let Some(size) = props.reactive.size.as_ref().map(|path| reactive_text(path)) {
                    let width = if props.icon_only {
                        format!("doweButtonMinHeight({size})")
                    } else {
                        "ViewGroup.LayoutParams.WRAP_CONTENT".to_string()
                    };
                    output.push_str(&format!("        {view}.setLayoutParams(new LinearLayout.LayoutParams({width}, doweButtonMinHeight({size})));\n        {view}.setPadding(doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}), doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}));\n"));
                }
                output.push_str(&format!("        {view}.setBackground(doweInputBackground({container}, {border}, {radius}));\n"));
                if props.icon_only {
                    output.push_str(&format!(
                        "        {view}.setContentDescription(\"{}\");\n",
                        escape_java(props.label.as_deref().unwrap_or_default())
                    ));
                }
                if let Some(path) = props.swap_bind.as_deref() {
                    if let Some(icon) = props.icon_start.as_ref() {
                        let on_icon =
                            render_dev_android_icon_view(icon, counter, output, Some(&content));
                        output.push_str(&format!("        {on_icon}.setVisibility(doweBool(\"{}\") ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {on_icon});\n", escape_java(&context.signal_path(path))));
                    }
                    if let Some(icon) = props.swap_icon_off.as_ref() {
                        let off_icon =
                            render_dev_android_icon_view(icon, counter, output, Some(&content));
                        output.push_str(&format!("        {off_icon}.setVisibility(doweBool(\"{}\") ? View.GONE : View.VISIBLE);\n        doweAdd({view}, {off_icon});\n", escape_java(&context.signal_path(path))));
                    }
                } else if let Some(icon) = props.icon_start.as_ref() {
                    if let Some(path) = props.reactive.icon_start_when.as_ref() {
                        output.push_str(&format!(
                            "        if ({}) {{\n",
                            icon_condition(path, props.reactive.icon_start_comparison.as_ref())
                        ));
                        render_dev_android_side_nav_icon(
                            icon,
                            &view,
                            counter,
                            output,
                            Some(&content),
                        );
                        output.push_str("        }\n");
                    } else {
                        render_dev_android_side_nav_icon(
                            icon,
                            &view,
                            counter,
                            output,
                            Some(&content),
                        );
                    }
                }
                if !props.icon_only {
                    let label = next_dev_view(counter);
                    output.push_str(&format!("        TextView {label} = doweText({text}, {content}, {}, 400, 0f, 1.2f, {});\n        {label}.setTextIsSelectable(false);\n        doweAdd({view}, {label}, 8, true);\n", dev_text_size_expr(false, INPUT_TEXT_SIZE), dev_font_value(props.style.font.as_ref().or(inherited_font))));
                    if let Some(icon) = props.icon_end.as_ref() {
                        if let Some(path) = props.reactive.icon_end_when.as_ref() {
                            output.push_str(&format!(
                                "        if ({}) {{\n",
                                icon_condition(path, props.reactive.icon_end_comparison.as_ref())
                            ));
                            render_dev_android_side_nav_icon(
                                icon,
                                &view,
                                counter,
                                output,
                                Some(&content),
                            );
                            output.push_str("        }\n");
                        } else {
                            render_dev_android_side_nav_icon(
                                icon,
                                &view,
                                counter,
                                output,
                                Some(&content),
                            );
                        }
                    }
                }
                if let Some(path) = props.swap_bind.as_deref() {
                    let bind = escape_java(&context.signal_path(path));
                    let action_body = action
                        .as_deref()
                        .and_then(|value| {
                            value
                                .strip_prefix("{ ")
                                .and_then(|body| body.strip_suffix(" }"))
                        })
                        .unwrap_or("");
                    output.push_str(&format!("        {view}.setOnClickListener(v -> {{ doweWrite(\"{bind}\", !doweBool(\"{bind}\")); {action_body} renderCurrentRoute(false); }});\n"));
                } else if let Some(action) = action {
                    output.push_str(&format!(
                        "        {view}.setOnClickListener(v -> {action});\n"
                    ));
                }
                if let Some(variant) = props.reactive.variant.as_ref() {
                    output.push_str(&format!(
                        "        {view}.setTag(DOWE_VARIANT_TAG, \"{}\");\n",
                        escape_java(variant)
                    ));
                }
                if let Some(scheme) = props.reactive.scheme.as_ref() {
                    output.push_str(&format!(
                        "        {view}.setTag(DOWE_SCHEME_TAG, \"{}\");\n",
                        escape_java(scheme)
                    ));
                }
                if let Some(size) = props.reactive.size.as_ref() {
                    output.push_str(&format!(
                        "        {view}.setTag(DOWE_SIZE_TAG, \"{}\");\n",
                        escape_java(size)
                    ));
                }
                if let Some(disabled) = disabled.as_ref() {
                    let disabled_path = disabled_path.as_deref().unwrap_or_default();
                    output.push_str(&format!("        {view}.setTag(DOWE_DISABLED_PATH_TAG, \"{}\");\n        {view}.setEnabled(!({disabled}));\n        {view}.setAlpha({disabled} ? 0.5f : 1f);\n", escape_java(disabled_path)));
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
                output.push_str(&format!("        {view}.setTag(DOWE_COMPACT_WIDTH_TAG, {view}.getLayoutParams().width == ViewGroup.LayoutParams.WRAP_CONTENT);\n"));
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
                return true;
            }
            output.push_str(&format!(
                            "        Button {view} = new Button(this);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setText({});\n        {view}.setTextIsSelectable(false);\n        {view}.setAllCaps(false);\n        {view}.setTypeface(Typeface.create({}, android.graphics.Typeface.NORMAL));\n        {view}.setTextSize({});\n        {view}.setIncludeFontPadding(false);\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setMinWidth(0);\n        {view}.setMinimumWidth(0);\n        {view}.setMinHeight(0);\n        {view}.setMinimumHeight(0);\n        {view}.setTextColor({});\n        {view}.setBackgroundTintList(null);\n        {view}.setBackground(doweInputBackground({}, {}, {}));\n",
                            text,
                            dev_font_value(props.style.font.as_ref().or(inherited_font)),
                            dev_text_size_expr(false, INPUT_TEXT_SIZE),
                            content,
                            container,
                            border,
                            radius
                        ));
            if let Some(size) = props.reactive.size.as_ref().map(|path| reactive_text(path)) {
                output.push_str(&format!("        {view}.setMinHeight(doweButtonMinHeight({size}));\n        {view}.setMinimumHeight(doweButtonMinHeight({size}));\n        {view}.getLayoutParams().height = doweButtonMinHeight({size});\n        {view}.setPadding(doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}), doweButtonHorizontalPadding({size}), doweButtonVerticalPadding({size}));\n"));
            }
            if let Some(action) = action {
                output.push_str(&format!(
                    "        {view}.setOnClickListener(v -> {action});\n"
                ));
            }
            if let Some(disabled) = disabled.as_ref() {
                let disabled_path = disabled_path.as_deref().unwrap_or_default();
                output.push_str(&format!("        {view}.setTag(DOWE_DISABLED_PATH_TAG, \"{}\");\n        {view}.setEnabled(!({disabled}));\n        {view}.setAlpha({disabled} ? 0.5f : 1f);\n", escape_java(disabled_path)));
            }
            let mut button_style = props.style.clone();
            button_style.shadow = None;
            button_style.shadow_color = None;
            if props.reactive.rounded.is_some() {
                button_style.rounded = None;
            }
            apply_dev_android_style(&button_style, &view, false, output);
            apply_dev_android_shadow_with_radius(&props.style, &view, &radius, output);
            output.push_str(&format!("        {view}.setTag(DOWE_COMPACT_WIDTH_TAG, {view}.getLayoutParams().width == ViewGroup.LayoutParams.WRAP_CONTENT);\n"));
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
    true
}
