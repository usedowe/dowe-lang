#[test]
fn completes_form_validation_children_props_and_rule_values() {
    let document = LanguageDocument {
        path: PathBuf::from("/project/pages/form.dowe"),
        source: "page FormPage\n  Input \n    validate \n    validate rule:\n".to_string(),
    };

    let input_props = complete_document(Path::new("/project"), &document, 2, "  Input ".len() + 1);
    let validation_props = complete_document(
        Path::new("/project"),
        &document,
        3,
        "    validate ".len() + 1,
    );
    let rule_values = complete_document(
        Path::new("/project"),
        &document,
        4,
        "    validate rule:".len() + 1,
    );

    assert!(input_props.iter().any(|item| item.label == "helpText"));
    assert!(input_props.iter().any(|item| item.label == "errorText"));
    assert!(validation_props.iter().any(|item| item.label == "rule"));
    assert!(validation_props.iter().any(|item| item.label == "message"));
    assert!(rule_values.iter().any(|item| item.label == "\"required\""));
    assert!(
        rule_values
            .iter()
            .any(|item| item.label == "\"matches:form.confirmation\"")
    );
}
