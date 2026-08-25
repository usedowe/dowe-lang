fn button_tags(props: &VariantProps, context: &ReactiveRenderContext) -> (String, &'static str) {
    let mut classes = variant_classes("button", props);
    if props.icon_only {
        classes.push("icon-button".to_string());
    }
    let accessibility = props
        .swap_bind
        .as_deref()
        .map(|bind| format!(r#" aria-pressed="{}" data-dowe-swap data-dowe-swap-bind="{}""#, if props.style.element.bind.is_some() { "false" } else { "false" }, escape_attr(bind)))
        .unwrap_or_default()
        + &props
        .icon_only
        .then(|| {
            format!(
                r#" aria-label="{}""#,
                escape_attr(props.label.as_deref().unwrap_or_default())
            )
        })
        .unwrap_or_default()
        + &reactive_button_attrs(props, context);
    match props.navigation.as_ref() {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => {
            let href = internal_href(path, fragment.as_deref());
            (
                format!(
                    "<a{}>",
                    attrs(
                        classes,
                        Some(&props.element),
                        Some(&format!(
                            "{}{}",
                            navigation_attrs(&href, *operation),
                            accessibility
                        )),
                        context
                    )
                ),
                "</a>",
            )
        }
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => {
            let href = format!("#{fragment}");
            (
                format!(
                    "<a{}>",
                    attrs(
                        classes,
                        Some(&props.element),
                        Some(&format!(
                            "{}{}",
                            navigation_attrs(&href, *operation),
                            accessibility
                        )),
                        context
                    )
                ),
                "</a>",
            )
        }
        Some(NavigationAction::External {
            url,
            web_target,
            native_external_mode,
        }) => (
            format!(
                "<a{}>",
                attrs(
                    classes,
                    Some(&props.element),
                    Some(&format!(
                        "{}{}",
                        external_attrs(url, *web_target, *native_external_mode),
                        accessibility
                    )),
                    context
                )
            ),
            "</a>",
        ),
        Some(NavigationAction::Back) => (
            format!(
                r#"<button{}>"#,
                attrs(
                    classes,
                    Some(&props.element),
                    Some(&format!(
                        r#" type="button" data-dowe-history="back"{}"#,
                        accessibility
                    )),
                    context
                )
            ),
            "</button>",
        ),
        None => (
            format!(
                r#"<button{}>"#,
                attrs(
                    classes,
                    Some(&props.element),
                    Some(&format!(r#" type="button"{}"#, accessibility)),
                    context,
                )
            ),
            "</button>",
        ),
    }
}

fn brand_tags(props: &BrandProps, context: &ReactiveRenderContext) -> (String, &'static str) {
    let classes = brand_classes(&props.style);
    let label = props
        .label
        .as_deref()
        .map(|label| format!(r#" aria-label="{}""#, escape_attr(label)))
        .unwrap_or_default();
    match props.navigation.as_ref() {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => {
            let href = internal_href(path, fragment.as_deref());
            (
                format!(
                    "<a{}>",
                    attrs(
                        classes,
                        Some(&props.style.element),
                        Some(&format!("{}{}", navigation_attrs(&href, *operation), label)),
                        context
                    )
                ),
                "</a>",
            )
        }
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => {
            let href = format!("#{fragment}");
            (
                format!(
                    "<a{}>",
                    attrs(
                        classes,
                        Some(&props.style.element),
                        Some(&format!("{}{}", navigation_attrs(&href, *operation), label)),
                        context
                    )
                ),
                "</a>",
            )
        }
        Some(NavigationAction::External {
            url,
            web_target,
            native_external_mode,
        }) => (
            format!(
                "<a{}>",
                attrs(
                    classes,
                    Some(&props.style.element),
                    Some(&format!(
                        "{}{}",
                        external_attrs(url, *web_target, *native_external_mode),
                        label
                    )),
                    context
                )
            ),
            "</a>",
        ),
        Some(NavigationAction::Back) => (
            format!(
                "<button{}>",
                attrs(
                    classes,
                    Some(&props.style.element),
                    Some(&format!(
                        r#" type="button" data-dowe-history="back"{label}"#
                    )),
                    context
                )
            ),
            "</button>",
        ),
        None => {
            let accessibility = props
                .label
                .as_ref()
                .map(|_| format!(r#" role="img"{label}"#))
                .unwrap_or_default();
            (
                format!(
                    "<div{}>",
                    attrs(
                        classes,
                        Some(&props.style.element),
                        Some(&accessibility),
                        context
                    )
                ),
                "</div>",
            )
        }
    }
}

fn banner_tags(props: &BannerProps, context: &ReactiveRenderContext) -> (String, &'static str) {
    let classes = banner_classes(&props.style);
    let label = props
        .label
        .as_deref()
        .map(|label| format!(r#" aria-label="{}""#, escape_attr(label)))
        .unwrap_or_default();
    let NavigationAction::External {
        url,
        web_target,
        native_external_mode,
    } = &props.navigation
    else {
        unreachable!()
    };
    (
        format!(
            "<a{}>",
            attrs(
                classes,
                Some(&props.style.element),
                Some(&format!(
                    "{}{}",
                    external_attrs(url, *web_target, *native_external_mode),
                    label
                )),
                context
            )
        ),
        "</a>",
    )
}

fn reactive_button_attrs(props: &VariantProps, context: &ReactiveRenderContext) -> String {
    let mut attrs = String::new();
    for binding in props.bindings() {
        let name = binding.property.as_str();
        if matches!(name, "variant" | "scheme" | "size" | "rounded") {
            attrs.push_str(&format!(
                r#" data-dowe-variant-binding="true" data-dowe-{}="{}""#,
                name,
                escape_attr(&context.signal_path(&binding.binding.path))
            ));
        }
    }
    for (name, value) in [
        ("variant", props.reactive.variant.as_deref()),
        ("scheme", props.reactive.scheme.as_deref()),
        ("size", props.reactive.size.as_deref()),
        ("rounded", props.reactive.rounded.as_deref()),
        ("loading", props.reactive.loading.as_deref()),
        ("disabled", props.reactive.disabled.as_deref()),
        ("icon-start-when", props.reactive.icon_start_when.as_deref()),
        ("icon-end-when", props.reactive.icon_end_when.as_deref()),
    ] {
        if let Some(value) = value {
            attrs.push_str(&format!(
                r#" data-dowe-button-{}="{}""#,
                name,
                escape_attr(&context.signal_path(value))
            ));
        }
    }
    for (name, comparison) in [
        ("icon-start", props.reactive.icon_start_comparison.as_ref()),
        ("icon-end", props.reactive.icon_end_comparison.as_ref()),
    ] {
        if let Some(comparison) = comparison {
            attrs.push_str(&format!(
                r#" data-dowe-button-{}-operator="{}" data-dowe-button-{}-value="{}""#,
                name,
                comparison.operator.as_str(),
                name,
                escape_attr(&comparison.value)
            ));
        }
    }
    attrs
}

fn internal_href(path: &str, fragment: Option<&str>) -> String {
    if let Some(fragment) = fragment {
        format!("{path}#{fragment}")
    } else {
        path.to_string()
    }
}

