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
  signal formLogin value:{ email:"" password:"" accepted:false }
  fn submit
    validate formLogin
    request result method:"POST" route:"/api/login" body:formLogin
  Input bind:formLogin.email
    validate rule:"required" message:"Email is required."
  Password bind:formLogin.password
    validate rule:"required" message:"Password is required."
  Checkbox bind:formLogin.accepted
    validate rule:"required" message:"Accept terms."
  Button disabled:formLogin.isInvalid onClick:submit
    "Submit""#,
    )
    .expect("tree");
    let ViewNode::Scope { actions, .. } = &tree else {
        panic!("scope");
    };
    let ViewActionKind::Sequence(statements) = &actions[0].kind else {
        panic!("sequence");
    };
    assert!(
        matches!(&statements[0], ViewFunctionStatement::Validate { target } if target == "formLogin")
    );
    let forms = dowe_components::collect_view_forms(&tree);
    assert_eq!(forms.len(), 1);
    assert_eq!(forms[0].signal, "formLogin");
    assert_eq!(
        forms[0]
            .fields
            .iter()
            .map(|field| field.path.as_str())
            .collect::<Vec<_>>(),
        ["email", "password", "accepted"]
    );

    assert!(
        parse_page(
            r#"page invalidForm
  signal plain value:""
  fn submit
    validate plain
  Button disabled:plain.isInvalid
    "Submit""#,
        )
        .is_err()
    );
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

