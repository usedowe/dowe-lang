pub fn form_validation_rule(
    rule: impl AsRef<str>,
    message: impl AsRef<str>,
) -> ComponentResult<FormValidationRule> {
    let source = rule.as_ref().trim();
    let message = message.as_ref().trim();
    if message.is_empty() {
        return Err(ComponentError::invalid_prop("message", "non-empty string"));
    }
    let kind = match source {
        "required" => FormValidationRuleKind::Required,
        "email" => FormValidationRuleKind::Email,
        "url" => FormValidationRuleKind::Url,
        "phone" => FormValidationRuleKind::Phone,
        "alphanumeric" => FormValidationRuleKind::Alphanumeric,
        "numeric" => FormValidationRuleKind::Numeric,
        "alpha" => FormValidationRuleKind::Alpha,
        "strongPassword" => FormValidationRuleKind::StrongPassword,
        "creditCard" => FormValidationRuleKind::CreditCard,
        "date" => FormValidationRuleKind::Date,
        _ => parse_parameterized_validation_rule(source)?,
    };
    Ok(FormValidationRule {
        kind,
        message: message.to_string(),
    })
}

fn parse_parameterized_validation_rule(source: &str) -> ComponentResult<FormValidationRuleKind> {
    let Some((name, argument)) = source.split_once(':') else {
        return Err(ComponentError::invalid_prop("rule", "known validation rule"));
    };
    match name {
        "min" => Ok(FormValidationRuleKind::Min(validation_count(argument)?)),
        "max" => Ok(FormValidationRuleKind::Max(validation_count(argument)?)),
        "minWords" => Ok(FormValidationRuleKind::MinWords(validation_count(argument)?)),
        "maxWords" => Ok(FormValidationRuleKind::MaxWords(validation_count(argument)?)),
        "matches" if valid_validation_path(argument) => {
            Ok(FormValidationRuleKind::Matches(argument.to_string()))
        }
        "matches" => Err(ComponentError::invalid_prop(
            "rule",
            "matches:<Signal or View Store path>",
        )),
        "pattern" if valid_portable_validation_pattern(argument) => {
            Ok(FormValidationRuleKind::Pattern(argument.to_string()))
        }
        "pattern" => Err(ComponentError::invalid_prop(
            "rule",
            "pattern:<portable regular expression>",
        )),
        _ => Err(ComponentError::invalid_prop("rule", "known validation rule")),
    }
}

fn validation_count(value: &str) -> ComponentResult<u16> {
    value
        .parse::<u16>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| ComponentError::invalid_prop("rule", "positive validation count"))
}

fn valid_validation_path(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|segment| {
            let mut characters = segment.chars();
            characters
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
                && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
        })
}

fn valid_portable_validation_pattern(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 256
        || value.contains('\n')
        || value.contains('\r')
        || value.contains('\0')
        || value.contains("(?")
        || value.contains("\\p")
    {
        return false;
    }
    let mut escaped = false;
    let mut parentheses = 0usize;
    let mut brackets = 0usize;
    for character in value.chars() {
        if escaped {
            if character.is_ascii_digit() {
                return false;
            }
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        match character {
            '(' if brackets == 0 => parentheses += 1,
            ')' if brackets == 0 && parentheses > 0 => parentheses -= 1,
            ')' if brackets == 0 => return false,
            '[' => brackets += 1,
            ']' if brackets > 0 => brackets -= 1,
            ']' => return false,
            _ => {}
        }
    }
    !escaped && parentheses == 0 && brackets == 0
}

pub fn attach_form_validation(
    mut node: ViewNode,
    rules: Vec<FormValidationRule>,
) -> ComponentResult<ViewNode> {
    let element = match &mut node {
        ViewNode::Input { props } | ViewNode::Select { props, .. } => &mut props.element,
        ViewNode::Checkbox { props } => &mut props.style.element,
        ViewNode::Date { props } => &mut props.style.element,
        ViewNode::Phone { props } => &mut props.style.element,
        ViewNode::Pin { props } => &mut props.style.element,
        _ => {
            return Err(ComponentError::invalid_prop_combination(
                "validate is only supported by Input, Date, Pin, Phone, Select and Checkbox",
            ));
        }
    };
    element.form_validation_mut().rules = rules;
    Ok(node)
}
