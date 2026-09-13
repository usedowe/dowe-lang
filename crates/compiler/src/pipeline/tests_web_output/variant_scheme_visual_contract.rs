#[test]
fn keeps_variant_roles_and_direct_card_styles_when_the_theme_changes() {
    let page = r#"page loginPage
  Card
    Title
      "Inherited title"
    Text
      "Inherited content"
  Card variant:"outlined" scheme:"primary" bg:"danger" color:"white" borderColor:"secondary"
    Title
      "Direct title"
    Text
      "Direct content""#;
    let theme = |default_theme: &str, primary: &str, surface: &str| {
        format!(
            r##"theme
  design defaultTheme:"{default_theme}"
    Card variant:"outlined" scheme:"primary"
    theme name:"light"
      colors:
        primary color:"{primary}" text:"#eeeeee" title:"#ffffff"
        surface color:"{surface}" text:"#222222" title:"#111111"
        danger color:"#cc0000" text:"#ffffff" title:"#ffffff"
        secondary color:"#0055aa" text:"#ffffff" title:"#ffffff"
    theme name:"dark"
      colors:
        primary color:"#f0f0f0" text:"#111111" title:"#000000"
        surface color:"#202020" text:"#eeeeee" title:"#ffffff"
        danger color:"#ff7777" text:"#220000" title:"#220000"
        secondary color:"#77bbff" text:"#001122" title:"#001122""##,
        )
    };

    let light = TempDir::new().expect("light tempdir");
    write_fixture_with_views(
        light.path(),
        "layout AuthLayout\n  Box\n    children",
        page,
    );
    fs::write(
        light.path().join("theme.dowe"),
        theme("light", "#112233", "#fafafa"),
    )
    .expect("light theme");
    let light_project = compile_dev(light.path()).expect("light project");

    let dark = TempDir::new().expect("dark tempdir");
    write_fixture_with_views(
        dark.path(),
        "layout AuthLayout\n  Box\n    children",
        page,
    );
    fs::write(
        dark.path().join("theme.dowe"),
        theme("dark", "#aabbcc", "#101010"),
    )
    .expect("dark theme");
    let dark_project = compile_dev(dark.path()).expect("dark project");

    for project in [&light_project, &dark_project] {
        assert_eq!(
            project.view_routes.web[0].page_tree,
            project.view_routes.desktop[0].page_tree
        );
        assert_eq!(
            project.view_routes.web[0].page_tree,
            project.view_routes.android[0].page_tree
        );
        assert_eq!(
            project.view_routes.web[0].page_tree,
            project.view_routes.ios[0].page_tree
        );
        let body = &project.web.pages[0].body_html;
        assert!(body.contains("is-outlined is-primary"));
        assert!(body.contains("card-bg-danger"));
        assert!(body.contains("card-color-white"));
        assert!(body.contains("border-color-secondary"));
    }

    let light_css = fs::read_to_string(
        light
            .path()
            .join(".dowe/web")
            .join(light_project.web.design_file_name()),
    )
    .expect("light css");
    assert!(light_css.contains("--dowe-primary:#112233;"));
    assert!(light_css.contains("--dowe-surface:#fafafa;"));
    let light_page_css = fs::read_to_string(
        light.path().join(".dowe/web").join(generated_css_chunk(
            &light_project.web.pages[0].css_chunks,
            "chunks/pages/",
        )),
    )
    .expect("light page css");
    assert!(light_page_css.contains(
        ".card.is-outlined.is-primary{--dowe-content-text:var(--dowe-primary);--dowe-content-title:var(--dowe-surfaceTitle);background-color:var(--dowe-surface);color:var(--dowe-primary);border:1px solid var(--dowe-primary);}"
    ));
    assert!(light_page_css.contains(".card-bg-danger{background-color:var(--dowe-danger) !important;}"));
    assert!(light_page_css.contains(".card-color-white{"));

    let dark_css = fs::read_to_string(
        dark
            .path()
            .join(".dowe/web")
            .join(dark_project.web.design_file_name()),
    )
    .expect("dark css");
    assert!(dark_css.contains("--dowe-primary:#aabbcc;"));
    assert!(dark_css.contains("--dowe-surface:#101010;"));

    let android = fs::read_to_string(
        light
            .path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    assert!(android.contains("DoweDesign.surface"));
    assert!(android.contains(
        "CardDefaults.cardColors(containerColor = DoweDesign.surface, contentColor = DoweDesign.primary)"
    ));
    assert!(android.contains("BorderStroke(1.dp, DoweDesign.primary)"));
    assert!(android.contains("DoweDesign.danger"));
    assert!(android.contains("DoweDesign.secondary"));

    let android_dev = android_dev_output(light.path());
    assert!(android_dev.contains("DOWE_SURFACE"));
    assert!(android_dev.contains("DOWE_SURFACE_TEXT"));
    assert!(android_dev.contains("DOWE_PRIMARY"));
    assert!(android_dev.contains("DOWE_SECONDARY"));

    let ios = ios_swift_output(light.path());
    assert!(ios.contains("DoweDesign.surface"));
    assert!(ios.contains(".foregroundStyle(DoweDesign.primary)"));
    assert!(ios.contains("stroke(DoweDesign.primary"));
    assert!(ios.contains("DoweDesign.danger"));
    assert!(ios.contains("DoweDesign.secondary"));
}
