#[test]
fn parses_advanced_form_components_and_structural_children() {
    let tree = parse_page(
        r#"page advancedPage
  signal form value:{ role:"editor" notes:"" password:"" phone:"" pin:"" bio:"" avatar:"" }
  fn saveNotes
    set form.notes value:form.notes
  Box
    ComboBox bind:form.role label:"Role" placeholder:"Choose" clearable:true
      comboOption value:"admin" label:"Admin" description:"Full access"
      comboOption value:"editor" label:"Editor"
    CsvField label:"Import" buttonText:"Upload CSV"
      csvColumn name:"email" label:"Email"
    DragDrop label:"Tasks" direction:"horizontal"
      dragGroup id:"todo" title:"Todo"
        dragItem id:"draft" label:"Draft" description:"Prepare"
    Editor bind:form.notes language:"dowe" label:"Notes" placeholder:"Write notes" minHeight:180 onSave:saveNotes
    ImageCropper bind:form.avatar label:"Avatar" shape:"circle"
    Password bind:form.password label:"Password" hideStrength:false
      validate rule:"required" message:"Enter your password."
      validate rule:"strongPassword" message:"Use a stronger password."
    Phone bind:form.phone label:"Phone" country:"US"
    Pin bind:form.pin label:"Code" length:6 type:"number"
    Textarea bind:form.bio label:"Bio" rows:4 maxLength:160"#,
    )
    .expect("tree");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Box { children, .. } = &children[0] else {
        panic!("box");
    };

    assert!(matches!(
        &children[0],
        ViewNode::ComboBox { props, options }
            if props.clearable
                && props.style.element.bind.as_deref() == Some("form.role")
                && options.len() == 2
                && options[0].description.as_deref() == Some("Full access")
    ));
    assert!(matches!(
        &children[1],
        ViewNode::CsvField { props, columns }
            if props.button_text == "Upload CSV"
                && columns.len() == 1
                && columns[0].name == "email"
    ));
    assert!(matches!(
        &children[2],
        ViewNode::DragDrop { props, groups, .. }
            if props.direction.as_str() == "horizontal"
                && groups.len() == 1
                && groups[0].items[0].id == "draft"
    ));
    assert!(matches!(
        &children[3],
        ViewNode::Editor { props }
            if props.min_height == 180
                && props.language == dowe_components::CodeLanguage::Dowe
                && props.style.element.bind.as_deref() == Some("form.notes")
                && props.on_save.as_deref() == Some("saveNotes")
    ));
    assert!(matches!(
        &children[4],
        ViewNode::ImageCropper { props }
            if props.shape.as_str() == "circle"
                && props.style.element.bind.as_deref() == Some("form.avatar")
    ));
    assert!(matches!(
        &children[5],
        ViewNode::Password { props }
            if !props.hide_strength
                && props.style.element.form_validation().is_some_and(|validation| validation.rules.len() == 2)
    ));
    assert!(matches!(
        &children[6],
        ViewNode::Phone { props } if props.country.as_deref() == Some("US")
    ));
    assert!(matches!(
        &children[7],
        ViewNode::Pin { props }
            if props.length == 6 && props.kind.as_str() == "number"
    ));
    assert!(matches!(
        &children[8],
        ViewNode::Textarea { props } if props.rows == 4 && props.max_length == Some(160)
    ));
}

#[test]
fn rejects_unknown_editor_save_action() {
    let error = parse_page(
        r#"page editorPage
  signal source value:""
  Editor bind:source language:"dowe" onSave:missing"#,
    )
    .expect_err("unknown editor save action");

    assert!(error.to_string().contains("unknown fn `missing`"));
}

#[test]
fn rejects_removed_phone_field_component_name() {
    let error = parse_page(
        r#"page contactPage
  PhoneField country:"US""#,
    )
    .expect_err("removed phone component name");

    assert!(error.to_string().contains("unknown component `PhoneField`"));
}

#[test]
fn rejects_removed_pin_field_component_name() {
    let error = parse_page(
        r#"page verificationPage
  PinField length:6 type:"number""#,
    )
    .expect_err("removed pin component name");

    assert!(error.to_string().contains("unknown component `PinField`"));
}

