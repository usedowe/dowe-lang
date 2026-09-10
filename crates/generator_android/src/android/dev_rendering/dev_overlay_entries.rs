fn render_dev_android_overlay_entry(
    entry: &OverlayEntry,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    dismiss: Option<&str>,
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
            dismiss,
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
    dismiss: Option<&str>,
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
            dismiss,
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
                    dismiss,
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
    dismiss: Option<&str>,
) {
    let view = next_dev_view(counter);
    let action = if item.disabled {
        None
    } else {
        dev_android_overlay_item_action(item, context, dismiss)
    };
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setPadding(doweDp(16), doweDp(10), doweDp(16), doweDp(10));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n        TextView {view}Label = doweText(\"{}\", {}, 14f, 700, 0f, 1.2f, {});\n        doweAdd({view}, {view}Label);\n",
        if action.is_some() {
            dev_variant_container(props)
        } else {
            "Color.TRANSPARENT"
        },
        escape_java(&item.label),
        dev_variant_content(props),
        dev_font_value(props.style.font.as_ref().or(inherited_font))
    ));
    if let Some(description) = item.description.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Description = doweText(\"{}\", doweAlpha({}, 0.68f), 12f, 400, 0f, 1.2f, {});\n        doweAdd({view}, {view}Description, 4, false);\n",
            escape_java(description),
            dev_variant_content(props),
            dev_font_value(props.style.font.as_ref().or(inherited_font))
        ));
    }
    if item.disabled {
        output.push_str(&format!("        {view}.setAlpha(0.48f);\n"));
    }
    if let Some(action) = action {
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> {{ {action} }});\n"
        ));
    }
    output.push_str(&dev_add(parent, &view, None, false));
}

fn dev_android_overlay_item_action(
    item: &OverlayItemProps,
    context: &ComposeReactiveContext,
    dismiss: Option<&str>,
) -> Option<String> {
    let action = item
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null);", escape_java(id)))
        .or_else(|| {
            dev_android_navigation_action(item.navigation.as_ref())
                .map(|action| format!("{action};"))
        })?;
    let close = dismiss
        .map(|value| format!("{value}; "))
        .unwrap_or_default();
    Some(format!("{close}{action}"))
}
