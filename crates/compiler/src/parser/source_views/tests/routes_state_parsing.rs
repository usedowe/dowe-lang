#[test]
fn rejects_nested_view_groups() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/views/routes.dowe"),
        r#"views viewRoutes
  group path:"/" layout:RootLayout
    group path:"admin" layout:AdminLayout
      route path:"" page:DashboardPage"#
            .to_string(),
    )
    .expect("source");

    let error = match view_declarations(&file) {
        Ok(_) => panic!("nested view group must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains(
            "view route groups cannot contain another `group`; use sibling groups or direct `route` children"
        ));
}

#[test]
fn parses_sequential_request_function_and_toast() {
    let tree = parse_page(
            r#"page authPage
  signal session value:{}
  signal loginForm value:{ email:"" password:"" }
  fn login
    request res method:"POST" route:"/api/auth/login" body:loginForm
    if res.ok
      set session value:res.data
      set loginForm value:{ email:"" password:"" }
      toast value:{ type:"success" title:"Success" message:"Signed in." visible:true } variant:"outlined" scheme:"surface" position:"top-right"
    else
      toast value:{ type:"error" title:"Error" message:"Login failed." visible:true }
  Text
    "Auth""#,
        )
        .expect("tree");
    let ViewNode::Scope { actions, .. } = tree else {
        panic!("scope")
    };
    let ViewActionKind::Sequence(statements) = &actions[0].kind else {
        panic!("sequence")
    };
    assert_eq!(statements.len(), 2);
    let ViewFunctionStatement::If { success, .. } = &statements[1] else {
        panic!("branch")
    };
    let ViewFunctionStatement::Toast(toast) = &success[2] else {
        panic!("toast")
    };
    assert_eq!(toast.variant.as_deref(), Some("outlined"));
    assert_eq!(toast.scheme.as_deref(), Some("surface"));
    assert_eq!(toast.position.as_deref(), Some("top-right"));
}

#[test]
fn parses_redirect_in_view_function_and_init_branch() {
    let function = parse_page(
        r#"page LoginPage
  fn finish
    redirect path:"/dashboard"
  Button onClick:finish
    "Continue""#,
    )
    .expect("redirect function");
    let ViewNode::Scope { actions, .. } = function else {
        panic!("scope")
    };
    assert!(matches!(
        &actions[0].kind,
        ViewActionKind::Sequence(statements)
            if matches!(statements.first(), Some(ViewFunctionStatement::Redirect { path }) if path == "/dashboard")
    ));

    let init = parse_page(
        r#"page HomePage
  init
    request session method:"GET" route:"/api/session"
    if session.ok
      toast value:{ type:"success" message:"Ready" visible:true }
    else
      redirect path:"/login"
  Text
    "Home""#,
    )
    .expect("redirect init branch");
    let ViewNode::Scope { actions, .. } = init else {
        panic!("scope")
    };
    let ViewActionKind::Sequence(statements) = &actions[0].kind else {
        panic!("sequence")
    };
    let ViewFunctionStatement::If { error, .. } = &statements[1] else {
        panic!("branch")
    };
    assert!(matches!(
        error.first(),
        Some(ViewFunctionStatement::Redirect { path }) if path == "/login"
    ));
}

