#[test]
fn parses_advanced_form_components_and_structural_children() {
    let tree = parse_page(
        r#"page advancedPage
  signal form value:{ role:"editor" notes:"" password:"" phone:"" pin:"" bio:"" avatar:"" }
  Box
    ComboBox bind:form.role label:"Role" placeholder:"Choose" clearable:true
      comboOption value:"admin" label:"Admin" description:"Full access"
      comboOption value:"editor" label:"Editor"
    CsvField label:"Import" buttonText:"Upload CSV"
      csvColumn name:"email" label:"Email"
    DragDrop label:"Tasks" direction:"horizontal"
      dragGroup id:"todo" title:"Todo"
        dragItem id:"draft" label:"Draft" description:"Prepare"
    Editor bind:form.notes label:"Notes" placeholder:"Write notes" minHeight:180
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
                && props.style.element.bind.as_deref() == Some("form.notes")
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

#[test]
fn parses_reactive_button_visual_props_and_conditional_icon() {
    let page = parse_page(
            r#"page buttonPage
  signal variantChoice value:"solid"
  signal schemeChoice value:"primary"
  signal sizeChoice value:"md"
  signal roundedChoice value:"md"
  signal loadingChoice value:false
  signal startIconVisible value:true
  Button variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice loading:loadingChoice iconStart:{ when:startIconVisible value:"add-circle" }
    "Create""#,
        )
        .expect("reactive button");
    let ViewNode::Scope { children, .. } = page else {
        panic!("scope")
    };
    let ViewNode::Button { props, .. } = &children[0] else {
        panic!("button")
    };
    assert_eq!(props.reactive.variant.as_deref(), Some("variantChoice"));
    assert_eq!(props.reactive.scheme.as_deref(), Some("schemeChoice"));
    assert_eq!(props.reactive.size.as_deref(), Some("sizeChoice"));
    assert_eq!(props.reactive.rounded.as_deref(), Some("roundedChoice"));
    assert_eq!(props.reactive.loading.as_deref(), Some("loadingChoice"));
    assert_eq!(
        props.reactive.icon_start_when.as_deref(),
        Some("startIconVisible")
    );
}

#[test]
fn rejects_removed_button_icon_visibility_props() {
    let error = parse_page(
        r#"page buttonPage
  signal visible value:true
  Button iconStart:"add-circle" showIconStart:visible
    "Create""#,
    )
    .expect_err("removed prop");
    assert!(error.to_string().contains("unknown prop `showIconStart`"));
}

#[test]
fn rejects_static_button_loading_values() {
    let error = parse_page(
        r#"page buttonPage
  Button loading:true
    "Create""#,
    )
    .expect_err("static loading");
    assert!(error.to_string().contains("loading"));
    assert!(error.to_string().contains("signal bool path"));
}

#[test]
fn omits_empty_button_icon_strings() {
    let page = parse_page(
        r#"page buttonPage
  Button iconStart:"" iconEnd:""
    "Create""#,
    )
    .expect("empty icons");
    let ViewNode::Button { props, .. } = &page else {
        panic!("button")
    };
    assert!(props.icon_start.is_none());
    assert!(props.icon_end.is_none());
}

#[test]
fn parses_numeric_conditional_button_icon() {
    let page = parse_page(
        r#"page buttonPage
  signal count value:11
  Button iconStart:{ when:count gt:10 value:"bell" }
    "Notifications""#,
    )
    .expect("numeric condition");
    let ViewNode::Scope { children, .. } = page else {
        panic!("scope")
    };
    let ViewNode::Button { props, .. } = &children[0] else {
        panic!("button")
    };
    let comparison = props
        .reactive
        .icon_start_comparison
        .as_ref()
        .expect("comparison");
    assert_eq!(comparison.operator.as_str(), ">");
    assert_eq!(comparison.value, "10");
}

#[test]
fn rejects_i18n_with_reactive_text_child() {
    let error = parse_page(
        r#"page profilePage
  signal profile value:{ title:"" }
  Text i18n:"profile.title"
    "{profile.title}""#,
    )
    .expect_err("reactive fallback");

    assert!(
        error
            .to_string()
            .contains("`i18n` requires a static fallback text child")
    );
}

#[test]
fn parses_show_visibility_conditions() {
    let tree = parse_page(
        r#"page readyPage
  signal isReady value:false
  signal rows value:[{ id:"1" ready:true }]
  Box show:{ xs:false md:true }
    Text show:isReady
      "Ready"
    each in:rows as:row key:row.id
      Text show:row.ready
        "Row""#,
    )
    .expect("tree");

    let ViewNode::Scope {
        signals, children, ..
    } = tree
    else {
        panic!("scope");
    };
    assert_eq!(signals[0].name, "isReady");

    let ViewNode::Box {
        props,
        children: box_children,
    } = &children[0]
    else {
        panic!("box");
    };
    match props.element.show.as_ref().expect("box show") {
        VisibilityCondition::Static(value) => {
            assert_eq!(value.entries.len(), 2);
            assert_eq!(value.entries[0].breakpoint, Breakpoint::Xs);
            assert!(!value.entries[0].value);
            assert_eq!(value.entries[1].breakpoint, Breakpoint::Md);
            assert!(value.entries[1].value);
        }
        VisibilityCondition::Signal(_) => panic!("static show"),
        VisibilityCondition::NumberComparison { .. } => panic!("static show"),
        VisibilityCondition::StringEquality { .. } => panic!("static show"),
    }

    let ViewNode::Text { props, .. } = &box_children[0] else {
        panic!("text");
    };
    assert_eq!(
        props.style.element.show,
        Some(VisibilityCondition::Signal("isReady".to_string()))
    );

    let ViewNode::Each {
        children: row_children,
        ..
    } = &box_children[1]
    else {
        panic!("each");
    };
    let ViewNode::Text { props, .. } = &row_children[0] else {
        panic!("row text");
    };
    assert_eq!(
        props.style.element.show,
        Some(VisibilityCondition::Signal("row.ready".to_string()))
    );
}

#[test]
fn parses_and_validates_numeric_show_condition() {
    let page = parse_page(
        r#"page countPage
  signal itemCount value:11
  Section show:{ when:itemCount gt:10 }
    Text
      "Many items""#,
    )
    .expect("numeric show");
    let ViewNode::Scope { children, .. } = page else {
        panic!("scope")
    };
    let ViewNode::Section { props, .. } = &children[0] else {
        panic!("section")
    };
    let Some(VisibilityCondition::NumberComparison { path, comparison }) =
        props.element.show.as_ref()
    else {
        panic!("numeric show")
    };
    assert_eq!(path, "itemCount");
    assert_eq!(comparison.operator.as_str(), ">");
    assert_eq!(comparison.value, "10");

    let error = parse_page(
        r#"page countPage
  signal label value:"many"
  Section show:{ when:label gt:10 }
    Text
      "Many items""#,
    )
    .expect_err("string comparison");
    assert!(
        error
            .to_string()
            .contains("invalid signal path `label` in `show.when`: expected number")
    );

    let error = parse_page(
        r#"page countPage
  signal itemCount value:11
  Section show:{ when:itemCount gt:10 lte:20 }
    Text
      "Many items""#,
    )
    .expect_err("multiple comparisons");
    assert!(
        error
            .to_string()
            .contains("show conditions accept one numeric comparator")
    );
}

#[test]
fn parses_interactive_motion_props() {
    let tree = parse_page(
            r#"page motionPage
  signal selected value:""
  fn selectMobile
    set selected value:"mobile"
  Flex direction:"column" animation:"fadeIn"
    Chip variant:"solid" scheme:"warning" size:"sm" rotate:-7 scale:1.05 translateX:-1.5 translateY:{ xs:0 md:2 } transition:"spring" gesture:"lift" onClick:selectMobile
      "Mobile Apps""#,
        )
        .expect("tree");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Flex { props, children } = &children[0] else {
        panic!("flex");
    };
    assert_eq!(props.style.animation(), Some(ViewAnimation::FadeIn));

    let ViewNode::Chip { props, .. } = &children[0] else {
        panic!("chip");
    };
    let motion = props.style.style.motion();
    assert_eq!(
        motion.rotate.as_ref().unwrap().entries[0].value,
        ViewRotation(-7)
    );
    assert_eq!(
        motion.scale.as_ref().unwrap().entries[0].value,
        ViewScale(105)
    );
    assert_eq!(
        motion.translate_x.as_ref().unwrap().entries[0].value,
        ViewTranslation(-3)
    );
    assert_eq!(motion.transition, Some(ViewTransition::Spring));
    assert_eq!(motion.gesture, Some(ViewGesture::Lift));
    assert_eq!(
        props.style.element.on_click.as_deref(),
        Some("selectMobile")
    );
}

#[test]
fn parses_section_background_props() {
    let tree = parse_page(
        r#"page landingPage
  Section id:"hero" background:{ xs:"soft" md:"aurora" } color:"backgroundText" boxed:true p:8
    Text
      "Hero""#,
    )
    .expect("tree");

    let ViewNode::Section { props, children } = tree else {
        panic!("section");
    };
    assert_eq!(props.element.id.as_deref(), Some("hero"));
    assert_eq!(
        props.background.expect("background").entries[1].value,
        SectionBackground::Aurora
    );
    assert!(props.text.is_some());
    assert!(props.boxed);
    assert_eq!(children.len(), 1);
}

#[test]
fn parses_section_center_responsive_prop() {
    let tree = parse_page(
        r#"page landingPage
  Section centerX:{ xs:false md:true }
    Text
      "Hero""#,
    )
    .expect("tree");

    let ViewNode::Section { props, .. } = tree else {
        panic!("section");
    };
    let center = props.center_x.expect("center");
    assert_eq!(center.entries[0].breakpoint, Breakpoint::Xs);
    assert!(!center.entries[0].value);
    assert_eq!(center.entries[1].breakpoint, Breakpoint::Md);
    assert!(center.entries[1].value);
}

#[test]
fn parses_section_gap_responsive_prop() {
    let tree = parse_page(
        r#"page landingPage
  Section gap:{ xs:2 md:4 }
    Text
      "Hero""#,
    )
    .expect("tree");

    let ViewNode::Section { props, .. } = tree else {
        panic!("section");
    };
    let gap = props.gap.expect("gap");
    assert_eq!(gap.entries[0].breakpoint, Breakpoint::Xs);
    assert_eq!(
        gap.entries[0].value,
        GapValue::Single(GapSize::Scale(ScaleValue(4)))
    );
    assert_eq!(gap.entries[1].breakpoint, Breakpoint::Md);
    assert_eq!(
        gap.entries[1].value,
        GapValue::Single(GapSize::Scale(ScaleValue(8)))
    );
}

#[test]
fn rejects_responsive_section_boxed_prop() {
    let error = parse_page(
        r#"page landingPage
  Section boxed:{ xs:true md:false }
    Text
      "Hero""#,
    )
    .expect_err("boxed");

    assert!(
        error
            .to_string()
            .contains("invalid value for prop `boxed`: expected boolean")
    );
}

#[test]
fn rejects_invalid_show_visibility_conditions() {
    let non_bool = parse_page(
        r#"page readyPage
  signal profile value:{ name:"" }
  Text show:profile.name
    "Ready""#,
    )
    .expect_err("non bool");
    assert!(
        non_bool
            .to_string()
            .contains("invalid signal path `profile.name` in `show`: expected bool")
    );

    let responsive_string = parse_page(
        r#"page readyPage
  Text show:{ xs:"false" }
    "Ready""#,
    )
    .expect_err("responsive string");
    assert!(
        responsive_string
            .to_string()
            .contains("invalid value for prop `show`: expected boolean")
    );
}
