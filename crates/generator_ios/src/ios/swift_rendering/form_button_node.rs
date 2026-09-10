fn render_swift_button_node(
    props: &VariantProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
let current_font = props.style.font.as_ref().or(inherited_font);
let reactive_text = |path: &str, fallback: &str| {
    context
        .item_value(path)
        .map(|item| {
            format!(
                "state.text(\"{}\", item: {item})",
                escape_swift(&context.item_path(path).expect("item path"))
            )
        })
        .unwrap_or_else(|| {
            format!(
                "state.text(\"{}\", fallback: \"{fallback}\")",
                escape_swift(&context.signal_path(path))
            )
        })
};
let reactive_bool = |path: &str| {
    context
        .item_value(path)
        .map(|item| {
            format!(
                "state.bool(\"{}\", item: {item})",
                escape_swift(&context.item_path(path).expect("item path"))
            )
        })
        .unwrap_or_else(|| {
            format!(
                "state.bool(\"{}\", fallback: true)",
                escape_swift(&context.signal_path(path))
            )
        })
};
let icon_condition =
    |path: &str, comparison: Option<&dowe_components::ReactiveNumberComparison>| {
        comparison
            .map(|comparison| {
                format!(
                    "(Double({}) ?? 0) {} {}",
                    reactive_text(path, "0"),
                    comparison.operator.as_str(),
                    comparison.value
                )
            })
            .unwrap_or_else(|| reactive_bool(path))
    };
let action = props
    .element
    .on_click
    .as_deref()
    .and_then(|name| context.action_id(name))
    .map(|id| {
        let item = context
            .active_item()
            .map(|value| format!(", item: {value}"))
            .unwrap_or_default();
        format!("{{ state.run(\"{}\"{item}) }}", escape_swift(id))
    })
    .unwrap_or_else(|| swift_navigation_action(props.navigation.as_ref()));
let loading = props
    .reactive
    .loading
    .as_ref()
    .map(|path| reactive_bool(path));
let action = if let Some(path) = props.swap_bind.as_deref() {
    let bind = escape_swift(&context.signal_path(path));
    let body = action
        .strip_prefix("{ ")
        .and_then(|value| value.strip_suffix(" }"))
        .unwrap_or("");
    format!("{{ state.write(\"{bind}\", value: !state.bool(\"{bind}\")); {body} }}")
} else {
    action
};
let disabled = props
    .reactive
    .disabled
    .as_ref()
    .map(|path| reactive_bool(path));
let variant = props
    .reactive
    .variant
    .as_ref()
    .map(|path| reactive_text(path, "solid"));
let scheme = props
    .reactive
    .scheme
    .as_ref()
    .map(|path| reactive_text(path, "primary"));
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
    variant_content(props).to_string()
};
output.push_str(&format!("{pad}Button(action: {action}) {{\n"));
let render_contents =
    |content_indent: usize, opacity: Option<&str>, output: &mut String| {
        let content_pad = " ".repeat(content_indent);
        output.push_str(&format!("{content_pad}HStack(spacing: 8) {{\n"));
        if let Some(path) = props.swap_bind.as_deref() {
            output.push_str(&format!(
                "{content_pad}    if state.bool(\"{}\") {{\n",
                escape_swift(&context.signal_path(path))
            ));
            if let Some(icon) = props.icon_start.as_ref() {
                render_swift_button_icon(icon, &content, content_indent + 8, output);
            }
            output.push_str(&format!("{content_pad}    }} else {{\n"));
            if let Some(icon) = props.swap_icon_off.as_ref() {
                render_swift_button_icon(icon, &content, content_indent + 8, output);
            }
            output.push_str(&format!("{content_pad}    }}\n"));
        } else if let Some(icon) = props.icon_start.as_ref() {
            if let Some(path) = props.reactive.icon_start_when.as_ref() {
                output.push_str(&format!(
                    "{content_pad}    if {} {{\n",
                    icon_condition(path, props.reactive.icon_start_comparison.as_ref())
                ));
                render_swift_button_icon(icon, &content, content_indent + 8, output);
                output.push_str(&format!("{content_pad}    }}\n"));
            } else {
                render_swift_button_icon(icon, &content, content_indent + 4, output);
            }
        }
        for child in children {
            render_swift_node_in_flow(
                child,
                content_indent + 4,
                output,
                NativeFlow::Inline,
                current_font,
                default_family,
                context,
            );
        }
        if let Some(icon) = props.icon_end.as_ref() {
            if let Some(path) = props.reactive.icon_end_when.as_ref() {
                output.push_str(&format!(
                    "{content_pad}    if {} {{\n",
                    icon_condition(path, props.reactive.icon_end_comparison.as_ref())
                ));
                render_swift_button_icon(icon, &content, content_indent + 8, output);
                output.push_str(&format!("{content_pad}    }}\n"));
            } else {
                render_swift_button_icon(icon, &content, content_indent + 4, output);
            }
        }
        output.push_str(&format!("{content_pad}}}\n"));
        output.push_str(&format!("{content_pad}    .lineLimit(1)\n"));
        output.push_str(&format!(
            "{content_pad}    .fixedSize(horizontal: true, vertical: false)\n"
        ));
        output.push_str(&format!("{content_pad}    .textSelection(.disabled)\n"));
        if let Some(value) = opacity {
            output.push_str(&format!("{content_pad}    .opacity({value})\n"));
        }
    };
