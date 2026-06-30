#[test]
fn generates_compose_box_and_text() {
    let output = generate_android(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(
        views
            .content
            .contains("Column(modifier = Modifier.fillMaxWidth()) {")
    );
    assert!(
        views
            .content
            .contains("Card(modifier = Modifier.fillMaxWidth()")
    );
    assert!(
        views
            .content
            .contains("Box(modifier = Modifier.fillMaxSize().background(DoweDesign.background))")
    );
    assert!(
        views
            .content
            .contains("Modifier.fillMaxSize().safeDrawingPadding().verticalScroll(scrollState)")
    );
    assert!(
        !views
            .content
            .contains("import androidx.compose.foundation.layout.matchParentSize")
    );
    assert_eq!(
        views
            .content
            .matches("private fun doweFontFamily(value: DoweFont?): FontFamily")
            .count(),
        1
    );
    assert!(
        views
            .content
            .contains("Text(\"Layout\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(
        views
            .content
            .contains("Text(\"Login\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(
        views
            .content
            .contains("Font(R.font.inter_light, FontWeight.Thin)")
    );
    assert!(
        views
            .content
            .contains("Font(R.font.inter_regular, FontWeight.Normal)")
    );
    assert!(
        views
            .content
            .contains("Font(R.font.inter_extrabold, FontWeight.Black)")
    );
    assert!(views.content.contains("DoweFont.Inter -> DoweFonts.inter"));

    let root_gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("build.gradle.kts"))
        .expect("root gradle");
    assert!(
        root_gradle
            .content
            .contains("org.jetbrains.kotlin.plugin.compose")
    );
    let gradle_properties = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("gradle.properties"))
        .expect("gradle properties");
    assert!(
        gradle_properties
            .content
            .contains("android.useAndroidX=true")
    );
    let app_gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("app/build.gradle.kts"))
        .expect("app gradle");
    assert!(app_gradle.content.contains("JvmTarget.JVM_17"));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains("root.setGravity(Gravity.TOP | Gravity.START)")
    );
    assert!(
        dev.content
            .contains("private static final int DOWE_BACKGROUND = Color.rgb(255, 255, 255);")
    );
    assert!(
        dev.content
            .contains("background.setBackgroundColor(DOWE_BACKGROUND)")
    );
    assert!(
        dev.content
            .contains("root.setBackgroundColor(DOWE_BACKGROUND)")
    );
    assert!(
        dev.content
            .contains("getWindow().setStatusBarColor(Color.TRANSPARENT)")
    );
    assert!(
        dev.content
            .contains("getWindow().setNavigationBarColor(Color.TRANSPARENT)")
    );
    assert!(
        dev.content
            .contains("getWindow().setDecorFitsSystemWindows(false)")
    );
    assert!(dev.content.contains("view.setOnApplyWindowInsetsListener"));
    assert!(dev.content.contains(
            "new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT)"
        ));
    assert!(dev.content.contains("doweCard(DOWE_PRIMARY, null)"));
    assert!(dev.content.contains(
        "private GradientDrawable doweInputBackground(int color, Integer strokeColor, float radius)"
    ));
    assert!(dev.content.contains("if (strokeColor != null)"));
    assert!(dev.content.contains("doweText(\"Layout\""));
    assert!(dev.content.contains("doweText(\"Login\""));
    assert!(
        dev.content
            .contains("private void renderLogin(LinearLayout root)")
    );
    assert!(
        dev.content
            .contains("renderLayout0(root, this::renderLoginPage);")
    );
    assert!(
        dev.content
            .contains("private void renderLayout0(ViewGroup root, Consumer<ViewGroup> page)")
    );
    assert!(dev.content.contains("page.accept(view0);"));
    assert!(
        dev.content
            .contains("private void renderLoginPage(ViewGroup root)")
    );
    assert!(dev.content.contains("doweFontName(null)"));
    assert!(
        dev.content
            .contains("return value == null ? \"Inter\" : value;")
    );
    assert!(
        output
            .files
            .iter()
            .any(|file| file.relative_path.ends_with("dev/AndroidManifest.xml"))
    );
    let manifest = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .ends_with("app/src/main/AndroidManifest.xml")
        })
        .expect("manifest");
    assert!(manifest.content.contains(r#"android:scheme="dowe-dev""#));
    assert!(
        manifest
            .content
            .contains(r#"android:windowSoftInputMode="adjustResize""#)
    );
    let main_activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("MainActivity.kt"))
        .expect("main activity");
    assert!(
        main_activity
            .content
            .contains("import androidx.activity.enableEdgeToEdge")
    );
    assert!(main_activity.content.contains("enableEdgeToEdge()"));
}

#[test]
fn generates_android_box_border_for_compose_and_dev_launcher() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps {
            border: Some(ResponsiveValue::scalar(BorderWidth(2))),
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
            ..Default::default()
        },
        children: vec![text("Bordered")],
    };

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains(
        ".doweBorder(width = doweResponsive(viewportWidth, xs = 2.dp), radius = doweResponsive(viewportWidth, xs = 12.dp))"
    ));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(dev.content.contains(
        "private GradientDrawable doweStyledBackground(int color, Integer strokeColor, Integer strokeWidth, float radius)"
    ));
    assert!(dev.content.contains(
        "view0.setBackground(doweStyledBackground(Color.TRANSPARENT, DOWE_ON_BACKGROUND, doweResponsiveInt(viewportWidth, 2, null, null, null, null), doweFloat(doweResponsiveFloat(viewportWidth, 12f, null, null, null, null), DOWE_RADIUS)))"
    ));
    assert!(dev
        .content
        .contains("background.setStroke(doweDp(strokeWidth), strokeColor)"));
}

