#[test]
fn generates_android_app_metadata() {
    let output = generate_android_with_app_and_translations(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
    );
    let gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("app/build.gradle.kts"))
        .expect("gradle");
    let app_manifest = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .ends_with("app/src/main/AndroidManifest.xml")
        })
        .expect("app manifest");
    let dev_manifest = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("dev/AndroidManifest.xml"))
        .expect("dev manifest");
    let dev = dev_java_source(&output);

    assert!(
        gradle
            .content
            .contains(r#"applicationId = "com.example.clinic""#)
    );
    assert!(
        app_manifest
            .content
            .contains(r#"android:label="Clinic Desk""#)
    );
    assert!(
        dev_manifest
            .content
            .contains(r#"package="com.example.clinic""#)
    );
    assert!(
        dev_manifest
            .content
            .contains(r#"android:label="Clinic Desk""#)
    );
    assert!(dev.content.contains("import com.example.clinic.R;"));
}

fn stateful_scaffold_drawer_layout_route(boxed: bool) -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Scope {
            constants: Vec::new(),
            signals: vec![
                ViewSignal {
                    id: "layout.drawer.open".to_string(),
                    name: "drawerOpen".to_string(),
                    storage_key: "drawerOpen".to_string(),
                    scope: dowe_components::ViewSignalScope::Page,
                    storage: dowe_components::ViewSignalStorage::None,
                    initial: ViewSignalValue::Bool(false),
                    schema: None,
                },
                ViewSignal {
                    id: "layout.drawer.visible".to_string(),
                    name: "drawerVisible".to_string(),
                    storage_key: "drawerVisible".to_string(),
                    scope: dowe_components::ViewSignalScope::Page,
                    storage: dowe_components::ViewSignalStorage::None,
                    initial: ViewSignalValue::Bool(true),
                    schema: None,
                },
            ],
            actions: vec![ViewAction {
                id: "layout.drawer.open.action".to_string(),
                name: "openDrawer".to_string(),
                params: Vec::new(),
                return_type: None,
                kind: ViewActionKind::Assign(ViewAssignAction {
                    target: "drawerOpen".to_string(),
                    source: "drawerVisible".to_string(),
                    literal: None,
                    call: None,
                }),
            }],
            children: vec![ViewNode::Scaffold {
                props: ScaffoldProps {
                    boxed,
                    ..Default::default()
                },
                app_bar: vec![ViewNode::AppBar {
                    props: BarProps {
                        position: BarPosition::Fixed,
                        ..bar_props(false)
                    },
                    top: Vec::new(),
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
                    bottom: Vec::new(),
                mobile_menu: None,
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
                            reactive_wide: None,
                        },
                        items: vec![SideNavItem::Item(SideNavItemProps {
                            label: "Overview".to_string(),
                            i18n: None,
                            description: None,
                            description_i18n: None,
                            status: None,
                            status_i18n: None,
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
                                reactive_wide: None,
                            },
                            items: vec![SideNavItem::Item(SideNavItemProps {
                                label: "Overview".to_string(),
                                i18n: None,
                                description: None,
                                description_i18n: None,
                                status: None,
                                status_i18n: None,
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
                overlays: Vec::new(),
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

#[test]
fn generates_intrinsic_brand_navigation_without_button_chrome() {
    let mut brand_route = route();
    brand_route.layout_tree = ViewNode::Children;
    brand_route.page_tree = ViewNode::Brand {
        props: BrandProps {
            style: StyleProps {
                sizing: SizingProps {
                    w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(64),
                    ))),
                    h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            navigation: Some(NavigationAction::Internal {
                path: "/".to_string(),
                fragment: None,
                operation: NavigationOperation::Push,
            }),
            label: Some("Dowe home".to_string()),
        },
        children: vec![text("Dowe")],
    };
    let output = generate_android(
        &[brand_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("pages");
    let dev = dev_java_source(&output);

    assert!(pages.content.contains("Row(modifier = Modifier"));
    assert!(
        pages
            .content
            .contains(".clickable(onClick = { navigate(\"push\", \"/\", null) })")
    );
    assert!(
        pages
            .content
            .contains(".semantics { contentDescription = \"Dowe home\" }")
    );
    assert!(pages.content.contains("DoweSize.Fixed(128.dp)"));
    assert!(pages.content.contains("DoweSize.Fixed(32.dp)"));
    assert!(!pages.content.contains(" Button(modifier ="));
    assert!(dev.content.contains("doweContainer(true)"));
    assert!(dev.content.contains("setContentDescription(\"Dowe home\")"));
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/\", null))")
    );
}

#[test]
fn generates_external_banner_without_button_chrome() {
    let mut banner_route = route();
    banner_route.layout_tree = ViewNode::Children;
    banner_route.page_tree = ViewNode::Banner {
        props: BannerProps {
            style: StyleProps {
                spacing: SpacingProps {
                    p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(12))),
                    ..Default::default()
                },
                ..Default::default()
            },
            navigation: NavigationAction::External {
                url: "https://dowe.dev/cloud".to_string(),
                web_target: dowe_components::WebTarget::Blank,
                native_external_mode: dowe_components::NativeExternalMode::System,
            },
            label: Some("Explore Dowe Cloud".to_string()),
        },
        children: vec![text("Build beyond code")],
    };
    let output = generate_android(
        &[banner_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("pages");
    let dev = dev_java_source(&output);

    assert!(pages.content.contains("Column(modifier = Modifier"));
    assert!(pages.content.contains(
        ".clickable(onClick = { openExternal(\"system\", \"https://dowe.dev/cloud\") })"
    ));
    assert!(
        pages
            .content
            .contains(".semantics { contentDescription = \"Explore Dowe Cloud\" }")
    );
    assert!(!pages.content.contains(" Button(modifier ="));
    assert!(dev.content.contains("doweContainer(false)"));
    assert!(
        dev.content
            .contains("setContentDescription(\"Explore Dowe Cloud\")")
    );
    assert!(dev.content.contains(
        "setOnClickListener(v -> doweOpenExternal(\"system\", \"https://dowe.dev/cloud\"))"
    ));
}
