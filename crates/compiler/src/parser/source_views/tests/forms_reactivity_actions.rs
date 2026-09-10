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
fn parses_reactive_visual_props_and_events_for_form_controls() {
    let tree = parse_page(
        r#"page formPage
  signal variantChoice value:"outlined"
  signal schemeChoice value:"success"
  signal sizeChoice value:"lg"
  signal roundedChoice value:"full"
  signal form value:{ email:"" role:"" notes:"" avatar:"" password:"" phone:"" pin:"" bio:"" accepted:false color:"" date:"" start:"" end:"" choice:"" slider:0 }
  fn fieldChanged
    set form.email value:form.email
  Box
    Input bind:form.email variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    Select bind:form.role variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
      Option value:"admin" label:"Admin"
    ComboBox bind:form.role variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
      comboOption value:"admin" label:"Admin"
    CsvField variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged
      csvColumn name:"email" label:"Email"
    DragDrop variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged
      dragItem id:"draft" label:"Draft"
    Editor bind:form.notes variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    ImageCropper bind:form.avatar variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged
    Password bind:form.password variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    Phone bind:form.phone variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    Pin bind:form.pin variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    Textarea bind:form.bio variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    Checkbox bind:form.accepted variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged
    Color bind:form.color variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    Date bind:form.date variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    DateRange start:form.start end:form.end variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    RadioGroup bind:form.choice variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged
      item value:"basic" label:"Basic"
    Toggle bind:form.accepted variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged
    Slider bind:form.slider variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged onInput:fieldChanged
    Dropzone variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice onChange:fieldChanged"#,
    )
    .expect("reactive form controls");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Box { children, .. } = &children[0] else {
        panic!("box");
    };
    assert_eq!(children.len(), 19);

    for node in children {
        let style = match node {
            ViewNode::Input { props } | ViewNode::Select { props, .. } => props,
            ViewNode::Checkbox { props } => &props.style,
            ViewNode::Color { props } => &props.style,
            ViewNode::Date { props } => &props.style,
            ViewNode::DateRange { props } => &props.style,
            ViewNode::RadioGroup { props, .. } => &props.style,
            ViewNode::Toggle { props } => &props.style,
            ViewNode::Slider { props } => &props.style,
            ViewNode::Dropzone { props } => &props.style,
            ViewNode::ComboBox { props, .. } => &props.style,
            ViewNode::CsvField { props, .. } => &props.style,
            ViewNode::DragDrop { props, .. } => &props.style,
            ViewNode::Editor { props } => &props.style,
            ViewNode::ImageCropper { props } => &props.style,
            ViewNode::Password { props } => &props.style,
            ViewNode::Phone { props } => &props.style,
            ViewNode::Pin { props } => &props.style,
            ViewNode::Textarea { props } => &props.style,
            _ => panic!("unexpected form node"),
        };
        assert_eq!(style.reactive.variant.as_deref(), Some("variantChoice"));
        assert_eq!(style.reactive.scheme.as_deref(), Some("schemeChoice"));
        assert_eq!(style.reactive.size.as_deref(), Some("sizeChoice"));
        assert_eq!(style.reactive.rounded.as_deref(), Some("roundedChoice"));
        let element = dowe_components::node_element_props(node).expect("element props");
        assert_eq!(element.on_change.as_deref(), Some("fieldChanged"));
        if matches!(
            node,
            ViewNode::Select { .. } | ViewNode::Date { .. } | ViewNode::DateRange { .. }
        ) {
            assert_eq!(element.on_input.as_deref(), Some("fieldChanged"));
        }
    }
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

