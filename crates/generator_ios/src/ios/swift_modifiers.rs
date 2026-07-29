fn swift_modifiers_for_container_style(props: &StyleProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    if flow.is_block() && props.sizing.w.is_none() {
        modifiers.push(".frame(maxWidth: .infinity, alignment: .leading)".to_string());
    }
    modifiers.extend(swift_modifiers_for_style_with_width_alignment(
        props,
        Some(".leading"),
    ));
    modifiers
}

fn swift_modifiers_for_positioned_box(props: &PositionProps) -> Vec<String> {
    let mut modifiers = Vec::new();
    if let Some(value) = props.top.as_ref() {
        modifiers.push(format!(".padding(.top, {})", swift_scale_value(value)));
    }
    if let Some(value) = props.right.as_ref() {
        modifiers.push(format!(".padding(.trailing, {})", swift_scale_value(value)));
    }
    if let Some(value) = props.bottom.as_ref() {
        modifiers.push(format!(".padding(.bottom, {})", swift_scale_value(value)));
    }
    if let Some(value) = props.left.as_ref() {
        modifiers.push(format!(".padding(.leading, {})", swift_scale_value(value)));
    }
    let vertical = if props.bottom.is_some() { "bottom" } else { "top" };
    let horizontal = if props.right.is_some() {
        "Trailing"
    } else {
        "Leading"
    };
    modifiers.push(format!(
        ".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .{vertical}{horizontal})"
    ));
    modifiers
}

fn swift_modifiers_for_section_container(props: &StyleProps, flow: NativeFlow) -> Vec<String> {
    let mut outer = props.clone();
    outer.spacing = Default::default();
    swift_modifiers_for_container_style(&outer, flow)
}

fn swift_modifiers_for_section_content(props: &StyleProps) -> Vec<String> {
    let mut content = StyleProps::default();
    content.spacing = dowe_components::section_content_spacing(&props.spacing);
    let mut modifiers = swift_modifiers_for_style(&content);
    if props.boxed {
        modifiers.push(".frame(maxWidth: CGFloat(1536), alignment: .leading)".to_string());
        modifiers.push(".frame(maxWidth: .infinity, alignment: .center)".to_string());
    }
    modifiers
}

fn swift_modifiers_for_layout(props: &LayoutProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    let should_fill = flow.is_block() && props.style.sizing.w.is_none();
    if should_fill || props.justify.is_some() {
        modifiers.push(format!(
            ".frame(maxWidth: .infinity, alignment: {})",
            swift_frame_alignment(props.justify.as_ref())
        ));
    }
    modifiers.extend(swift_modifiers_for_style(&props.style));
    modifiers
}

fn swift_modifiers_for_column_layout(props: &LayoutProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    let should_fill = flow.is_block() && props.style.sizing.w.is_none();
    if should_fill || props.align.is_some() {
        modifiers.push(format!(
            ".frame(maxWidth: .infinity, alignment: {})",
            swift_column_frame_alignment(props.align.as_ref())
        ));
    }
    modifiers.extend(swift_modifiers_for_style(&props.style));
    modifiers
}

fn swift_modifiers_for_grid(props: &GridProps, flow: NativeFlow) -> Vec<String> {
    swift_modifiers_for_container_style(&props.style, flow)
}