#[test]
fn reuses_identical_dev_layout_methods_across_routes() {
    let first = route();
    let mut second = route();
    second.id = "signup".to_string();
    second.route_path = "/signup".to_string();
    second.page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("Signup")],
    };

    let output = generate_android(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert_eq!(
        dev.content
            .matches("private void renderLayout0(ViewGroup root")
            .count(),
        1
    );
    assert!(
        dev.content
            .contains("renderLayout0(root, this::renderLoginPage);")
    );
    assert!(
        dev.content
            .contains("renderLayout0(root, this::renderSignupPage);")
    );
}

#[test]
fn omits_absent_dev_padding_branches() {
    let props = StyleProps {
        spacing: SpacingProps {
            px: Some(responsive_scale(&[(Breakpoint::Xs, 8)])),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut output = String::new();

    super::apply_dev_android_style(&props, "view0", true, &mut output);

    assert!(output.contains("Integer view0PaddingX = doweResponsiveInt"));
    assert!(!output.contains("Integer view0Padding ="));
    assert!(!output.contains("Integer view0PaddingY ="));
    assert!(!output.contains("Integer view0PaddingLeft ="));
    assert!(!output.contains("Integer view0PaddingRight ="));
    assert!(!output.contains("Integer view0PaddingTop ="));
    assert!(!output.contains("Integer view0PaddingBottom ="));
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

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(!dev.content.contains("private void renderLayout0("));
    assert!(!dev.content.contains("private void renderLoginPage("));
    assert!(
        dev.content
            .contains("doweTextValue(\"layout.message\", null)")
    );
}

#[test]
fn reuses_stateful_dev_layout_when_page_does_not_read_layout_state() {
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

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(dev.content.contains("private void renderLayout0("));
    assert!(dev.content.contains("private void renderLoginPage("));
    assert!(
        dev.content
            .contains("renderLayout0(root, this::renderLoginPage);")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.open\", false);")
    );
}

#[test]
fn reuses_stateful_scaffold_drawer_layout_when_page_mentions_binding_literals() {
    let contextual = stateful_scaffold_drawer_layout_route();

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(dev.content.contains("private void renderLayout0("));
    assert!(dev.content.contains("private void renderLoginPage("));
    assert!(
        dev.content
            .contains("renderLayout0(root, this::renderLoginPage);")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.drawer.open\", false);")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.drawer.visible\", true);")
    );
    assert!(
        dev.content
            .contains("doweActions.put(\"layout.drawer.open.action\", DoweAction.assign(\"layout.drawer.open\", \"layout.drawer.visible\"));")
    );
}

#[test]
fn generates_compose_and_dev_section_backgrounds() {
    let output = generate_android(
        &[section_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(
        views
            .content
            .contains("private enum class DoweSectionBackground")
    );
    assert!(views.content.contains("DoweSectionBackgroundBox("));
    assert!(views.content.contains("background = doweResponsive(viewportWidth, xs = DoweSectionBackground.Aurora, md = DoweSectionBackground.Ocean)"));
    assert!(views.content.contains("Brush.linearGradient(listOf(DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary))"));
    assert!(views.content.contains("DoweCoverBox("));
    assert!(views.content.contains("https://example.com/hero.jpg"));
    assert!(
        views
            .content
            .contains("DoweOverlay.Solid(Color.Black.copy(alpha = 0.35f))")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains("private GradientDrawable doweSectionBackground(String value)")
    );
    assert!(
        dev.content
            .contains("String view1SectionBackground = doweResponsiveString(viewportWidth, \"aurora\", null, \"ocean\", null, null)")
    );
    assert!(
        dev.content
            .contains("view1.setBackground(doweSectionBackground(view1SectionBackground));")
    );
}

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
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

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
