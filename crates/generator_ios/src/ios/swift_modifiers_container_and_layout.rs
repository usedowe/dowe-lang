fn swift_modifiers_for_container_style(props: &StyleProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    if flow.is_block() && props.sizing.w.is_none() {
        modifiers.push(format!(
            ".frame(maxWidth: .infinity, alignment: {})",
            swift_box_horizontal_alignment(props.center_x.as_ref())
        ));
    }
    if flow == NativeFlow::GridItem {
        if props.sizing.h.is_none() {
            modifiers.push(".frame(maxHeight: .infinity, alignment: .top)".to_string());
        }
    }
    let width_alignment = props
        .center_x
        .as_ref()
        .map(|value| swift_box_horizontal_alignment(Some(value)))
        .unwrap_or_else(|| ".leading".to_string());
    modifiers.extend(swift_modifiers_for_style_with_width_alignment(
        props,
        Some(&width_alignment),
    ));
    modifiers.extend(swift_modifiers_for_style_bindings(props));
    if let Some(center_y) = props.center_y.as_ref() {
        modifiers.push(format!(
            ".frame(maxHeight: .infinity, alignment: {})",
            swift_vertical_center_alignment(center_y)
        ));
    }
    append_swift_flex_item_modifiers(&mut modifiers, props.flex.as_ref(), flow);
    if flow == NativeFlow::GridItem {
        modifiers.push(format!(
            ".doweGridItemStretches({})",
            swift_grid_item_stretches_height(props.sizing.h.as_ref())
        ));
    }
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
    let vertical = if props.bottom.is_some() {
        "bottom"
    } else {
        "top"
    };
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
    content.sizing = props.sizing.clone();
    let section_spacing = content.spacing.clone();
    content.sizing.h = content
        .sizing
        .h
        .as_ref()
        .map(|value| swift_section_bounded_size(value, &section_spacing));
    content.sizing.min_h = content
        .sizing
        .min_h
        .as_ref()
        .map(|value| swift_section_bounded_size(value, &section_spacing));
    let mut modifiers = swift_modifiers_for_style(&content);
    if props.sizing.h.is_some() || props.sizing.min_h.is_some() {
        modifiers.push(".frame(maxHeight: .infinity)".to_string());
    }
    if props.center_x.is_some() && !props.boxed {
        modifiers.push(".frame(maxWidth: .infinity, alignment: .leading)".to_string());
    }
    if let Some(center_y) = props.center_y.as_ref() {
        modifiers.push(format!(
            ".frame(maxHeight: .infinity, alignment: {})",
            swift_vertical_center_alignment(center_y)
        ));
    }
    if props.boxed {
        modifiers.push(".frame(maxWidth: CGFloat(1536), alignment: .leading)".to_string());
        modifiers.push(".frame(maxWidth: .infinity, alignment: .center)".to_string());
    }
    modifiers
}

fn swift_section_bounded_size(
    value: &ResponsiveValue<SizeValue>,
    spacing: &dowe_components::SpacingProps,
) -> ResponsiveValue<SizeValue> {
    ResponsiveValue::ordered(
        value
            .entries
            .iter()
            .map(|entry| {
                let top = swift_section_spacing_edge(spacing, entry.breakpoint, true);
                let bottom = swift_section_spacing_edge(spacing, entry.breakpoint, false);
                let value = match entry.value {
                    SizeValue::ViewportMinus(inset) => {
                        SizeValue::ViewportMinus(ScaleValue::from_half_steps(
                            inset.0.saturating_sub(top.saturating_add(bottom)),
                        ))
                    }
                    value => value,
                };
                dowe_components::ResponsiveEntry {
                    breakpoint: entry.breakpoint,
                    value,
                }
            })
            .collect(),
    )
}

fn swift_section_spacing_edge(
    spacing: &dowe_components::SpacingProps,
    breakpoint: Breakpoint,
    top: bool,
) -> u16 {
    let value = if let Some(all) = spacing.p.as_ref() {
        swift_responsive_scale_at(all, breakpoint)
    } else if top {
        spacing
            .pt
            .as_ref()
            .or(spacing.py.as_ref())
            .and_then(|value| swift_responsive_scale_at(value, breakpoint))
    } else {
        spacing
            .pb
            .as_ref()
            .or(spacing.py.as_ref())
            .and_then(|value| swift_responsive_scale_at(value, breakpoint))
    };
    value.map(|value| value.0).unwrap_or_default()
}

fn swift_responsive_scale_at(
    value: &ResponsiveValue<ScaleValue>,
    breakpoint: Breakpoint,
) -> Option<ScaleValue> {
    value
        .entries
        .iter()
        .rev()
        .find(|entry| entry.breakpoint.min_width() <= breakpoint.min_width())
        .map(|entry| entry.value)
}

