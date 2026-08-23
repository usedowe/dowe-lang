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
            ".frame(maxWidth: .infinity, minHeight: CGFloat(48), alignment: .center)".to_string(),
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

fn swift_modifiers_for_style(props: &StyleProps) -> Vec<String> {
    let mut modifiers = swift_modifiers_for_style_with_width_alignment(props, None);
    for binding in props.bindings() {
        let path = escape_swift(&binding.binding.path);
        let value = format!("state.text(\"{}\")", path);
        let modifier = match binding.property {
            dowe_components::StyleBindingProperty::TextColor => format!(".foregroundStyle(doweDynamicColor({value}))"),
            dowe_components::StyleBindingProperty::BackgroundColor => format!(".background(doweDynamicColor({value}))"),
            dowe_components::StyleBindingProperty::Padding => format!(".padding(CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::PaddingInline => format!(".padding(.horizontal, CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::PaddingBlock => format!(".padding(.vertical, CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::PaddingLeft => format!(".padding(.leading, CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::PaddingRight => format!(".padding(.trailing, CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::PaddingTop => format!(".padding(.top, CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::PaddingBottom => format!(".padding(.bottom, CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::Width => format!(".frame(width: CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::Height => format!(".frame(height: CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::MinWidth => format!(".frame(minWidth: CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::MinHeight => format!(".frame(minHeight: CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::MaxWidth => format!(".frame(maxWidth: CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::MaxHeight => format!(".frame(maxHeight: CGFloat(Double({value}) ?? 0) / 8.0)"),
            dowe_components::StyleBindingProperty::BorderWidth => format!(".overlay(RoundedRectangle(cornerRadius: 0).stroke(Color.primary, lineWidth: CGFloat(Double({value}) ?? 0)) )"),
            dowe_components::StyleBindingProperty::BorderRadius => format!(".clipShape(RoundedRectangle(cornerRadius: CGFloat(Double({value}) ?? 0)))"),
        };
        modifiers.push(modifier);
    }
    modifiers
}

fn swift_modifiers_for_svg(props: &SvgProps) -> Vec<String> {
    let mut modifiers = swift_modifiers_for_style(&props.style);
    let has_full_dimension = props.style.sizing.w.as_ref().is_some_and(|value| {
        value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
    }) || props.style.sizing.h.as_ref().is_some_and(|value| {
        value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
    });
    let has_single_dimension = props.style.sizing.w.is_some() != props.style.sizing.h.is_some();
    if props.data.is_none()
        && props.icon_name.is_none()
        && props.motion.is_none()
        && (has_single_dimension || has_full_dimension)
        && let Some(ratio) = props.view_box.aspect_ratio()
    {
        modifiers.push(format!(
            ".aspectRatio(CGFloat({ratio:.6}), contentMode: .fit)"
        ));
    }
    modifiers
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
            swift_padding_edge(
                props.spacing.pt.as_ref(),
                props.spacing.py.as_ref(),
                props.spacing.p.as_ref()
            ),
            swift_padding_edge(
                props.spacing.pl.as_ref(),
                props.spacing.px.as_ref(),
                props.spacing.p.as_ref()
            ),
            swift_padding_edge(
                props.spacing.pb.as_ref(),
                props.spacing.py.as_ref(),
                props.spacing.p.as_ref()
            ),
            swift_padding_edge(
                props.spacing.pr.as_ref(),
                props.spacing.px.as_ref(),
                props.spacing.p.as_ref()
            )
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
        let expression = swift_size_value(value);
        modifiers.push(format!(".frame(minWidth: doweFixedSize({expression}))"));
    }
    if let Some(value) = props.sizing.min_h.as_ref() {
        modifiers.push(format!(
            ".frame(minHeight: doweFixedSize({}, viewportHeight: viewportHeight))",
            swift_size_value(value)
        ));
        if value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
        {
            modifiers.push(format!(
                ".frame(maxHeight: doweMaxSize({}))",
                swift_size_value(value)
            ));
        }
    }
    if let Some(value) = props.sizing.max_w.as_ref() {
        modifiers.push(format!(
            ".frame(maxWidth: doweFixedSize({}))",
            swift_size_value(value)
        ));
    }
    if let Some(value) = props.sizing.max_h.as_ref() {
        let expression = swift_size_value(value);
        modifiers.push(format!(
            ".frame(maxHeight: doweFixedSize({expression}, viewportHeight: viewportHeight))"
        ));
        modifiers.push(format!(".doweMaxHeight({expression})"));
    }
    let has_percentage_width = props
        .sizing
        .w
        .iter()
        .chain(props.sizing.min_w.iter())
        .flat_map(|value| &value.entries)
        .any(|entry| matches!(entry.value, SizeValue::Percent(_)));
    if has_percentage_width {
        let width = props
            .sizing
            .w
            .as_ref()
            .map(swift_size_value)
            .unwrap_or_else(|| "nil".to_string());
        let min_width = props
            .sizing
            .min_w
            .as_ref()
            .map(swift_size_value)
            .unwrap_or_else(|| "nil".to_string());
        modifiers.push(format!(
            ".dowePercentageWidth(width: {width}, minWidth: {min_width})"
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
            ".foregroundStyle({} ?? DoweDesign.backgroundText)",
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
            .unwrap_or("DoweDesign.backgroundText");
        modifiers.push(format!(
            ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({border_color}, lineWidth: {} ?? CGFloat(0)))",
            swift_border_value(value)
        ));
    }
    if let Some(modifier) = swift_shadow_modifier(props) {
        modifiers.push(modifier);
    }
    let motion = props.motion();
    if let Some(value) = motion.rotate.as_ref() {
        modifiers.push(format!(
            ".rotationEffect(.degrees({} ?? Double(0)))",
            swift_responsive_value(value, |value| format!("Double({})", value.degrees()))
        ));
    }
    if let Some(value) = motion.scale.as_ref() {
        modifiers.push(format!(
            ".scaleEffect(CGFloat({} ?? Double(1)))",
            swift_responsive_value(value, |value| format!("Double({})", value.factor()))
        ));
    }
    let translate_x = motion
        .translate_x
        .as_ref()
        .map(|value| {
            format!(
                "CGFloat({} ?? Double(0))",
                swift_responsive_value(value, |value| format!("Double({})", value.native_units()))
            )
        })
        .unwrap_or_else(|| "CGFloat(0)".to_string());
    let translate_y = motion
        .translate_y
        .as_ref()
        .map(|value| {
            format!(
                "CGFloat({} ?? Double(0))",
                swift_responsive_value(value, |value| format!("Double({})", value.native_units()))
            )
        })
        .unwrap_or_else(|| "CGFloat(0)".to_string());
    if motion.translate_x.is_some() || motion.translate_y.is_some() {
        modifiers.push(format!(".offset(x: {translate_x}, y: {translate_y})"));
    }
    if let Some(modifier) = swift_gesture_modifier(props) {
        modifiers.push(modifier);
    }
    if let Some(animation) = props.animation() {
        modifiers.push(format!(
            ".modifier(DoweAnimationModifier(preset: {}))",
            swift_animation_preset(animation)
        ));
    }
    modifiers
}

fn swift_gesture_modifier(props: &StyleProps) -> Option<String> {
    let motion = props.motion();
    let gesture = motion.gesture?;
    if gesture == ViewGesture::None {
        return None;
    }
    let transition = motion.transition.unwrap_or(ViewTransition::Smooth);
    Some(format!(
        ".modifier(DoweGestureModifier(preset: .{}, transition: .{}))",
        gesture.as_str(),
        transition.as_str()
    ))
}

fn swift_style_without_gesture(props: &StyleProps) -> StyleProps {
    let mut style = props.clone();
    if style.motion().gesture.is_some() {
        style.motion_mut().gesture = None;
    }
    style
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

fn swift_shadow_modifier_with_radius(props: &StyleProps, corner_radius: &str) -> Option<String> {
    swift_shadow_spec(props).map(|shadow| {
        format!(".background(DoweShadowSurface(shadow: {shadow}, cornerRadius: {corner_radius}))")
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
                .unwrap_or("DoweDesign.backgroundText");
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
