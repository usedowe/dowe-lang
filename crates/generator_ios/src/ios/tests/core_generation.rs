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
fn keeps_layouts_composed_when_page_reads_layout_state() {
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
fn reuses_stateful_swiftui_layout_when_page_does_not_read_layout_state() {
    let mut contextual = route();
    contextual.layout_tree = ViewNode::Scope {
        signals: vec![ViewSignal {
            id: "layout.open".to_string(),
            name: "open".to_string(),
            initial: ViewSignalValue::Bool(false),
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
        value: "Login".to_string(),
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

    assert!(layouts.content.contains("struct DoweLayout0<"));
    assert!(login.content.contains("DoweLayout0("));
    assert!(login.content.contains("\"layout.open\": false"));
}

#[test]
fn reuses_stateful_scaffold_drawer_layout_when_page_mentions_binding_literals() {
    let contextual = stateful_scaffold_drawer_layout_route();

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

    assert!(layouts.content.contains("struct DoweLayout0<"));
    assert!(layouts.content.contains("DoweDrawer(open: state.bool(\"layout.drawer.open\")"));
    assert!(login.content.contains("DoweLayout0("));
    assert!(login.content.contains("\"layout.drawer.open\": false"));
    assert!(login.content.contains("\"layout.drawer.visible\": true"));
    assert!(login.content.contains(
        "\"layout.drawer.open.action\": .assign(\"layout.drawer.open\", \"layout.drawer.visible\", nil)"
    ));
    assert!(!login.content.contains("DoweDrawer(open: state.bool(\"layout.drawer.open\")"));
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

fn stateful_scaffold_drawer_layout_route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Scope {
            signals: vec![
                ViewSignal {
                    id: "layout.drawer.open".to_string(),
                    name: "drawerOpen".to_string(),
                    initial: ViewSignalValue::Bool(false),
                    schema: None,
                },
                ViewSignal {
                    id: "layout.drawer.visible".to_string(),
                    name: "drawerVisible".to_string(),
                    initial: ViewSignalValue::Bool(true),
                    schema: None,
                },
            ],
            actions: vec![ViewAction {
                id: "layout.drawer.open.action".to_string(),
                name: "openDrawer".to_string(),
                kind: ViewActionKind::Assign(ViewAssignAction {
                    target: "drawerOpen".to_string(),
                    source: "drawerVisible".to_string(),
                    call: None,
                }),
            }],
            children: vec![ViewNode::Scaffold {
                props: ScaffoldProps::default(),
                app_bar: vec![ViewNode::AppBar {
                    props: bar_props(false),
                    start: vec![ViewNode::Button {
                        props: VariantProps {
                            element: ElementProps {
                                on_click: Some("openDrawer".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        children: vec![text("Menu")],
                    }],
                    center: vec![text("Docs")],
                    end: Vec::new(),
                }],
                start: vec![ViewNode::Sidebar {
                    props: SidebarProps {
                        style: VariantProps::default(),
                    },
                    header: Vec::new(),
                    body: vec![ViewNode::SideNav {
                        props: SideNavProps {
                            style: VariantProps::default(),
                            size: SideNavSize::Sm,
                            wide: true,
                        },
                        items: vec![SideNavItem::Item(SideNavItemProps {
                            label: "Overview".to_string(),
                            description: None,
                            status: None,
                            icon: None,
                            on_click: None,
                            navigation: None,
                        })],
                    }],
                    footer: Vec::new(),
                }],
                main: vec![
                    ViewNode::Drawer {
                        props: DrawerProps {
                            style: VariantProps::default(),
                            open: "drawerOpen".to_string(),
                            position: DrawerPosition::Start,
                            disable_overlay_close: false,
                            hide_close_button: false,
                        },
                        header: Vec::new(),
                        body: vec![ViewNode::SideNav {
                            props: SideNavProps {
                                style: VariantProps::default(),
                                size: SideNavSize::Sm,
                                wide: true,
                            },
                            items: vec![SideNavItem::Item(SideNavItemProps {
                                label: "Overview".to_string(),
                                description: None,
                                status: None,
                                icon: None,
                                on_click: None,
                                navigation: None,
                            })],
                        }],
                        footer: Vec::new(),
                    },
                    ViewNode::Children,
                ],
                end: Vec::new(),
                bottom_bar: Vec::new(),
            }],
        },
        page_tree: ViewNode::RichText {
            props: TextProps::default(),
            marks: vec![RichTextMark {
                text: "drawerOpen openDrawer".to_string(),
                style: RichTextMarkStyle::Mark,
                color: ColorFamily::Primary,
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
