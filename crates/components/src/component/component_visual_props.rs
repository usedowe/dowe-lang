fn conditional_icon_value(value: &str) -> (Option<String>, &str, Option<ReactiveNumberComparison>) {
    let Some(value) = value.strip_prefix("@conditional-icon:") else {
        return (None, value, None);
    };
    let mut parts = value.split(':');
    let (Some(condition), Some(name)) = (parts.next(), parts.next()) else {
        return (None, value, None);
    };
    let comparison = match (parts.next(), parts.next()) {
        (Some(operator), Some(value)) => Some(ReactiveNumberComparison {
            operator: match operator {
                "gt" => NumberComparisonOperator::GreaterThan,
                "gte" => NumberComparisonOperator::GreaterThanOrEqual,
                "lt" => NumberComparisonOperator::LessThan,
                "lte" => NumberComparisonOperator::LessThanOrEqual,
                _ => return (None, value, None),
            },
            value: value.to_string(),
        }),
        _ => None,
    };
    (Some(condition.to_string()), name, comparison)
}

fn parse_bar_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<BarProps> {
    let mut bar = BarProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "bordered" => bar.bordered = parse_static_bool(&prop.name, &prop.value)?,
            "blurred" => bar.blurred = parse_static_bool(&prop.name, &prop.value)?,
            "boxed" => bar.boxed = parse_static_bool(&prop.name, &prop.value)?,
            "floating"
                if matches!(
                    component,
                    BuiltinComponent::AppBar | BuiltinComponent::BottomBar
                ) =>
            {
                bar.floating = parse_static_bool(&prop.name, &prop.value)?
            }
            "position" if component == BuiltinComponent::AppBar => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                bar.position = BarPosition::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("position", "static, sticky or fixed")
                })?;
            }
            "hideOnScroll" if component == BuiltinComponent::AppBar => {
                bar.hide_on_scroll = parse_static_bool(&prop.name, &prop.value)?
            }
            "dockOnScroll" if component == BuiltinComponent::AppBar => {
                bar.dock_on_scroll = parse_static_bool(&prop.name, &prop.value)?
            }
            _ => style_props.push(prop.clone()),
        }
    }

    if bar.dock_on_scroll && (!bar.floating || bar.position != BarPosition::Fixed) {
        return Err(ComponentError::invalid_prop_combination(
            "`dockOnScroll:true` requires `floating:true` and `position:\"fixed\"` on `AppBar`",
        ));
    }

    let mut style = parse_variant_props(component, &style_props)?;
    if component == BuiltinComponent::Footer {
        let horizontal_padding = ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(8),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(12),
            },
        ]);
        let top_padding = ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(20),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(32),
            },
        ]);
        let bottom_padding = horizontal_padding.clone();
        style.style.spacing = style
            .style
            .spacing
            .with_horizontal_padding_default(horizontal_padding)
            .with_vertical_padding_defaults(top_padding, bottom_padding);
    }
    bar.style = style;
    Ok(bar)
}

fn parse_tabs_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<TabsProps> {
    let mut variant = TabsVariant::Pills;
    let mut color = ColorFamily::Primary;
    let mut variant_explicit = false;
    let mut color_explicit = false;
    let mut position = TabsPosition::Top;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "variant" => {
                variant = parse_tabs_variant_prop(&prop.name, &prop.value)?;
                variant_explicit = true;
            }
            "scheme" => {
                color = parse_family_prop(component, &prop.name, &prop.value)?;
                color_explicit = true;
            }
            "position" => {
                position = parse_tabs_position_prop(&prop.name, &prop.value)?;
            }
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Tabs`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop.clone()),
        }
    }

    Ok(TabsProps {
        style: parse_style_props(component, &style_props, StylePropMode::Variant)?,
        variant,
        color,
        position,
        variant_explicit,
        color_explicit,
    })
}

fn parse_stepper_props(props: &[ComponentProp]) -> ComponentResult<TabsProps> {
    let mut color = ColorFamily::Primary;
    let mut color_explicit = false;
    let mut position = TabsPosition::Top;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "scheme" => {
                color = parse_family_prop(BuiltinComponent::Stepper, &prop.name, &prop.value)?;
                color_explicit = true;
            }
            "orientation" => {
                let orientation = parse_required_string(&prop.name, &prop.value)?;
                position = match orientation.as_str() {
                    "horizontal" => TabsPosition::Top,
                    "vertical" => TabsPosition::Start,
                    _ => {
                        return Err(ComponentError::invalid_prop(
                            "orientation",
                            "horizontal or vertical",
                        ));
                    }
                };
            }
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Stepper`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop.clone()),
        }
    }

    Ok(TabsProps {
        style: parse_style_props(
            BuiltinComponent::Stepper,
            &style_props,
            StylePropMode::Variant,
        )?,
        variant: TabsVariant::Stepper,
        color,
        position,
        variant_explicit: true,
        color_explicit,
    })
}

