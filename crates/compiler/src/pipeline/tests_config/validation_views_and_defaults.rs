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
  Card variant:"solid" scheme:"happy"
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
    assert_eq!(dark.color_value(happy.color_token()), "#55c2cc");

    let body = &project.web.pages[0].body_html;
    assert!(body.contains("is-solid is-happy"), "{body}");

    let design_css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("design css");
    assert!(design_css.contains("--dowe-happy:#176c75;"), "{design_css}");
    assert!(design_css.contains("--dowe-happyText:#fffffe;"), "{design_css}");
    assert!(design_css.contains("--dowe-happyTitle:#fffefe;"), "{design_css}");
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
        page_css.contains(".card.is-solid.is-happy")
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
fn rejects_undeclared_custom_scheme() {
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

