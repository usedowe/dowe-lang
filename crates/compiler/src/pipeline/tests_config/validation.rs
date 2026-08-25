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
    assert!(source.contains(r#""moduleRoot": ".""#));
    assert!(source.contains(r#""projectRootAlias": "@/""#));
    assert!(source.contains(r#""assetsImportable": false"#));
    assert!(source.contains("dynamic text uses exactly one braced binding"));
    assert!(views.contains("main views:[dashboardRoutes docsRoutes]"));
    assert!(views.contains("userRoutes"));
    assert!(views.contains("blogRoutes"));
    assert!(server.contains(r#""root": "main.dowe""#));
    assert!(server.contains(r#""req.json""#));
    assert!(server.contains(r#""resolvedLogValues": true"#));
    assert!(server.contains(r#""const body:Type value:req.json""#));
    assert!(server.contains("functionName result args:{ input:value }"));
    assert!(!server.contains("let result = functionName"));
    assert!(server.contains(r#""nodeRuntime": false"#));
    assert!(views.contains(r#""root": "main.dowe""#));
    assert!(views.contains(r#""Box""#));
    assert!(views.contains(r#""Alert""#));
    assert!(views.contains("BottomBar tab href label Icon featured"));
    assert!(views.contains(r#""Svg""#));
    assert!(views.contains(r#""Path""#));
    assert!(views.contains(r#""Code""#));
    assert!(views.contains(r#""Video""#));
    assert!(views.contains(r#""Divider""#));
    assert!(views.contains("Section boxed:true"));
    assert!(views.contains("center:{ xs:false md:true }"));
    assert!(views.contains("gap:{ xs:2 md:4 }"));
    assert!(views.contains(r#""Input bind:signal.field""#));
    assert!(views.contains(r#""signalPathValidation""#));
    assert!(views.contains(r#"\"{blog.title}\" dynamic text child"#));
    assert!(views.contains(r#""signal rows type:Row[] value:[]""#));
    assert!(views.contains("any imported .dowe module"));
    assert!(views.contains("classified by store declaration"));
    assert!(!views.contains("views/store/**/*.dowe"));
    assert!(views.contains(r#""routing""#));
    assert!(views.contains("platform values"));
    assert!(views.contains(r#""metadata""#));
    assert!(views.contains(r#"meta name:\"title\" content:\"Page title\""#));
    assert!(views.contains("active layout then page by name"));
    assert!(views.contains("web SSR and browser routing only"));
    assert!(!views.contains(r#""Body""#));
    assert!(views.contains(r#""children""#));
    assert!(views.contains(r#""serverApisAvailable": false"#));
    assert!(config.contains(r#""themeRoot": "theme.dowe""#));
    assert!(config.contains(r#""envRoot": ".env""#));
    assert!(config.contains(r#""envExampleRoot": ".env.example""#));
    assert!(config.contains(r#""serverRoot": "main.dowe""#));
    assert!(config.contains(
        r#""obsoleteConfig": ["dowe.json", "env.dowe", "src/config.dowe", "src/main.dowe", "src/theme.dowe", "src/env.dowe"]"#
    ));
    assert!(config.contains(r#""defaultTheme": "light""#));
    assert!(config.contains(
        r##""declaration": "colors: -> primary color:\"#2563eb\" text:\"#ffffff\" title:\"#ffffff\"""##
    ));
    assert!(config.contains(r#""roles": ["color", "text", "title"]"#));
    assert!(config.contains(r#""flatRoleAuthoring": "rejected""#));
    assert!(config.contains(r#""fontSlots": ["text", "title"]"#));
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
    assert!(message.contains("theme.dowe"));
    assert!(message.contains("no longer supported"));
}
#[test]
fn rejects_entry_and_configuration_files_under_src() {
    for file_name in ["main.dowe", "theme.dowe"] {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::create_dir_all(temp.path().join("src")).expect("legacy source directory");
        fs::write(temp.path().join("src").join(file_name), "legacy\n").expect("legacy source");

        let error = compile_dev(temp.path()).expect_err("legacy location");
        let message = error.to_string();

        assert!(message.contains(&format!("src/{file_name}")));
        assert!(message.contains(&format!("project-root `{file_name}`")));
    }
}

#[test]
fn rejects_removed_environment_dowe_files() {
    for relative in ["env.dowe", "src/env.dowe"] {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let path = temp.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent");
        }
        fs::write(&path, "env\n").expect("removed env source");

        let error = compile_dev(temp.path()).expect_err("removed env.dowe");
        let message = error.to_string();

        assert!(message.contains(relative));
        assert!(message.contains(".env"));
        assert!(message.contains("no longer supported"));
    }
}

#[test]
fn rejects_invalid_theme_dowe_theme() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"dark"
    theme name:"light"
      colors:
        primary color:"#000000""##,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(
        error
            .to_string()
            .contains("default theme `dark` is not declared")
    );
}

#[test]
fn accepts_grouped_theme_color_families_and_rejects_flat_roles() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        primary color:"#1F3A5F" text:"#FFFFFF" title:"#FFFFFE"
        background color:"#FFFFFF" text:"#17263A" title:"#17263E"
        surface color:"#F7F9FC" text:"#17263A" title:"#17263E"
        softPrimary color:"#CCFBF3" text:"#073B35" title:"#073B35"
    theme name:"brand" extends:"light"
      colors:
        primary title:"#FFFEEE""##,
    )
    .expect("theme");

    compile_dev(temp.path()).expect("grouped theme color families");

    for flat in [
        "primary:\"#1F3A5F\"",
        "primaryText:\"#FFFFFF\"",
        "primaryTitle:\"#FFFFFE\"",
        "onPrimary:\"#FFFFFF\"",
        "onSuccess:\"#FFFFFF\"",
        "onSoftPrimary:\"#073B35\"",
    ] {
        fs::write(
            temp.path().join("theme.dowe"),
            format!(
                r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        {flat}"##
            ),
        )
        .expect("flat theme");

        let message = compile_dev(temp.path())
            .expect_err("flat theme role")
            .to_string();
        assert!(message.contains("grouped color families"), "{message}");
    }

    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        softBackground color:"#FFFFFF" text:"#17263A" title:"#17263E""##,
    )
    .expect("unknown family");
    let message = compile_dev(temp.path())
        .expect_err("unknown color family")
        .to_string();
    assert!(message.contains("unknown color family `softBackground`"), "{message}");
}

#[test]
fn compiles_custom_theme_color_families_for_all_view_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        r#"page loginPage
  Card scheme:"happy"
    Title
      "Saved"
    Text
      "Your changes are ready."
  Card variant:"soft" scheme:"happy"
    Title
      "Gentle success""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        happy color:"#176c75" text:"#fffffe" title:"#fffefe"
        softHappy color:"#d9f3f1" text:"#124d53" title:"#124d53"
    theme name:"dark"
      colors:
        happy color:"#55c2cc" text:"#071e20" title:"#071e20""##,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("custom theme family");
    let happy = dowe_components::ColorFamily::from_name("happy").expect("happy family");
    let light = project.design_config.theme("light").expect("light theme");
    let dark = project.design_config.theme("dark").expect("dark theme");

    assert_eq!(light.color_value(happy.color_token()), "#176c75");
    assert_eq!(light.color_value(happy.text_token()), "#fffffe");
    assert_eq!(light.color_value(happy.title_token()), "#fffefe");
    assert_eq!(light.color_value(happy.color_token()), "#d9f3f1");
    assert_eq!(dark.color_value(happy.color_token()), "#55c2cc");
    assert_eq!(dark.color_value(happy.color_token()), "#d9f3f1");

    let body = &project.web.pages[0].body_html;
    assert!(body.contains("is-solid is-happy"), "{body}");
    assert!(body.contains("is-soft is-happy"), "{body}");

    let design_css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("design css");
    assert!(design_css.contains("--dowe-happy:#176c75;"), "{design_css}");
    assert!(design_css.contains("--dowe-happyText:#fffffe;"), "{design_css}");
    assert!(design_css.contains("--dowe-happyTitle:#fffefe;"), "{design_css}");
    assert!(design_css.contains("--dowe-softHappy:#d9f3f1;"), "{design_css}");
    assert!(
        design_css.contains("[data-dowe-theme=\"dark\"]{")
            && design_css.contains("--dowe-happy:#55c2cc;"),
        "{design_css}"
    );
    let page_css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(generated_css_chunk(
                &project.web.pages[0].css_chunks,
                "chunks/pages/",
            )),
    )
    .expect("page css");
    assert!(
        page_css.contains(".card.is-solid.is-happy")
            && page_css.contains("var(--dowe-happyText)")
            && page_css.contains("var(--dowe-happyTitle)"),
        "{page_css}"
    );
    assert!(
        page_css.contains(".card.is-soft.is-happy")
            && page_css.contains("var(--dowe-happyText)")
            && page_css.contains("var(--dowe-happyTitle)"),
        "{page_css}"
    );

    let android_theme = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweTheme.kt"),
    )
    .expect("android theme");
    assert!(android_theme.contains("\"happy\" to Color(0xFF176C75)"));
    assert!(android_theme.contains("\"softHappy\" to Color(0xFFD9F3F1)"));
    let android_pages = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    assert!(android_pages.contains("DoweDesign.happy"), "{android_pages}");
    assert!(android_pages.contains("DoweDesign.happyText"), "{android_pages}");
    assert!(android_pages.contains("DoweDesign.happyTitle"), "{android_pages}");
    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("DOWE_HAPPY"), "{android_dev}");

    let ios_theme =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweTheme.swift")).expect("ios theme");
    assert!(ios_theme.contains("\"happy\": Color("), "{ios_theme}");
    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("DoweDesign.happy"), "{ios}");
    assert!(ios.contains("DoweDesign.happyText"), "{ios}");
    assert!(ios.contains("DoweDesign.happyTitle"), "{ios}");
}

#[test]
fn rejects_undeclared_custom_scheme_and_missing_custom_soft_family() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Card scheme:\"happy\"\n    Text\n      \"Missing\"",
    );
    let undeclared = match compile_dev(temp.path()) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected undeclared custom scheme to fail"),
    };
    assert!(undeclared.contains("happy"), "{undeclared}");

    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        happy color:"#176c75" text:"#fffffe" title:"#fffffe""##,
    )
    .expect("theme");
    fs::write(
        temp.path().join("pages/login.dowe"),
        "page loginPage\n  Card variant:\"soft\" scheme:\"happy\"\n    Text\n      \"Missing soft\"",
    )
    .expect("page");
    let missing_soft = match compile_dev(temp.path()) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected missing custom soft family to fail"),
    };
    assert!(missing_soft.contains("softHappy"), "{missing_soft}");
}

#[test]
fn rejects_removed_slot_defaults_and_unknown_component_default_props() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    radius panel:md
    theme name:"light""##,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("`radius` is not valid inside `design`"));

    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    Card columns:3
    theme name:"light""##,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(
        error
            .to_string()
            .contains("`columns` is not a theme default prop for `Card`")
    );
}

#[test]
fn accepts_navigation_and_chip_theme_defaults() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    Chip variant:"outlined" scheme:"primary"
    SideNav variant:"soft" scheme:"surface"
    Sidebar variant:"ghost" scheme:"surface"
    NavMenu variant:"solid" scheme:"surface"
    theme name:"light""#,
    )
    .expect("theme");

    compile_dev(temp.path()).expect("navigation and chip theme defaults");
}

#[test]
fn rejects_invalid_text_and_title_theme_defaults() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    for (line, expected) in [
        ("Text variant:\"soft\"", "`variant` is not a theme default prop for `Text`"),
        ("Title radius:\"md\"", "`radius` is not a theme default prop for `Title`"),
        ("Text font:\"arial\"", "unknown font token `arial` in `Text.font`"),
        ("Title font:\"Inter\"", "unknown font token `Inter` in `Title.font`"),
    ] {
        fs::write(
            temp.path().join("theme.dowe"),
            format!(
                "theme\n  design defaultTheme:\"light\"\n    {line}\n    theme name:\"light\""
            ),
        )
        .expect("theme");

        let error = compile_dev(temp.path()).expect_err("invalid text theme default");

        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn rejects_removed_theme_radius_surface() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    theme name:"light" radius:10"#,
    )
    .expect("theme");

    assert!(
        compile_dev(temp.path())
            .expect_err("removed radius prop")
            .to_string()
            .contains("`radius` is not valid on `theme`")
    );

    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    theme name:"light" radiusBox:12"#,
    )
    .expect("theme");
    assert!(
        compile_dev(temp.path())
            .expect_err("removed radius prop")
            .to_string()
            .contains("`radiusBox` is not valid on `theme`")
    );

    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    theme name:"light"
      radii radius:10"#,
    )
    .expect("theme");
    assert!(
        compile_dev(temp.path())
            .expect_err("removed radii block")
            .to_string()
            .contains("`radii` is not valid inside `theme`")
    );
}

#[test]
fn rejects_unquoted_static_config_strings() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
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
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:server methods:["GET"]
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let cors_error = compile_dev(temp.path()).expect_err("cors error");
    assert!(
        cors_error
            .to_string()
            .contains("quoted static string literal")
    );
}

