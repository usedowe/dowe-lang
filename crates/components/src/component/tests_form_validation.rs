#[test]
fn parses_the_closed_form_validation_rule_set() {
    let cases = [
        ("required", super::FormValidationRuleKind::Required),
        ("email", super::FormValidationRuleKind::Email),
        ("min:3", super::FormValidationRuleKind::Min(3)),
        ("max:40", super::FormValidationRuleKind::Max(40)),
        ("url", super::FormValidationRuleKind::Url),
        ("phone", super::FormValidationRuleKind::Phone),
        (
            "pattern:^[A-Za-z]+$",
            super::FormValidationRuleKind::Pattern("^[A-Za-z]+$".to_string()),
        ),
        ("alphanumeric", super::FormValidationRuleKind::Alphanumeric),
        ("numeric", super::FormValidationRuleKind::Numeric),
        ("alpha", super::FormValidationRuleKind::Alpha),
        (
            "matches:profile.password",
            super::FormValidationRuleKind::Matches("profile.password".to_string()),
        ),
        (
            "strongPassword",
            super::FormValidationRuleKind::StrongPassword,
        ),
        ("creditCard", super::FormValidationRuleKind::CreditCard),
        ("date", super::FormValidationRuleKind::Date),
        ("minWords:2", super::FormValidationRuleKind::MinWords(2)),
        ("maxWords:8", super::FormValidationRuleKind::MaxWords(8)),
    ];

    for (source, expected) in cases {
        assert_eq!(
            super::form_validation_rule(source, "Invalid value")
                .expect("validation rule")
                .kind,
            expected
        );
    }
}

#[test]
fn rejects_invalid_form_validation_contracts() {
    for rule in [
        "custom",
        "min:0",
        "max:nope",
        "matches:profile..password",
        "pattern:(unclosed",
        "pattern:(?=lookahead)",
        "pattern:(a)\\1",
    ] {
        assert!(super::form_validation_rule(rule, "Invalid").is_err());
    }
    assert!(super::form_validation_rule("required", " ").is_err());
}

#[test]
fn attaches_validation_and_form_messages_to_supported_controls() {
    let rule = super::form_validation_rule("required", "Required").expect("rule");
    let input = super::input_node(vec![
        string_prop("helpText", "Use your work email"),
        string_prop("errorText", "Server rejected this value"),
    ])
    .expect("input");
    let input = super::attach_form_validation(input, vec![rule.clone()]).expect("validation");
    let ViewNode::Input { props } = input else {
        panic!("input");
    };
    let validation = props.element.form_validation().expect("form validation");
    assert_eq!(validation.help_text.as_deref(), Some("Use your work email"));
    assert_eq!(
        validation.error_text.as_deref(),
        Some("Server rejected this value")
    );
    assert_eq!(validation.rules, vec![rule.clone()]);

    let checkbox = super::checkbox_component_node(vec![
        string_prop("helpText", "Required to continue"),
        string_prop("errorText", "Accept the terms"),
    ])
    .expect("checkbox");
    let checkbox = super::attach_form_validation(checkbox, vec![rule.clone()]).expect("validation");
    let ViewNode::Checkbox { props } = checkbox else {
        panic!("checkbox");
    };
    let validation = props
        .style
        .element
        .form_validation()
        .expect("form validation");
    assert_eq!(
        validation.help_text.as_deref(),
        Some("Required to continue")
    );
    assert_eq!(validation.error_text.as_deref(), Some("Accept the terms"));
    assert_eq!(validation.rules, vec![rule]);

    assert!(super::attach_form_validation(
        super::text_node("Unsupported").expect("text"),
        Vec::new()
    )
    .is_err());
}