fn swift_modifiers_for_bar(props: &BarProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    if flow.is_block() && props.style.style.sizing.w.is_none() {
        modifiers.push(".frame(maxWidth: .infinity, minHeight: CGFloat(48), alignment: .center)".to_string());
    } else {
        modifiers.push(".frame(minHeight: CGFloat(48), alignment: .center)".to_string());
    }
    modifiers.extend(swift_modifiers_for_style(&props.style.style));
    modifiers.push(format!(".background({})", variant_container(&props.style)));
    modifiers.push(format!(".foregroundStyle({})", variant_content(&props.style)));
    if props.position != BarPosition::Static {
        modifiers.push(".zIndex(1)".to_string());
    }
    let radius = if props.floating {
        "DoweDesign.radius"
    } else {
        "CGFloat(0)"
    };
    if props.floating {
        modifiers.push(format!(
            ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
        ));
    }
    if props.floating {
        modifiers.push(format!(
            ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
    }
    if props.floating {
        modifiers.push(".padding(.horizontal, CGFloat(16))".to_string());
        modifiers.push(".padding(.vertical, CGFloat(8))".to_string());
    }
    modifiers
}

fn swift_modifiers_for_divider(props: &DividerProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = vec![format!(".fill({})", color_ref(family_color(props.color)))];
    match props.orientation {
        DividerOrientation::Horizontal => {
            if flow.is_block() && props.style.sizing.w.is_none() {
                modifiers.push(".frame(maxWidth: .infinity)".to_string());
            }
            if props.style.sizing.h.is_none() {
                modifiers.push(".frame(height: CGFloat(1))".to_string());
            }
        }
        DividerOrientation::Vertical => {
            if props.style.sizing.w.is_none() {
                modifiers.push(".frame(width: CGFloat(1))".to_string());
            }
            if props.style.sizing.h.is_none() {
                modifiers.push(".frame(maxHeight: .infinity)".to_string());
            }
        }
    }
    modifiers.extend(swift_modifiers_for_style(&props.style));
    modifiers
}

fn swift_grid_column_count(value: Option<&ResponsiveValue<GridTracks>>) -> String {
    value
        .map(|value| {
            format!(
                "{} ?? 1",
                swift_responsive_value(value, |value| value.count().unwrap_or(1).to_string())
            )
        })
        .unwrap_or_else(|| "1".to_string())
}

fn swift_grid_row_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| {
            swift_responsive_value(value, |value| match value {
                GapValue::Single(value) | GapValue::Pair(value, _) => swift_gap_size(value),
            })
        })
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_grid_column_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| {
            swift_responsive_value(value, |value| match value {
                GapValue::Single(value) | GapValue::Pair(_, value) => swift_gap_size(value),
            })
        })
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_navigation_action(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => format!(
            r#"{{ navigate("{}", "{}", {}) }}"#,
            operation.as_str(),
            escape_swift(path),
            fragment
                .as_ref()
                .map(|value| format!(r#""{}""#, escape_swift(value)))
                .unwrap_or_else(|| "nil".to_string())
        ),
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => {
            format!(
                r#"{{ navigate("{}", "", "{}") }}"#,
                operation.as_str(),
                escape_swift(fragment)
            )
        }
        Some(NavigationAction::External {
            url,
            native_external_mode,
            ..
        }) => format!(
            r#"{{ openExternal("{}", "{}") }}"#,
            native_external_mode.as_str(),
            escape_swift(url)
        ),
        Some(NavigationAction::Back) => "{ goBack() }".to_string(),
        None => "{}".to_string(),
    }
}

fn swift_modifiers_for_style(props: &StyleProps) -> Vec<String> {
    swift_modifiers_for_style_with_width_alignment(props, None)
}

fn swift_modifiers_for_style_with_width_alignment(
    props: &StyleProps,
    width_alignment: Option<&str>,
) -> Vec<String> {
    let mut modifiers = Vec::new();
    if let Some(id) = props.element.id.as_ref() {
        modifiers.push(format!(".id(\"{}\")", escape_swift(id)));
    }
    if props.spacing.p.is_some()
        || props.spacing.px.is_some()
        || props.spacing.py.is_some()
        || props.spacing.pl.is_some()
        || props.spacing.pr.is_some()
        || props.spacing.pt.is_some()
        || props.spacing.pb.is_some()
    {
        modifiers.push(format!(
            ".padding(EdgeInsets(top: {}, leading: {}, bottom: {}, trailing: {}))",
            swift_padding_edge(props.spacing.pt.as_ref(), props.spacing.py.as_ref(), props.spacing.p.as_ref()),
            swift_padding_edge(props.spacing.pl.as_ref(), props.spacing.px.as_ref(), props.spacing.p.as_ref()),
            swift_padding_edge(props.spacing.pb.as_ref(), props.spacing.py.as_ref(), props.spacing.p.as_ref()),
            swift_padding_edge(props.spacing.pr.as_ref(), props.spacing.px.as_ref(), props.spacing.p.as_ref())
        ));
    }
    if let Some(value) = props.sizing.w.as_ref() {
        let expression = swift_size_value(value);
        let alignment = width_alignment
            .map(|value| format!(", alignment: {value}"))
            .unwrap_or_default();
        modifiers.push(format!(
            ".frame(width: doweFixedSize({expression}){alignment})"
        ));
        modifiers.push(format!(
            ".frame(maxWidth: doweMaxSize({expression}){alignment})"
        ));
    }
    if let Some(value) = props.sizing.h.as_ref() {
        let expression = swift_size_value(value);
        modifiers.push(format!(
            ".frame(height: doweFixedSize({0}, viewportHeight: viewportHeight))",
            expression
        ));
        modifiers.push(format!(".frame(maxHeight: doweMaxSize({0}))", expression));
    }
    if let Some(value) = props.sizing.min_w.as_ref() {
        modifiers.push(format!(
            ".frame(minWidth: doweFixedSize({}))",
            swift_size_value(value)
        ));
    }
    if let Some(value) = props.sizing.min_h.as_ref() {
        modifiers.push(format!(
            ".frame(minHeight: doweFixedSize({}, viewportHeight: viewportHeight))",
            swift_size_value(value)
        ));
    }
    if let Some(value) = props.bg.as_ref() {
        modifiers.push(format!(
            ".background({} ?? Color.clear)",
            swift_color_value(value)
        ));
    }
    if let Some(value) = props.text.as_ref() {
        modifiers.push(format!(
            ".foregroundStyle({} ?? DoweDesign.onBackground)",
            swift_color_value(value)
        ));
    }
    if let Some(value) = props.rounded.as_ref() {
        modifiers.push(format!(
            ".clipShape(RoundedRectangle(cornerRadius: {} ?? DoweDesign.radius))",
            swift_rounded_value(value)
        ));
    }
    if let Some(value) = props.border.as_ref() {
        let radius = props
            .rounded
            .as_ref()
            .map(|value| format!("{} ?? DoweDesign.radius", swift_rounded_value(value)))
            .unwrap_or_else(|| "DoweDesign.radius".to_string());
        let border_color = props
            .border_color
            .map(family_color)
            .map(color_ref)
            .unwrap_or("DoweDesign.onBackground");
        modifiers.push(format!(
            ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({border_color}, lineWidth: {} ?? CGFloat(0)))",
            swift_border_value(value)
        ));
    }
    if let Some(modifier) = swift_shadow_modifier(props) {
        modifiers.push(modifier);
    }
    if let Some(animation) = props.animation {
        modifiers.push(format!(
            ".modifier(DoweAnimationModifier(preset: {}))",
            swift_animation_preset(animation)
        ));
    }
    modifiers
}

