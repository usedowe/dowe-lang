fn parse_layout_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<LayoutProps> {
    let mut layout = LayoutProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "direction" => layout.direction = parse_flex_direction_prop(&prop.name, &prop.value)?,
            "wrap" => layout.wrap = parse_static_bool(&prop.name, &prop.value)?,
            "justify" => layout.justify = Some(parse_justify_prop(&prop.name, &prop.value)?),
            "align" => layout.align = Some(parse_align_prop(&prop.name, &prop.value)?),
            "gap" => layout.gap = Some(parse_gap_prop(&prop.name, &prop.value, false)?),
            _ => style_props.push(prop.clone()),
        }
    }

    layout.style = parse_style_props(component, &style_props, StylePropMode::Layout)?;
    Ok(layout)
}

fn parse_grid_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<GridProps> {
    let mut grid = GridProps {
        columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
        rows: Some(ResponsiveValue::scalar(GridTracks::Auto)),
        justify: Some(ResponsiveValue::scalar(GridAlignment::Stretch)),
        align: Some(ResponsiveValue::scalar(GridAlignment::Stretch)),
        gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
            ScaleValue::from_half_steps(0),
        )))),
        ..GridProps::default()
    };
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "columns" => {
                grid.columns = Some(parse_grid_tracks_prop(
                    &prop.name,
                    &prop.value,
                    false,
                    Some(12),
                )?)
            }
            "rows" => {
                grid.rows = Some(parse_grid_tracks_prop(&prop.name, &prop.value, true, None)?)
            }
            "justify" => grid.justify = Some(parse_grid_alignment_prop(&prop.name, &prop.value)?),
            "align" => grid.align = Some(parse_grid_alignment_prop(&prop.name, &prop.value)?),
            "gap" => grid.gap = Some(parse_gap_prop(&prop.name, &prop.value, true)?),
            _ => style_props.push(prop.clone()),
        }
    }

    grid.style = parse_style_props(component, &style_props, StylePropMode::Grid)?;
    if grid.style.sizing.w.is_none() {
        grid.style.sizing.w = Some(ResponsiveValue::scalar(SizeValue::Full));
    }
    Ok(grid)
}

