fn render_dev_android_overlay_node(
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
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            render_dev_android_drawer(
                props,
                header,
                body,
                footer,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::Avatar { props, icon } => {
            let label = props
                .name
                .as_deref()
                .or(Some(props.alt.as_str()))
                .and_then(|value| value.chars().next())
                .map(|value| value.to_uppercase().to_string())
                .unwrap_or_else(|| "A".to_string());
            let (_static_size, static_text_size) = match props.size {
                AvatarSize::Xs => (24, 12),
                AvatarSize::Sm => (32, 14),
                AvatarSize::Md => (40, 16),
                AvatarSize::Lg => (48, 18),
                AvatarSize::Xl => (64, 24),
                AvatarSize::Xxl => (80, 28),
                AvatarSize::Xxxl => (96, 32),
                AvatarSize::Xxxxl => (112, 36),
                AvatarSize::Xxxxxl => (128, 40),
                AvatarSize::Xxxxxxl => (144, 44),
                AvatarSize::Xxxxxxxl => (160, 48),
            };
            let size_value = props
                .size_binding
                .as_ref()
                .map(|binding| dev_text_expression(&binding.path, None, context))
                .unwrap_or_else(|| format!("\"{}\"", props.size.as_str()));
            let size = format!("doweAvatarSize({size_value})");
            let text_size = props
                .size_binding
                .as_ref()
                .map(|binding| format!("doweAvatarTextSize({})", dev_text_expression(&binding.path, None, context)))
                .unwrap_or_else(|| format!("{static_text_size}f"));
            let view = next_dev_view(counter);
            if let Some(source) = props.src.as_deref() {
                output.push_str(&format!(
                    "        FrameLayout {view} = doweAvatarImage(\"{}\", \"{}\", {}, {}, {}, {text_size}, {});\n",
                    escape_java(source),
                    escape_java(&props.alt),
                    dev_text_expression(&label, None, context),
                    dev_variant_container(&props.style),
                    dev_variant_content(&props.style),
                    dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                ));
            } else if let Some(icon) = icon {
                output.push_str(&format!(
                    "        FrameLayout {view} = new FrameLayout(this);\n"
                ));
                let icon_view = format!("view{counter}");
                let icon_node = ViewNode::Svg {
                    props: icon.props.clone(),
                    paths: icon.paths.clone(),
                };
                render_dev_android_node(
                    &icon_node,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    inherited_font,
                    inherited_color.clone(),
                    context,
                    children_method,
                );
                output.push_str(&format!(
                    "        FrameLayout.LayoutParams {icon_view}Params = new FrameLayout.LayoutParams(doweDp((int)({size} * .6f)), doweDp((int)({size} * .6f)));\n        {icon_view}Params.gravity = Gravity.CENTER;\n        {icon_view}.setLayoutParams({icon_view}Params);\n"
                ));
            } else {
                output.push_str(&format!(
                    "        TextView {view} = doweText({}, {}, {text_size}, 600, 0f, 1.2f, {});\n        {view}.setGravity(Gravity.CENTER);\n",
                    dev_text_expression(&label, None, context),
                    dev_variant_content(&props.style),
                    dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                ));
            }
            let mut avatar_style = props.style.style.clone();
            avatar_style.shadow = None;
            avatar_style.shadow_color = None;
            avatar_style.rounded = None;
            apply_dev_android_style(&avatar_style, &view, false, output);
            output.push_str(&format!(
                "        {view}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({size}), doweDp({size})));\n        {view}.setBackground(doweBackground({}, 999f));\n        doweRound({view}, 999f);\n",
                dev_variant_container(&props.style)
            ));
            apply_dev_android_shadow_with_radius(
                &props.style.style,
                &view,
                "999f",
                output,
            );
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Badge { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = inherited_color.clone();
            let view = next_dev_view(counter);
            let content = next_dev_view(counter);
            let label = next_dev_view(counter);
            let (gravity, translation_x, translation_y) = match props.position {
                OverlayCornerPosition::TopLeft => (
                    "Gravity.TOP | Gravity.START",
                    "-v.getWidth() / 2f",
                    "-v.getHeight() / 2f",
                ),
                OverlayCornerPosition::TopRight => (
                    "Gravity.TOP | Gravity.END",
                    "v.getWidth() / 2f",
                    "-v.getHeight() / 2f",
                ),
                OverlayCornerPosition::BottomLeft => (
                    "Gravity.BOTTOM | Gravity.START",
                    "-v.getWidth() / 2f",
                    "v.getHeight() / 2f",
                ),
                OverlayCornerPosition::BottomRight => (
                    "Gravity.BOTTOM | Gravity.END",
                    "v.getWidth() / 2f",
                    "v.getHeight() / 2f",
                ),
            };
            output.push_str(&format!(
                "        FrameLayout {view} = new DoweBadgeLayout(this);\n        LinearLayout {content} = doweContainer(false);\n        {view}.addView({content}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            for child in children {
                render_dev_android_node(
                    child,
                    &content,
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
            output.push_str(&format!(
                "        TextView {label} = doweText({}, {}, 12f, 700, 0f, 1f, {});\n        {label}.setSingleLine(true);\n        {label}.setGravity(Gravity.CENTER);\n        {label}.setPadding(doweDp(6), doweDp(2), doweDp(6), doweDp(2));\n        {label}.setBackground(doweBackground({}, 999f));\n        FrameLayout.LayoutParams {label}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, doweDp(20), {gravity});\n        {view}.addView({label}, {label}Params);\n        {label}.addOnLayoutChangeListener((v, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom) -> {{\n            v.setTranslationX({translation_x});\n            v.setTranslationY({translation_y});\n        }});\n",
                dev_text_expression(&props.text, None, context),
                dev_variant_content(&props.style),
                dev_font_value(current_font),
                dev_variant_container(&props.style),
            ));
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Chip {
            props,
            value,
            start,
            end,
        } => render_dev_android_chip(
            props,
            value,
            start.as_ref(),
            end.as_ref(),
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Skeleton { props } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                            "        View {view} = new View(this);\n        {view}.setMinimumHeight(doweDp(16));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n",
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
            render_dev_android_modal(
                props,
                header,
                body,
                footer,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::AlertDialog { props } => {
            render_dev_android_alert_dialog(props, counter, output, inherited_font, context);
        }
        ViewNode::Tooltip { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            for child in children {
                render_dev_android_node(
                    child,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    current_font,
                    inherited_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Toast { props } => {
            render_dev_android_toast(props, counter, output, inherited_font, context);
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            render_dev_android_dropdown(
                props,
                trigger,
                header,
                entries,
                footer,
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
        ViewNode::Command { props, entries } => {
            render_dev_android_command(props, entries, counter, output, inherited_font, context);
        }
        ViewNode::Children => {
            if let Some(method) = children_method {
                output.push_str(&format!("        {method}({parent});\n"));
            }
        }
        _ => {}
    }
}

fn render_dev_android_chip(
    props: &ChipProps,
    value: &str,
    start: Option<&SideNavIcon>,
    end: Option<&SideNavIcon>,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    let (height, horizontal_padding, text_size) = match size {
        ButtonSize::Xs => (20, 12, 12),
        ButtonSize::Sm => (24, 12, 12),
        ButtonSize::Md => (32, 16, 14),
        ButtonSize::Lg => (40, 20, 18),
        ButtonSize::Xl => (48, 24, 24),
    };
    let icon_size = size.chip_icon_size().native_units();
    let view = next_dev_view(counter);
    let content = dev_variant_content(&props.style);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp({horizontal_padding}), 0, doweDp({horizontal_padding}), 0);\n        {view}.setMinimumHeight(doweDp({height}));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n        doweWrapContentWidth({view});\n",
        dev_variant_container(&props.style)
    ));
    if let Some(icon) = start {
        let icon_view = render_dev_android_icon_view(icon, counter, output, Some(&content));
        output.push_str(&format!(
            "        {icon_view}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({icon_size}), doweDp({icon_size})));\n        doweAdd({view}, {icon_view});\n"
        ));
    }
    let label = next_dev_view(counter);
    output.push_str(&format!(
        "        TextView {label} = doweText({}, {}, {text_size}f, 500, 0f, 1.2f, {});\n        {label}.setSingleLine(true);\n        doweAdd({view}, {label}, 8, true);\n",
        dev_text_expression(value, None, context),
        content,
        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
    ));
    if let Some(icon) = end {
        let icon_view = render_dev_android_icon_view(icon, counter, output, Some(&content));
        output.push_str(&format!(
            "        {icon_view}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({icon_size}), doweDp({icon_size})));\n        doweAdd({view}, {icon_view}, 8, true);\n"
        ));
    }
    if let Some(action) = props.on_close.as_deref().and_then(|name| context.action_id(name)) {
        let close = next_dev_view(counter);
        output.push_str(&format!(
            "        TextView {close} = doweText(\"x\", {}, {text_size}f, 700, 0f, 1.2f, {});\n        {close}.setGravity(Gravity.CENTER);\n        {close}.setContentDescription(\"Close\");\n        {close}.setOnClickListener(v -> doweRunAction(\"{}\", null));\n        doweAdd({view}, {close}, 8, true);\n",
            content,
            dev_font_value(props.style.style.font.as_ref().or(inherited_font)),
            escape_java(action)
        ));
    }
    apply_dev_android_style(&props.style.style, &view, false, output);
    apply_dev_android_click(&props.style.style, &view, context, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