fn swift_shadow_modifier(props: &StyleProps) -> Option<String> {
    props.shadow.as_ref().map(|value| {
        format!(
            ".shadow(color: {}, radius: {} ?? CGFloat(0), x: CGFloat(0), y: {} ?? CGFloat(0))",
            swift_shadow_color(props, value),
            swift_shadow_value(value),
            swift_shadow_offset_value(value)
        )
    })
}

fn swift_shadow_modifier_with_radius(
    props: &StyleProps,
    corner_radius: &str,
) -> Option<String> {
    swift_shadow_spec(props).map(|shadow| {
        format!(
            ".background(DoweShadowSurface(shadow: {shadow}, cornerRadius: {corner_radius}))"
        )
    })
}

fn swift_shadow_spec(props: &StyleProps) -> Option<String> {
    props.shadow.as_ref().map(|value| {
        format!(
            "DoweShadowSpec(color: {}, blurRadius: {} ?? CGFloat(0), offsetY: {} ?? CGFloat(0))",
            swift_shadow_color(props, value),
            swift_shadow_value(value),
            swift_shadow_offset_value(value)
        )
    })
}

fn swift_style_border(
    props: &StyleProps,
    fallback_color: &str,
    fallback_width: &str,
) -> (String, String) {
    props.border.as_ref().map_or_else(
        || (fallback_color.to_string(), fallback_width.to_string()),
        |width| {
            let width = swift_border_value(width);
            let color = props
                .border_color
                .map(family_color)
                .map(color_ref)
                .unwrap_or("DoweDesign.onBackground");
            (
                format!("({width}) == nil ? {fallback_color} : Optional({color})"),
                format!("{width} ?? {fallback_width}"),
            )
        },
    )
}

