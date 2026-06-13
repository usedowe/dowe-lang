fn parse_id_prop(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if is_valid_section_id(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(
            name,
            "portable section id using letters, numbers, underscore or hyphen",
        ))
    }
}

fn parse_required_string(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::String(value) if !value.is_empty() => Ok(value.clone()),
        PropValue::String(_) => Err(ComponentError::invalid_prop(name, "non-empty string")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "static string"))
        }
    }
}

fn parse_navigation_operation(
    name: &str,
    value: &PropValue,
) -> ComponentResult<NavigationOperation> {
    let value = parse_required_string(name, value)?;
    NavigationOperation::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "push or replace"))
}

fn parse_history_prop(name: &str, value: &PropValue) -> ComponentResult<NavigationAction> {
    let value = parse_required_string(name, value)?;
    if value == "back" {
        Ok(NavigationAction::Back)
    } else {
        Err(ComponentError::invalid_prop(name, "back"))
    }
}

fn parse_web_target(name: &str, value: &PropValue) -> ComponentResult<WebTarget> {
    let value = parse_required_string(name, value)?;
    WebTarget::from_name(&value).ok_or_else(|| ComponentError::invalid_prop(name, "self or blank"))
}

fn parse_native_external_mode(
    name: &str,
    value: &PropValue,
) -> ComponentResult<NativeExternalMode> {
    let value = parse_required_string(name, value)?;
    NativeExternalMode::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "system or webview"))
}

fn parse_navigation_props(
    component: BuiltinComponent,
    href: Option<String>,
    navigate: Option<NavigationOperation>,
    history: Option<NavigationAction>,
    target: Option<WebTarget>,
    external_mode: Option<NativeExternalMode>,
) -> ComponentResult<Option<NavigationAction>> {
    if !matches!(
        component,
        BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty
    ) {
        return Ok(None);
    }

    if href.is_some() && history.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            format!(
                "`href` and `history` cannot be used on the same `{}`",
                component.as_str()
            ),
        ));
    }

    if href.is_none() {
        return if history.is_some() {
            if navigate.is_some() || target.is_some() || external_mode.is_some() {
                Err(ComponentError::invalid_prop_combination(
                    format!(
                        "`navigate`, `target` and `externalMode` require `href` on `{}`",
                        component.as_str()
                    ),
                ))
            } else {
                Ok(history)
            }
        } else {
            parse_link_navigation_props(component.as_str(), href, navigate, target, external_mode)
        };
    }

    parse_link_navigation_props(component.as_str(), href, navigate, target, external_mode)
}

fn parse_link_navigation_props(
    owner: &str,
    href: Option<String>,
    navigate: Option<NavigationOperation>,
    target: Option<WebTarget>,
    external_mode: Option<NativeExternalMode>,
) -> ComponentResult<Option<NavigationAction>> {
    let Some(href) = href else {
        if navigate.is_some() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "`navigate` requires `href` on `{owner}`",
            )));
        }
        if target.is_some() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "`target` requires `href` on `{owner}`",
            )));
        }
        if external_mode.is_some() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "`externalMode` requires `href` on `{owner}`",
            )));
        }
        return Ok(None);
    };

    let operation = navigate.unwrap_or(NavigationOperation::Push);
    let web_target = target.unwrap_or(WebTarget::SelfTarget);
    let native_external_mode = external_mode.unwrap_or(NativeExternalMode::System);
    classify_href(&href, operation, web_target, native_external_mode).map(Some)
}

fn classify_href(
    href: &str,
    operation: NavigationOperation,
    web_target: WebTarget,
    native_external_mode: NativeExternalMode,
) -> ComponentResult<NavigationAction> {
    if href.starts_with("https://") {
        validate_https_url(href)?;
        return Ok(NavigationAction::External {
            url: href.to_string(),
            web_target,
            native_external_mode,
        });
    }

    if href.starts_with("//")
        || href.starts_with("javascript:")
        || href.starts_with("data:")
        || href.starts_with("file:")
        || href.contains("://")
    {
        return Err(ComponentError::invalid_prop(
            "href",
            "internal route, fragment or https URL",
        ));
    }

    if let Some(fragment) = href.strip_prefix('#') {
        if is_valid_section_id(fragment) {
            return Ok(NavigationAction::Section {
                fragment: fragment.to_string(),
                operation,
            });
        }
        return Err(ComponentError::invalid_prop("href", "valid fragment id"));
    }

    if !href.starts_with('/') || href.contains('?') {
        return Err(ComponentError::invalid_prop(
            "href",
            "absolute internal route, fragment or https URL",
        ));
    }

    let (path, fragment) = split_internal_href(href)?;
    Ok(NavigationAction::Internal {
        path,
        fragment,
        operation,
    })
}

fn split_internal_href(href: &str) -> ComponentResult<(String, Option<String>)> {
    if let Some((path, fragment)) = href.split_once('#') {
        if path.is_empty() || !path.starts_with('/') {
            return Err(ComponentError::invalid_prop(
                "href",
                "absolute internal route",
            ));
        }
        if !is_valid_section_id(fragment) {
            return Err(ComponentError::invalid_prop("href", "valid fragment id"));
        }
        Ok((normalize_path_value(path), Some(fragment.to_string())))
    } else {
        Ok((normalize_path_value(href), None))
    }
}

fn normalize_path_value(path: &str) -> String {
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn validate_https_url(value: &str) -> ComponentResult<()> {
    let rest = value
        .strip_prefix("https://")
        .ok_or_else(|| ComponentError::invalid_prop("href", "https URL"))?;
    let host = rest
        .split(['/', '#', '?'])
        .next()
        .filter(|host| !host.is_empty())
        .ok_or_else(|| ComponentError::invalid_prop("href", "https URL with host"))?;
    if host.chars().any(|value| value.is_control() || value == ' ') {
        return Err(ComponentError::invalid_prop("href", "https URL with host"));
    }
    Ok(())
}

fn is_valid_section_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '_' || value == '-')
}
