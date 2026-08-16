    #[test]
    fn parses_request_route_blocks_and_api_base_default() {
        let tree = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  signal alert value:{ type:"info" message:"" visible:false }
  fn load
    request GET route:"/api/blogs" update:blogs autoload:true
      onError alert:"No se pudieron cargar los blogs"
  Box
    Text size:"md"
      "Crear y editar entradas usando signals, Input bind, Button onClick y Store.""#,
        )
        .expect("tree");
        let ViewNode::Scope {
            actions, children, ..
        } = tree
        else {
            panic!("scope");
        };
        let ViewActionKind::Request(request) = &actions[0].kind else {
            panic!("request");
        };

        assert_eq!(actions[0].name, "load");
        assert_eq!(request.path, "/api/blogs");
        assert_eq!(request.base_env.as_deref(), Some("BACKEND_URL"));
        assert_eq!(request.error_alert.as_deref(), Some("alert"));
        assert_eq!(
            request.error_message.as_deref(),
            Some("No se pudieron cargar los blogs")
        );
        assert!(request.autoload);
        assert!(matches!(
            &children[0],
            ViewNode::Box { children, .. }
                if matches!(&children[0], ViewNode::Text { value, .. }
                    if value == "Crear y editar entradas usando signals, Input bind, Button onClick y Store.")
        ));
    }

    #[test]
    fn parses_signal_validation_statement_and_derived_form_metadata() {
        let tree = parse_page(
            r#"page loginPage
  signal formLogin value:{ email:"" accepted:false }
  fn submit
    validate formLogin
    request result method:"POST" route:"/api/login" body:formLogin
  Input bind:formLogin.email
    validate rule:"required" message:"Email is required."
  Checkbox bind:formLogin.accepted
    validate rule:"required" message:"Accept terms."
  Button disabled:formLogin.isInvalid onClick:submit
    "Submit""#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = &tree else { panic!("scope"); };
        let ViewActionKind::Sequence(statements) = &actions[0].kind else { panic!("sequence"); };
        assert!(matches!(&statements[0], ViewFunctionStatement::Validate { target } if target == "formLogin"));
        let forms = dowe_components::collect_view_forms(&tree);
        assert_eq!(forms.len(), 1);
        assert_eq!(forms[0].signal, "formLogin");
        assert_eq!(forms[0].fields.iter().map(|field| field.path.as_str()).collect::<Vec<_>>(), ["email", "accepted"]);

        assert!(parse_page(
            r#"page invalidForm
  signal plain value:""
  fn submit
    validate plain
  Button disabled:plain.isInvalid
    "Submit""#,
        )
        .is_err());
    }

    #[test]
    fn parses_request_headers() {
        let tree = parse_page(
            r#"page blogsPage
  signal session value:{ authorization:"Bearer token" }
  signal draft value:{ title:"" }
  fn create
    request POST route:"/api/blogs/create" body:draft headers:{ Authorization:session.authorization XExample:"public" } update:draft
  Button onClick:create
    "Create""#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let ViewActionKind::Request(request) = &actions[0].kind else {
            panic!("request");
        };

        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers[0].name, "Authorization");
        assert!(matches!(
            &request.headers[0].value,
            dowe_components::ViewRequestHeaderValue::Signal(value)
                if value == "session.authorization"
        ));
        assert_eq!(request.headers[1].name, "XExample");
        assert!(matches!(
            &request.headers[1].value,
            dowe_components::ViewRequestHeaderValue::Static(value) if value == "public"
        ));
    }

    #[test]
    fn rejects_invalid_request_header_value() {
        let error = parse_page(
            r#"page blogsPage
  fn create
    request POST route:"/api/blogs/create" headers:{ Authorization:{ token:"nope" } }
  Button onClick:create
    "Create""#,
        )
        .expect_err("invalid header");

        assert!(
            error
                .message()
                .contains("`request headers` values must be strings or Signal references")
        );
    }

    #[test]
    fn rejects_unknown_request_header_signal() {
        let error = parse_page(
            r#"page blogsPage
  fn create
    request POST route:"/api/blogs/create" headers:{ Authorization:session.authorization }
  Button onClick:create
    "Create""#,
        )
        .expect_err("unknown header signal");

        assert!(error.message().contains("unknown request header source"));
    }

    #[test]
    fn parses_global_local_signal() {
        let tree = parse_page(
            r#"page authPage
  signal session scope:"global" storage:"local" value:{ authorization:"" }
  Text
    "Auth""#,
        )
        .expect("tree");
        let ViewNode::Scope { signals, .. } = tree else {
            panic!("scope");
        };

        assert_eq!(signals[0].scope, ViewSignalScope::Global);
        assert_eq!(signals[0].storage, ViewSignalStorage::Local);
    }

    #[test]
    fn rejects_local_storage_without_global_scope() {
        let error = parse_page(
            r#"page authPage
  signal session storage:"local" value:{ authorization:"" }
  Text
    "Auth""#,
        )
        .expect_err("storage");

        assert!(
            error
                .message()
                .contains("`signal storage:\"local\"` requires `scope:\"global\"`")
        );
    }

    #[test]
    fn parses_request_path_alias_and_success_block_target() {
        let tree = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  signal feedback value:{ type:"info" message:"" visible:false }
  fn create
    request POST path:"/api/blogs" update:blogs
      onSuccess target:feedback alert:"Blog creado"
  Box
    Text
      "Blogs""#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let ViewActionKind::Request(request) = &actions[0].kind else {
            panic!("request");
        };

        assert_eq!(request.path, "/api/blogs");
        assert_eq!(request.success_alert.as_deref(), Some("feedback"));
        assert_eq!(request.success_message.as_deref(), Some("Blog creado"));
    }

    #[test]
    fn parses_stdlib_set_action() {
        let tree = parse_page(
            r#"page profilePage
  signal form value:{ name:"  Ada  " }
  signal normalized value:""
  fn normalize
    set normalized source:str.trim value:form.name
  Box
    Text
      "Profile""#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let ViewActionKind::Assign(assign) = &actions[0].kind else {
            panic!("set");
        };
        let call = assign.call.as_ref().expect("stdlib call");

        assert_eq!(assign.target, "normalized");
        assert_eq!(assign.source, "str.trim");
        assert_eq!(call.namespace, "str");
        assert_eq!(call.function, "trim");
        assert_eq!(call.args[0].name, "value");
    }

    #[test]
    fn parses_svg_conversion_action() {
        let tree = parse_page(
            r#"page svgPage
  signal source value:"<svg viewBox='0 0 10 10'><path d='M0 0L10 10'/></svg>"
  signal output value:""
  signal preview value:""
  fn convert
    set output source:parse.svg value:source fallback:"" colors:"original" format:"source"
    set preview source:parse.svg value:source fallback:"" colors:"original" format:"data"
  Code content:"{output}" template:true"#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let ViewActionKind::Sequence(statements) = &actions[0].kind else {
            panic!("sequence");
        };
        let ViewFunctionStatement::Assign(assign) = &statements[0] else {
            panic!("source set");
        };
        let call = assign.call.as_ref().expect("stdlib call");

        assert_eq!(call.namespace, "parse");
        assert_eq!(call.function, "svg");
        assert_eq!(call.args.len(), 4);
        let ViewFunctionStatement::Assign(preview) = &statements[1] else {
            panic!("preview set");
        };
        assert_eq!(
            preview
                .call
                .as_ref()
                .expect("preview call")
                .args
                .iter()
                .find(|argument| argument.name == "format")
                .map(|argument| &argument.value),
            Some(&dowe_stdlib::StdlibValue::String("data".to_string()))
        );
    }

    #[test]
    fn parses_portable_standard_library_view_syntax() {
        let tree = parse_page(
            r#"page standardLibraryPage
  signal text value:"  value  "
  signal values value:[]
  signal result value:""
  fn run
    set result source:str.trim value:text
    set result source:math.sum values:values
    set result source:parse.int value:text fallback:0
    set result source:url.querySet value:text name:"page" param:text
    set result source:csv.parse value:text header:true
    set result source:sort.by values:values field:"score" direction:"desc"
    set result source:list.filterContains values:values field:"name" value:text
    set result source:json.get value:text path:"name" fallback:""
    set result source:date.now"#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let ViewActionKind::Sequence(statements) = &actions[0].kind else {
            panic!("sequence");
        };
        let names = statements
            .iter()
            .filter_map(|statement| match statement {
                ViewFunctionStatement::Assign(assign) => assign
                    .call
                    .as_ref()
                    .map(|call| call.name()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let names = names.iter().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "str.trim",
                "math.sum",
                "parse.int",
                "url.querySet",
                "csv.parse",
                "sort.by",
                "list.filterContains",
                "json.get",
                "date.now",
            ]
        );
    }

    #[test]
    fn parses_every_fill_emitted_by_svg_conversion() {
        let svg = r##"<svg viewBox="0 0 16 8">
<path d="M0 0L1 1Z" fill="#000001"/>
<path d="M1 0L2 1Z" fill="#000002"/>
<path d="M2 0L3 1Z" fill="#000003"/>
<path d="M3 0L4 1Z" fill="#000004"/>
<path d="M4 0L5 1Z" fill="#000005"/>
<path d="M5 0L6 1Z" fill="#000006"/>
<path d="M6 0L7 1Z" fill="#000007"/>
<path d="M7 0L8 1Z" fill="#000008"/>
</svg>"##;
        let call = dowe_stdlib::StdlibCall {
            namespace: "parse".to_string(),
            function: "svg".to_string(),
            args: vec![dowe_stdlib::StdlibArgument {
                name: "value".to_string(),
                value: dowe_stdlib::StdlibValue::String(svg.to_string()),
            }],
        };
        let converted = dowe_stdlib::evaluate(&call, |_| None)
            .expect("conversion")
            .as_str()
            .expect("source")
            .lines()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n");

        parse_page(&format!("page converted\n{converted}")).expect("converted svg");
    }

    #[test]
    fn parses_set_reference_negation_and_literals() {
        let tree = parse_page(
            r#"page menuPage
  signal openMenu value:false
  signal drawerVisible value:true
  fn copy
    set openMenu value:drawerVisible
  fn toggle
    set openMenu value:!openMenu
  fn open
    set openMenu value:true
  fn close
    set openMenu value:false
  Box
    Text
      "Menu""#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let sources = actions
            .iter()
            .map(|action| match &action.kind {
                ViewActionKind::Assign(action) => action.source.as_str(),
                _ => panic!("set"),
            })
            .collect::<Vec<_>>();

        assert_eq!(
            sources,
            [
                "drawerVisible",
                "!openMenu",
                "$dowe:bool:true",
                "$dowe:bool:false"
            ]
        );
    }

    #[test]
    fn keeps_bare_on_click_as_action_reference() {
        let tree = parse_page(
            r#"page menuPage
  signal openDrawer value:false
  fn openDrawer
    set openDrawer value:true
  IconButton label:"menu" variant:"ghost" onClick:openDrawer icon:"menu-dots""#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "openDrawer");
        let ViewActionKind::Assign(action) = &actions[0].kind else {
            panic!("set action");
        };
        assert_eq!(action.target, "openDrawer");
        assert_eq!(action.source, "$dowe:bool:true");
    }

    #[test]
    fn lowers_inline_on_click_state_updates() {
        let tree = parse_page(
            r#"page menuPage
  signal openDrawer value:false
  signal counter value:0
  signal name value:""
  Button onClick:{ set:openDrawer value:!openDrawer }
    "Toggle"
  Button onClick:{ set:counter add:1 }
    "Increment"
  Button onClick:{ set:name value:"Ada" }
    "Name"
  Button onClick:{ set:name append:"!" }
    "Append""#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };

        assert_eq!(actions.len(), 4);
        let assignments = actions
            .iter()
            .map(|action| {
                let ViewActionKind::Assign(assign) = &action.kind else {
                    panic!("inline set");
                };
                assign
            })
            .collect::<Vec<_>>();
        assert_eq!(assignments[0].target, "openDrawer");
        assert_eq!(assignments[0].source, "!openDrawer");
        assert_eq!(assignments[1].target, "counter");
        assert_eq!(assignments[1].source, "$dowe:onClick:add");
        assert_eq!(assignments[2].target, "name");
        assert_eq!(assignments[2].source, "$dowe:string:Ada");
        assert_eq!(assignments[3].target, "name");
        assert_eq!(assignments[3].source, "$dowe:onClick:append");
        assert_eq!(
            assignments[1].call.as_ref().expect("add call").function,
            "add"
        );
        assert_eq!(
            assignments[3].call.as_ref().expect("append call").function,
            "join"
        );
    }

    #[test]
    fn rejects_inline_on_click_add_for_non_number() {
        let error = parse_page(
            r#"page menuPage
  signal label value:"Menu"
  Button onClick:{ set:label add:1 }
    "Open""#,
        )
        .expect_err("non-number inline target");

        assert!(
            error
                .message()
                .contains("invalid signal path `label` in `onClick add target`: expected number")
        );
    }

    #[test]
    fn rejects_inline_on_click_append_for_non_string() {
        let error = parse_page(
            r#"page menuPage
  signal counter value:0
  Button onClick:{ set:counter append:"!" }
    "Open""#,
        )
        .expect_err("non-string inline target");

        assert!(
            error.message().contains(
                "invalid signal path `counter` in `onClick append target`: expected string"
            )
        );
    }

    #[test]
    fn rejects_assign_view_action() {
        let error = parse_page(
            r#"page menuPage
  signal openMenu value:false
  fn open
    assign openMenu source:true
  Box
    Text
      "Menu""#,
        )
        .expect_err("assign");

        assert!(
            error
                .message()
                .contains("`assign` was replaced by `set target value:<value>`")
        );
    }

    #[test]
    fn rejects_negating_a_non_boolean_set_value() {
        let error = parse_page(
            r#"page menuPage
  signal count value:1
  fn toggle
    set count value:!count
  Box
    Text
      "Menu""#,
        )
        .expect_err("non-boolean set value");

        assert!(
            error
                .message()
                .contains("`set value:!count` must reference a boolean")
        );
    }