fn swift_shadow_color(props: &StyleProps, value: &ResponsiveValue<ShadowSize>) -> String {
    props
        .shadow_color
        .map(family_color)
        .map(color_ref)
        .map(|color| format!("{color}.opacity(0.28)"))
        .unwrap_or_else(|| {
            format!(
                "Color.black.opacity({} ?? Double(0))",
                swift_shadow_opacity_value(value)
            )
        })
}

fn swift_padding_edge(
    side: Option<&ResponsiveValue<ScaleValue>>,
    axis: Option<&ResponsiveValue<ScaleValue>>,
    all: Option<&ResponsiveValue<ScaleValue>>,
) -> String {
    let values = [side, axis, all]
        .into_iter()
        .flatten()
        .map(swift_scale_value)
        .collect::<Vec<_>>();
    if values.is_empty() {
        "CGFloat(0)".to_string()
    } else {
        format!("{} ?? CGFloat(0)", values.join(" ?? "))
    }
}

fn append_swift_modifiers(output: &mut String, indent: usize, modifiers: &[String]) {
    let pad = " ".repeat(indent + 4);
    for modifier in modifiers {
        output.push_str(&format!("{pad}{modifier}\n"));
    }
}

fn swift_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| swift_responsive_value(value, |value| swift_gap_value(value)))
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_flex_justify(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(swift_justify_value)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_flex_align(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(swift_align_value)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_vertical_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(|value| format!("doweVerticalAlignment({})", swift_align_value(value)))
        .unwrap_or_else(|| ".center".to_string())
}

fn swift_horizontal_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(|value| format!("doweHorizontalAlignment({})", swift_align_value(value)))
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_frame_alignment(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(|value| format!("doweFrameAlignment({})", swift_justify_value(value)))
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_column_frame_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(|value| format!("doweColumnFrameAlignment({})", swift_align_value(value)))
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_grid_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    value
        .map(swift_grid_alignment_value)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_control_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(|value| format!("{} ?? DoweDesign.radius", swift_rounded_value(value)))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn swift_card_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(swift_rounded_value)
        .map(|value| format!("{value} ?? DoweDesign.radius"))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn swift_drawer_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(swift_rounded_value)
        .map(|value| format!("{value} ?? CGFloat(0)"))
        .unwrap_or_else(|| "CGFloat(0)".to_string())
}

fn swift_animation_preset(value: ViewAnimation) -> &'static str {
    match value {
        ViewAnimation::None => ".none",
        ViewAnimation::FadeIn => ".fadeIn",
        ViewAnimation::SlideUp => ".slideUp",
        ViewAnimation::SlideDown => ".slideDown",
        ViewAnimation::SlideLeft => ".slideLeft",
        ViewAnimation::SlideRight => ".slideRight",
        ViewAnimation::ScaleIn => ".scaleIn",
    }
}
