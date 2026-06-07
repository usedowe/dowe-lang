    #[test]
    fn compiles_design_system_components_and_responsive_props() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box bg:"background" color:"onBackground" p:{ xs:2 md:4 }
    Text size:"sm"
      Shell
    children"#,
            r#"page loginPage
  Box p:10 px:0.5 w:"full"
    Flex justify:"center" align:"center" gap:{ xs:2 lg:6 }
      Card variant:"soft" scheme:"primary" rounded:"lg" border:1 p:{ xs:4 md:8 }
        Title size:"2xl" bg:"softPrimary" weight:"extrabold" spacing:"tight" p:4
          Welcome
        Text size:"md" bg:"surface" color:"onPrimary" weight:"bold" spacing:"wide" rounded:"md" border:1
          Login
        Button variant:"solid" scheme:"danger"
          Save
        Button variant:"soft" scheme:"warning" size:"lg" rounded:"full"
          Warn
        Input variant:"outlined" scheme:"info"
        Card scheme:"primary"
          Text
            Default"#,
        );

        let project = compile_dev(temp.path()).expect("project");
        let body = &project.web.pages[0].body_html;

        assert!(body.contains(r#"class="box bg-background color-onBackground p-2 md:p-4""#));
        assert!(body.contains(r#"class="box p-10 px-0.5 w-full""#));
        assert!(body.contains(r#"class="flex justify-center align-center gap-2 lg:gap-6""#));
        assert!(
            body.contains(
                r#"class="card p-4 md:p-8 rounded-lg border-1 is-soft is-primary""#
            )
        );
        assert!(body.contains(
            r#"class="title-2xl bg-softPrimary p-4 weight-extrabold tracking-tight""#
        ));
        assert!(body.contains(
            r#"class="text-md bg-surface color-onPrimary rounded-md border-1 weight-bold tracking-wide""#
        ));
        assert!(
            body.contains(r#"class="button button-md px-4 py-2.5 min-h-10 is-solid is-danger""#)
        );
        assert!(body.contains(
            r#"class="button button-lg px-5 py-3 min-h-11 rounded-full is-soft is-warning""#
        ));
        assert!(
            body.contains(
                r#"<div class="control is-outlined is-info"><input class="input"></div>"#
            )
        );
        assert!(body.contains(r#"class="card is-solid is-primary""#));
        assert!(!body.contains(r#"class="card p-4 md:p-6 lg:p-8"#));

        let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
        assert!(css.contains("--dowe-primary"));
        assert!(css.contains("--dowe-softDanger"));
        assert!(!css.contains(".p-96"));
        let layout_css_path = temp
            .path()
            .join(".dowe/web")
            .join(&project.web.pages[0].css_chunks[0]);
        let layout_css = fs::read_to_string(layout_css_path).expect("layout css");
        assert!(layout_css.contains(".color-onBackground{color:var(--dowe-onBackground);}"));

        let page_css_path = temp
            .path()
            .join(".dowe/web")
            .join(&project.web.pages[0].css_chunks[1]);
        let page_css = fs::read_to_string(page_css_path).expect("page css");
        assert!(page_css.contains(".p-10{padding:2.5rem;}"));
        assert!(page_css.contains(".px-0\\.5{padding-left:0.125rem;padding-right:0.125rem;}"));
        assert!(page_css.contains(".md\\:p-8"));
        assert!(!page_css.contains(".lg\\:p-8"));
        assert!(page_css.contains(".lg\\:gap-6"));
        assert!(page_css.contains(".title-2xl{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(1.75rem, 1.4rem + 1vw, 2.25rem);line-height:1.2;font-weight:700;letter-spacing:-0.025em;margin:0;}"));
        assert!(page_css.contains(".text-md{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(0.875rem, 0.82rem + 0.25vw, 1rem);line-height:1.6;font-weight:400;margin:0;}"));
        assert!(page_css.contains(".color-onPrimary{color:var(--dowe-onPrimary);}"));
        assert!(page_css.contains(".button-md{padding:0.625rem 1rem;min-height:2.5rem;}"));
        assert!(page_css.contains(".button-lg{padding:0.75rem 1.25rem;min-height:2.75rem;}"));
        assert!(page_css.contains(".min-h-10{min-height:2.5rem;}"));
        assert!(page_css.contains(".rounded-full{border-radius:9999px;}"));
        assert!(page_css.contains(".weight-extrabold{font-weight:800;}"));
        assert!(page_css.contains(".tracking-wide{letter-spacing:0.02em;}"));
        assert!(!page_css.contains(".p-96"));

        let android = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
        )
        .expect("android");
        assert!(android.contains("object DoweDesign"));
        assert!(android.contains("val softWarning"));
        assert!(android.contains("Button("));
        assert!(android.contains("DoweDesign.danger"));
        assert!(android.contains("DoweDesign.softWarning"));
        assert!(android.contains("DoweDesign.onSoftWarning"));
        assert!(android.contains("all = doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)"));
        assert!(android.contains("horizontal = doweResponsive(viewportWidth, xs = 20.dp)"));
        assert!(android.contains("vertical = doweResponsive(viewportWidth, xs = 12.dp)"));
        assert!(android.contains("doweResponsive(viewportWidth, xs = DoweSize.Fixed(44.dp))"));
        assert!(android.contains(
            "RoundedCornerShape(doweResponsive(viewportWidth, xs = 999.dp) ?: DoweDesign.radiusUi)"
        ));
        assert!(android.contains("DoweInput("));
        assert!(android.contains("DoweDesign.info"));
        assert!(android.contains("FontWeight.ExtraBold"));
        assert!(android.contains("xs = (-0.02f).em"));
        assert!(android.contains("DoweDesign.softPrimary"));

        let ios = ios_swift_output(temp.path());
        assert!(ios.contains("enum DoweDesign"));
        assert!(ios.contains("static let softWarning"));
        assert!(ios.contains("Button(action: {})"));
        assert!(ios.contains("DoweDesign.danger"));
        assert!(ios.contains("DoweDesign.softWarning"));
        assert!(ios.contains("DoweDesign.onSoftWarning"));
        assert!(
            ios.contains(
                ".padding(doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0))"
            )
        );
        assert!(!ios.contains("xs: CGFloat(16), md: CGFloat(24), lg: CGFloat(32)"));
        assert!(ios.contains(
            ".padding(.horizontal, doweResponsive(viewportWidth, xs: CGFloat(20)) ?? CGFloat(0))"
        ));
        assert!(ios.contains(
            ".padding(.vertical, doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0))"
        ));
        assert!(ios.contains("DoweSize.fixed(CGFloat(44))"));
        assert!(ios.contains(
            "RoundedRectangle(cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(999)) ?? DoweDesign.radiusUi)"
        ));
        assert!(ios.contains("DoweInputField(value: nil, label: nil, placeholder: \"\", floating: false"));
        assert!(ios.contains("Font.Weight.heavy"));
        assert!(ios.contains("doweTextTracking"));
        assert!(ios.contains("DoweDesign.softPrimary"));
    }

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
      Weighted text
    Title weight:"black"
      Weighted title"#,
        );

        let project = compile_dev(temp.path()).expect("project");
        let body = &project.web.pages[0].body_html;

        assert!(body.contains(
            r#"class="text-md weight-thin md:weight-extralight lg:weight-black""#
        ));
        assert!(body.contains(r#"class="title-md weight-black""#));

        let page_css_path = temp
            .path()
            .join(".dowe/web")
            .join(&project.web.pages[0].css_chunks[1]);
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

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(android_dev.contains(
            "doweTextWeight(doweResponsiveInt(viewportWidth, 100, null, 200, 900, null), 400)"
        ));
        assert!(android_dev.contains("doweTextWeight(doweResponsiveInt(viewportWidth, 900, null, null, null, null), 400)"));

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
      Layout
    children"#,
            r#"page loginPage
  Box font:{ xs:"inter" md:"lato" }
    Text
      Inherited
    Text font:"manrope"
      Lead
    Title font:"poppins"
      Login
    Button font:"montserrat"
      Submit
    Input font:"roboto"
    Text font:"lora"
      Caption"#,
        );

        let project = compile_dev(temp.path()).expect("project");
        let body = &project.web.pages[0].body_html;

        assert!(body.contains(r#"class="box font-roboto""#));
        assert!(body.contains(r#"class="box font-inter md:font-lato""#));
        assert!(body.contains(r#"class="text-md font-manrope""#));
        assert!(body.contains(r#"class="title-md font-poppins""#));
        assert!(body.contains(
            r#"class="button button-md font-montserrat px-4 py-2.5 min-h-10 is-solid is-primary""#
        ));
        assert!(body.contains(r#"class="control font-roboto is-solid is-primary""#));
        assert!(body.contains(r#"class="text-md font-lora""#));

        let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
        assert!(css.contains("body{margin:0;"));
        assert!(css.contains("p,h1,h2,h3,h4,h5,h6{margin:0;"));
        assert!(css.contains("a{color:inherit;text-decoration:inherit;}"));
        assert!(css.contains("button,input,textarea,select{font:inherit;color:inherit;margin:0;}"));
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
            .join(&project.web.pages[0].css_chunks[1]);
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
            android
                .contains("doweResponsive(viewportWidth, xs = DoweFont.Inter, md = DoweFont.Lato)")
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

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(android_dev.contains("setAllCaps(false)"));
        assert!(android_dev.contains(
            "doweResponsiveString(viewportWidth, \"Inter\", null, \"Lato\", null, null)"
        ));

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
        let plist =
            fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("plist");
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
      Layout
    children"#,
            r#"page loginPage
  Box
    Text
      Login"#,
        );
        fs::write(
            temp.path().join("src/config.dowe"),
            r#"config
  fonts default:"manrope" install:["lora"]"#,
        )
        .expect("config");

        let project = compile_dev(temp.path()).expect("project");
        assert_eq!(
            project.font_config.default_family,
            dowe_components::FontFamily::Manrope
        );

        let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
        assert!(css.contains("html{font-family:var(--dowe-font-manrope);"));
        assert!(css.contains("body{margin:0;"));
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

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(android_dev.contains("return value == null ? \"Manrope\" : value;"));

        let plist =
            fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("plist");
        assert!(plist.contains("Fonts/manrope-regular.ttf"));
        assert!(plist.contains("Fonts/lora-regular.ttf"));
        assert!(!plist.contains("Fonts/inter-regular.ttf"));
    }

    #[test]
    fn compiles_design_tokens_from_config_dowe() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box
    Text
      Layout
    children"#,
            r#"page loginPage
  Card scheme:"primary"
    Text
      Login"#,
        );
        fs::write(
            temp.path().join("src/config.dowe"),
            r##"config
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    theme name:"light"
      colors primary:"#000000" onPrimary:"#ffffff"
      radii radius:10 radiusBox:14 radiusUi:9
    theme name:"dark"
      colors primary:"#ffffff" onPrimary:"#000000""##,
        )
        .expect("config");

        let project = compile_dev(temp.path()).expect("project");

        assert_eq!(project.design_config.default_theme, "light");
        assert!(project.design_config.theme("dark").is_some());

        let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
        assert!(css.contains("--dowe-primary:#000000;"));
        assert!(css.contains("--dowe-radius:10px;"));
        assert!(css.contains("--dowe-radiusBox:14px;"));
        assert!(css.contains("--dowe-radiusUi:9px;"));
        assert!(css.contains("[data-dowe-theme=\"dark\"]{"));
        assert!(css.contains("--dowe-primary:#ffffff;"));

        let android_theme = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweTheme.kt"),
        )
        .expect("android theme");
        assert!(android_theme.contains("const val defaultTheme = \"light\""));
        assert!(android_theme.contains("\"dark\""));
        assert!(android_theme.contains("\"primary\" to Color(0xFF000000)"));
        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(
            android_dev.contains("private static final int DOWE_PRIMARY = Color.rgb(0, 0, 0);")
        );
        assert!(android_dev.contains("private static final float DOWE_RADIUS_UI = 9f;"));

        let ios_theme =
            fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweTheme.swift"))
                .expect("ios theme");
        assert!(ios_theme.contains("static let defaultTheme = \"light\""));
        assert!(ios_theme.contains("\"dark\""));
        assert!(ios_theme.contains("\"primary\": Color(red: 0.000, green: 0.000, blue: 0.000)"));
    }

    #[test]
    fn compiles_mobile_responsive_runtime_values() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture_with_views(
            temp.path(),
            r#"layout AuthLayout
  Box p:{ xs:4 md:8 }
    Text
      Layout
    children"#,
            r#"page loginPage
  Box p:{ md:8 }
    Text size:{ md:"lg" }
      Login"#,
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
            "fun IndexScreen(viewportWidth: Dp, sectionRegistry: DoweSectionRegistry, navigate:"
        ));
        assert!(android.contains("doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)"));
        assert!(android.contains("doweResponsive(viewportWidth, md = 32.dp)"));
        assert!(android.contains(
            "doweResponsive(viewportWidth, md = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)) ?: doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)"
        ));

        let android_dev = fs::read_to_string(
            temp.path()
                .join(".dowe/apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java"),
        )
        .expect("android dev");
        assert!(
            android_dev
                .contains("viewportWidth = getResources().getConfiguration().screenWidthDp;")
        );
        assert!(android_dev.contains("int viewportWidth = this.viewportWidth;"));
        assert!(android_dev.contains("doweResponsiveInt(viewportWidth, 16, null, 32, null, null)"));
        assert!(
            android_dev.contains("doweResponsiveInt(viewportWidth, null, null, 32, null, null)")
        );

        let ios = ios_swift_output(temp.path());
        assert!(ios.contains("GeometryReader { geometry in"));
        assert!(ios.contains("let viewportWidth: CGFloat"));
        assert!(
            ios.contains(
                ".padding(doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0))"
            )
        );
        assert!(
            ios.contains(".padding(doweResponsive(viewportWidth, md: CGFloat(32)) ?? CGFloat(0))")
        );
        assert!(ios.contains(
            ".font(doweFont(.inter, size: doweResponsive(viewportWidth, md: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))) ?? doweTextSize(viewportWidth, min: CGFloat(14), preferredBase: CGFloat(13.12), preferredViewport: CGFloat(0.25), max: CGFloat(16))))"
        ));
        assert!(ios.contains(".fontWeight(Font.Weight.regular)"));
    }
