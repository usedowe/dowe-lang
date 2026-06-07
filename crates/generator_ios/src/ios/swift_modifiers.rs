fn swift_modifiers_for_container_style(props: &StyleProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    if flow == NativeFlow::Block && props.sizing.w.is_none() {
        modifiers.push(".frame(maxWidth: .infinity, alignment: .leading)".to_string());
    }
    modifiers.extend(swift_modifiers_for_style(props));
    modifiers
}

fn swift_modifiers_for_layout(props: &LayoutProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    let should_fill = flow == NativeFlow::Block && props.style.sizing.w.is_none();
    if should_fill || props.justify.is_some() {
        modifiers.push(format!(
            ".frame(maxWidth: .infinity, alignment: {})",
            swift_frame_alignment(props.justify.as_ref())
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
    if flow == NativeFlow::Block && props.style.style.sizing.w.is_none() {
        modifiers.push(".frame(maxWidth: .infinity, minHeight: CGFloat(48), alignment: .center)".to_string());
    } else {
        modifiers.push(".frame(minHeight: CGFloat(48), alignment: .center)".to_string());
    }
    modifiers.extend(swift_modifiers_for_style(&props.style.style));
    modifiers.push(format!(".background({})", variant_container(&props.style)));
    modifiers.push(format!(".foregroundStyle({})", variant_content(&props.style)));
    let radius = if props.floating {
        "DoweDesign.radiusBox"
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
            if flow == NativeFlow::Block && props.style.sizing.w.is_none() {
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

fn swift_grid_columns(props: &GridProps) -> String {
    format!(
        "doweGridColumns({}, spacing: {})",
        swift_grid_column_count(props.columns.as_ref()),
        swift_grid_column_gap(props.gap.as_ref())
    )
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
    let mut modifiers = Vec::new();
    if let Some(id) = props.element.id.as_ref() {
        modifiers.push(format!(".id(\"{}\")", escape_swift(id)));
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
    if let Some(value) = props.spacing.p.as_ref() {
        modifiers.push(format!(
            ".padding({} ?? CGFloat(0))",
            swift_scale_value(value)
        ));
    }
    if let Some(value) = props.spacing.px.as_ref() {
        modifiers.push(format!(
            ".padding(.horizontal, {} ?? CGFloat(0))",
            swift_scale_value(value)
        ));
    }
    if let Some(value) = props.spacing.py.as_ref() {
        modifiers.push(format!(
            ".padding(.vertical, {} ?? CGFloat(0))",
            swift_scale_value(value)
        ));
    }
    if let Some(value) = props.spacing.pl.as_ref() {
        modifiers.push(format!(
            ".padding(.leading, {} ?? CGFloat(0))",
            swift_scale_value(value)
        ));
    }
    if let Some(value) = props.spacing.pr.as_ref() {
        modifiers.push(format!(
            ".padding(.trailing, {} ?? CGFloat(0))",
            swift_scale_value(value)
        ));
    }
    if let Some(value) = props.spacing.pt.as_ref() {
        modifiers.push(format!(
            ".padding(.top, {} ?? CGFloat(0))",
            swift_scale_value(value)
        ));
    }
    if let Some(value) = props.spacing.pb.as_ref() {
        modifiers.push(format!(
            ".padding(.bottom, {} ?? CGFloat(0))",
            swift_scale_value(value)
        ));
    }
    if let Some(value) = props.sizing.w.as_ref() {
        let expression = swift_size_value(value);
        modifiers.push(format!(
            ".frame(width: doweFixedSize({0}))",
            expression
        ));
        modifiers.push(format!(".frame(maxWidth: doweMaxSize({0}))", expression));
    }
    if let Some(value) = props.sizing.h.as_ref() {
        let expression = swift_size_value(value);
        modifiers.push(format!(
            ".frame(height: doweFixedSize({0}))",
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
            ".frame(minHeight: doweFixedSize({}))",
            swift_size_value(value)
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
        modifiers.push(format!(
            ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke(DoweDesign.onBackground, lineWidth: {} ?? CGFloat(0)))",
            swift_border_value(value)
        ));
    }
    if let Some(animation) = props.animation {
        modifiers.push(format!(
            ".modifier(DoweAnimationModifier(preset: {}))",
            swift_animation_preset(animation)
        ));
    }
    modifiers
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

fn swift_vertical_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(|value| format!("doweVerticalAlignment({})", swift_align_value(value)))
        .unwrap_or_else(|| ".center".to_string())
}

fn swift_frame_alignment(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(|value| format!("doweFrameAlignment({})", swift_justify_value(value)))
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_grid_horizontal_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    value
        .map(|value| {
            format!(
                "doweHorizontalAlignment({})",
                swift_grid_alignment_value(value)
            )
        })
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_control_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(|value| format!("{} ?? DoweDesign.radiusUi", swift_rounded_value(value)))
        .unwrap_or_else(|| "DoweDesign.radiusUi".to_string())
}

fn swift_card_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(swift_rounded_value)
        .map(|value| format!("{value} ?? DoweDesign.radiusBox"))
        .unwrap_or_else(|| "DoweDesign.radiusBox".to_string())
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