#[test]
fn lowers_validation_rules_for_all_supported_form_controls() {
    let tree = parse_page(
        r#"page validationPage
  signal form value:{ email:"" birthday:"" code:"" phone:"" role:"" accepted:false password:"" }
  Box
    Input bind:form.email label:"Email" helpText:"Use your work email"
      validate rule:"required" message:"Email is required"
      validate rule:"email" message:"Enter a valid email"
    Date bind:form.birthday label:"Birthday"
      validate rule:"date" message:"Enter a valid date"
    Pin bind:form.code label:"Code"
      validate rule:"min:6" message:"Enter all six digits"
    Phone bind:form.phone label:"Phone"
      validate rule:"phone" message:"Enter a valid phone"
    Select bind:form.role label:"Role" errorText:"Role is unavailable"
      Option value:"admin" label:"Admin"
      validate rule:"required" message:"Choose a role"
    Checkbox bind:form.accepted label:"Accept" helpText:"Required to continue"
      validate rule:"required" message:"Accept the terms""#,
    )
    .expect("validated controls");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Box { children, .. } = &children[0] else {
        panic!("box");
    };

    let rules = children
        .iter()
        .map(|node| {
            super::node_element_props(node)
                .and_then(|props| props.form_validation())
                .expect("validation")
        })
        .collect::<Vec<_>>();
    assert_eq!(rules[0].rules.len(), 2);
    assert_eq!(rules[0].help_text.as_deref(), Some("Use your work email"));
    assert_eq!(rules[1].rules[0].kind.name(), "date");
    assert_eq!(rules[2].rules[0].kind.argument(), Some("6".to_string()));
    assert_eq!(rules[3].rules[0].kind.name(), "phone");
    assert_eq!(rules[4].error_text.as_deref(), Some("Role is unavailable"));
    assert_eq!(rules[5].help_text.as_deref(), Some("Required to continue"));
}

#[test]
fn rejects_invalid_validation_structure_rules_and_matches_types() {
    let outside = parse_page(
        r#"page validationPage
  validate rule:"required" message:"Required""#,
    )
    .expect_err("contextual validation");
    assert!(outside.to_string().contains("can only be used inside"));

    let invalid_rule = parse_page(
        r#"page validationPage
  Input
    validate rule:"custom" message:"Invalid""#,
    )
    .expect_err("closed rule set");
    assert!(invalid_rule.to_string().contains("known validation rule"));

    let invalid_child = parse_page(
        r#"page validationPage
  Phone
    Text
      "No""#,
    )
    .expect_err("validate-only children");
    assert!(
        invalid_child
            .to_string()
            .contains("only accepts validate children")
    );

    let invalid_match = parse_page(
        r#"page validationPage
  signal form value:{ accepted:false count:1 }
  Checkbox bind:form.accepted
    validate rule:"matches:form.count" message:"Must match""#,
    )
    .expect_err("typed matches path");
    assert!(
        invalid_match
            .to_string()
            .contains("in `validate matches`: expected bool"),
        "{invalid_match}"
    );
}

#[test]
fn rejects_duplicate_request_path_forms() {
    let error = parse_page(
        r#"page blogsPage
  signal blogs value:[]
  fn load
    request GET "/api/blogs" route:"/api/blogs" update:blogs
  Box
    Text
      "Blogs""#,
    )
    .expect_err("error");

    assert!(error.to_string().contains("only one route path"));
}

#[test]
fn rejects_text_prop_on_text_component() {
    let error = parse_page(
        r#"page blogsPage
  Text text:"Blogs""#,
    )
    .expect_err("error");

    assert!(error.to_string().contains("unknown prop `text`"));
}

#[test]
fn rejects_unknown_and_incompatible_signal_paths() {
    let missing = parse_page(
        r#"page blogsPage
  signal blog value:{ title:"" visible:false }
  Box
    Input bind:blog.missing"#,
    )
    .expect_err("missing field");
    assert!(
        missing
            .to_string()
            .contains("unknown signal path `blog.missing`")
    );

    let incompatible = parse_page(
        r#"page blogsPage
  signal alert value:{ message:"" visible:false }
  Alert type:"info" message:alert.visible visible:alert.message"#,
    )
    .expect_err("incompatible field");
    assert!(
        incompatible
            .to_string()
            .contains("invalid signal path `alert.visible` in `message`: expected string")
    );
}

