    use super::compile_dev;
    use crate::model::{
        EndpointBehavior, EnvironmentValueSource, EnvironmentVisibility, HttpMethod,
        ServerLogLevel, ServerLogValue, ServerStatement,
    };
    use crate::parser::validate_design_copilot_dowe;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn compiles_example_project_and_writes_chunks() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());

        let project = compile_dev(temp.path()).expect("project");

        assert_eq!(project.backend.port, 8080);
        assert!(project.backend.has_websocket("/ws"));
        assert_eq!(project.web.pages[0].route_path, "/");
        assert_eq!(project.web.pages[0].layout_text, "Layout");
        assert_eq!(project.web.pages[0].page_text, "Login");
        assert!(project.web.pages[0].body_html.contains(r#"<p class="text-md">Layout</p>"#));
        assert!(project.web.pages[0].body_html.contains(r#"<p class="text-md">Login</p>"#));
        assert!(
            project.web.pages[0]
                .body_html
                .contains(r#"data-dowe-boundary="page:"#)
        );
        assert!(temp.path().join(".dowe/web/chunks/layouts").exists());
        assert!(temp.path().join(".dowe/web/chunks/pages").exists());
        assert!(temp.path().join(".dowe/web/index.html").exists());
        assert!(temp.path().join(".dowe/web/pages/index.html").exists());
        let manifest =
            fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("manifest");
        assert!(manifest.contains(r#""path":"/""#));
        assert!(manifest.contains(r#""staticFile":"web/pages/index.html""#));
        assert!(manifest.contains(r#""cssChunks""#));
        assert_eq!(project.web.chunks.len(), 2);
    }

    #[test]
    fn compiles_i18n_catalogs_for_web_and_native_targets() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    children"#,
            r#"page loginPage
  Title i18n:"home.hero.title"
    "Dowe builds systems.""#,
        );
        write_translation_catalogs(temp.path());

        let project = compile_dev(temp.path()).expect("project");
        let body = &project.web.pages[0].body_html;

        assert_eq!(project.translations.default_locale.as_deref(), Some("en"));
        assert_eq!(project.web.translation_chunks.len(), 2);
        assert!(body.contains(r#"data-dowe-i18n="home.hero.title""#));
        assert!(body.contains("Dowe builds systems."));
        assert!(project.web.router_js.contains("navigator.languages"));
        assert!(project.web.router_js.contains("hydrateTranslations"));
        assert!(
            project
                .web
                .translation_chunks
                .iter()
                .any(|chunk| chunk.content.contains("Dowe construye sistemas."))
        );

        let manifest =
            fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("manifest");
        assert!(manifest.contains(r#""translationChunks""#));
        assert!(manifest.contains(r#""defaultLocale":"en""#));
        assert!(temp.path().join(".dowe/apps/desktop/web/chunks/i18n").is_dir());

        let android = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
        )
        .expect("android");
        assert!(android.contains("stringResource(R.string.dowe_home_hero_title_"));
        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(android_dev.contains("getString(R.string.dowe_home_hero_title_"));
        let android_spanish = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/res/values-es/strings.xml"),
        )
        .expect("android spanish");
        assert!(android_spanish.contains("Dowe construye sistemas."));

        let ios = ios_swift_output(temp.path());
        assert!(ios.contains(r#"String(localized: "home.hero.title")"#));
        let ios_spanish = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/ios/es.lproj/Localizable.strings"),
        )
        .expect("ios spanish");
        assert!(ios_spanish.contains("Dowe construye sistemas."));
    }

    #[test]
    fn compiles_platform_specific_view_routes() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::create_dir_all(temp.path().join("src/layouts")).expect("layouts");
        fs::create_dir_all(temp.path().join("src/pages")).expect("pages");
        fs::write(
            temp.path().join("src/layouts/marketing.dowe"),
            r#"layout MarketingLayout
  Box
    Text
      "Marketing"
    children"#,
        )
        .expect("marketing layout");
        fs::write(
            temp.path().join("src/pages/landing.dowe"),
            r#"page landingPage
  Box
    Text
      "Landing""#,
        )
        .expect("landing");
        fs::write(
            temp.path().join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import MarketingLayout from "./layouts/marketing"
import loginPage from "./pages/login"
import landingPage from "./pages/landing"

views
  route path:"/" layout:MarketingLayout platform:"web"
    page path:"" component:landingPage
  route path:"/" layout:AuthLayout platform:["desktop","ios","android"]
    page path:"" component:loginPage"#,
        )
        .expect("views");

        let project = compile_dev(temp.path()).expect("project");

        assert_eq!(project.web.pages[0].layout_text, "Marketing");
        assert_eq!(project.web.pages[0].page_text, "Landing");
        assert_eq!(project.desktop_web.pages[0].layout_text, "Layout");
        assert_eq!(project.desktop_web.pages[0].page_text, "Login");
        assert_eq!(project.view_routes.web.len(), 1);
        assert_eq!(project.view_routes.desktop.len(), 1);
        assert_eq!(project.view_routes.android.len(), 1);
        assert_eq!(project.view_routes.ios.len(), 1);
        let web_index =
            fs::read_to_string(temp.path().join(".dowe/web/index.html")).expect("web index");
        let desktop_index = fs::read_to_string(
            temp.path().join(".dowe/apps/desktop/web/index.html"),
        )
        .expect("desktop index");
        assert!(web_index.contains("Landing"));
        assert!(!web_index.contains("Login"));
        assert!(desktop_index.contains("Login"));
        assert!(!desktop_index.contains("Landing"));
        let manifest =
            fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("manifest");
        assert!(manifest.contains(r#""routesByTarget""#));
        assert!(manifest.contains(r#""desktopWebManifest":"apps/desktop/web/manifest.json""#));
    }

    #[test]
    fn rejects_overlapping_platform_route_paths() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"

views
  route path:"/" layout:AuthLayout platform:["web","desktop"]
    page path:"" component:loginPage
  route path:"/" layout:AuthLayout platform:"web"
    page path:"" component:loginPage"#,
        )
        .expect("views");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("duplicate view path `/` for platform `web`")
        );
    }

    #[test]
    fn rejects_invalid_platform_values() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"

views
  route path:"/" layout:AuthLayout platform:["web","watch"]
    page path:"" component:loginPage"#,
        )
        .expect("views");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("got `watch`"));

        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"

views
  route path:"/" layout:AuthLayout platform:web
    page path:"" component:loginPage"#,
        )
        .expect("views");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("quoted static string literal")
        );

        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"

views
  route path:"/" layout:AuthLayout platform:["web","web"]
    page path:"" component:loginPage"#,
        )
        .expect("views");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("duplicate platform `web`"));
    }

    #[test]
    fn writes_source_language_artifacts() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());

        compile_dev(temp.path()).expect("project");

        let source = fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
            .expect("source format");
        let server = fs::read_to_string(temp.path().join(".dowe/language/server-surface.json"))
            .expect("server surface");
        let views = fs::read_to_string(temp.path().join(".dowe/language/views-surface.json"))
            .expect("views surface");
        let config = fs::read_to_string(temp.path().join(".dowe/language/config-surface.json"))
            .expect("config surface");

        assert!(source.contains(r#""extension": ".dowe""#));
        assert!(source.contains(r#""declaredTypes""#));
        assert!(source.contains(r#""unsupportedAuthoringExtensions""#));
        assert!(source.contains(r#""packages": "rejected""#));
        assert!(server.contains(r#""root": "src/main.dowe""#));
        assert!(server.contains(r#""req.json()""#));
        assert!(server.contains(r#""resolvedLogValues": true"#));
        assert!(server.contains(r#""let body:Type = await req.json()""#));
        assert!(server.contains(r#""nodeRuntime": false"#));
        assert!(views.contains(r#""root": "src/views.dowe""#));
        assert!(views.contains(r#""Box""#));
        assert!(views.contains(r#""Alert""#));
        assert!(views.contains(r#""Svg""#));
        assert!(views.contains(r#""Path""#));
        assert!(views.contains(r#""Code""#));
        assert!(views.contains(r#""Video""#));
        assert!(views.contains(r#""Divider""#));
        assert!(views.contains(r#""Input bind:signal.field""#));
        assert!(views.contains(r#""signalPathValidation""#));
        assert!(views.contains(r#""signal rows type:Row[] value:[]""#));
        assert!(views.contains(r#""routing""#));
        assert!(views.contains("platform values"));
        assert!(!views.contains(r#""Body""#));
        assert!(views.contains(r#""children""#));
        assert!(views.contains(r#""serverApisAvailable": false"#));
        assert!(config.contains(r#""root": "src/config.dowe""#));
        assert!(config.contains(r#""obsoleteConfig": "dowe.json""#));
        assert!(config.contains(r#""defaultTheme": "light""#));
        assert!(config.contains(r#""cors""#));
        assert!(config.contains(r#""devOrigins""#));
        assert!(!temp.path().join(".dowe/tsconfig.json").exists());
        assert!(!temp.path().join(".dowe/types").exists());
    }

    #[test]
    fn rejects_root_dowe_json_configuration() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("dowe.json"),
            r#"{"fonts":{"default":"inter","install":["inter"]}}"#,
        )
        .expect("json config");

        let error = compile_dev(temp.path()).expect_err("error");
        let message = error.to_string();

        assert!(message.contains("dowe.json"));
        assert!(message.contains("src/config.dowe"));
        assert!(message.contains("no longer supported"));
    }

    #[test]
    fn rejects_legacy_server_dowe_entry() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(temp.path().join("src/server.dowe"), "main\n").expect("legacy server");

        let error = compile_dev(temp.path()).expect_err("error");
        let message = error.to_string();

        assert!(message.contains("src/server.dowe"));
        assert!(message.contains("src/main.dowe"));
    }

    #[test]
    fn rejects_invalid_config_dowe_theme() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r##"config
  fonts default:"inter" install:["inter"]
  design defaultTheme:"dark"
    theme name:"light"
      colors primary:"#000000""##,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("default theme `dark` is not declared"));
    }

    #[test]
    fn rejects_unquoted_static_config_strings() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:inter install:["inter"]"#,
        )
        .expect("config");

        let font_error = compile_dev(temp.path()).expect_err("font error");
        assert!(
            font_error
                .to_string()
                .contains("quoted static string literal")
        );

        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:server methods:["GET"]"#,
        )
        .expect("config");

        let cors_error = compile_dev(temp.path()).expect_err("cors error");
        assert!(
            cors_error
                .to_string()
                .contains("quoted static string literal")
        );
    }

    #[test]
    fn compiles_app_metadata_from_config() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  app name:"Clinic Desk" bundle:"com.example.clinic"
  fonts default:"inter" install:["inter"]"#,
        )
        .expect("config");

        let project = compile_dev(temp.path()).expect("project");
        let apps_manifest =
            fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("manifest");
        let android_gradle =
            fs::read_to_string(temp.path().join(".dowe/apps/android/app/build.gradle.kts"))
                .expect("android gradle");
        let android_manifest =
            fs::read_to_string(temp.path().join(".dowe/apps/android/dev/AndroidManifest.xml"))
                .expect("android manifest");
        let android_activity = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android activity");
        let ios_plist =
            fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("ios plist");
        let desktop_manifest = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/desktop/macos/dowe-desktop.json"),
        )
        .expect("desktop manifest");

        assert_eq!(project.app_config.name, "Clinic Desk");
        assert_eq!(project.app_config.bundle, "com.example.clinic");
        assert!(apps_manifest.contains(r#""name":"Clinic Desk""#));
        assert!(apps_manifest.contains(r#""bundle":"com.example.clinic""#));
        assert!(android_gradle.contains(r#"applicationId = "com.example.clinic""#));
        assert!(android_manifest.contains(r#"package="com.example.clinic""#));
        assert!(android_manifest.contains(r#"android:label="Clinic Desk""#));
        assert!(android_activity.contains("import com.example.clinic.R;"));
        assert!(ios_plist.contains("<string>Clinic Desk</string>"));
        assert!(ios_plist.contains("<string>com.example.clinic</string>"));
        assert!(desktop_manifest.contains(r#""title":"Clinic Desk""#));
    }

    #[test]
    fn rejects_invalid_app_metadata() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  app name:"" bundle:"example""#,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("app.name"));

        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  app name:"Example" bundle:"example""#,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("app.bundle"));
    }

    #[test]
    fn replaces_stale_typecheck_artifacts_with_language_support() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());

        fs::create_dir_all(temp.path().join(".dowe/types")).expect("types");
        fs::write(temp.path().join(".dowe/tsconfig.json"), "stale").expect("root config");
        fs::write(temp.path().join(".dowe/types/views.d.ts"), "stale").expect("views types");

        compile_dev(temp.path()).expect("project");

        let first_source =
            fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
                .expect("source format");

        assert!(!temp.path().join(".dowe/tsconfig.json").exists());
        assert!(!temp.path().join(".dowe/types").exists());
        assert!(first_source.contains("dowe-source-format"));

        compile_dev(temp.path()).expect("project");

        let second_source =
            fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
                .expect("source format");

        assert_eq!(first_source, second_source);
    }

    #[test]
    fn ignores_agents_as_application_source_and_output() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::create_dir_all(temp.path().join(".agents/src/pages")).expect("agents");
        fs::write(temp.path().join(".agents/src/pages/bad.dowe"), "Stack\n")
        .expect("bad agent source");

        compile_dev(temp.path()).expect("project");

        let web_manifest =
            fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("web");
        let apps_manifest =
            fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("apps");

        assert!(!web_manifest.contains(".agents"));
        assert!(!apps_manifest.contains(".agents"));
    }

    #[test]
    fn parses_expected_endpoints() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());

        let project = compile_dev(temp.path()).expect("project");
        let status = project
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/status")
            .expect("status");
        let posts = project
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/posts")
            .expect("posts");

        assert_eq!(
            status.endpoint.behavior,
            EndpointBehavior::StaticText("OK".to_string())
        );
        assert_eq!(posts.endpoint.behavior, EndpointBehavior::CreatePostJson);
    }

    #[test]
    fn compiles_blog_crud_signals_actions_and_handlers() {
        let temp = TempDir::new().expect("tempdir");
        write_blog_fixture(temp.path());

        let project = compile_dev(temp.path()).expect("project");
        let list = project
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/blogs")
            .expect("list blogs");
        let create = project
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/blogs")
            .expect("create blog");
        let update = project
            .backend
            .find_endpoint(&HttpMethod::Patch, "/api/blogs/01HX")
            .expect("update blog");

        assert!(matches!(
            list.endpoint.behavior,
            EndpointBehavior::StoreActionJson(_)
        ));
        assert!(matches!(
            create.endpoint.behavior,
            EndpointBehavior::StoreActionJson(_)
        ));
        assert!(matches!(
            update.endpoint.behavior,
            EndpointBehavior::StoreActionJson(_)
        ));

        let page = project
            .web
            .pages
            .iter()
            .find(|page| page.route_path == "/blogs")
            .expect("blogs page");
        let alert_paths = attribute_values(&page.body_html, "data-dowe-alert-visible");
        let click_actions = attribute_values(&page.body_html, "data-dowe-click");
        let bindings = attribute_values(&page.body_html, "data-dowe-bind");
        let collections = attribute_values(&page.body_html, "data-dowe-each");

        assert_eq!(alert_paths.len(), 2);
        assert_ne!(alert_paths[0], alert_paths[1]);
        assert!(alert_paths.iter().all(|path| short_root(path, ".visible")));
        assert!(bindings.iter().any(|path| short_root(path, ".title")));
        assert!(collections.iter().any(|path| short_root(path, "")));
        assert!(click_actions.len() >= 3);
        assert!(!click_actions.iter().any(|action| *action == "close"));
        assert!(page.body_html.contains(r#"data-dowe-alert"#));

        let page_chunk = project
            .web
            .chunks
            .iter()
            .find(|chunk| chunk.id == page.page_chunk_id)
            .expect("page chunk");
        let layout_chunk = project
            .web
            .chunks
            .iter()
            .find(|chunk| chunk.id == page.layout_chunk_id)
            .expect("layout chunk");
        assert!(page_chunk.content.contains("dowePage"));
        assert!(page_chunk.content.contains(r#""id":"#));
        assert!(layout_chunk.content.contains("doweLayout"));
        assert!(page_chunk.content.contains("/api/blogs"));
        assert!(page_chunk.content.contains("\"create\""));
        assert!(project.web.router_js.contains("dowe:request"));
        assert!(project.web.router_js.contains("doweLayout"));

        let android_pages = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
        )
        .expect("android pages");
        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        let ios_pages = ios_swift_output(temp.path());

        assert!(android_pages.contains("DoweReactiveState"));
        assert!(android_pages.contains("state.text("));
        assert!(android_pages.contains("state.rows("));
        assert!(android_pages.contains("state.run("));
        assert!(android_pages.contains("DoweEnvironment.BACKEND_URL"));
        assert!(android_pages.contains(".verticalScroll(scrollState)"));
        assert!(android_pages.contains("if (path == \"item\" && item != null)"));
        assert!(!android_pages.contains("Text(\"alert.message\""));
        assert!(!android_pages.contains("Text(\"item.title\""));

        assert!(android_dev.contains("doweState"));
        assert!(android_dev.contains("doweRows("));
        assert!(android_dev.contains("doweRunAction("));
        assert!(android_dev.contains("DoweEnvironment.BACKEND_URL"));
        assert!(android_dev.contains("ScrollView scrollView"));
        assert!(android_dev.contains("doweInputBackground("));
        assert!(android_dev.contains("doweAdd(ViewGroup parent, View child, Integer gap"));
        assert!(!android_dev.contains("doweText(\"alert.message\""));
        assert!(!android_dev.contains("doweText(\"item.title\""));

        assert!(ios_pages.contains("DoweReactiveState"));
        assert!(ios_pages.contains("state.binding("));
        assert!(ios_pages.contains("state.rows("));
        assert!(ios_pages.contains("state.run("));
        assert!(ios_pages.contains("DoweEnvironment.BACKEND_URL"));
        assert!(ios_pages.contains("ScrollView {"));
        assert!(ios_pages.contains("DoweInputField(value: state.binding("));
        assert!(ios_pages.contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)"));
        assert!(ios_pages.contains("if path == \"item\", let item"));
        assert!(!ios_pages.contains("Text(\"alert.message\")"));
        assert!(!ios_pages.contains("Text(\"item.title\")"));
    }

    #[test]
    fn writes_app_targets_from_shared_view_tree() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());

        compile_dev(temp.path()).expect("project");

        let root = temp.path().join(".dowe/apps");
        assert!(root.join("desktop/macos/DoweMacOSApp.swift").exists());
        assert!(root.join("desktop/windows/DoweWindowsApp.cs").exists());
        assert!(root.join("desktop/linux/dowe_linux_app.c").exists());
        assert!(
            root.join("android/app/src/main/AndroidManifest.xml")
                .exists()
        );
        assert!(
            root.join("android/app/src/main/java/dev/dowe/generated/MainActivity.kt")
                .exists()
        );
        assert!(
            root.join("android/app/src/main/java/dev/dowe/generated/DoweRouting.kt")
                .exists()
        );
        assert!(
            root.join("android/app/src/main/java/dev/dowe/generated/DoweLayouts.kt")
                .exists()
        );
        assert!(
            root.join("android/app/src/main/java/dev/dowe/generated/DowePages.kt")
                .exists()
        );
        assert!(
            root.join("android/app/src/main/java/dev/dowe/generated/DoweTheme.kt")
                .exists()
        );
        assert!(
            root.join("android/app/src/main/java/dev/dowe/generated/DoweResponsive.kt")
                .exists()
        );
        assert!(root.join("ios/DoweIosApp.swift").exists());
        assert!(root.join("ios/DoweRouting.swift").exists());
        assert!(root.join("ios/DoweLayouts.swift").exists());
        assert!(root.join("ios/DowePages.swift").exists());
        assert!(root.join("ios/DoweTheme.swift").exists());
        assert!(root.join("ios/DoweResponsive.swift").exists());
        assert!(root.join("manifest.json").exists());

        let android = fs::read_to_string(
            root.join("android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
        )
        .expect("android views");
        let ios = ios_apps_swift_output(&root);
        let manifest = fs::read_to_string(root.join("manifest.json")).expect("apps manifest");

        assert!(android.contains("Column(modifier = Modifier.fillMaxWidth()) {"));
        assert!(android.contains("Text(\"Layout\", modifier = Modifier, color = Color.Unspecified"));
        assert!(android.contains("Text(\"Login\", modifier = Modifier, color = Color.Unspecified"));
        assert!(ios.contains("VStack(alignment: .leading, spacing: 0)"));
        assert!(!ios.contains("VStack(alignment: .leading) {"));
        assert!(ios.contains("Text(\"Layout\")"));
        assert!(ios.contains("Text(\"Login\")"));
        assert!(manifest.contains("desktop-macos"));
        assert!(manifest.contains("desktop-windows"));
        assert!(manifest.contains("desktop-linux"));
        assert!(manifest.contains("android"));
        assert!(manifest.contains("ios"));
        assert!(manifest.contains("web/manifest.json"));
        assert!(manifest.contains(r#""deepLinks""#));
        assert!(manifest.contains(r#""scheme":"dowe-dev""#));
    }

    #[test]
    fn preserves_nested_box_order_and_children_position() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    Text
      "Before"
    children
    Text
      "After""#,
            r#"page loginPage
  Box
    Box
      Text
        "Login""#,
        );

        let project = compile_dev(temp.path()).expect("project");

        assert!(project.web.pages[0].body_html.contains(r#"<p class="text-md">Before</p>"#));
        assert!(
            project.web.pages[0]
                .body_html
                .contains(r#"<div class="box"><div class="box"><p class="text-md">Login</p></div></div>"#)
        );
        assert!(project.web.pages[0].body_html.contains(r#"<p class="text-md">After</p>"#));
    }

    #[test]
    fn compiles_client_environment_for_request_base() {
        let temp = TempDir::new().expect("tempdir");
        write_blog_fixture(temp.path());
        fs::write(
            temp.path().join(".env"),
            "BACKEND_URL=https://api.example.com\nINTERNAL_TOKEN=secret\n",
        )
        .expect("dotenv");

        let project = compile_dev(temp.path()).expect("project");
        let backend = project
            .environment_config
            .variable("BACKEND_URL")
            .expect("backend url");
        let internal = project
            .environment_config
            .variable("INTERNAL_TOKEN")
            .expect("internal token");
        assert_eq!(backend.visibility, EnvironmentVisibility::Client);
        assert_eq!(backend.resolved_source, EnvironmentValueSource::DotEnv);
        assert_eq!(backend.resolved_value.as_deref(), Some("https://api.example.com"));
        assert_eq!(internal.visibility, EnvironmentVisibility::Server);

        let page = project
            .web
            .pages
            .iter()
            .find(|page| page.route_path == "/blogs")
            .expect("blogs page");
        let page_chunk = project
            .web
            .chunks
            .iter()
            .find(|chunk| chunk.id == page.page_chunk_id)
            .expect("page chunk");
        assert!(page_chunk.content.contains(r#""baseEnv":"BACKEND_URL""#));
        assert!(project.web.router_js.contains("env.json"));

        let env_json = fs::read_to_string(temp.path().join(".dowe/web/env.json")).expect("env");
        assert!(env_json.contains(r#""BACKEND_URL":"https://api.example.com""#));
        assert!(!env_json.contains("INTERNAL_TOKEN"));

        let android = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweEnvironment.kt"),
        )
        .expect("android env");
        let ios =
            fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweEnvironment.swift"))
                .expect("ios env");
        assert!(android.contains(r#"const val BACKEND_URL = "https://api.example.com""#));
        assert!(ios.contains(r#"static let BACKEND_URL = "https://api.example.com""#));
        assert!(!android.contains("INTERNAL_TOKEN"));
        assert!(!ios.contains("INTERNAL_TOKEN"));
    }

    #[test]
    fn resolves_environment_from_operating_system_when_dotenv_omits_key() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"DOWE_TEST_BACKEND_URL" visibility:"client" required:true"#,
        )
        .expect("config");
        unsafe {
            std::env::set_var("DOWE_TEST_BACKEND_URL", "https://os.example.com");
        }

        let project = compile_dev(temp.path()).expect("project");

        unsafe {
            std::env::remove_var("DOWE_TEST_BACKEND_URL");
        }
        let variable = project
            .environment_config
            .variable("DOWE_TEST_BACKEND_URL")
            .expect("variable");
        assert_eq!(variable.resolved_source, EnvironmentValueSource::Os);
        assert_eq!(variable.resolved_value.as_deref(), Some("https://os.example.com"));
    }

    #[test]
    fn rejects_server_environment_variable_from_view_request_base() {
        let temp = TempDir::new().expect("tempdir");
        write_blog_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"BACKEND_URL" visibility:"server" default:"https://api.example.com"
    variable name:"INTERNAL_TOKEN" visibility:"server" default:"secret""#,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("server-only"));
        assert!(error.to_string().contains("BACKEND_URL"));
    }

    #[test]
    fn rejects_invalid_request_base_url() {
        let temp = TempDir::new().expect("tempdir");
        write_blog_fixture(temp.path());
        fs::write(temp.path().join(".env"), "BACKEND_URL=file:///tmp/api\n").expect("dotenv");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("http or https URL"));
        assert!(error.to_string().contains("BACKEND_URL"));
    }

    #[test]
    fn rejects_missing_required_environment_variable() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"REQUIRED_TOKEN" visibility:"server" required:true"#,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("REQUIRED_TOKEN"));
        assert!(error.to_string().contains("required environment variable"));
    }

    #[test]
    fn compiles_backend_cors_config() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" devOrigins:true origins:["http://127.0.0.1:56035"] methods:["GET","POST","PATCH","DELETE"] headers:["Content-Type"] exposeHeaders:["X-Request-Id"] credentials:false maxAge:600"#,
        )
        .expect("config");

        let project = compile_dev(temp.path()).expect("project");

        assert!(project.backend.cors.enabled);
        assert!(project.backend.cors.allow_dev_origins);
        assert_eq!(
            project.backend.cors.origins,
            vec!["http://127.0.0.1:56035".to_string()]
        );
        assert!(project.backend.cors.methods.contains(&"GET".to_string()));
        assert!(project.backend.cors.methods.contains(&"POST".to_string()));
        assert_eq!(
            project.backend.cors.headers,
            vec!["Content-Type".to_string()]
        );
        assert_eq!(
            project.backend.cors.expose_headers,
            vec!["X-Request-Id".to_string()]
        );
        assert_eq!(project.backend.cors.max_age, Some(600));
    }

    #[test]
    fn expands_cors_target_all_to_backend_and_desktop() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/main.dowe"),
            r#"main
  server port:8080
    route "/api/status"
      response text:"OK"
  desktop
    server port:4500
      route "/api/status"
        response text:"OK""#,
        )
        .expect("server");
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"all" origins:["https://app.example.com"] headers:["Content-Type"]"#,
        )
        .expect("config");

        let project = compile_dev(temp.path()).expect("project");

        assert!(project.backend.cors.enabled);
        assert!(
            project
                .desktop_server
                .as_ref()
                .expect("desktop")
                .cors
                .enabled
        );
    }

    #[test]
    fn rejects_invalid_cors_origin() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" origins:["https://app.example.com/path"]"#,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("invalid CORS origin"));
    }

    #[test]
    fn rejects_wildcard_cors_with_credentials() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" origins:["*"] credentials:true"#,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("credentials:true"));
    }

    #[test]
    fn rejects_duplicate_cors_policy_for_target() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" origins:["https://one.example.com"]
    cors target:"server" origins:["https://two.example.com"]"#,
        )
        .expect("config");

        let error = compile_dev(temp.path()).expect_err("error");

        assert!(error.to_string().contains("duplicate CORS policy"));
    }

    fn write_blog_fixture(root: &Path) {
        write_fixture_with_views(
            root,
            r#"layout AuthLayout
  signal alert value:{ type:"info" message:"Layout alert" visible:true }
  action close
    reset alert
  Box
    Text
      "Layout"
    Alert type:"info" message:alert.message visible:alert.visible onClose:close
    children"#,
            r#"page loginPage
  Box
    Text
      "Login""#,
        );
        fs::write(
            root.join("src/config.dowe"),
            r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"BACKEND_URL" visibility:"client" required:false default:""
    variable name:"INTERNAL_TOKEN" visibility:"server" required:false"#,
        )
        .expect("config");
        fs::create_dir_all(root.join("src/handlers")).expect("handlers");
        fs::write(
            root.join("src/main.dowe"),
            r#"import listBlogs from "./handlers/blogs"
import createBlog from "./handlers/blogs"
import readBlog from "./handlers/blogs"
import updateBlog from "./handlers/blogs"
import deleteBlog from "./handlers/blogs"

main
  server port:8080
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
    route "/api/blogs/:id"
      method GET handler:readBlog
      method PATCH handler:updateBlog
      method DELETE handler:deleteBlog"#,
        )
        .expect("server");
        fs::write(
            root.join("src/handlers/blogs.dowe"),
        r#"handler listBlogs req
  let db = store database:"app"
  let blogs = db.list table:"blogs"
  return response json:{ ok:true data:blogs }

handler createBlog async req
  let body = await req.json()
  let db = store database:"app"
  let created = db.insert table:"blogs" value:{ title:body.title content:body.content createdAt:now updatedAt:now } required:["title","content"]
  let blogs = db.list table:"blogs"
  return response status:201 json:{ ok:true data:blogs }

handler readBlog req
  let db = store database:"app"
  let blog = db.read table:"blogs" where:{ id:req.params.id } required:true
  return response json:{ ok:true data:blog }

handler updateBlog async req
  let body = await req.json()
  let db = store database:"app"
  let updated = db.update table:"blogs" where:{ id:req.params.id } value:{ title:body.title content:body.content updatedAt:now } required:true match:{ id:req.params.id }
  let blogs = db.list table:"blogs"
  return response json:{ ok:true data:blogs }

handler deleteBlog req
  let db = store database:"app"
  let deleted = db.delete table:"blogs" where:{ id:req.params.id } required:true
  let blogs = db.list table:"blogs"
  return response json:{ ok:true data:blogs }"#,
        )
        .expect("handlers");
        fs::write(
            root.join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"
import blogsPage from "./pages/blogs"

views
  route path:"/" layout:AuthLayout
    page path:"" component:loginPage
    page path:"blogs" component:blogsPage"#,
        )
        .expect("views");
        fs::write(
            root.join("src/pages/blogs.dowe"),
            r#"page blogsPage
  signal blog value:{ id:null title:"" content:"" }
  signal blogs value:[]
  signal alert value:{ type:"info" message:"" visible:false }
  action load
    request GET route:"/api/blogs" update:blogs autoload:true
      onError alert:"No se pudieron cargar los blogs"
  action create
    request POST route:"/api/blogs" body:blog update:blogs reset:blog
      onSuccess alert:"Blog creado"
      onError alert:"No se pudo crear el blog"
  action edit
    assign blog source:item
  action update
    request PATCH route:"/api/blogs/:id" body:blog update:blogs reset:blog
      onSuccess alert:"Blog actualizado"
      onError alert:"No se pudo actualizar el blog"
  action delete
    request DELETE route:"/api/blogs/:id" body:item update:blogs
      onSuccess alert:"Blog eliminado"
      onError alert:"No se pudo eliminar el blog"
  action close
    reset alert
  Box
    Title
      "Blogs"
    Alert type:"info" message:alert.message visible:alert.visible onClose:close
    Input bind:blog.title
    Button onClick:create
      "Crear"
    each item in blogs key:item.id
      Card
        Title
          "item.title"
        Text
          "item.content"
        Button onClick:edit
          "Editar""#,
        )
        .expect("blogs");
    }

    fn attribute_values<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
        let prefix = format!(r#"{name}=""#);
        html.match_indices(&prefix)
            .filter_map(|(start, _)| {
                let value = &html[start + prefix.len()..];
                value.find('"').map(|end| &value[..end])
            })
            .collect()
    }

    fn short_root(value: &str, suffix: &str) -> bool {
        value.strip_suffix(suffix).is_some_and(|root| {
            root.len() == 8
                && root
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        })
    }

    fn ios_swift_output(root: &Path) -> String {
        ios_swift_output_from(&root.join(".dowe/apps/ios"))
    }

    fn ios_apps_swift_output(root: &Path) -> String {
        ios_swift_output_from(&root.join("ios"))
    }

    fn ios_swift_output_from(ios_root: &Path) -> String {
        let mut swift_files = fs::read_dir(ios_root)
            .expect("ios output")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("swift"))
            .collect::<Vec<_>>();
        swift_files.sort();
        swift_files
            .into_iter()
            .map(|path| fs::read_to_string(path).expect("ios swift"))
            .collect::<Vec<_>>()
            .join("\n")
    }