fn swift_box_horizontal_alignment(value: Option<&ResponsiveValue<bool>>) -> String {
    value
        .map(|value| {
            format!(
                "({} ?? false) ? .center : .leading",
                swift_bool_value(value)
            )
        })
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_section_horizontal_alignment(value: Option<&ResponsiveValue<bool>>) -> String {
    value
        .map(|value| {
            format!(
                "({} ?? false) ? .center : .leading",
                swift_bool_value(value)
            )
        })
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_vertical_center_alignment(value: &ResponsiveValue<bool>) -> String {
    format!(
        "({} ?? false) ? .center : .top",
        swift_bool_value(value)
    )
}

fn swift_section_vertical_spacing(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| {
            format!(
                "{} ?? CGFloat(0)",
                swift_responsive_value(value, swift_gap_value)
            )
        })
        .unwrap_or_else(|| "0".to_string())
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
    append_swift_flex_item_modifiers(&mut modifiers, props.style.flex.as_ref(), flow);
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
    append_swift_flex_item_modifiers(&mut modifiers, props.style.flex.as_ref(), flow);
    modifiers
}

fn append_swift_flex_item_modifiers(
    modifiers: &mut Vec<String>,
    value: Option<&ResponsiveValue<FlexItem>>,
    flow: NativeFlow,
) {
    if !flow.is_flex_item() {
        return;
    }
    let Some(value) = value else {
        return;
    };
    let value = swift_responsive_value(value, |value| match value {
        FlexItem::Initial => "DoweFlexItem.initial".to_string(),
        FlexItem::Auto => "DoweFlexItem.auto".to_string(),
        FlexItem::None => "DoweFlexItem.none".to_string(),
        FlexItem::Fill => "DoweFlexItem.fill".to_string(),
    });
    modifiers.push(format!(".doweFlexItem({value} ?? .initial, horizontal: {})", flow.is_inline()));
}

fn swift_modifiers_for_grid(props: &GridProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = swift_modifiers_for_container_style(&props.style, flow);
    if flow == NativeFlow::Block && swift_grid_has_full_height(props) {
        modifiers.push(format!(
            ".frame(minHeight: CGFloat(0), maxHeight: ({}) ? .infinity : nil, alignment: .topLeading)",
            swift_grid_size_fills_height(props)
        ));
    }
    modifiers
}

fn swift_grid_item_stretches_height(value: Option<&ResponsiveValue<SizeValue>>) -> String {
    value
        .map(|value| {
            format!(
                "({} ?? true)",
                swift_responsive_value(value, |value| {
                    matches!(value, SizeValue::Auto | SizeValue::Full).to_string()
                })
            )
        })
        .unwrap_or_else(|| "true".to_string())
}

fn swift_grid_has_full_height(props: &GridProps) -> bool {
    props.style.sizing.h.as_ref().is_some_and(|value| {
        value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
    })
}

fn swift_grid_size_fills_height(props: &GridProps) -> String {
    props
        .style
        .sizing
        .h
        .as_ref()
        .map(|value| {
            format!(
                "({} ?? false)",
                swift_responsive_value(value, |value| matches!(value, SizeValue::Full).to_string())
            )
        })
        .unwrap_or_else(|| "false".to_string())
}

fn swift_grid_fills_height(props: &GridProps, flow: NativeFlow) -> String {
    let size = swift_grid_size_fills_height(props);
    if flow == NativeFlow::GridItem {
        return size;
    }
    let Some(flex_value) = props.style.flex.as_ref() else {
        return size;
    };
    let flex = format!(
        "({} ?? false)",
        swift_responsive_value(flex_value, |value| matches!(value, FlexItem::Fill).to_string())
    );
    format!("({size}) || ({flex})")
}

fn swift_modifiers_for_bar(props: &BarProps, flow: NativeFlow) -> Vec<String> {
    let mut modifiers = Vec::new();
    if flow.is_block() && props.style.style.sizing.w.is_none() {
        modifiers.push(
            ".frame(minWidth: CGFloat(0), maxWidth: .infinity, minHeight: CGFloat(48), alignment: .center)".to_string(),
        );
    } else {
        modifiers.push(".frame(minHeight: CGFloat(48), alignment: .center)".to_string());
    }
    modifiers.extend(swift_modifiers_for_style(&props.style.style));
    modifiers.push(format!(".background({})", variant_container(&props.style)));
    modifiers.push(format!(
        ".foregroundStyle({})",
        variant_content(&props.style)
    ));
    if props.position != BarPosition::Static {
        modifiers.push(".zIndex(1)".to_string());
    }
    if props.dock_on_scroll {
        modifiers.push(format!(
            ".modifier(DoweDockingAppBarModifier(backgroundColor: {}, contentColor: {}))",
            variant_container(&props.style),
            variant_content(&props.style)
        ));
        return modifiers;
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

fn swift_grid_tracks(value: Option<&ResponsiveValue<GridTracks>>) -> String {
    value
        .map(|value| {
            format!(
                "{} ?? [CGFloat(1)]",
                swift_responsive_value(value, |value| match value {
                    GridTracks::Count(count) =>
                        format!("[{}]", vec!["CGFloat(1)"; *count as usize].join(", ")),
                    GridTracks::Fractions(weights) => format!(
                        "[{}]",
                        weights
                            .iter()
                            .map(|weight| format!("CGFloat({weight})"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    GridTracks::Auto => "[CGFloat(1)]".to_string(),
                })
            )
        })
        .unwrap_or_else(|| "[CGFloat(1)]".to_string())
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

