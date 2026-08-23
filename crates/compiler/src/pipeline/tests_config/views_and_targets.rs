
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
    assert!(page.body_html.contains(">item.literal</p>"));

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
    let android_dev = android_dev_output(temp.path());
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
    assert!(android_pages.contains("Text(\"item.literal\""));

    assert!(android_dev.contains("doweState"));
    assert!(android_dev.contains("doweRows("));
    assert!(android_dev.contains("doweRunAction("));
    assert!(android_dev.contains("DoweEnvironment.BACKEND_URL"));
    assert!(android_dev.contains("ScrollView scrollView"));
    assert!(android_dev.contains("doweInputBackground("));
    assert!(android_dev.contains("doweAdd(ViewGroup parent, View child, Integer gap"));
    assert!(!android_dev.contains("doweText(\"alert.message\""));
    assert!(!android_dev.contains("doweText(\"item.title\""));
    assert!(android_dev.contains("doweText(\"item.literal\""));

    assert!(ios_pages.contains("DoweReactiveState"));
    assert!(ios_pages.contains("state.binding("));
    assert!(ios_pages.contains("state.rows("));
    assert!(ios_pages.contains("state.run("));
    assert!(ios_pages.contains("DoweEnvironment.BACKEND_URL"));
    assert!(ios_pages.contains("ScrollView {"));
    assert!(ios_pages.contains("DoweInputField(value: state.binding("));
    assert!(ios_pages.contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)"));
    assert!(ios_pages.contains("if path == \"item\", let item"));
    assert!(!ios_pages.contains("Text(verbatim: \"alert.message\")"));
    assert!(!ios_pages.contains("Text(verbatim: \"item.title\")"));
    assert!(ios_pages.contains("Text(verbatim: \"item.literal\")"));
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

    let android =
        fs::read_to_string(root.join("android/app/src/main/java/dev/dowe/generated/DowePages.kt"))
            .expect("android views");
    let ios = ios_apps_swift_output(&root);
    let manifest = fs::read_to_string(root.join("manifest.json")).expect("apps manifest");

    assert!(android.contains("Column(modifier = Modifier.fillMaxWidth()) {"));
    assert!(android.contains("Text(\"Layout\", modifier = Modifier, color = Color.Unspecified"));
    assert!(android.contains("Text(\"Login\", modifier = Modifier, color = Color.Unspecified"));
    assert!(ios.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(!ios.contains("VStack(alignment: .leading) {"));
    assert!(ios.contains("Text(verbatim: \"Layout\")"));
    assert!(ios.contains("Text(verbatim: \"Login\")"));
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

    assert!(
        project.web.pages[0]
            .body_html
            .contains(r#"<p class="dowe-text text-md">Before</p>"#)
    );
    assert!(
        project.web.pages[0].body_html.contains(
            r#"<div class="box"><div class="box"><p class="dowe-text text-md">Login</p></div></div>"#
        )
    );
    assert!(
        project.web.pages[0]
            .body_html
            .contains(r#"<p class="dowe-text text-md">After</p>"#)
    );
}

#[test]
fn compiles_client_environment_for_request_base() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "BACKEND_URL=\nINTERNAL_TOKEN=\n",
    )
    .expect("env example");
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=\"https://api.example.com\"\nINTERNAL_TOKEN=secret\n",
    )
    .expect("env");

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
    assert_eq!(
        backend.resolved_value.as_deref(),
        Some("https://api.example.com")
    );
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
    let ios = fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweEnvironment.swift"))
        .expect("ios env");
    assert!(android.contains(r#"const val BACKEND_URL = "https://api.example.com""#));
    assert!(ios.contains(r#"static let BACKEND_URL = "https://api.example.com""#));
    assert!(!android.contains("INTERNAL_TOKEN"));
    assert!(!ios.contains("INTERNAL_TOKEN"));
}

