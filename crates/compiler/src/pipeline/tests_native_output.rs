#[test]
fn compiles_flex_defaults_and_alignment_across_native_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Flex justify:"center" align:"center"
    Text
      "Flex""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    let ios = ios_swift_output(temp.path());
    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    let page_css_path = temp.path().join(".dowe/web").join(generated_css_chunk(
        &project.web.pages[0].css_chunks,
        "chunks/pages/",
    ));
    let page_css = fs::read_to_string(page_css_path).expect("page css");

    assert!(
        project.web.pages[0]
            .body_html
            .contains("flex direction-row justify-center align-center")
    );
    assert!(css.contains(".flex{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));width:100%;height:auto;}"));
    assert!(page_css.contains(".justify-center{justify-content:center;}"));
    assert!(page_css.contains(".align-center{align-items:center;}"));
    assert!(android.contains(
        "Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = doweVerticalArrangement"
    ));
    assert!(android.contains("horizontalAlignment = doweHorizontalAlignment"));
    assert!(ios.contains("VStack(alignment:") || ios.contains("HStack(alignment:"));
    assert!(ios.contains(".frame(maxWidth: .infinity"));
    assert!(!android.contains("doweHeight(DoweSize"));
    assert!(!ios.contains("frame(height: doweFixedSize"));
}

