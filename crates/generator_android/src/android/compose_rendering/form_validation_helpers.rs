fn compose_control_icon(icon: Option<&SideNavIcon>) -> String {
    icon.map(|icon| format!(
        "{{ DoweSvg(viewBox = {}, modifier = Modifier.width(24.dp).height(24.dp), color = LocalContentColor.current, paths = {}) }}",
        compose_svg_view_box(&icon.props.view_box),
        compose_svg_paths(&icon.paths)
    ))
    .unwrap_or_else(|| "null".to_string())
}

fn compose_password_icon(icon: &SideNavIcon) -> String {
    format!(
        "{{ DoweSvg(viewBox = {}, modifier = Modifier.width(20.dp).height(20.dp), color = LocalContentColor.current, paths = {}) }}",
        compose_svg_view_box(&icon.props.view_box),
        compose_svg_paths(&icon.paths)
    )
}

fn compose_phone_country_catalog() -> String {
    let countries = phone_countries()
        .iter()
        .filter_map(|country| {
            let icon = phone_country_flag_icon(country.code)?;
            Some(format!(
                "DowePhoneCountry(code = {}, name = {}, dialCode = {}, viewBox = {}, paths = {})",
                compose_string_literal(country.code),
                compose_string_literal(country.name),
                compose_string_literal(country.dial),
                compose_svg_view_box(&icon.props.view_box),
                compose_svg_paths(&icon.paths)
            ))
        })
        .collect::<Vec<_>>();
    let mut output = String::new();
    let mut parts = Vec::new();
    for (index, countries) in countries.chunks(16).enumerate() {
        let name = format!("dowePhoneCountries{index}");
        output.push_str(&format!(
            "private fun {name}(): List<DowePhoneCountry> = listOf({})\n",
            countries.join(", ")
        ));
        parts.push(format!("addAll({name}())"));
    }
    output.push_str(&format!(
        "private val dowePhoneCountries: List<DowePhoneCountry> = buildList {{ {} }}",
        parts.join("; ")
    ));
    output
}

fn compose_string_list(values: &[String]) -> String {
    format!(
        "listOf({})",
        values
            .iter()
            .map(|value| compose_string_literal(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn compose_validation_help(element: &ElementProps) -> String {
    compose_optional_string(
        element
            .form_validation()
            .and_then(|validation| validation.help_text.as_deref()),
    )
}

fn compose_validation_error(element: &ElementProps) -> String {
    compose_optional_string(
        element
            .form_validation()
            .and_then(|validation| validation.error_text.as_deref()),
    )
}

fn compose_validation_rules(element: &ElementProps, context: &ComposeReactiveContext) -> String {
    let Some(validation) = element.form_validation() else {
        return "emptyList()".to_string();
    };
    let values = validation
        .rules
        .iter()
        .map(|rule| {
            let argument = match &rule.kind {
                dowe_components::FormValidationRuleKind::Matches(path) => format!(
                    "state.text(\"{}\")",
                    escape_kotlin(&context.signal_path(path))
                ),
                _ => rule
                    .kind
                    .argument()
                    .as_deref()
                    .map(compose_string_literal)
                    .unwrap_or_else(|| "null".to_string()),
            };
            format!(
                "DoweValidationRule(kind = {}, argument = {argument}, message = {})",
                compose_string_literal(rule.kind.name()),
                compose_string_literal(&rule.message)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_boolean_validation_rules(
    element: &ElementProps,
    context: &ComposeReactiveContext,
) -> String {
    let Some(validation) = element.form_validation() else {
        return "emptyList()".to_string();
    };
    let values = validation
        .rules
        .iter()
        .map(|rule| {
            let argument = match &rule.kind {
                dowe_components::FormValidationRuleKind::Matches(path) => format!(
                    "state.bool(\"{}\").toString()",
                    escape_kotlin(&context.signal_path(path))
                ),
                _ => rule
                    .kind
                    .argument()
                    .as_deref()
                    .map(compose_string_literal)
                    .unwrap_or_else(|| "null".to_string()),
            };
            format!(
                "DoweValidationRule(kind = {}, argument = {argument}, message = {})",
                compose_string_literal(rule.kind.name()),
                compose_string_literal(&rule.message)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