if let Some(loading) = loading.as_ref() {
    output.push_str(&format!("{pad}    ZStack {{\n"));
    let opacity = format!("{loading} ? 0 : 1");
    render_contents(indent + 8, Some(&opacity), output);
    output.push_str(&format!("{pad}        if {loading} {{\n"));
    if let Some(icon) = props.loading_icon.as_ref() {
        render_swift_button_spinner(icon, &content, indent + 12, output);
    }
    output.push_str(&format!("{pad}        }}\n"));
    output.push_str(&format!("{pad}    }}\n"));
} else {
    render_contents(indent + 4, None, output);
}
output.push_str(&format!("{pad}}}\n"));
let gesture_modifier = swift_gesture_modifier(&props.style);
let mut button_style = swift_style_without_gesture(&props.style);
button_style.shadow = None;
button_style.shadow_color = None;
button_style.set_animation(None);
let mut modifiers = swift_modifiers_for_style(&button_style);
if let Some(size) = props
    .reactive
    .size
    .as_ref()
    .map(|path| reactive_text(path, "md"))
{
    modifiers.push(format!(
        ".padding(.horizontal, doweButtonHorizontalPadding({size}))"
    ));
    modifiers.push(format!(
        ".padding(.vertical, doweButtonVerticalPadding({size}))"
    ));
    if props.icon_only {
        modifiers.push(format!(
            ".frame(width: doweButtonMinHeight({size}), height: doweButtonMinHeight({size}))"
        ));
    } else {
        modifiers.push(format!(".frame(height: doweButtonMinHeight({size}))"));
    }
}
if flow.is_grid_item() && props.style.sizing.w.is_none() && !props.icon_only {
    modifiers.push(".frame(maxWidth: .infinity, alignment: .center)".to_string());
}
modifiers.push(".contentShape(Rectangle())".to_string());
let container = if reactive_visual {
    format!("doweButtonContainer({variant_value}, {scheme_value})")
} else {
    variant_container(props).to_string()
};
if let Some(disabled) = disabled.as_deref() {
    modifiers.push(format!(
        ".background({container}.opacity({disabled} ? 0.5 : 1))"
    ));
} else {
    modifiers.push(format!(".background({container})"));
}
modifiers.push(format!(".foregroundStyle({content})"));
let radius = props
    .reactive
    .rounded
    .as_ref()
    .map(|path| format!("doweButtonRadius({})", reactive_text(path, "md")))
    .unwrap_or_else(|| swift_control_radius(&props.style));
modifiers.push(format!(
    ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
));
if reactive_visual {
    modifiers.push(format!(".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({content}, lineWidth: {variant_value} == \"outlined\" ? CGFloat(1) : CGFloat(0)))"));
} else if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined
{
    modifiers.push(format!(
        ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
        variant_content(props)
    ));
}
modifiers.push(".buttonStyle(.plain)".to_string());
if props.icon_only {
    modifiers.push(".accessibilityElement(children: .ignore)".to_string());
    modifiers.push(format!(
        ".accessibilityLabel(Text(\"{}\"))",
        escape_swift(props.label.as_deref().unwrap_or_default())
    ));
}
if loading.is_some() && disabled.is_some() {
    let loading_value = loading.as_deref().unwrap_or("false");
    let disabled_value = disabled.as_deref().unwrap_or("false");
    modifiers.push(format!(
        ".disabled(({loading_value}) || ({disabled_value}))"
    ));
} else if let Some(loading) = loading.as_deref() {
    modifiers.push(format!(".disabled({loading})"));
} else if let Some(disabled) = disabled.as_deref() {
    modifiers.push(format!(".disabled({disabled})"));
}
if let Some(modifier) = swift_shadow_modifier_with_radius(&props.style, &radius) {
    modifiers.push(modifier);
}
if let Some(modifier) = gesture_modifier {
    modifiers.push(modifier);
}
if let Some(animation) = props.style.animation() {
    modifiers.push(format!(
        ".modifier(DoweAnimationModifier(preset: {}))",
        swift_animation_preset(animation)
    ));
}
append_swift_modifiers(output, indent, &modifiers);
}
