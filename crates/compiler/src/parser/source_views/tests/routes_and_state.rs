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
    toast value:{ type:"info" message:"Saved" visible:true } scheme:"onPrimary"
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

    #[test]
    fn parses_each_over_an_immutable_view_constant() {
        let tree = parse_page(
            r#"page catalogPage
  const options value:[{ id:"primary" label:"Primary" }]
  Grid
    each in:options as:option key:option.id
      Button scheme:option.id
        "option.label""#,
        )
        .expect("constant each");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("constant scope");
        };
        let ViewNode::Grid { children, .. } = &children[0] else {
            panic!("grid");
        };
        assert!(
            matches!(&children[0], ViewNode::Each { item, collection, key, .. } if item == "option" && collection == "options" && key == "option.id")
        );
    }

    #[test]
    fn parses_each_props_independently_of_order() {
        parse_page(
            r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each key:option.id as:option in:options
    Text
      "option.id""#,
        )
        .expect("ordered each props");
    }

    #[test]
    fn rejects_positional_each_syntax() {
        let error = parse_page(
            r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each option in options key:option.id
    Text
      "option.id""#,
        )
        .expect_err("positional each");

        assert!(
            error
                .to_string()
                .contains("`each` must use `each in:collection as:item key:item.id`")
        );
    }

    #[test]
    fn rejects_invalid_each_props() {
        let quoted = parse_page(
            r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:"options" as:option key:option.id
    Text
      "option.id""#,
        )
        .expect_err("quoted collection");
        assert!(quoted.to_string().contains("`in` must be a reference"));

        let missing = parse_page(
            r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:options key:option.id
    Text
      "option.id""#,
        )
        .expect_err("missing alias");
        assert!(missing.to_string().contains("missing `as`"));

        let quoted_key = parse_page(
            r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:options as:option key:"option.id"
    Text
      "option.id""#,
        )
        .expect_err("quoted key");
        assert!(quoted_key.to_string().contains("`key` must be a reference"));

        let unknown = parse_page(
            r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:options as:option key:option.id index:index
    Text
      "option.id""#,
        )
        .expect_err("unknown each prop");
        assert!(
            unknown
                .to_string()
                .contains("unknown prop `index` on `each`")
        );
    }

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
