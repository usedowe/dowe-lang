    #[test]
    fn compiles_refactored_container_props() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    children"#,
            r#"page loginPage
  Grid columns:{ xs:1 md:3 } rows:"100px auto" gap:"10px 20px" justify:"center" align:"end"
    Box colSpan:{ md:2 } cover:{ xs:"/mobile.jpg" md:"/desktop.jpg" } overlay:true
      Text
        "Hero"
    Card variant:"soft" scheme:"surface" rounded:"full" rowSpan:2 cover:"/images/card.jpg" overlay:0.6
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
        assert!(body.contains("is-soft is-surface"));

        let page_css_path = temp
            .path()
            .join(".dowe/web")
            .join(&project.web.pages[0].css_chunks[1]);
        let page_css = fs::read_to_string(page_css_path).expect("page css");
        assert!(page_css.contains("grid-template-columns:repeat(3,minmax(0,1fr));"));
        assert!(page_css.contains("grid-template-rows:100px auto;"));
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
        assert!(android.contains("all = doweResponsive(viewportWidth, xs = 16.dp, lg = 20.dp)"));
        assert!(android.contains("DoweGrid(modifier ="));
        assert!(
            android.contains("columns = doweResponsive(viewportWidth, xs = 1, md = 3) ?: 1")
        );
        assert!(android.contains(
            "horizontalGap = doweResponsive(viewportWidth, xs = 20.dp) ?: 0.dp"
        ));
        assert!(
            android.contains("verticalGap = doweResponsive(viewportWidth, xs = 10.dp) ?: 0.dp")
        );

        let ios = ios_swift_output(temp.path());
        assert!(ios.contains("DoweCoverImage"));
        assert!(ios.contains("\"/desktop.jpg\""));
        assert!(ios.contains("DoweOverlay.color(Color.black.opacity(0.6))"));
        assert!(ios.contains(
            ".padding(doweResponsive(viewportWidth, xs: CGFloat(16), lg: CGFloat(20)) ?? CGFloat(0))"
        ));
        assert!(ios.contains(
            "LazyVGrid(columns: doweGridColumns(doweResponsive(viewportWidth, xs: 1, md: 3) ?? 1, spacing: doweResponsive(viewportWidth, xs: CGFloat(20))),"
        ));
    }

    #[test]
    fn compiles_layout_bars_without_ios_dividers() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    AppBar variant:"soft" scheme:"surface" bordered:true boxed:true
      start
        Text
          "Dowe"
    children
    BottomBar variant:"soft" scheme:"surface" bordered:true boxed:true
      center
        Text
          "Home"
    Footer scheme:"background" bordered:true boxed:true
      end
        Text
          "Built with Dowe""#,
            r#"page loginPage
  Text
    "Login""#,
        );

        compile_dev(temp.path()).expect("project");

        let ios = ios_swift_output(temp.path());
        assert!(!ios.contains(
            ".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))"
        ));
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

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
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
        let router = fs::read_to_string(temp.path().join(".dowe/web/router.js")).expect("router");
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
        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
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
                .contains(r#"href="https://example.com/docs" data-dowe-external-mode="webview" target="_blank" rel="noopener""#)
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
        assert!(android_dev.contains(
            "setOnClickListener(v -> doweNavigate(\"replace\", currentPath, \"hero\"))"
        ));
        assert!(ios_routing.contains("dowe-dev://generated/signup"));
        assert!(ios_pages.contains("struct DoweRouteEntry: Hashable"));
        assert!(ios_pages.contains("@State private var navigationPath: [DoweRouteEntry] = []"));
        assert!(ios_pages.contains("routeContent(currentEntry, viewportWidth:"));
        assert!(ios_pages.contains(".simultaneousGesture(backSwipeGesture)"));
        assert!(ios_pages.contains(r#"{ navigate("replace", "", "hero") }"#));
        assert!(ios_pages.contains(r#"{ navigate("push", "/signup", "join") }"#));
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
    fn rejects_invalid_design_system_props() {
        assert_compile_error(
            r#"page loginPage
  Box bg:"softBackground"
    Text
      "Login""#,
            "invalid value for prop `bg`",
        );

        assert_compile_error(
            r#"page loginPage
  Box text:onBackground
    Text
      "Login""#,
            "unknown prop `text`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Button color:"primary"
      "Login""#,
            "unknown prop `color`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Text text:"Login""#,
            "unknown prop `text`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Title text:"Login""#,
            "unknown prop `text`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Button text:"Login""#,
            "unknown prop `text`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Card color:"primary"
      Text
        "Login""#,
            "unknown prop `color`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Input color:"primary""#,
            "unknown prop `color`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Alert type:"success" message:"Saved" color:"primary""#,
            "unknown prop `color`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Button scheme:"onPrimary"
      "Login""#,
            "invalid value for prop `scheme`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Flex aling:center
      Text
        "Login""#,
            "unknown prop `aling`",
        );

        assert_compile_error(
            r#"page loginPage
  Box p:13
    Text
      "Login""#,
            "invalid value for prop `p`",
        );

        assert_compile_error(
            r#"page loginPage
  Box p:97
    Text
      "Login""#,
            "invalid value for prop `p`",
        );

        assert_compile_error(
            r#"page loginPage
  Box border:5
    Text
      "Login""#,
            "invalid value for prop `border`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Text weight:"heavy"
      "Login""#,
            "invalid value for prop `weight`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Title spacing:"-0.02em"
      "Login""#,
            "invalid value for prop `spacing`",
        );

        assert_compile_error(
            r#"page loginPage
  Grid justify:"between"
    Text
      "Login""#,
            "invalid value for prop `justify`",
        );

        assert_compile_error(
            r#"page loginPage
  Box colSpan:2
    Text
      "Login""#,
            "`colSpan` and `rowSpan` can only be used",
        );

        assert_compile_error(
            r#"page loginPage
  Box overlay:true
    Text
      "Login""#,
            "`overlay` requires `cover`",
        );

        assert_compile_error(
            r#"page loginPage
  Box cover:"javascript:alert(1)"
    Text
      "Login""#,
            "invalid value for prop `cover`",
        );
    }

    #[test]
    fn validates_design_copilot_dowe_before_generation() {
        let valid = validate_design_copilot_dowe(
            r#"page loginPage
  Box p:{ xs:2 md:4 }
    Text size:"md"
      "Login""#,
        )
        .expect("valid copilot dowe");

        assert_eq!(
            dowe_components::first_text(&valid),
            Some("Login".to_string())
        );

        let error = validate_design_copilot_dowe(
            r#"page loginPage
  Box
    Button scheme:"onPrimary"
      "Login""#,
        )
        .expect_err("invalid copilot dowe");

        assert!(error.to_string().contains("dowe-copilot.dowe"));
        assert!(error.to_string().contains("invalid value for prop `scheme`"));
    }

    #[test]
    fn rejects_invalid_select_options() {
        assert_compile_error(
            r#"page loginPage
  Box
    Option value:"admin" label:"Admin""#,
            "Option can only be used inside Select",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Select
      Option value:"admin" label:"Admin"
      Option value:"admin" label:"Again""#,
            "duplicate Select option value `admin`",
        );

        assert_compile_error(
            r#"page loginPage
  Box
    Select
      Option value:admin label:"Admin""#,
            "expected quoted static string literal",
        );
    }

    #[test]
    fn compiles_code_blocks_with_cross_target_copy_and_highlighting() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    children"#,
            r#"page loginPage
  Box
    Text
      "Documentation"
    Code language:"dowe" variant:"soft" scheme:"surface" lines:["page docsPage","  Card variant:\"soft\" p:4 show:true","    Text","      \"Documentation\""]
    Code language:"typescript" scheme:"info" lines:["const value: string = \"Dowe\""]
    Code language:"go" scheme:"success" lines:["func main() {}"]
    Code language:"rust" scheme:"warning" lines:["fn main() {}"]"#,
        );

        let project = compile_dev(temp.path()).expect("project");
        let body = &project.web.pages[0].body_html;
        assert!(body.contains("data-dowe-code"));
        assert!(body.contains("code-token-keyword"));
        assert!(body.contains("code-token-type"));
        assert!(body.contains("docsPage"));
        assert!(project.web.router_js.contains("navigator.clipboard"));

        let android = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
        )
        .expect("android");
        assert!(android.contains("private fun DoweCode("));
        assert!(android.contains("clipboard.setText(AnnotatedString(source))"));
        assert!(android.contains("DoweCode(source = \"page docsPage\\n  Card variant:\\\"soft\\\" p:4 show:true\\n    Text\\n      \\\"Documentation\\\"\""));
        assert!(android.contains("DoweDesign.primary"));
        assert!(android.contains("DoweDesign.info"));
        assert!(android.contains("DoweDesign.success"));
        assert!(android.contains("DoweDesign.warning"));
        assert!(android.contains("DoweDesign.tertiary"));
        assert!(android.contains("DoweDesign.danger"));

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(android_dev.contains("private LinearLayout doweCode("));
        assert!(android_dev.contains("ClipboardManager clipboard"));
        assert!(android_dev.contains("new ForegroundColorSpan(tokenColors[index])"));

        let ios = ios_swift_output(temp.path());
        assert!(ios.contains("struct DoweCodeView: View"));
        assert!(ios.contains("UIPasteboard.general.string = source"));
        assert!(ios.contains("DoweCodeView(source: \"page docsPage\\n  Card variant:\\\"soft\\\" p:4 show:true\\n    Text\\n      \\\"Documentation\\\"\""));
        assert!(ios.contains("DoweDesign.primary"));
        assert!(ios.contains("DoweDesign.info"));
        assert!(ios.contains("DoweDesign.success"));
        assert!(ios.contains("DoweDesign.warning"));
        assert!(ios.contains("DoweDesign.tertiary"));
        assert!(ios.contains("DoweDesign.danger"));
    }

    #[test]
    fn rejects_invalid_code_blocks() {
        assert_compile_error(
            r#"page loginPage
  Text
    "Documentation"
  Code"#,
            "`Code` requires `lines`",
        );
        assert_compile_error(
            r#"page loginPage
  Text
    "Documentation"
  Code lines:[]"#,
            "`Code lines` must be a non-empty string array",
        );
        assert_compile_error(
            r#"page loginPage
  Text
    "Documentation"
  Code lines:[docs]"#,
            "`Code lines` must contain quoted static strings",
        );
        assert_compile_error(
            r#"page loginPage
  Text
    "Documentation"
  Code language:"python" lines:["print()"]"#,
            "invalid value for prop `language`: expected dowe, typescript, go or rust",
        );
        assert_compile_error(
            r#"page loginPage
  Text
    "Documentation"
  Code lines:["docs"]
    Text
      "Child""#,
            "children are not valid for this component",
        );
    }

    #[test]
    fn compiles_video_with_cross_target_hls_playback() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    children"#,
            r#"page loginPage
  Box
    Text
      "Video"
    Video src:"https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8" poster:"/images/video.jpg" autoplay:true aspect:"vertical" variant:"outlined" scheme:"surface""#,
        );

        let project = compile_dev(temp.path()).expect("project");
        let body = &project.web.pages[0].body_html;
        assert!(body.contains("data-dowe-video"));
        assert!(body.contains("controls playsinline"));
        assert!(body.contains(r#"poster="/images/video.jpg""#));
        assert!(body.contains("video vertical is-outlined is-surface"));
        assert!(project.web.router_js.contains("application/vnd.apple.mpegurl"));
        assert!(project.web.router_js.contains("https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js"));

        let android = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
        )
        .expect("android");
        assert!(android.contains("private fun DoweVideo("));
        assert!(android.contains("VideoView(context)"));
        assert!(android.contains("MediaController(context)"));
        assert!(android.contains("aspect = \"vertical\""));

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(android_dev.contains("private FrameLayout doweVideo("));
        assert!(android_dev.contains("VideoView video = new VideoView(this)"));

        let ios = ios_swift_output(temp.path());
        assert!(ios.contains("import AVKit"));
        assert!(ios.contains("struct DoweVideoView: View"));
        assert!(ios.contains("VideoPlayer(player: player)"));
        assert!(ios.contains("aspect: \"vertical\""));
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
        assert!(project.web.chunks.iter().any(|chunk| chunk
            .css_content
            .contains(".divider.is-primary{background-color:var(--dowe-primary);")));

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

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
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
