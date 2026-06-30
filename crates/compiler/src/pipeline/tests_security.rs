    #[test]
    fn rejects_unsupported_actions_inside_views() {
        assert_compile_error(
            r#"page loginPage
  Box
    dowe.spawn command:"echo""#,
            "unknown component `dowe.spawn`",
        );
    }

    #[test]
    fn rejects_unknown_signal_bindings_in_views() {
        assert_compile_error(
            r#"page loginPage
  signal blog value:{ title:"" }
  Box
    Input bind:missing.title"#,
            "unknown signal path `missing.title`",
        );
    }

    #[test]
    fn rejects_unknown_signal_fields_in_views() {
        assert_compile_error(
            r#"page loginPage
  signal blog value:{ title:"" }
  Box
    Input bind:blog.content"#,
            "unknown signal path `blog.content`",
        );
    }

    #[test]
    fn rejects_unknown_button_actions_in_views() {
        assert_compile_error(
            r#"page loginPage
  signal blog value:{ title:"" }
  Box
    Button onClick:saveBlog
      "Save""#,
            "unknown action `saveBlog`",
        );
    }

    #[test]
    fn compiles_console_actions_by_runtime_surface() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let server = fs::read_to_string(temp.path().join("src/main.dowe")).expect("server");
        fs::write(
            temp.path().join("src/main.dowe"),
            server.replace(
                "return response text:\"Hello User {req.params.id}!\"",
                "warn \"User handler\" 42 true null { ready:true }\n        return response text:\"Hello User {req.params.id}!\"",
            ),
        )
        .expect("server");

        let project = compile_dev(temp.path()).expect("project");
        let user = project
            .backend
            .find_endpoint(&HttpMethod::Get, "/users/123")
            .expect("user");

        assert_eq!(project.backend.init_action.statements.len(), 1);
        assert_log(
            &project.backend.init_action.statements[0],
            ServerLogLevel::Log,
            &[ServerLogValue::String("Server inicializado".to_string())],
        );
        assert_eq!(user.endpoint.action.statements.len(), 1);
        assert_log(
            &user.endpoint.action.statements[0],
            ServerLogLevel::Warn,
            &[
                ServerLogValue::String("User handler".to_string()),
                ServerLogValue::Number("42".to_string()),
                ServerLogValue::Boolean(true),
                ServerLogValue::Null,
                ServerLogValue::JsonLiteral("{ ready:true }".to_string()),
            ],
        );
    }

    #[test]
    fn rejects_unsupported_actions_in_server() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let server = fs::read_to_string(temp.path().join("src/main.dowe")).expect("server");
        fs::write(
            temp.path().join("src/main.dowe"),
            server.replace(
                "log \"Server inicializado\"",
                "console.debug \"Server inicializado\"",
            ),
        )
        .expect("server");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("unsupported server action")
        );
    }

    #[test]
    fn rejects_client_style_actions_in_server() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let server = fs::read_to_string(temp.path().join("src/main.dowe")).expect("server");
        fs::write(
            temp.path().join("src/main.dowe"),
            server.replace(
                "log \"Server inicializado\"",
                "window.alert \"Server inicializado\"",
            ),
        )
        .expect("server");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("unsupported server action")
        );
    }

    #[test]
    fn rejects_spawn_actions_in_server_until_contract_exists() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let server = fs::read_to_string(temp.path().join("src/main.dowe")).expect("server");
        fs::write(
            temp.path().join("src/main.dowe"),
            server.replace(
                "log \"Server inicializado\"",
                "dowe.spawn command:\"echo\" pty:true stderr:\"pipe\"",
            ),
        )
        .expect("server");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("unsupported server action")
        );
    }

    #[test]
    fn rejects_typescript_and_tsx_sources_under_src() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(temp.path().join("src/legacy.tsx"), "<Box />").expect("legacy");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("unsupported source `src/legacy.tsx`"));
        assert!(error.to_string().contains("use `.dowe` files"));
    }

    #[test]
    fn rejects_i18n_key_without_translation_catalogs() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    children"#,
            r#"page loginPage
  Text i18n:"home.hero.title"
    "Dowe builds systems.""#,
        );

        let error = compile_dev(temp.path()).expect_err("compile error");

        assert!(
            error
                .to_string()
                .contains("requires catalogs under `src/i18n/<locale>.dowe`")
        );
    }

    #[test]
    fn rejects_i18n_key_missing_from_any_locale() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    children"#,
            r#"page loginPage
  Text i18n:"home.hero.title"
    "Dowe builds systems.""#,
        );
        write_translation_catalogs(temp.path());
        fs::write(
            temp.path().join("src/i18n/es.dowe"),
            r#"translations
  home
    other "Otro""#,
        )
        .expect("spanish");

        let error = compile_dev(temp.path()).expect_err("compile error");

        assert!(
            error
                .to_string()
                .contains("translation key `home.hero.title` is missing for locale `es`")
        );
    }

    #[test]
    fn rejects_translation_catalogs_without_default_locale() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::create_dir_all(temp.path().join("src/i18n")).expect("i18n");
        fs::write(
            temp.path().join("src/i18n/en.dowe"),
            r#"translations
  home
    hero
      title "Dowe builds systems.""#,
        )
        .expect("english");

        let error = compile_dev(temp.path()).expect_err("compile error");

        assert!(
            error
                .to_string()
                .contains("must declare exactly one `translations default:true` locale")
        );
    }

    #[test]
    fn rejects_i18n_prop_on_unsupported_components() {
        assert_compile_error(
            r#"page loginPage
  Box i18n:"home.hero.title""#,
            "unknown prop `i18n` on `Box`",
        );
    }

    fn assert_log(statement: &ServerStatement, level: ServerLogLevel, values: &[ServerLogValue]) {
        match statement {
            ServerStatement::Log(log) => {
                assert_eq!(log.level, level);
                assert_eq!(log.values, values);
            }
            ServerStatement::RequestJson { .. } => panic!("expected log statement"),
            ServerStatement::Stdlib(_) => panic!("expected log statement"),
            ServerStatement::Http(_) => panic!("expected log statement"),
            ServerStatement::AgentChat(_) => panic!("expected log statement"),
            ServerStatement::WebSocketJson(_) => panic!("expected log statement"),
            ServerStatement::WebSocketSendJson(_) => panic!("expected log statement"),
            ServerStatement::WebSocketSseBridge(_) => panic!("expected log statement"),
            ServerStatement::Store(_) => panic!("expected log statement"),
            ServerStatement::Kv(_) => panic!("expected log statement"),
        }
    }

    fn write_fixture(root: &Path) {
        write_fixture_with_views(
            root,
            r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
            r#"page loginPage
  Box
    Text
      "Login""#,
        );
    }

    fn write_fixture_with_views(root: &Path, layout_source: &str, page_source: &str) {
        fs::create_dir_all(root.join("src/layouts")).expect("layouts");
        fs::create_dir_all(root.join("src/pages")).expect("pages");
        fs::write(
            root.join("src/main.dowe"),
            r#"main
  server port:8080
    route "/api/status"
      response text:"OK"
    route "/users/:id"
      handler req
        return response text:"Hello User {req.params.id}!"
    route "/api/posts"
      method GET
        return response text:"List posts"
      method POST async req
        let body = await req.json()
        return response json:{ created:true ...body }
    websocket "/ws"
      open ws
      message ws data
      close ws code reason
      drain ws
    init
      log "Server inicializado"
  desktop
    server port:4500
      route "/api/status"
        response text:"OK"
      init
        log "Desktop server inicializado""#,
        )
        .expect("server");
        fs::write(
            root.join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"

views
  route path:"/" layout:AuthLayout
    page path:"" component:loginPage"#,
        )
        .expect("views");
        fs::write(root.join("src/layouts/auth.dowe"), layout_source).expect("layout");
        fs::write(root.join("src/pages/login.dowe"), page_source).expect("page");
    }

    fn write_translation_catalogs(root: &Path) {
        fs::create_dir_all(root.join("src/i18n")).expect("i18n");
        fs::write(
            root.join("src/i18n/en.dowe"),
            r#"translations default:true
  home
    hero
      title "Dowe builds systems.""#,
        )
        .expect("english");
        fs::write(
            root.join("src/i18n/es.dowe"),
            r#"translations
  home
    hero
      title "Dowe construye sistemas.""#,
        )
        .expect("spanish");
    }

    fn write_fixture_with_two_pages(
        root: &Path,
        layout_source: &str,
        login_source: &str,
        signup_source: &str,
    ) {
        write_fixture_with_views(root, layout_source, login_source);
        fs::write(
            root.join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"
import signupPage from "./pages/signup"

views
  route path:"/" layout:AuthLayout
    page path:"" component:loginPage
    page path:"signup" component:signupPage"#,
        )
        .expect("views");
        fs::write(root.join("src/pages/signup.dowe"), signup_source).expect("signup");
    }

    fn assert_compile_error(page_source: &str, expected: &str) {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
            page_source,
        );

        let error = compile_dev(temp.path()).expect_err("compile error");
        let message = error.to_string();

        assert!(message.contains("src/pages/login.dowe"));
        assert!(message.contains(expected));
    }
