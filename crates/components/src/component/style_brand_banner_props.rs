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