fn parse_variant_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<VariantProps> {
    let mut variant_props = VariantProps::default();
    let mut style_props = Vec::new();
    let mut href = None;
    let mut navigate = None;
    let mut history = None;
    let mut target = None;
    let mut external_mode = None;
    let mut form_help_text = None;
    let mut form_error_text = None;
    let mut swap_bind = None;

    for prop in props {
        if matches!(
            component,
            BuiltinComponent::Button
                | BuiltinComponent::Card
                | BuiltinComponent::Input
                | BuiltinComponent::Select
                | BuiltinComponent::IconButton
                | BuiltinComponent::Swap
                | BuiltinComponent::Avatar
                | BuiltinComponent::Badge
                | BuiltinComponent::Chip
                | BuiltinComponent::SideNav
                | BuiltinComponent::RailNav
                | BuiltinComponent::Sidebar
                | BuiltinComponent::NavMenu
                | BuiltinComponent::Tabs
                | BuiltinComponent::Stepper
                | BuiltinComponent::Fab
        ) && matches!(
            prop.name.as_str(),
            "variant" | "scheme" | "size" | "rounded" | "loading" | "disabled"
        ) && !view_prop_declared(component, &prop.name)
        {
            return Err(ComponentError::unknown_prop(component, &prop.name));
        }
        let binding = prop.value.binding().cloned();
        let value = prop.value.binding_fallback().unwrap_or_else(|| prop.value.clone());
        match prop.name.as_str() {
            "bind" if component == BuiltinComponent::Swap => {
                swap_bind = Some(
                    prop.value
                        .binding()
                        .map(|binding| binding.path.clone())
                        .ok_or_else(|| ComponentError::invalid_prop(&prop.name, "boolean Signal path"))?,
                );
            }
            "variant" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.variant = reactive_reference(&prop.value)
            }
            "scheme" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.scheme = reactive_reference(&prop.value)
            }
            "size" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.size = reactive_reference(&prop.value)
            }
            "rounded" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.rounded = reactive_reference(&prop.value)
            }
            "loading" if component == BuiltinComponent::Button => {
                let loading = reactive_reference(&prop.value).ok_or_else(|| {
                    ComponentError::invalid_prop(&prop.name, "boolean Signal or View Store path")
                })?;
                variant_props.reactive.loading = Some(loading);
                variant_props.loading_icon = Some(svg_spinner_control_icon("3-dots-move")?);
            }
            "disabled" if matches!(component, BuiltinComponent::Button | BuiltinComponent::Swap) => {
                let disabled = reactive_reference(&prop.value).ok_or_else(|| {
                    ComponentError::invalid_prop(&prop.name, "boolean Signal or View Store path")
                })?;
                variant_props.reactive.disabled = Some(disabled);
            }
            "i18n" if component == BuiltinComponent::Button => {
                variant_props.i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?);
            }
            "iconStart"
                if matches!(
                    component,
                    BuiltinComponent::Button | BuiltinComponent::Input
                ) =>
            {
                let name = parse_static_string(&prop.name, &prop.value)?;
                let (condition, name, comparison) = conditional_icon_value(&name);
                variant_props.reactive.icon_start_when = condition;
                variant_props.reactive.icon_start_comparison = comparison;
                if !name.is_empty() {
                    variant_props.icon_start = Some(solar_control_icon(name)?);
                }
            }
            "iconEnd"
                if matches!(
                    component,
                    BuiltinComponent::Button | BuiltinComponent::Input
                ) =>
            {
                let name = parse_static_string(&prop.name, &prop.value)?;
                let (condition, name, comparison) = conditional_icon_value(&name);
                variant_props.reactive.icon_end_when = condition;
                variant_props.reactive.icon_end_comparison = comparison;
                if !name.is_empty() {
                    variant_props.icon_end = Some(solar_control_icon(name)?);
                }
            }
            "icon" if component == BuiltinComponent::IconButton => {
                let name = parse_static_string(&prop.name, &prop.value)?;
                variant_props.icon_start = Some(solar_control_icon(&name)?);
            }
            "iconOn" if component == BuiltinComponent::Swap => {
                let name = parse_static_string(&prop.name, &prop.value)?;
                variant_props.icon_start = Some(solar_control_icon(&name)?);
            }
            "iconOff" if component == BuiltinComponent::Swap => {
                let name = parse_static_string(&prop.name, &prop.value)?;
                variant_props.swap_icon_off = Some(solar_control_icon(&name)?);
            }
            "variant" => {
                variant_props.variant = Some(parse_variant_prop(&prop.name, &value)?);
                variant_props.variant_binding = binding;
            }
            "scheme" => {
                variant_props.color = Some(parse_family_prop(component, &prop.name, &value)?);
                variant_props.color_binding = binding
            }
            "size"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Swap
                        | BuiltinComponent::Chip
                        | BuiltinComponent::AvatarGroup
                        | BuiltinComponent::ToggleTheme
                        | BuiltinComponent::SelectTheme
                        | BuiltinComponent::Fab
                ) =>
            {
                variant_props.size = Some(parse_button_size_prop(&prop.name, &value)?);
                variant_props.size_binding = binding
            }
            "size"
                if matches!(
                    component,
                    BuiltinComponent::Input | BuiltinComponent::Select
                ) =>
            {
                variant_props.size = Some(parse_control_size_prop(&prop.name, &value)?);
                variant_props.size_binding = binding
            }
            "label" if matches!(component, BuiltinComponent::IconButton | BuiltinComponent::Swap) => {
                variant_props.label = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "label"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::CsvField
                        | BuiltinComponent::DragDrop
                        | BuiltinComponent::Editor
                        | BuiltinComponent::ImageCropper
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Pin
                        | BuiltinComponent::Textarea
                        | BuiltinComponent::Checkbox
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::DateRange
                        | BuiltinComponent::RadioGroup
                        | BuiltinComponent::Toggle
                        | BuiltinComponent::Slider
                        | BuiltinComponent::Dropzone
                        | BuiltinComponent::Fab
                ) =>
            {
                variant_props.label = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "placeholder"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::Editor
                        | BuiltinComponent::ImageCropper
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Textarea
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::DateRange
                        | BuiltinComponent::Dropzone
                ) =>
            {
                variant_props.placeholder = Some(parse_static_string(&prop.name, &prop.value)?)
            }
            "labelFloating"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Textarea
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::DateRange
                ) =>
            {
                variant_props.label_floating = parse_static_bool(&prop.name, &prop.value)?
            }
            "helpText"
                if matches!(
                    component,
                    BuiltinComponent::Input | BuiltinComponent::Select
                ) =>
            {
                form_help_text = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "errorText"
                if matches!(
                    component,
                    BuiltinComponent::Input | BuiltinComponent::Select
                ) =>
            {
                form_error_text = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "href"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                href = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "navigate"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?)
            }
            "history"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                history = Some(parse_history_prop(&prop.name, &prop.value)?)
            }
            "target"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                target = Some(parse_web_target(&prop.name, &prop.value)?)
            }
            "externalMode"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            _ => style_props.push(prop.clone()),
        }
    }

    variant_props.style = parse_style_props(
        component,
        &style_props,
        if component == BuiltinComponent::Card {
            StylePropMode::Card
        } else {
            StylePropMode::Variant
        },
    )?;
    if component == BuiltinComponent::Card {
        normalize_card_visual_props(&mut variant_props);
    }
    if form_help_text.is_some() || form_error_text.is_some() {
        let validation = variant_props.style.element.form_validation_mut();
        validation.help_text = form_help_text;
        validation.error_text = form_error_text;
    }
    variant_props.element = variant_props.style.element.clone();
    if component == BuiltinComponent::Swap {
        variant_props.element.bind = swap_bind.clone();
        variant_props.style.element.bind = swap_bind.clone();
        variant_props.swap_bind = swap_bind.clone();
    }
    variant_props.navigation =
        parse_navigation_props(component, href, navigate, history, target, external_mode)?;
    if component == BuiltinComponent::Swap {
        variant_props.icon_only = true;
        if variant_props.icon_start.is_none() || variant_props.swap_icon_off.is_none() {
            return Err(ComponentError::invalid_prop("iconOn/iconOff", "known quoted Solar icon names"));
        }
        if swap_bind.is_none() {
            return Err(ComponentError::invalid_prop("bind", "boolean Signal path"));
        }
        variant_props.swap_bind = swap_bind.clone();
    }
    if component == BuiltinComponent::IconButton {
        variant_props.icon_only = true;
        if variant_props.icon_start.is_none() {
            return Err(ComponentError::invalid_prop(
                "icon",
                "known quoted Solar icon name",
            ));
        }
        if variant_props.label.as_deref().is_none_or(str::is_empty) {
            return Err(ComponentError::invalid_prop(
                "label",
                "non-empty accessibility label",
            ));
        }
    }
    Ok(variant_props)
}

fn parse_brand_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<BrandProps> {
    let mut style_props = Vec::new();
    let mut href = None;
    let mut label = None;

    for prop in props {
        match prop.name.as_str() {
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop.clone()),
        }
    }

    Ok(BrandProps {
        style: parse_style_props(component, &style_props, StylePropMode::Layout)?,
        navigation: parse_link_navigation_props(component.as_str(), href, None, None, None)?,
        label,
    })
}

fn parse_banner_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<BannerProps> {
    let mut style_props = Vec::new();
    let mut href = None;
    let mut label = None;

    for prop in props {
        match prop.name.as_str() {
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop.clone()),
        }
    }

    let href = href.ok_or_else(|| ComponentError::invalid_prop("href", "required https URL"))?;
    let navigation = classify_href(
        &href,
        NavigationOperation::Push,
        WebTarget::Blank,
        NativeExternalMode::System,
    )?;
    if !matches!(navigation, NavigationAction::External { .. }) {
        return Err(ComponentError::invalid_prop("href", "https URL"));
    }

    Ok(BannerProps {
        style: parse_style_props(component, &style_props, StylePropMode::Banner)?,
        navigation,
        label,
    })
}

