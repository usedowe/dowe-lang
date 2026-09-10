#[test]
fn compiles_navigation_actions_sections_and_deep_link_metadata() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_two_pages(
        temp.path(),
        r#"layout AuthLayout
  Box id:"shell"
    Text
      "Layout"
    children"#,
        r##"page loginPage
  Box id:"hero"
    Button href:"#hero" navigate:"replace"
      "Hero"
    Button href:"/signup#join"
      "Signup"
    Button href:"https://example.com/docs" target:"blank" externalMode:"webview"
      "Docs"
    Button history:"back"
      "Back""##,
        r#"page signupPage
  Box id:"join"
    Text
      "Signup""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let login = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/")
        .expect("login");
    let manifest =
        fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("manifest");
    let router = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.router_file_name()),
    )
    .expect("router");
    let android_routing = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweRouting.kt"),
    )
    .expect("android routing");
    let android_pages = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    let android_dev = android_dev_output(temp.path());
    let ios_routing = fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweRouting.swift"))
        .expect("ios routing");
    let ios_pages = ios_swift_output(temp.path());

    assert!(login.body_html.contains(r#"id="hero""#));
    assert!(
        login
            .body_html
            .contains(r##"href="#hero" data-dowe-nav="replace""##)
    );
    assert!(
        login
            .body_html
            .contains(r##"href="/signup#join" data-dowe-nav="push""##)
    );
    assert!(
            login
                .body_html
                .contains(r#"href="https://example.com/docs" data-dowe-external-mode="webview" target="_blank" rel="noopener noreferrer""#)
        );
    assert!(login.body_html.contains(r#"data-dowe-history="back""#));
    assert!(manifest.contains(r#""sections":["shell","hero"]"#));
    assert!(manifest.contains(r#""navigationActions""#));
    assert!(manifest.contains(r#""nativeExternalMode":"webview""#));
    assert!(manifest.contains(r#""deepLinks""#));
    assert!(router.contains("history.pushState"));
    assert!(router.contains("history.replaceState"));
    assert!(router.contains("popstate"));
    assert!(router.contains("scrollToFragment"));
    assert!(router.contains("suppressPageEntranceAnimations"));
    assert!(router.contains("runCssPageTransition"));
    assert!(router.contains("await Promise.all([loadRouteCss(route),cachedRouteModules(route),])"));
    assert!(android_routing.contains("dowe-dev://generated/signup"));
    assert!(android_pages.contains("private data class DoweRouteEntry"));
    assert!(android_pages.contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(android_pages.contains(r#"{ navigate("push", "/signup", "join") }"#));
    assert!(android_pages.contains("ExitTransition.None"));
    assert!(android_pages.contains("durationMillis = 280"));
    assert!(
        android_dev
            .contains("setOnClickListener(v -> doweNavigate(\"replace\", currentPath, \"hero\"))")
    );
    assert!(android_dev.contains("doweStartPageTransition();"));
    assert!(android_dev.contains("setDuration(280)"));
    assert!(ios_routing.contains("dowe-dev://generated/signup"));
    assert!(ios_pages.contains("struct DoweRouteEntry: Hashable"));
    assert!(ios_pages.contains("@State private var navigationPath: [DoweRouteEntry] = []"));
    assert!(ios_pages.contains("routeContent(currentEntry, viewportWidth:"));
    assert!(ios_pages.contains(".transition(.asymmetric(insertion: .opacity, removal: .identity)"));
    assert!(ios_pages.contains("duration: 0.28"));
    assert!(ios_pages.contains("pageEntranceSuppressed"));
    assert!(ios_pages.contains(".simultaneousGesture(backSwipeGesture)"));
    assert!(ios_pages.contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(ios_pages.contains(r#"{ navigate("push", "/signup", "join") }"#));
}

#[test]
fn compiles_external_banner_across_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Banner href:"https://dowe.dev/cloud" label:"Explore Dowe Cloud" p:6
    Title
      "Build beyond code"
    Text
      "Explore Dowe Cloud""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    let android_dev = android_dev_output(temp.path());
    let ios = ios_swift_output(temp.path());

    assert!(body.contains(r#"<a class="banner p-6""#));
    assert!(body.contains(r#"href="https://dowe.dev/cloud""#));
    assert!(body.contains(r#"target="_blank" rel="noopener noreferrer""#));
    assert!(body.contains(r#"aria-label="Explore Dowe Cloud""#));
    assert!(android.contains(
        ".clickable(onClick = { openExternal(\"system\", \"https://dowe.dev/cloud\") })"
    ));
    assert!(android.contains(".semantics { contentDescription = \"Explore Dowe Cloud\" }"));
    assert!(android_dev.contains(
        "setOnClickListener(v -> doweOpenExternal(\"system\", \"https://dowe.dev/cloud\"))"
    ));
    assert!(
        ios.contains("Button(action: { openExternal(\"system\", \"https://dowe.dev/cloud\") })")
    );
    assert!(ios.contains(".accessibilityLabel(Text(\"Explore Dowe Cloud\"))"));
}

#[test]
fn rejects_navigation_to_unknown_route() {
    assert_compile_error(
        r#"page loginPage
  Box
    Button href:"/missing"
      "Missing""#,
        "unknown navigation route `/missing`",
    );
}
#[test]
fn rejects_redirect_to_unknown_route() {
    assert_compile_error(
        r#"page loginPage
  init
    redirect path:"/missing"
  Text
    "Login""#,
        "unknown navigation route `/missing`",
    );
}

#[test]
fn rejects_navigation_to_unknown_section() {
    assert_compile_error(
        r##"page loginPage
  Box id:"hero"
    Button href:"#missing"
      "Missing""##,
        "unknown section `#missing`",
    );
}

#[test]
fn rejects_duplicate_section_ids() {
    assert_compile_error(
        r#"page loginPage
  Box id:"hero"
    Box id:"hero"
      Text
        "Login""#,
        "duplicate section id `hero`",
    );
}

#[test]
fn rejects_unsafe_external_href() {
    assert_compile_error(
        r#"page loginPage
  Box
    Button href:"javascript:alert(1)"
      "Bad""#,
        "invalid value for prop `href`",
    );
}

#[test]
fn rejects_unknown_components() {
    assert_compile_error(
        r#"page loginPage
  Stack
    Text
      "Login""#,
        "unknown component `Stack`",
    );

    assert_compile_error(
        r#"page loginPage
  Body text:"Login""#,
        "unknown component `Body`",
    );
}
