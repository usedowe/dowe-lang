#[test]
fn compiles_expanded_text_weight_overrides() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text weight:{ xs:"thin" md:"extralight" lg:"black" }
      "Weighted text"
    Title weight:"black"
      "Weighted title""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(
        body.contains(
            r#"class="dowe-text text-md weight-thin md:weight-extralight lg:weight-black""#
        )
    );
    assert!(body.contains(r#"class="dowe-title title-md weight-black""#));

    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(generated_css_chunk(
            &project.web.pages[0].css_chunks,
            "chunks/pages/",
        ));
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains(".weight-thin{font-weight:100;}"));
    assert!(page_css.contains(".md\\:weight-extralight{font-weight:200;}"));
    assert!(page_css.contains(".lg\\:weight-black{font-weight:900;}"));
    assert!(page_css.contains(".weight-black{font-weight:900;}"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("FontWeight.Thin"));
    assert!(android.contains("FontWeight.ExtraLight"));
    assert!(android.contains("FontWeight.Black"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains(
        "doweTextWeight(doweResponsiveInt(viewportWidth, 100, null, 200, 900, null), 400)"
    ));
    assert!(android_dev.contains(
        "doweTextWeight(doweResponsiveInt(viewportWidth, 900, null, null, null, null), 400)"
    ));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("Font.Weight.ultraLight"));
    assert!(ios.contains("Font.Weight.thin"));
    assert!(ios.contains("Font.Weight.black"));
}

#[test]
fn compiles_platform_reset_and_font_tokens() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box font:"roboto"
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box font:{ xs:"inter" md:"lato" }
    Text
      "Inherited"
    Text font:"manrope"
      "Lead"
    Title font:"poppins"
      "Login"
    Button font:"montserrat"
      "Submit"
    Input font:"roboto"
    Text font:"lora"
      "Caption""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains(r#"class="box font-roboto""#));
    assert!(body.contains(r#"class="box font-inter md:font-lato""#));
    assert!(body.contains(r#"class="dowe-text text-md font-manrope""#));
    assert!(body.contains(r#"class="dowe-title title-md font-poppins""#));
    assert!(body.contains(
        "is-solid is-primary"
    ));
    assert!(body.contains("font-roboto"));
    assert!(body.contains("is-md font-roboto is-outlined is-primary"));
    assert!(body.contains(r#"class="dowe-text text-md font-lora""#));

    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    assert!(css.contains("body{--dowe-content-text:var(--dowe-backgroundText);--dowe-content-title:var(--dowe-backgroundTitle);margin:0;"));
    assert!(css.contains("scroll-behavior:smooth;"));
    assert!(css.contains("p,h1,h2,h3,h4,h5,h6{margin:0;"));
    assert!(css.contains("a{color:inherit;text-decoration:inherit;}"));
    assert!(css.contains("button,input,textarea,select{font:inherit;color:inherit;margin:0;}"));
    assert!(css.contains("::-webkit-scrollbar{width:var(--dowe-scrollbar-size,6px);"));
    assert!(css.contains("::-webkit-scrollbar-thumb{background:color-mix(in oklch,currentColor 80%,transparent);border-radius:999px;}"));
    assert!(css.contains("@supports not selector(::-webkit-scrollbar){*{scrollbar-width:thin;"));
    assert!(css.contains("--dowe-font-inter"));
    assert!(css.contains("@font-face{font-family:\"Dowe Inter\""));
    assert!(css.contains("font-weight:100;src:url(\"/fonts/inter/inter-light.ttf\")"));
    assert!(css.contains("src:url(\"/fonts/inter/inter-regular.ttf\") format(\"truetype\")"));
    assert!(css.contains("font-weight:900;src:url(\"/fonts/inter/inter-extrabold.ttf\")"));
    assert!(
        temp.path()
            .join(".dowe/fonts/inter/inter-regular.ttf")
            .is_file()
    );
    assert!(!temp.path().join(".dowe/fonts/quicksand").exists());

    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(generated_css_chunk(
            &project.web.pages[0].css_chunks,
            "chunks/pages/",
        ));
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains(".font-poppins{font-family:var(--dowe-font-poppins);}"));
    assert!(page_css.contains(".font-manrope{font-family:var(--dowe-font-manrope);}"));
    assert!(page_css.contains(".font-lora{font-family:var(--dowe-font-lora);}"));
    assert!(page_css.contains(".md\\:font-lato{font-family:var(--dowe-font-lato);}"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("private enum class DoweFont"));
    assert!(android.contains("Font(R.font.inter_light, FontWeight.Thin)"));
    assert!(android.contains("Font(R.font.inter_regular, FontWeight.Normal)"));
    assert!(android.contains("Font(R.font.inter_extrabold, FontWeight.Black)"));
    assert!(android.contains("DoweFont.Lato -> DoweFonts.lato"));
    assert!(android.contains("DoweFont.Manrope -> DoweFonts.manrope"));
    assert!(android.contains("DoweFont.Lora -> DoweFonts.lora"));
    assert!(
        android.contains("doweResponsive(viewportWidth, xs = DoweFont.Inter, md = DoweFont.Lato)")
    );
    assert!(android.contains("xs = DoweFont.Poppins"));
    assert!(android.contains("contentPadding = PaddingValues(0.dp)"));
    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/font/inter_regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/font/manrope_regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/font/lora_regular.ttf")
            .is_file()
    );

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("setAllCaps(false)"));
    assert!(
        android_dev
            .contains("doweResponsiveString(viewportWidth, \"Inter\", null, \"Lato\", null, null)")
    );

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("enum DoweFont"));
    assert!(ios.contains("doweResponsive(viewportWidth, xs: .inter, md: .lato)"));
    assert!(ios.contains("xs: .poppins"));
    assert!(ios.contains("xs: .manrope"));
    assert!(ios.contains("xs: .lora"));
    assert!(ios.contains(".buttonStyle(.plain)"));
    assert!(ios.contains(".textFieldStyle(.plain)"));
    assert!(
        temp.path()
            .join(".dowe/apps/ios/Fonts/inter-regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/ios/Fonts/manrope-regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/ios/Fonts/lora-regular.ttf")
            .is_file()
    );
    let plist = fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("plist");
    assert!(plist.contains("UIAppFonts"));
    assert!(plist.contains("Fonts/inter-regular.ttf"));
}

#[test]
fn compiles_configured_font_install_set() {
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
      "Login""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  fonts default:"manrope" install:["lora"]"#,
    )
    .expect("config");

    let project = compile_dev(temp.path()).expect("project");
    assert_eq!(
        project.font_config.default_family,
        dowe_components::FontFamily::Manrope
    );

    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    assert!(css.contains("--dowe-font-default:var(--dowe-font-manrope);"));
    assert!(css.contains("html{font-family:var(--dowe-font-default);"));
    assert!(css.contains("body{--dowe-content-text:var(--dowe-backgroundText);--dowe-content-title:var(--dowe-backgroundTitle);margin:0;"));
    assert!(css.contains("--dowe-font-manrope"));
    assert!(css.contains("--dowe-font-lora"));
    assert!(!css.contains("--dowe-font-inter"));
    assert!(!css.contains("--dowe-font-poppins"));
    assert!(
        temp.path()
            .join(".dowe/fonts/manrope/manrope-regular.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/fonts/lora/lora-regular.ttf")
            .is_file()
    );
    assert!(!temp.path().join(".dowe/fonts/inter").exists());

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("null -> DoweFonts.manrope"));
    assert!(android.contains("DoweFont.Lora -> DoweFonts.lora"));
    assert!(!android.contains("R.font.inter_regular"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("return value == null ? \"Manrope\" : value;"));

    let plist = fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("plist");
    assert!(plist.contains("Fonts/manrope-regular.ttf"));
    assert!(plist.contains("Fonts/lora-regular.ttf"));
    assert!(!plist.contains("Fonts/inter-regular.ttf"));
}

#[test]
fn compiles_syne_jost_and_puritan_across_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text font:"jost"
      "Jost"
    Text font:"puritan"
      "Puritan""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  fonts default:"syne" install:["jost","puritan"]"#,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("project");

    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    assert!(css.contains("--dowe-font-syne"));
    assert!(css.contains("/fonts/syne/syne-variable.ttf"));
    assert!(css.contains("--dowe-font-jost"));
    assert!(css.contains("--dowe-font-puritan"));
    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/font/jost_variable.ttf")
            .is_file()
    );
    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/font/puritan_bold.ttf")
            .is_file()
    );

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("case syne"));
    assert!(ios.contains("case jost"));
    assert!(ios.contains("case puritan"));
    assert!(
        temp.path()
            .join(".dowe/apps/ios/Fonts/syne-variable.ttf")
            .is_file()
    );
}

