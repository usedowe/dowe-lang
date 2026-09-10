#[test]
fn parses_select_options_from_an_immutable_view_constant() {
    let tree = parse_page(
        r#"page catalogPage
  const options value:[{ id:"primary" value:"primary" label:"Primary" }]
  Select
    each in:options as:option key:option.id
      Option value:option.value label:option.label"#,
    )
    .expect("constant select options");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("constant scope");
    };
    assert!(matches!(
        &children[0],
        ViewNode::Select {
            option_each: Some(option_each),
            ..
        } if option_each.collection == "options"
    ));
}

#[test]
fn rejects_constant_bindings() {
    let error = parse_page(
        r#"page settings
  const name value:"Dowe"
  Input bind:name"#,
    )
    .expect_err("constant binding");
    assert!(
        error
            .to_string()
            .contains("constant path `name` cannot be used in `bind`")
    );
}

#[test]
fn rejects_signal_and_constant_name_collisions() {
    let error = parse_page(
        r#"page settings
  const name value:"Dowe"
  signal name value:"Dowe"
  Text
    "Settings""#,
    )
    .expect_err("duplicate view value");
    assert!(error.to_string().contains("duplicate view value `name`"));
}

#[test]
fn rejects_multiple_layout_root_nodes() {
    let error = parse_page(
        r#"layout AppLayout
  Box
    children
  Box
    Text
      "Footer""#,
    )
    .expect_err("layout roots");
    assert!(
        error
            .to_string()
            .contains("layout exports must contain one root view node")
    );
}

#[test]
fn parses_layout_init_and_splash_boundary() {
    let tree = parse_page(
        r#"layout AppLayout
  signal isLoading value:true
  init
    request session method:"GET" route:"/api/session"
    if session.ok
      set isLoading value:false
    else
      set isLoading value:false
  Scaffold
    main
      children
  Splash bind:isLoading
    Section
      Text
        "Loading application""#,
    )
    .expect("layout init and Splash");

    let ViewNode::Scope {
        actions, children, ..
    } = tree
    else {
        panic!("layout scope");
    };
    assert_eq!(actions.len(), 1);
    assert!(actions[0].is_init());
    assert!(matches!(
        &actions[0].kind,
        ViewActionKind::Sequence(statements)
            if matches!(statements.first(), Some(dowe_components::ViewFunctionStatement::Request { result, .. }) if result == "session")
                && matches!(statements.get(1), Some(dowe_components::ViewFunctionStatement::If { result, .. }) if result == "session")
    ));
    let ViewNode::Splash {
        binding,
        initial,
        content,
        children,
    } = &children[0]
    else {
        panic!("Splash boundary");
    };
    assert_eq!(binding, "isLoading");
    assert!(*initial);
    assert!(matches!(&content[0], ViewNode::Scaffold { .. }));
    assert!(matches!(&children[0], ViewNode::Section { .. }));
}

#[test]
fn parses_page_splash_with_multiple_normal_roots() {
    let tree = parse_page(
        r#"page UsersPage
  signal isLoading value:false
  Section
    Text
      "Users"
  Section
    Text
      "Results"
  Splash bind:isLoading"#,
    )
    .expect("page Splash");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("page scope");
    };
    let ViewNode::Splash {
        initial,
        content,
        children,
        ..
    } = &children[0]
    else {
        panic!("Splash boundary");
    };
    assert!(!initial);
    assert_eq!(content.len(), 2);
    assert!(children.is_empty());
}

#[test]
fn rejects_invalid_view_init_and_splash_forms() {
    let duplicate_init = parse_page(
        r#"page HomePage
  init
    toast value:{ type:"info" title:"First" message:"First" visible:true }
  init
    toast value:{ type:"info" title:"Second" message:"Second" visible:true }
  Text
    "Home""#,
    )
    .expect_err("duplicate init");
    assert!(duplicate_init.to_string().contains("one `init` hook"));

    let duplicate_splash = parse_page(
        r#"page HomePage
  signal loading value:true
  Text
    "Home"
  Splash bind:loading
  Splash bind:loading"#,
    )
    .expect_err("duplicate Splash");
    assert!(
        duplicate_splash
            .to_string()
            .contains("only one root `Splash`")
    );

    let named_init = parse_page(
        r#"page HomePage
  fn init
    set ready value:true
  signal ready value:false
  Text
    "Home""#,
    )
    .expect_err("named init");
    assert!(
        named_init
            .to_string()
            .contains("`init` is a reserved view hook")
    );

    let non_boolean = parse_page(
        r#"page HomePage
  signal loading value:"yes"
  Text
    "Home"
  Splash bind:loading"#,
    )
    .expect_err("non-boolean Splash binding");
    assert!(
        non_boolean
            .to_string()
            .contains("boolean Signal or View Store")
    );

    let nested = parse_page(
        r#"page HomePage
  signal loading value:true
  Section
    Splash bind:loading"#,
    )
    .expect_err("nested Splash");
    assert!(
        nested
            .to_string()
            .contains("direct child of a layout or page"),
        "{nested}"
    );

    let layout_roots = parse_page(
        r#"layout AppLayout
  signal loading value:true
  Scaffold
    main
      children
  Section
    children
  Splash bind:loading"#,
    )
    .expect_err("multiple normal layout roots with Splash");
    assert!(
        layout_roots
            .to_string()
            .contains("layout exports must contain one root view node")
    );
}
