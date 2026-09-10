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
            .contains("Box(modifier = Modifier.fillMaxSize().verticalScroll(scrollState))")
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
    assert!(
        gradle_properties
            .content
            .contains("org.gradle.jvmargs=-Xmx2048m")
    );
    assert!(
        gradle_properties
            .content
            .contains("kotlin.daemon.jvmargs=-Xmx8192m")
    );
    let app_gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("app/build.gradle.kts"))
        .expect("app gradle");
    assert!(app_gradle.content.contains("JvmTarget.JVM_17"));
    assert!(
        app_gradle
            .content
            .contains("androidx.compose:compose-bom:2026.06.01")
    );
    assert!(app_gradle.content.contains("androidx.compose.ui:ui\")"));

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("root.setGravity(Gravity.TOP | Gravity.START)")
    );
    assert!(
        dev.content
.contains("DOWE_BACKGROUND")
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
    assert!(
        dev.content
            .contains("boolean useDarkStatusIcons = doweColorLuminance(doweSafeAreaTopColor) > 0.179f")
    );
    assert!(dev.content.contains(
        "getWindow().getInsetsController().setSystemBarsAppearance(appearance, statusMask | navigationMask)"
    ));
    assert!(
        dev.content
            .contains("doweApplyTheme(name);\n        doweUpdateSafeAreaColors();\n        doweApplySystemBarAppearance();")
    );
    assert!(dev.content.contains("view.setOnApplyWindowInsetsListener"));
    assert!(dev.content.contains("scrollView.setClipToPadding(true);"));
    assert!(dev.content.contains(
        "scrollView.setOnScrollChangeListener((view, scrollX, scrollY, oldScrollX, oldScrollY) -> doweUpdatePinnedAppBarDock(scrollY > doweDp(100), true));"
    ));
    assert!(dev.content.contains("scrollView.addView(root"));
    assert!(dev.content.contains("private boolean dowePageTransitioning = false;"));
    assert!(dev.content.contains("private void doweStartPageTransition()"));
    assert!(dev.content.contains("private void doweFinishPageTransition()"));
    assert!(dev.content.contains("setDuration(280)"));
    assert!(dev.content.contains("new PathInterpolator(0.22f, 0.61f, 0.36f, 1f)"));
    assert!(dev.content.contains("if (dowePageEntranceSuppressed)"));
    assert!(dev.content.contains("doweStartPageTransition();"));
    assert!(dev.content.contains("doweFinishPageTransition();"));
    assert!(dev.content.contains("withEndAction(() ->"));
    assert!(dev.content.contains("dowePageTransitioning = false;"));
    assert!(dev.content.contains(
            "new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT)"
        ));
    assert!(
        dev.content
            .contains("doweCard(DOWE_PRIMARY, (\"outlined\".equals(\"solid\") ? null : null))")
    );
    assert!(dev.content.contains(
        "private GradientDrawable doweInputBackground(int color, Integer strokeColor, float radius)"
    ));
    assert!(dev.content.contains("if (strokeColor != null)"));
    assert!(dev.content.contains("doweText(\"Layout\""));
    assert!(dev.content.contains("doweText(\"Login\""));
    assert!(dev.content.contains("final class DoweDevRouteLogin"));
    assert!(
        dev.content
            .contains("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
    );
    assert!(dev.content.contains("final class DoweDevLayout0"));
    assert!(dev.content.contains("page.accept(doweCreatePageContainer("));
    assert!(dev.content.contains("private static void renderPage("));
    assert!(dev.content.contains("doweFontName(null)"));
    assert!(
        dev.content
            .contains("return value == null ? \"Inter\" : value;")
    );
    assert!(
        dev.content
            .contains("Typeface bundled = getResources().getFont(resource);")
    );
    assert!(dev.content.contains("return R.font.inter_light;"));
    assert!(dev.content.contains("return R.font.inter_regular;"));
    assert!(
        dev.content
            .contains("view.setTypeface(doweTypeface(font, weight));")
    );
    assert!(
        !dev.content
            .contains("Typeface baseTypeface = Typeface.create(font, Typeface.NORMAL);")
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
    assert!(
        main_activity
            .content
            .contains("val useDarkStatusBarIcons = doweSafeAreaTopColor(incomingPath).luminance() > 0.179f")
    );
    assert!(
        main_activity
            .content
            .contains("isAppearanceLightStatusBars = useDarkStatusBarIcons")
    );
    assert!(
        main_activity
            .content
            .contains("isAppearanceLightNavigationBars = useDarkNavigationBarIcons")
    );
    assert!(
        main_activity
            .content
            .contains("restoreThemePreference()\n        applyIntentRoute(intent)")
    );
    assert!(main_activity.content.contains(
        "getSharedPreferences(\"dowe\", MODE_PRIVATE)\n            .getString(\"theme-preference\", DoweThemeModule.defaultTheme)"
    ));
    assert!(
        main_activity
            .content
            .contains("DoweDesign.applyTheme(storedTheme)")
    );
    assert!(
        !views
            .content
            .contains("val storedTheme = context.getSharedPreferences")
    );

    let hot_host = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevHostActivity.java"))
        .expect("hot host");
    assert!(hot_host.content.contains("DexClassLoader"));
    assert!(
        hot_host
            .content
            .contains("/_dowe/dev/modules/manifest.json")
    );
    assert!(hot_host.content.contains("getConstructor(Activity.class)"));
    assert!(hot_host.content.contains("resolveEndpoint(getIntent())"));
    assert!(
        hot_host
            .content
            .contains("getSharedPreferences(HMR_PREFERENCES, MODE_PRIVATE)")
    );
    assert!(
        hot_host
            .content
            .contains("putString(HMR_ENDPOINT, value).apply()")
    );
    assert!(hot_host.content.contains("getString(HMR_ENDPOINT, \"\")"));
    assert!(
        hot_host
            .content
            .contains("setContentView(loading);\n        restoreCachedModule();\n        poll();")
    );
    assert!(
        hot_host
            .content
            .contains("new File(getFilesDir(), \"dowe-modules\")")
    );
    assert!(hot_host.content.contains("getString(HMR_VERSION, \"\")"));
    assert!(hot_host.content.contains("putString(HMR_VERSION, version)"));
    assert!(dev.content.contains("extends ContextThemeWrapper"));
    assert!(
        dev.content
            .contains("public void mount(String preferredPath, Intent launchIntent)")
    );
}