fn normalize_button_visual_props(props: &mut VariantProps) {
    let size = *props.size.get_or_insert(ButtonSize::Md);
    if props.reactive.size.is_none() {
        apply_button_size_defaults(&mut props.style, size);
    }
}

fn normalize_icon_button_visual_props(props: &mut VariantProps) {
    let size = *props.size.get_or_insert(ButtonSize::Md);
    if props.reactive.size.is_some() {
        return;
    }

    let control_size = ResponsiveValue::scalar(SizeValue::Scale(size.icon_button_control_size()));
    props.style.sizing.w = Some(control_size.clone());
    props.style.sizing.h = Some(control_size);

    if let Some(icon) = props.icon_start.as_mut() {
        let icon_size = ResponsiveValue::scalar(SizeValue::Scale(size.icon_button_icon_size()));
        icon.props.style.sizing.w = Some(icon_size.clone());
        icon.props.style.sizing.h = Some(icon_size);
    }
}

fn normalize_card_visual_props(props: &mut VariantProps) {
    props.style.spacing = props
        .style
        .spacing
        .with_padding_default(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(8),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Lg,
                value: ScaleValue::from_half_steps(10),
            },
        ]));
}

fn apply_button_size_defaults(style: &mut StyleProps, size: ButtonSize) {
    if style.spacing.p.is_none() {
        apply_horizontal_button_padding(&mut style.spacing, size.padding_x());
        apply_vertical_button_padding(&mut style.spacing, size.padding_y());
    }

    if style.sizing.h.is_none() {
        style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Scale(size.min_height())));
    }
}

fn apply_horizontal_button_padding(spacing: &mut SpacingProps, value: ScaleValue) {
    if spacing.px.is_some() {
        return;
    }

    match (spacing.pl.is_some(), spacing.pr.is_some()) {
        (false, false) => spacing.px = Some(ResponsiveValue::scalar(value)),
        (false, true) => spacing.pl = Some(ResponsiveValue::scalar(value)),
        (true, false) => spacing.pr = Some(ResponsiveValue::scalar(value)),
        (true, true) => {}
    }
}

fn apply_vertical_button_padding(spacing: &mut SpacingProps, value: ScaleValue) {
    if spacing.py.is_some() {
        return;
    }

    match (spacing.pt.is_some(), spacing.pb.is_some()) {
        (false, false) => spacing.py = Some(ResponsiveValue::scalar(value)),
        (false, true) => spacing.pt = Some(ResponsiveValue::scalar(value)),
        (true, false) => spacing.pb = Some(ResponsiveValue::scalar(value)),
        (true, true) => {}
    }
}

fn parse_text_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<TextProps> {
    let mut text = TextProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        let value = prop.value.binding_fallback().unwrap_or_else(|| prop.value.clone());
        let binding = prop.value.binding().cloned();
        match prop.name.as_str() {
            "align" if matches!(component, BuiltinComponent::Text | BuiltinComponent::Title) => {
                text.align = Some(parse_text_align_prop(&prop.name, &prop.value)?)
            }
            "size" => {
                text.size = Some(parse_text_size_prop(&prop.name, &value)?);
                text.size_binding = binding;
            }
            "weight" => {
                text.weight = Some(parse_text_weight_prop(&prop.name, &value)?);
                text.weight_binding = binding;
            }
            "spacing" => {
                text.letter_spacing = Some(parse_text_spacing_prop(&prop.name, &value)?);
                text.letter_spacing_binding = binding;
            }
            "i18n" => text.i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
            "as" if component == BuiltinComponent::Title => {
                let value = match &prop.value {
                    PropValue::String(value) if matches!(value.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") => value.clone(),
                    _ => return Err(ComponentError::invalid_prop(&prop.name, "h1, h2, h3, h4, h5 or h6")),
                };
                text.as_tag = Some(value);
            }
            "title" if component == BuiltinComponent::RichText => {
                text.title = parse_static_bool(&prop.name, &prop.value)?
            }
            _ => style_props.push(prop.clone()),
        }
    }

    text.style = parse_style_props(component, &style_props, StylePropMode::Text)?;

    Ok(text)
}