#[test]
fn compiles_refactored_container_props() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Grid columns:{ xs:1 md:3 } rows:2 gap:"10px 20px" justify:"center" align:"end"
    Box colSpan:{ md:2 } cover:{ xs:"/mobile.jpg" md:"/desktop.jpg" } overlay:0.4
      Text
        "Hero"
    Card variant:"solid" scheme:"surface" rounded:"full" rowSpan:2 cover:"/images/card.jpg" overlay:0.6
      Text
        "Card""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(body.contains("grid-cols-1 md:grid-cols-3"));
    assert!(body.contains("grid-rows-"));
    assert!(body.contains("gap-"));
    assert!(body.contains("grid-justify-center"));
    assert!(body.contains("grid-align-end"));
    assert!(body.contains("md:col-span-2"));
    assert!(body.contains("has-cover"));
    assert!(body.contains("has-overlay"));
    assert!(body.contains("p-4 lg:p-5"));
    assert!(body.contains("is-solid is-surface"));

    let page_css_path = temp.path().join(".dowe/web").join(generated_css_chunk(
        &project.web.pages[0].css_chunks,
        "chunks/pages/",
    ));
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains("grid-template-columns:repeat(3,minmax(0,1fr));"));
    assert!(page_css.contains("grid-template-rows:repeat(2,minmax(0,1fr));"));
    assert!(page_css.contains("row-gap:10px;column-gap:20px;"));
    assert!(page_css.contains("background-image:url(\"/mobile.jpg\")"));
    assert!(page_css.contains("background-image:url(\"/desktop.jpg\")"));
    assert!(page_css.contains(".lg\\:p-5"));
    assert!(page_css.contains("rgba(0,0,0,0.4)"));
    assert!(page_css.contains("rgba(0,0,0,0.6)"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("DoweCoverBox"));
    assert!(android.contains("\"/desktop.jpg\""));
    assert!(android.contains("DoweOverlay.Solid(Color.Black.copy(alpha = 0.6f))"));
    assert!(android.contains("PaddingValues(start = doweResponsive(viewportWidth, xs = 16.dp, lg = 20.dp)"));
    assert!(android.contains("DoweGrid(modifier ="));
    assert!(android.contains("tracks = doweResponsive(viewportWidth, xs = listOf(1f), md = listOf(1f, 1f, 1f)) ?: listOf(1f)"));
    assert!(android.contains("horizontalGap = doweResponsive(viewportWidth, xs = 20.dp) ?: 0.dp"));
    assert!(android.contains("verticalGap = doweResponsive(viewportWidth, xs = 10.dp) ?: 0.dp"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("DoweCoverImage"));
    assert!(ios.contains("\"/desktop.jpg\""));
    assert!(ios.contains("DoweOverlay.color(Color.black.opacity(0.6))"));
    assert!(ios.contains(
            ".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(16), lg: CGFloat(20)) ?? CGFloat(0)"
        ));
    assert!(ios.contains("DoweGridLayout("));
}

#[test]
fn compiles_container_foreground_inheritance_for_all_view_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box color:"primaryText"
    Text
      "Box inherited"
    Text color:"danger"
      "Box override"
  Card variant:"solid" scheme:"muted"
    Text
      "Card inherited"
    Title color:"warning"
      "Card override""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains("box color-primaryText"));
    assert!(body.contains("Box override"));
    assert!(body.contains("color-danger"));
    assert!(body.contains("card p-4 lg:p-5 rounded-md is-solid is-muted"));
    assert!(body.contains("Card override"));
    assert!(body.contains("color-warning"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(
            "CompositionLocalProvider(LocalContentColor provides (doweResponsive(viewportWidth, xs = DoweDesign.primaryText) ?: LocalContentColor.current))"
        ));
    assert!(
        android.contains("Text(\"Box inherited\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(android.contains("DoweDesign"));
    assert!(
        android.contains("Text(\"Card inherited\", modifier = Modifier, color = Color.Unspecified")
    );

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("Text(verbatim: \"Box inherited\")"));
    assert!(ios.contains(
            ".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.primaryText) ?? DoweDesign.backgroundText)"
        ));
    assert!(ios.contains("Text(verbatim: \"Card inherited\")"));
    assert!(ios.contains("DoweDesign"));
}

#[test]
fn compiles_layout_bars_without_ios_dividers() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    AppBar variant:"solid" scheme:"surface" position:"sticky" bordered:true boxed:true
      start
        Text
          "Dowe"
    children
    BottomBar variant:"solid" scheme:"surface" bordered:true boxed:true
      tab href:"/login" label:"Home"
        Icon name:"home"
    Footer scheme:"background" bordered:true boxed:true
      end
        Text
          "info@dowe.dev""#,
        r#"page loginPage
  Text
    "Login""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    assert!(project.web.pages[0].body_html.contains("position-sticky"));
    assert!(
        project.web.pages[0]
            .body_html
            .contains("footer px-4 md:px-6")
    );

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(".zIndex(1)"));
    assert!(ios.contains("Text(verbatim: \"info@dowe.dev\")"));
    assert!(ios.contains(
        "leading: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(24)) ?? CGFloat(0)"
    ));
    assert!(!ios.contains(".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))"));
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(".zIndex(1f)"));
    assert!(android.contains("horizontal = doweResponsive(viewportWidth, xs = 16.dp, md = 24.dp)"));
    let android_dev = android_dev_output(temp.path());
    assert!(
        android_dev
            .contains("PaddingX = doweResponsiveInt(viewportWidth, 16, null, 24, null, null)")
    );
    assert!(!ios.contains(
            ".overlay(RoundedRectangle(cornerRadius: CGFloat(0)).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
}

#[test]
fn compiles_cross_target_typography_from_shared_metrics() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text size:"9xl"
      "Body"
    Title size:"9xl"
      "Title""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let css = project.web.pages[0]
        .css_chunks
        .iter()
        .map(|chunk| {
            fs::read_to_string(temp.path().join(".dowe/web").join(chunk)).expect("css chunk")
        })
        .collect::<Vec<_>>()
        .join("");
    assert!(css.contains(
            ".text-9xl{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(2.5rem, 1.9rem + 2.8vw, 3.75rem);line-height:1.2;font-weight:400;margin:0;}"
        ));
    assert!(css.contains(
            ".title-9xl{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(4.5rem, 3rem + 7vw, 8rem);line-height:1;font-weight:800;letter-spacing:-0.06em;margin:0;}"
        ));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(
            "doweTextSize(viewportWidth, min = 40f, preferredBase = 30.4f, preferredViewport = 2.8f, max = 60f)"
        ));
    assert!(android.contains(
            "doweTextSize(viewportWidth, min = 72f, preferredBase = 48f, preferredViewport = 7f, max = 128f)"
        ));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("doweFluidTextSize(40f, 30.4f, 2.8f, 60f)"));
    assert!(android_dev.contains("doweFluidTextSize(72f, 48f, 7f, 128f)"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(
            "doweTextSize(viewportWidth, min: CGFloat(40), preferredBase: CGFloat(30.4), preferredViewport: CGFloat(2.8), max: CGFloat(60))"
        ));
    assert!(ios.contains(
            "doweTextSize(viewportWidth, min: CGFloat(72), preferredBase: CGFloat(48), preferredViewport: CGFloat(7), max: CGFloat(128))"
        ));
}

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
    assert!(android_routing.contains("dowe-dev://generated/signup"));
    assert!(android_pages.contains("private data class DoweRouteEntry"));
    assert!(android_pages.contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(android_pages.contains(r#"{ navigate("push", "/signup", "join") }"#));
    assert!(
        android_dev
            .contains("setOnClickListener(v -> doweNavigate(\"replace\", currentPath, \"hero\"))")
    );
    assert!(ios_routing.contains("dowe-dev://generated/signup"));
    assert!(ios_pages.contains("struct DoweRouteEntry: Hashable"));
    assert!(ios_pages.contains("@State private var navigationPath: [DoweRouteEntry] = []"));
    assert!(ios_pages.contains("routeContent(currentEntry, viewportWidth:"));
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

#[test]
fn compiles_device_preview_with_fixed_profiles_and_zoom() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Text
    "Device preview"
  Device device:"laptop" w:"full" rounded:"md" border:1
    Iframe src:"/examples/appbar-one" title:"Responsive preview""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains("data-dowe-device-profile=\"laptop\""));
    assert!(body.contains("data-dowe-device-option=\"mobile\""));
    assert!(body.contains("width:1440px;height:900px"));
    assert!(body.contains("class=\"device"));
    assert!(body.contains("border-1"));
    assert!(body.contains("button icon-button button-md device-toggle"));
    assert!(body.contains("data-dowe-button-icon-start"));
    assert!(project.web.router_js.contains("ResizeObserver"));
    assert!(
        project
            .web
            .router_js
            .contains("Math.min(1,width/dimensions[0])")
    );

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("private fun DoweDevicePreview("));
    assert!(android.contains("1440f to 900f"));
    assert!(android.contains("DoweDeviceIcon(profile = \"mobile\""));
    assert!(android.contains("DoweSvg(viewBox = icon.viewBox"));
    assert!(android.contains(".border(doweResponsive("));
    assert!(android.contains("private fun DoweDeviceIconButton("));
    assert!(android.contains("Modifier.size(40.dp)"));
    assert!(android.contains("Modifier.size(24.dp)"));
    assert!(android.contains("containerColor = if (selected) DoweDesign.muted else Color.Transparent"));
    assert!(android.contains("contentColor = if (selected) DoweDesign.primary else DoweDesign.backgroundText"));
    assert!(android.contains("Row(modifier = Modifier.padding(4.dp)"));
    assert!(!android.contains("Text(option.second)"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("private FrameLayout doweDevice("));
    assert!(android_dev.contains("DoweDeviceOption[] options"));
    assert!(android_dev.contains("new DoweSvgView(this, 0f, 0f, 24f, 24f"));
    assert!(
        android_dev
            .contains("doweDevice(\"laptop\", \"/examples/appbar-one\", \"Responsive preview\"")
    );
    assert!(android_dev.contains("new DoweDeviceOption(\"mobile\""));
    assert!(android_dev.contains("doweStyledBackground(Color.TRANSPARENT, DOWE_BACKGROUND_TEXT,"));
    assert!(android_dev.contains("selected ? DOWE_MUTED : DOWE_BACKGROUND, selected ? DOWE_PRIMARY : DOWE_BACKGROUND_TEXT"));
    assert!(android_dev.contains(".setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));"));
    assert!(android_dev.contains("doweDeviceIconButtonBackground"));
    assert!(
        android_dev
            .contains("new FrameLayout.LayoutParams(doweDp(24), doweDp(24), Gravity.CENTER)")
    );
    assert!(android_dev.contains("setMargins(doweDp(2), doweDp(4), doweDp(2), doweDp(4))"));
    assert!(!android_dev.contains("button.setText(option[1])"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("struct DoweDevicePreview: View"));
    assert!(ios.contains("CGSize(width: 1920, height: 1080)"));
    assert!(ios.contains("DoweDeviceIcon(profile: \"mobile\""));
    assert!(ios.contains("DoweSvgView(viewBox: option.viewBox"));
    assert!(ios.contains(".frame(width: CGFloat(40), height: CGFloat(40))"));
    assert!(ios.contains(".frame(width: CGFloat(24), height: CGFloat(24))"));
    assert!(ios.contains(".background(profile == option.profile ? DoweDesign.muted : Color.clear)"));
    assert!(ios.contains(".padding(CGFloat(4))"));
    assert!(!ios.contains("Button(option.1)"));
}

#[test]
fn compiles_canvas_with_cross_target_scene_runtime() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  signal input value:{ x:48 y:90 }
  signal scene value:[{ type:"rect" x:0 y:0 width:320 height:180 fill:"surface" },{ type:"circle" x:48 y:90 radius:18 fill:"primary" bind:{ x:"input.x" y:"input.y" } },{ type:"text" x:160 y:28 text:"Canvas" fill:"surfaceText" size:18 align:"center" }]
  fn capture
    set input value:item
  Box
    Text
      "Canvas demo"
    Canvas scene:scene viewWidth:320 viewHeight:180 fit:"contain" fps:60 autoplay:true background:"background" pixelated:true label:"Animated Canvas" onPointer:capture onKey:capture onMotion:capture motionRate:30 w:"full" h:48 rounded:"md" border:1"#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    let runtime_path = project.web.pages[0]
        .runtime_chunks
        .iter()
        .find(|path| path.contains("visualization-"))
        .expect("visualization route dependency");
    assert!(body.contains("data-dowe-canvas"));
    assert!(body.contains("data-dowe-canvas-scene="));
    assert!(body.contains("aria-label=\"Animated Canvas\""));
    let visualization = project
        .web
        .runtime_chunks()
        .into_iter()
        .find(|chunk| chunk.name == "visualization")
        .expect("visualization runtime");
    assert!(visualization.content.contains("drawCanvasCommand"));
    assert!(visualization.content.contains("devicePixelRatio"));
    assert!(visualization.content.contains("closeCanvasFrames"));
    assert!(
        project.web.pages[0]
            .html_document
            .contains(&format!(r#"rel="modulepreload" href="/{runtime_path}""#))
    );
    assert!(
        dowe_generator_web::manifest(&project.web)
            .contains(&format!(r#""runtimeChunks":["{runtime_path}"]"#))
    );
    assert!(temp.path().join(".dowe/web").join(runtime_path).is_file());
    assert!(body.contains("data-dowe-canvas-on-pointer="));
    assert!(body.contains("data-dowe-canvas-on-key="));
    assert!(body.contains("data-dowe-canvas-on-motion="));
    assert!(visualization.content.contains("boundCanvasCommand"));
    assert!(visualization.content.contains("canvasLogicalPoint"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("private fun DoweCanvas("));
    assert!(android.contains("doweDrawCanvasCommand"));
    assert!(android.contains("DoweCanvas(state = state, scenePath = \""));
    assert!(android.contains("doweBoundCanvasCommand"));
    assert!(android.contains("pointerInput(onPointer"));
    assert!(android.contains("Sensor.TYPE_ROTATION_VECTOR"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("private DoweCanvasView doweCanvas("));
    assert!(android_dev.contains("doweDrawCanvasCommand"));
    assert!(android_dev.contains("onTouchEvent(MotionEvent event)"));
    assert!(android_dev.contains("doweStartCanvasSensors"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("struct DoweCanvasView: View"));
    assert!(ios.contains("TimelineView(.animation"));
    assert!(ios.contains("DoweCanvasView(state: state, scenePath: \""));
    assert!(ios.contains("pixelated: true"));
    assert!(ios.contains("interpolation(pixelated ? .none : .medium)"));
    assert!(ios.contains("accessibilityAddTraits(.isImage)"));
    assert!(ios.contains("DoweCanvasInputBridge"));
    assert!(ios.contains("CMMotionManager"));
    assert!(ios.contains("boundCommand"));
    let ios_info =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("ios info");
    assert!(ios_info.contains("NSMotionUsageDescription"));
}

#[test]
fn compiles_divider_across_native_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text
      "Divider"
    Divider scheme:"primary"
    Divider orientation:"vertical" scheme:"secondary""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains("divider divider-horizontal is-primary"));
    assert!(body.contains("divider divider-vertical is-secondary"));
    assert!(project.web.chunks.iter().any(|chunk| {
        chunk
            .css_content
            .contains(".divider.is-primary{background-color:var(--dowe-primary);")
    }));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(
        "Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(DoweDesign.primary))"
    ));
    assert!(android.contains(
        "Box(modifier = Modifier.width(1.dp).fillMaxHeight().background(DoweDesign.secondary))"
    ));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("setBackgroundColor(DOWE_PRIMARY)"));
    assert!(android_dev.contains("setBackgroundColor(DOWE_SECONDARY)"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(".fill(DoweDesign.primary)"));
    assert!(ios.contains(".fill(DoweDesign.secondary)"));
    assert!(ios.contains(".frame(height: CGFloat(1))"));
    assert!(ios.contains(".frame(width: CGFloat(1))"));
}

#[test]
fn rejects_invalid_divider_components() {
    assert_compile_error(
        r#"page loginPage
  Text
    "Divider"
  Divider orientation:"diagonal""#,
        "expected horizontal or vertical",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Divider"
  Divider
    Text
      "Child""#,
        "children are not valid for this component",
    );
}

#[test]
fn rejects_invalid_video_components() {
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video"#,
        "invalid value for prop `src`: expected https URL",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"http://example.com/video.mp4""#,
        "invalid value for prop `src`: expected https URL",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"https://example.com/video.mp4" aspect:"wide""#,
        "invalid value for prop `aspect`: expected horizontal, vertical or square",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"https://example.com/video.mp4" autoplay:"true""#,
        "invalid value for prop `autoplay`: expected boolean",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"https://example.com/video.mp4"
    Text
      "Child""#,
        "children are not valid for this component",
    );
}

#[test]
fn rejects_invalid_iframe_components() {
    assert_compile_error(
        r#"page loginPage
  Iframe src:"https://example.com""#,
        "invalid value for prop `title`: expected non-empty string",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"http://example.com" title:"Example""#,
        "invalid value for prop `src`: expected https URL or internal route",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"//example.com" title:"Example""#,
        "invalid value for prop `src`: expected https URL or internal route",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"/examples/../admin" title:"Example""#,
        "invalid value for prop `src`: expected https URL or internal route",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"https://example.com" title:"Example" sandbox:"scripts unknown""#,
        "invalid value for prop `sandbox`: expected portable iframe policy tokens",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"https://example.com" title:"Example"
    Text
      "Child""#,
        "children are not valid for this component",
    );
}

#[test]
fn rejects_invalid_device_components() {
    assert_compile_error(
        r#"page loginPage
  Device device:"watch"
    Iframe src:"/preview" title:"Preview""#,
        "invalid value for prop `device`: expected mobile, tablet, laptop or monitor",
    );
    assert_compile_error(
        r#"page loginPage
  Device"#,
        "Device requires exactly one Iframe child",
    );
    assert_compile_error(
        r#"page loginPage
  Device
    Text
      "Invalid""#,
        "Device can only contain one Iframe child",
    );
}

#[test]
fn rejects_empty_text() {
    assert_compile_error(
        r#"page loginPage
  Box
    Text"#,
        "Text requires a text child",
    );

    assert_compile_error(
        r#"page loginPage
  Box
    Button"#,
        "Button requires a text child",
    );

    assert_compile_error(
        r#"page loginPage
  Box
    Text
      "   ""#,
        "Text requires static text",
    );
}

#[test]
fn rejects_component_children_inside_text() {
    assert_compile_error(
        r#"page loginPage
  Box
    Text
      Box
        Text
          "Nested""#,
        "must be a quoted static string literal",
    );
}

#[test]
fn rejects_children_inside_page() {
    assert_compile_error(
        r#"page loginPage
  Box
    children"#,
        "children can only be used inside layouts",
    );
}
