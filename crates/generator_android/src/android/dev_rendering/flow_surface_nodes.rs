fn render_dev_android_surface_flow_node(
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
) -> bool {
    if !matches!(node, ViewNode::Card { .. } | ViewNode::Brand { .. } | ViewNode::Banner { .. }) {
        return false;
    }
    match node {
        ViewNode::Card { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let view = next_dev_view(counter);
            let reactive_scheme = props.reactive.scheme.as_ref().map(|path| {
                let item = context.active_item().unwrap_or("null");
                let path = context
                    .item_path(path)
                    .unwrap_or_else(|| context.signal_path(path));
                format!("runtime.doweTextValue(\"{}\", {item})", escape_java(&path))
            });
            let reactive_variant = props.reactive.variant.as_ref().map(|path| {
                let item = context.active_item().unwrap_or("null");
                let path = context
                    .item_path(path)
                    .unwrap_or_else(|| context.signal_path(path));
                format!("runtime.doweTextValue(\"{}\", {item})", escape_java(&path))
            });
            let variant = reactive_variant.as_deref().unwrap_or("\"solid\"");
            let scheme = reactive_scheme.as_deref().unwrap_or("\"primary\"");
            let current_color = if props.style.text.is_some() {
                dev_inherited_color(&props.style, inherited_color.as_deref())
            } else if reactive_scheme.is_some() || reactive_variant.is_some() {
                Some(dev_content_colors(
                    &format!("runtime.doweButtonContent({variant}, {scheme})"),
                    &format!("runtime.doweButtonContent({variant}, {scheme})"),
                ))
            } else {
                Some(dev_content_colors(
                    dev_card_variant_content(props),
                    dev_card_variant_title(props),
                ))
            };
            let container = if reactive_scheme.is_some() || reactive_variant.is_some() {
                format!("runtime.doweButtonContainer({variant}, {scheme})")
            } else {
                dev_card_variant_container(props).to_string()
            };
            let border = if props.reactive.variant.is_some() {
                format!("runtime.doweButtonContent({variant}, {scheme})")
            } else {
                dev_card_border(props).to_string()
            };
            let has_cover = props.style.cover.is_some();
            if has_cover {
                output.push_str(&format!(
                    "        FrameLayout {view} = new FrameLayout(this);\n"
                ));
            } else {
                output.push_str(&format!(
                    "        LinearLayout {view} = runtime.doweCard({container}, (\"outlined\".equals({variant}) ? {border} : null));\n"
                ));
            }
            if let Some(path) = props.reactive.scheme.as_ref() {
                output.push_str(&format!(
                    "        {view}.setTag(DOWE_SCHEME_TAG, \"{}\");\n",
                    escape_java(path)
                ));
            }
            let mut card_style = props.style.clone();
            if has_cover {
                card_style.spacing = Default::default();
            }
            apply_dev_android_style(&card_style, &view, false, output);
            if let Some(border) = props.style.border.as_ref() {
                let border_color = props
                    .style
                    .border_color
                    .map(|family| java_color(family_color(family)).to_string())
                    .unwrap_or_else(|| "DOWE_BACKGROUND_TEXT".to_string());
                output.push_str(&format!(
                    "        {view}.setBackground(doweStyledBackground({container}, {border_color}, {}, {}));\n",
                    dev_border_value(border),
                    dev_style_radius(&props.style)
                ));
            }
            let child_parent = if has_cover {
                let source = dev_responsive_string_value(
                    props.style.cover.as_ref().expect("cover"),
                    |value| format!("\"{}\"", escape_java(&value.0)),
                );
                let image = format!("{view}CoverImage");
                let overlay = format!("{view}CoverOverlay");
                let content = format!("{view}Content");
                output.push_str(&format!(
                    "        FrameLayout {image} = runtime.doweImage({source}, \"\", \"auto\", \"cover\", {container}, null);\n        {view}.addView({image}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        View {overlay} = new View(this);\n"
                ));
                if let Some(value) = props.style.overlay.as_ref() {
                    output.push_str(&format!(
                        "        {overlay}.setBackgroundColor({});\n",
                        dev_overlay_value(value)
                    ));
                }
                output.push_str(&format!(
                    "        {view}.addView({overlay}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        LinearLayout {content} = doweContainer(false);\n"
                ));
                let mut content_style = StyleProps::default();
                content_style.spacing = props.style.spacing.clone();
                apply_dev_android_style(&content_style, &content, false, output);
                output.push_str(&format!(
                    "        {view}.addView({content}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n"
                ));
                content
            } else {
                view.clone()
            };
            apply_dev_android_click(&props.style, &view, context, output);
            apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
            apply_dev_android_flex_item(&props.style, parent, &view, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &child_parent,
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
        ViewNode::Banner { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            if let Some(label) = props.label.as_deref() {
                output.push_str(&format!(
                    "        {view}.setContentDescription(\"{}\");\n",
                    escape_java(label)
                ));
            }
            if let Some(action) = dev_android_navigation_action(Some(&props.navigation)) {
                output.push_str(&format!(
                    "        {view}.setOnClickListener(v -> {action});\n"
                ));
            }
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
        _ => {}
    }
    true
}
