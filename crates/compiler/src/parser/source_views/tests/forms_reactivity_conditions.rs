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
fn accepts_calculated_card_animation_from_each_item_paths() {
    let tree = parse_page(
        r#"page cardAnimationPage
  const cardAnimations value:[{ id:"one" label:"One" animation:"slideUp" }]
  each in:cardAnimations as:card key:card.id
    Card variant:"solid" scheme:"surface" animation:card.animation
      Text
        "Card""#,
    )
    .expect("dynamic card animation");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Each { children, .. } = &children[0] else {
        panic!("each");
    };
    let ViewNode::Card { props, .. } = &children[0] else {
        panic!("card");
    };
    assert_eq!(props.style.animation(), Some(ViewAnimation::None));
    assert_eq!(
        props.style.animation_binding.as_ref().map(|binding| binding.path.as_str()),
        Some("card.animation")
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
  Section id:"hero" background:{ xs:"aurora" md:"aurora" } color:"backgroundText" boxed:true p:8
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
