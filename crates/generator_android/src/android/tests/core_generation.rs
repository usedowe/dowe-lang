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
fn keeps_contextual_dev_layouts_composed_with_their_pages() {
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