#[test]
fn rejects_invalid_redirect_statements() {
    for (source, expected) in [
        (
            "page HomePage\n  fn leave\n    redirect\n  Text\n    \"Home\"",
            "`path` must be a quoted string",
        ),
        (
            "page HomePage\n  fn leave\n    redirect path:\"login\"\n  Text\n    \"Home\"",
            "`redirect` path must start with `/`",
        ),
        (
            "page HomePage\n  fn leave\n    redirect path:\"/login\" mode:\"push\"\n  Text\n    \"Home\"",
            "`redirect` does not support `mode`",
        ),
    ] {
        let error = parse_page(source).expect_err("invalid redirect");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn rejects_non_card_global_toast_surface_values() {
    let variant = parse_page(
        r#"page HomePage
  fn notify
    toast value:{ type:"info" message:"Saved" visible:true } variant:"line"
  Text
    "Home""#,
    )
    .expect_err("toast line variant");
    assert!(variant.to_string().contains("toast variant"));

    let scheme = parse_page(
        r#"page HomePage
  fn notify
    toast value:{ type:"info" message:"Saved" visible:true } scheme:"primaryText"
  Text
    "Home""#,
    )
    .expect_err("toast scheme");
    assert!(scheme.to_string().contains("toast scheme"));
}

#[test]
fn parses_multiple_page_root_nodes_as_one_logical_scope() {
    let tree = parse_page(
        r#"page landingPage
  Section
    Text
      "First"
  Section
    Text
      "Second"
  Section
    Text
      "Third""#,
    )
    .expect("page");
    let ViewNode::Scope {
        signals,
        actions,
        children,
        ..
    } = tree
    else {
        panic!("page scope");
    };
    assert!(signals.is_empty());
    assert!(actions.is_empty());
    assert_eq!(children.len(), 3);
    assert!(
        children
            .iter()
            .all(|child| matches!(child, ViewNode::Section { .. }))
    );
}

#[test]
fn accepts_flex_item_values_on_supported_layout_components() {
    parse_page(
        r#"page flexPage
  Section h:"vh-0"
    Box flex:"initial"
      Text
        "Box"
    Flex flex:"auto"
      Card flex:"none"
        Text
          "Card"
    Grid flex:{ xs:1 md:"none" }
      Grid flex:1
        Text
          "Grid""#,
    )
    .expect("flex items");
}

#[test]
fn accepts_direct_layout_and_page_metadata_without_adding_visual_roots() {
    parse_page(
        r#"layout HomeLayout
  meta name:"title" content:"Dowe"
  meta name:"og:image" content:"/images/social.png"
  Scaffold
    main
      children"#,
    )
    .expect("layout metadata");
    parse_page(
        r#"page ViewsPage
  meta name:"title" content:"Views | Dowe"
  meta name:"description" content:"Build fullstack views with Dowe."
  Title
    "Views""#,
    )
    .expect("page metadata");
}

#[test]
fn rejects_invalid_direct_view_metadata() {
    for (source, expected) in [
        (
            "page InvalidPage\n  meta content:\"Dowe\"\n  Text\n    \"Invalid\"",
            "missing `name` on `meta`",
        ),
        (
            "page InvalidPage\n  meta name:\"title\" content:\"\"\n  Text\n    \"Invalid\"",
            "`content` must not be empty",
        ),
        (
            "page InvalidPage\n  meta name:\"author\" content:\"Dowe\"\n  Text\n    \"Invalid\"",
            "unsupported meta name `author`",
        ),
        (
            "page InvalidPage\n  meta name:\"title\" content:\"Dowe\" media:\"all\"\n  Text\n    \"Invalid\"",
            "unknown prop `media` on `meta`",
        ),
        (
            "page InvalidPage\n  meta name:\"title\" content:titleSignal\n  Text\n    \"Invalid\"",
            "expected quoted static string literal",
        ),
        (
            "page InvalidPage\n  meta name:\"title\" content:\"One\"\n  meta name:\"title\" content:\"Two\"\n  Text\n    \"Invalid\"",
            "duplicate meta name `title`",
        ),
        (
            "page InvalidPage\n  meta invalid name:\"title\" content:\"Dowe\"\n  Text\n    \"Invalid\"",
            "accepts only `name` and `content` props and no children",
        ),
        (
            "page InvalidPage\n  meta name:\"title\" content:\"Dowe\"\n    Text\n      \"Invalid\"",
            "accepts only `name` and `content` props and no children",
        ),
        (
            "page InvalidPage\n  Box\n    meta name:\"title\" content:\"Nested\"",
            "unknown component `meta`",
        ),
    ] {
        let error = parse_page(source).expect_err("invalid metadata");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn parses_immutable_view_constants_for_table_data() {
    let tree = parse_page(
        r#"page catalog
  const rows value:[{ id:"one" name:"Starter" }]
  Table data:rows
    column field:"name" label:"Name""#,
    )
    .expect("constant table");
    let ViewNode::Scope {
        constants, signals, ..
    } = tree
    else {
        panic!("constant scope");
    };
    assert_eq!(constants.len(), 1);
    assert_eq!(constants[0].name, "rows");
    assert!(signals.is_empty());
}
