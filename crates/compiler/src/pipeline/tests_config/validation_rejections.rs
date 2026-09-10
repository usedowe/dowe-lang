#[test]
fn accepts_navigation_and_chip_theme_defaults() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    Chip variant:"outlined" scheme:"primary"
    SideNav variant:"solid" scheme:"surface"
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
        ("Text variant:\"solid\"", "`variant` is not a theme default prop for `Text`"),
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

