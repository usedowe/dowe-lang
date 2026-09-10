fn render_dev_android_relative_box(
    props: &StyleProps,
    children: &[ViewNode],
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
    let current_font = props.font.as_ref().or(inherited_font);
    let current_color = dev_inherited_color(props, inherited_color.as_deref());
    let view = next_dev_view(counter);
    let content = next_dev_view(counter);
    output.push_str(&format!(
        "        FrameLayout {view} = new FrameLayout(this);\n"
    ));
    if !parent_horizontal && props.sizing.w.is_none() {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
        ));
    }
    apply_dev_android_style(props, &view, true, output);
    apply_dev_android_click(props, &view, context, output);
    apply_dev_android_inline_width(props, &view, parent_horizontal, output);
    apply_dev_android_flex_item(props, parent, &view, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    render_dev_android_cover_image(props, &view, output);
    output.push_str(&format!(
        "        LinearLayout {content} = doweContainer(false);\n        {view}.addView({content}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.START));\n"
    ));
    for child in children.iter().filter(|child| {
        !matches!(child, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Absolute)
    }) {
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
    for child in children.iter().filter(|child| {
        matches!(child, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Absolute)
    }) {
        let show = node_element_props(child).and_then(|props| props.show.as_ref());
        let ViewNode::Box { props, children } = child else {
            unreachable!();
        };
        if let Some(show) = show {
            output.push_str(&format!(
                "        if ({}) {{\n",
                dev_show_condition(show, context)
            ));
        }
        render_dev_android_positioned_box(
            props,
            children,
            &view,
            counter,
            output,
            current_font,
            current_color.clone(),
            context,
            children_method,
        );
        if show.is_some() {
            output.push_str("        }\n");
        }
    }
}

fn render_dev_android_cover_image(props: &StyleProps, view: &str, output: &mut String) {
    let Some(cover) = props.cover.as_ref() else {
        return;
    };
    let source =
        dev_responsive_string_value(cover, |value| format!("\"{}\"", escape_java(&value.0)));
    output.push_str(&format!(
        "        String {view}Cover = {source};\n        if ({view}Cover != null && !{view}Cover.isEmpty()) {{\n            ImageView {view}CoverImage = new ImageView(this);\n            {view}CoverImage.setScaleType(ImageView.ScaleType.CENTER_CROP);\n            FrameLayout.LayoutParams {view}CoverParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT);\n            {view}.addView({view}CoverImage, {view}CoverParams);\n            new Thread(() -> {{\n                Bitmap {view}CoverBitmap = doweLoadImageBitmap({view}Cover);\n                if ({view}CoverBitmap != null) {{\n                    runOnUiThread(() -> {view}CoverImage.setImageBitmap({view}CoverBitmap));\n                }}\n            }}).start();\n        }}\n"
    ));
}

fn render_dev_android_positioned_box(
    props: &StyleProps,
    children: &[ViewNode],
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let current_font = props.font.as_ref().or(inherited_font);
    let current_color = dev_inherited_color(props, inherited_color.as_deref());
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(props, &view, true, output);
    apply_dev_android_click(props, &view, context, output);
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
    let params = format!("{view}PositionParams");
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {params} = doweFrameLayoutParams({view}.getLayoutParams());\n        {params}.gravity = {};\n        {params}.setMargins({}, {}, {}, {});\n        {parent}.addView({view}, {params});\n",
        dev_box_position_gravity(props.position()),
        dev_box_offset(props.position().left.as_ref()),
        dev_box_offset(props.position().top.as_ref()),
        dev_box_offset(props.position().right.as_ref()),
        dev_box_offset(props.position().bottom.as_ref()),
    ));
}

fn dev_overlay_value(value: &ResponsiveValue<OverlayPaint>) -> String {
    dev_responsive_value(value, |paint| match paint {
        OverlayPaint::BlackOpacity(opacity) => format!(
            "doweAlpha(Color.BLACK, {}f)",
            opacity
        ),
        OverlayPaint::Color(color) => java_color(*color).to_string(),
        OverlayPaint::Rgba(rgba) => format!("Color.parseColor(\"{}\")", escape_java(rgba)),
        OverlayPaint::LinearGradient(_) => "Color.TRANSPARENT".to_string(),
    })
}

fn render_dev_android_fixed_box(
    props: &StyleProps,
    children: &[ViewNode],
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let overlay = next_dev_view(counter);
    output.push_str(&format!(
        "        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setTag(\"dowe-fixed-box\");\n        {overlay}.setClipChildren(false);\n        {overlay}.setClipToPadding(false);\n"
    ));
    render_dev_android_positioned_box(
        props,
        children,
        &overlay,
        counter,
        output,
        inherited_font,
        inherited_color,
        context,
        children_method,
    );
    output.push_str(&format!(
        "        ((ViewGroup) scrollView.getParent()).addView({overlay}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        doweApplySystemInsets({overlay});\n"
    ));
}

fn dev_box_position_gravity(props: &PositionProps) -> String {
    let vertical = if props.bottom.is_some() {
        "Gravity.BOTTOM"
    } else {
        "Gravity.TOP"
    };
    let horizontal = if props.right.is_some() {
        "Gravity.END"
    } else {
        "Gravity.START"
    };
    format!("{vertical} | {horizontal}")
}

fn dev_box_offset(value: Option<&ResponsiveValue<ScaleValue>>) -> String {
    value
        .map(|value| format!("doweDp({})", dev_scale_value(value)))
        .unwrap_or_else(|| "0".to_string())
}
