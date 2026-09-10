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
    assert!(views.contains("Scaffold safeAreaTop:\\\"surface\\\" safeAreaBottom:\\\"primary\\\""));
    assert!(views.contains("omitted safe-area colors default to background"));
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
        primaryText color:"#FFFFFF" text:"#17263A" title:"#17263E""##,
    )
    .expect("unknown family");
    let message = compile_dev(temp.path())
        .expect_err("unknown color family")
        .to_string();
    assert!(message.contains("unknown color family `primaryText`"), "{message}");
}

