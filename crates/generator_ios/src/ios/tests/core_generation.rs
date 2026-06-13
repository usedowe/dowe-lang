fn swift_content(output: &IosOutput) -> String {
    output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .extension()
                .and_then(|value| value.to_str())
                == Some("swift")
        })
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn generates_swiftui_box_and_text() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(views.contains("AnyView("));
    assert!(views.contains("routeSection0()"));
    assert!(views.contains("private func routeSection0() -> some View"));
    assert!(views.contains("private let activePath = \"/login\""));
    assert!(!views.contains("        let activePath ="));
    assert!(!views.contains("VStack(alignment: .leading) {"));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .leading)"));
    assert!(views.contains(".background(DoweDesign.primary)"));
    assert!(views.contains("Text(\"Layout\")"));
    assert!(views.contains("Text(\"Login\")"));

    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleExecutable"));
    assert!(plist.content.contains("DoweIosApp"));
    assert!(plist.content.contains("CFBundleURLSchemes"));
    assert!(plist.content.contains("dowe-dev"));
    assert!(plist.content.contains("UILaunchScreen"));
    assert!(plist.content.contains("NSAllowsLocalNetworking"));
    assert!(plist.content.contains("UIAppFonts"));
    assert!(plist.content.contains("Fonts/inter-regular.ttf"));
}

#[test]
fn generates_shared_swiftui_layout_once_for_multiple_routes() {
    let mut first = route();
    first.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Box {
                props: Default::default(),
                children: vec![text("Layout")],
            },
            ViewNode::Children,
        ],
    };
    let mut second = first.clone();
    second.route_path = "/signup".to_string();
    second.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "Signup".to_string(),
    };

    let output = generate_ios(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayouts.swift"))
        .expect("layouts");
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");
    let signup = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageSignupView.swift"))
        .expect("signup");

    assert_eq!(
        layouts
            .content
            .matches("struct DoweLayout0<Content: View>")
            .count(),
        1
    );
    assert_eq!(layouts.content.matches("Text(\"Layout\")").count(), 1);
    assert!(layouts.content.contains("layoutSection0()"));
    assert!(
        layouts
            .content
            .contains("private func layoutSection0() -> some View")
    );
    assert!(login.content.contains("DoweLayout0("));
    assert!(signup.content.contains("DoweLayout0("));
    assert!(!login.content.contains("Text(\"Layout\")"));
    assert!(!signup.content.contains("Text(\"Layout\")"));
    assert!(login.content.contains("Text(\"Login\")"));
    assert!(signup.content.contains("Text(\"Signup\")"));
}

#[test]
fn keeps_contextual_swiftui_layout_composed_with_its_page() {
    let mut contextual = route();
    contextual.layout_tree = ViewNode::Scope {
        signals: vec![ViewSignal {
            id: "layout.message".to_string(),
            name: "message".to_string(),
            initial: ViewSignalValue::String("Layout message".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Children],
        }],
    };
    contextual.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "message".to_string(),
    };

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayouts.swift"))
        .expect("layouts");
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(!layouts.content.contains("struct DoweLayout0<"));
    assert!(!login.content.contains("DoweLayout0("));
    assert!(login.content.contains("state.text(\"layout.message\")"));
}

#[test]
fn generates_ios_app_metadata() {
    let output = generate_ios_with_app_and_translations(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
    );
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");

    assert!(plist.content.contains("<string>Clinic Desk</string>"));
    assert!(
        plist
            .content
            .contains("<string>com.example.clinic</string>")
    );
    assert!(plist.content.contains("<key>CFBundleName</key>"));
}

#[test]
fn generates_swiftui_section_backgrounds() {
    let output = generate_ios(
        &[section_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("enum DoweSectionBackground"));
    assert!(views.contains("DoweSectionBackgroundView(background: background)"));
    assert!(views.contains("doweResponsive(viewportWidth, xs: DoweSectionBackground.aurora, md: DoweSectionBackground.ocean)"));
    assert!(views.contains("LinearGradient(colors: [DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary]"));
    assert!(views.contains("DoweCoverImage(source:"));
    assert!(views.contains("https://example.com/hero.jpg"));
    assert!(views.contains("DoweOverlay.color(Color.black.opacity(0.35))"));
    assert!(views.contains("DoweOverlayView(overlay: overlay)"));
}

#[test]
fn generates_native_ios_translation_resources() {
    let mut localized_route = route();
    localized_route.page_tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    let output = generate_ios_with_translations(
        &[localized_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &translations(),
    );
    let views = swift_content(&output);
    assert!(views.contains(r#"String(localized: "home.hero.title")"#));
    let english = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("en.lproj/Localizable.strings"))
        .expect("english");
    assert!(english.content.contains("Dowe builds systems."));
    let spanish = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("es.lproj/Localizable.strings"))
        .expect("spanish");
    assert!(spanish.content.contains("Dowe construye sistemas."));
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleDevelopmentRegion"));
    assert!(plist.content.contains("<string>en</string>"));
}
