#[test]
fn compiles_design_tokens_from_theme_dowe() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box
    Card
      Text
        "Default"
      Button
        "Go"
    Tabs
      tab id:"overview" label:"Overview"
        Text
          "Overview"
    Card variant:"solid" scheme:"success" p:5 rounded:"md"
      Text
        "Override""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    Card radius:"md" shadow:"xs" shadowColor:"primary" border:1 borderColor:"primary" scheme:"surface" variant:"outline"
    Button radius:"md" scheme:"primary" variant:"solid" size:"md"
    Tabs variant:"line"
    Chip radius:"md"
    Avatar radius:"full"
    theme name:"light"
      colors:
        primary color:"#000000" text:"#ffffff" title:"#ffffff"
    theme name:"dark"
      colors:
        primary color:"#ffffff" text:"#000000" title:"#000000""##,
    )
    .expect("config");

    let project = compile_dev(temp.path()).expect("project");

    assert_eq!(project.design_config.default_theme, "light");
    assert!(project.design_config.theme("dark").is_some());
    assert_eq!(
        project
            .design_config
            .defaults
            .tabs_variant
            .get(&dowe_components::DesignComponentSlot::Tabs),
        Some(&dowe_components::TabsVariant::Line)
    );

    let body = &project.web.pages[0].body_html;
    assert!(body.contains("card p-4 lg:p-5 rounded-md border-1 border-color-primary shadow-xs shadow-color-primary is-outlined is-surface"));
    assert!(body.contains("button button-md"));
    assert!(body.contains("is-solid is-primary"));
    assert!(body.contains("card p-5 rounded-md border-1 border-color-primary shadow-xs shadow-color-primary is-solid is-success"));
    assert!(body.contains("tabs"));

    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    assert!(css.contains("--dowe-primary:#000000;"));
    assert!(css.contains("--dowe-radius:8px;"));
    assert!(css.contains("[data-dowe-theme=\"dark\"]{"));
    assert!(css.contains("--dowe-primary:#ffffff;"));
    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(generated_css_chunk(
            &project.web.pages[0].css_chunks,
            "chunks/pages/",
        ));
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains("border-color"));
    assert!(page_css.contains(".shadow-xs{box-shadow:"));
    assert!(page_css.contains(".shadow-color-primary{--dowe-shadow-color:"));

    let android_theme = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweTheme.kt"),
    )
    .expect("android theme");
    assert!(android_theme.contains("const val defaultTheme = \"light\""));
    assert!(android_theme.contains("\"dark\""));
    assert!(android_theme.contains("\"primary\" to Color(0xFF000000)"));
    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("DOWE_PRIMARY"));
    assert!(android_dev.contains("DOWE_RADIUS"));
    let android_pages = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    assert!(android_pages.contains(".doweShadow("));
    assert!(android_pages.contains("color = DoweDesign.primary"));

    let ios_theme =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweTheme.swift")).expect("ios theme");
    assert!(ios_theme.contains("static let defaultTheme = \"light\""));
    assert!(ios_theme.contains("\"dark\""));
    assert!(ios_theme.contains("\"primary\": Color(red: 0.000, green: 0.000, blue: 0.000)"));
    let ios_pages = ios_swift_output(temp.path());
    assert!(ios_pages.contains(
        ".background(DoweShadowSurface(shadow: DoweShadowSpec(color: DoweDesign.primary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(2)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(1)) ?? CGFloat(0))"
    ));
}

#[test]
fn compiles_text_and_title_font_defaults_from_theme_dowe() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box
    Text
      "Default text"
    Text font:"inter"
      "Override text"
    Title
      "Default title"
    Title font:"inter"
      "Override title""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    Text font:"manrope"
    Title font:"syne"
    theme name:"light""#,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("project");
    let defaults = &project.design_config.defaults;
    assert_eq!(
        defaults
            .font
            .get(&dowe_components::DesignComponentSlot::Text),
        Some(&dowe_components::FontFamily::Manrope)
    );
    assert_eq!(
        defaults
            .font
            .get(&dowe_components::DesignComponentSlot::Title),
        Some(&dowe_components::FontFamily::Syne)
    );

    let body = &project.web.pages[0].body_html;
    assert!(body.contains("font-manrope"));
    assert!(body.contains("font-inter"));
    assert!(body.contains("font-syne"));
    assert!(body.contains("font-inter"));

    assert!(
        temp.path()
            .join(".dowe/fonts/manrope/manrope-regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/fonts/syne/syne-variable.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/font/manrope_regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/font/syne_variable.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/ios/Fonts/manrope-regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/ios/Fonts/syne-variable.ttf")
            .is_file()
    );

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("xs = DoweFont.Manrope"));
    assert!(android.contains("xs = DoweFont.Syne"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("\"Manrope\".equals(font)"));
    assert!(android_dev.contains("return R.font.manrope_regular;"));
    assert!(android_dev.contains("\"Syne\".equals(font)"));
    assert!(android_dev.contains("return R.font.syne_variable;"));
    assert!(android_dev.contains("Typeface bundled = getResources().getFont(resource);"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("xs: .manrope"));
    assert!(ios.contains("xs: .syne"));
}

#[test]
fn compiles_mobile_responsive_runtime_values() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box p:{ xs:4 md:8 }
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box p:{ md:8 }
    Text size:{ md:"lg" }
      "Login""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(body.contains(r#"class="box p-4 md:p-8""#));
    assert!(body.contains(r#"class="box md:p-8""#));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("BoxWithConstraints"));
    assert!(android.contains(
        "fun IndexScreen(viewportWidth: Dp, scrollState: ScrollState, sectionRegistry: DoweSectionRegistry, navigate:"
    ));
    assert!(android.contains("doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)"));
    assert!(android.contains("doweResponsive(viewportWidth, md = 32.dp)"));
    assert!(android.contains(
            "doweResponsive(viewportWidth, md = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)) ?: doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)"
        ));

    let android_dev = android_dev_output(temp.path());
    assert!(
        android_dev.contains("viewportWidth = getResources().getConfiguration().screenWidthDp;")
    );
    assert!(android_dev.contains("int viewportWidth = this.viewportWidth;"));
    assert!(android_dev.contains("doweResponsiveInt(viewportWidth, 16, null, 32, null, null)"));
    assert!(android_dev.contains("doweResponsiveInt(viewportWidth, null, null, 32, null, null)"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("GeometryReader { geometry in"));
    assert!(ios.contains("let viewportWidth: CGFloat"));
    assert!(ios.contains(
        ".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0)"
    ));
    assert!(ios.contains("top: doweResponsive(viewportWidth, md: CGFloat(32)) ?? CGFloat(0)"));
    assert!(ios.contains(
            ".font(doweFont(.inter, size: doweResponsive(viewportWidth, md: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))) ?? doweTextSize(viewportWidth, min: CGFloat(14), preferredBase: CGFloat(13.12), preferredViewport: CGFloat(0.25), max: CGFloat(16))))"
        ));
    assert!(ios.contains(".fontWeight(Font.Weight.regular)"));
}
