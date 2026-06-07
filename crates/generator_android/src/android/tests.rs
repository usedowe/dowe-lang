use super::{
    generate_android, generate_android_with_app_and_translations,
    generate_android_with_translations,
};
use dowe_components::{
    AccordionItem, AccordionProps, Align, AlertDialogProps, AudioProps, AvatarProps, AvatarStatus,
    BadgeProps, BarProps, Breakpoint, ButtonSize, CarouselIndicatorType, CarouselOrientation,
    CarouselProps, CarouselSlide, CheckboxProps, ChipProps, ColorFamily, ColorProps, ColorToken,
    CommandEntry, CommandProps, ComponentProp, ComponentVariant, CoverSource, DateProps,
    DateRangeProps, DesignConfig, DividerOrientation, DividerProps, DrawerPosition, DrawerProps,
    DropdownProps, ElementProps, FontConfig, GapSize, GapValue, GridProps, GridTracks, ImageAspect,
    ImageLoading, ImageObjectFit, ImageProps, Justify, LayoutProps, ModalProps, NavMenuItem,
    NavMenuItemProps, NavMenuProps, NavigationAction, NavigationOperation, OverlayCornerPosition,
    OverlayEntry, OverlayItemProps, OverlayPaint, OverlayPosition, PropValue, RadioGroupProps,
    RadioOption, ResponsiveEntry, ResponsiveValue, RoundedSize, ScaleValue, ScaffoldProps,
    SectionBackground, SelectOption, SideNavIcon, SideNavItem, SideNavItemProps, SideNavProps,
    SideNavSize, SizeValue, SkeletonAnimation, SkeletonProps, SkeletonVariant, StyleProps, SvgPath,
    SvgPathFill, SvgProps, SvgViewBox, TabItem, TabsPosition, TabsProps, TabsVariant, TextProps,
    TextSize, TextWeight, ToastKind, ToastProps, ToggleProps, TooltipProps, TranslationCatalog,
    TranslationLocale, TranslationValue, VariantProps, ViewAnimation, ViewNode, ViewRoute,
    ViewSection, ViewSignal, ViewSignalValue, VisibilityCondition,
};
use std::path::PathBuf;

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
    assert!(
        dev.content
            .contains("private GradientDrawable doweInputBackground(int color, Integer strokeColor, float radius)")
    );
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
        .find(|file| file.relative_path.ends_with("app/src/main/AndroidManifest.xml"))
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

    assert!(gradle.content.contains(r#"applicationId = "com.example.clinic""#));
    assert!(app_manifest.content.contains(r#"android:label="Clinic Desk""#));
    assert!(dev_manifest.content.contains(r#"package="com.example.clinic""#));
    assert!(dev_manifest.content.contains(r#"android:label="Clinic Desk""#));
    assert!(dev.content.contains("import com.example.clinic.R;"));
}

#[test]
fn generates_native_android_translation_resources() {
    let mut localized_route = route();
    localized_route.page_tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    let output = generate_android_with_translations(
        &[localized_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &translations(),
    );
    let resource = dowe_components::translation_resource_name("home.hero.title");
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(
        views
            .content
            .contains(&format!("stringResource(R.string.{resource})"))
    );
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(
        dev.content
            .contains(&format!("getString(R.string.{resource})"))
    );
    let default_strings = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("res/values/strings.xml"))
        .expect("default strings");
    assert!(default_strings.content.contains("Dowe builds systems."));
    let spanish_strings = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("res/values-es/strings.xml"))
        .expect("spanish strings");
    assert!(spanish_strings.content.contains("Dowe construye sistemas."));
}

#[test]
fn generates_android_code_with_copy_and_theme_tokens() {
    let output = generate_android(
        &[code_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweCode("));
    assert!(
        views
            .content
            .contains("clipboard.setText(AnnotatedString(source))")
    );
    assert!(views.content.contains("DoweCode(source = \"page docsPage\\n  Card variant:\\\"soft\\\" p:4 show:true\\n    Text\\n      Documentation\""));
    assert!(views.content.contains("DoweDesign.primary"));
    assert!(views.content.contains("DoweDesign.info"));
    assert!(views.content.contains("DoweDesign.tertiary"));
    assert!(views.content.contains("DoweDesign.success"));
    assert!(views.content.contains("DoweDesign.warning"));
    assert!(views.content.contains("DoweDesign.danger"));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("private LinearLayout doweCode("));
    assert!(dev.content.contains("ClipboardManager clipboard"));
    assert!(
        dev.content
            .contains("new ForegroundColorSpan(tokenColors[index])")
    );
}

#[test]
fn generates_android_video_with_native_hls_player() {
    let output = generate_android(
        &[video_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweVideo("));
    assert!(views.content.contains("VideoView(context)"));
    assert!(views.content.contains("MediaController(context)"));
    assert!(
        views
            .content
            .contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8")
    );
    assert!(views.content.contains("poster = \"/images/video.jpg\""));
    assert!(views.content.contains("aspect = \"vertical\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("private FrameLayout doweVideo("));
    assert!(
        dev.content
            .contains("VideoView video = new VideoView(this)")
    );
    assert!(
        dev.content
            .contains("MediaController controls = new MediaController(this)")
    );
    assert!(
        dev.content
            .contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8")
    );
}

#[test]
fn generates_android_candlestick_with_canvas_and_stream_runtime() {
    let output = generate_android(
        &[candlestick_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweCandlestick("));
    assert!(
        views
            .content
            .contains("Canvas(modifier = Modifier.matchParentSize())")
    );
    assert!(
        views
            .content
            .contains("doweConnectCandlestickStream(stream, dataPath, maxPoints, state)")
    );
    assert!(
        views
            .content
            .contains("state.upsertCandles(dataPath, payload, maxPoints)")
    );
    assert!(views.content.contains(
        "DoweCandlestick(state = state, dataPath = \"candles\", stream = \"/api/candles\""
    ));
    assert!(views.content.contains("emptyLabel = \"Market closed\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(
        dev.content
            .contains("private DoweCandlestickView doweCandlestick(")
    );
    assert!(dev.content.contains("private void doweUpsertCandles("));
    assert!(dev.content.contains(
        "HttpURLConnection connection = (HttpURLConnection) new URL(address).openConnection()"
    ));
    assert!(dev.content.contains("DoweCandlestickView"));
}

#[test]
fn generates_android_table_for_compose_and_dev_runtime() {
    let output = generate_android(
        &[table_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweTable("));
    assert!(
        views
            .content
            .contains("DoweTable(state = state, dataPath = \"users\"")
    );
    assert!(views.content.contains("DoweTableColumn(field = \"status\", label = \"Status\", align = DoweTableColumnAlign.End, width = \"8rem\")"));
    assert!(views.content.contains("size = DoweTableSize.Lg"));
    assert!(
        views
            .content
            .contains("striped = true, bordered = true, dividers = true")
    );
    assert!(views.content.contains("emptyTitle = \"No users\""));
    assert!(
        views
            .content
            .contains("backgroundColor = DoweDesign.surface")
    );
    assert!(
        views
            .content
            .contains("contentColor = DoweDesign.onSurface")
    );
    assert!(views.content.contains("state.rows(dataPath)"));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("private LinearLayout doweTable("));
    assert!(
        dev.content
            .contains("LinearLayout view0 = doweTable(\"users\"")
    );
    assert!(dev.content.contains("new String[]{\"name\", \"status\"}"));
    assert!(
        dev.content
            .contains("new int[]{Gravity.START, Gravity.END}")
    );
    assert!(
        dev.content
            .contains("doweTableValue(rows.get(rowIndex), fields[columnIndex])")
    );
}

#[test]
fn generates_android_divider_with_native_view() {
    let output = generate_android(
        &[divider_route()],
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
        "Box(modifier = Modifier.width(1.dp).fillMaxHeight().background(DoweDesign.primary))"
    ));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("new View(this)"));
    assert!(dev.content.contains("setBackgroundColor(DOWE_PRIMARY)"));
    assert!(
        dev.content.contains(
            "new LinearLayout.LayoutParams(doweDp(1), ViewGroup.LayoutParams.MATCH_PARENT)"
        )
    );
}

#[test]
fn generates_compose_responsive_runtime_values() {
    let output = generate_android(
        &[responsive_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("BoxWithConstraints"));
    assert!(views.content.contains(
        "fun LoginScreen(viewportWidth: Dp, sectionRegistry: DoweSectionRegistry, navigate:"
    ));
    assert!(
        views
            .content
            .contains("doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)")
    );
    assert!(
        views
            .content
            .contains("doweResponsive(viewportWidth, md = 32.dp)")
    );
    assert!(
            views
                .content
                .contains("doweResponsive(viewportWidth, md = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)) ?: doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)")
        );
    assert!(
            views
                .content
                .contains("fontWeight = doweResponsive(viewportWidth, xs = FontWeight.Thin, md = FontWeight.ExtraLight, lg = FontWeight.Black) ?: FontWeight.Normal")
        );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(
        dev.content
            .contains("viewportWidth = getResources().getConfiguration().screenWidthDp;")
    );
    assert!(
        dev.content
            .contains("int viewportWidth = this.viewportWidth;")
    );
    assert!(
        dev.content
            .contains("doweResponsiveInt(viewportWidth, 16, null, 32, null, null)")
    );
    assert!(
        dev.content
            .contains("doweResponsiveInt(viewportWidth, null, null, 32, null, null)")
    );
    assert!(dev.content.contains(
        "doweTextWeight(doweResponsiveInt(viewportWidth, 100, null, 200, 900, null), 400)"
    ));
}

#[test]
fn generates_show_visibility_conditions() {
    let output = generate_android(
        &[show_route()],
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
            .contains("if (doweResponsive(viewportWidth, xs = false, md = true) ?: true) {")
    );
    assert!(views.content.contains("if (state.bool(\"ready01\")) {"));
    assert!(
        views
            .content
            .contains("if (state.bool(\"item.ready\", row.value)) {")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(dev.content.contains(
        "if (doweShow(doweResponsiveBool(viewportWidth, false, null, true, null, null))) {"
    ));
    assert!(dev.content.contains("if (doweBool(\"ready01\", null)) {"));
    assert!(dev.content.contains("if (doweBool(\"item.ready\", row"));
}

#[test]
fn generates_dev_flex_justify_and_align_gravity() {
    let output = generate_android(
        &[flex_alignment_route()],
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
            "horizontalArrangement = doweHorizontalArrangement(doweResponsive(viewportWidth, xs = DoweJustify.End), doweResponsive(viewportWidth, xs = 12.dp))"
        ));
    assert!(views.content.contains(
            "verticalAlignment = doweVerticalAlignment(doweResponsive(viewportWidth, xs = DoweAlign.Center))"
        ));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(
        dev.content
            .contains("private static final class DoweFlexLayout extends ViewGroup")
    );
    assert!(dev.content.contains(
            "DoweFlexLayout view0 = doweFlex(doweResponsiveInt(viewportWidth, DOWE_JUSTIFY_END, null, null, null, null), doweResponsiveInt(viewportWidth, DOWE_ALIGN_CENTER, null, null, null, null), doweResponsiveInt(viewportWidth, 12, null, null, null, null))"
        ));
}

#[test]
fn generates_fragment_aware_native_history_and_deep_links() {
    let output = generate_android(
        &[index_route_with_signup_link(), signup_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(dev.content.contains("private String currentPath = \"/\";"));
    assert!(
        dev.content
            .contains("private String currentFragment = null;")
    );
    assert!(
        dev.content
            .contains("private static final class DoweRouteEntry")
    );
    assert!(
        dev.content
            .contains("private void renderIndex(LinearLayout root)")
    );
    assert!(
        dev.content
            .contains("private void renderSignup(LinearLayout root)")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/signup\", \"join\"))")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"replace\", currentPath, \"hero\"))")
    );
    assert!(dev.content.contains("setOnClickListener(v -> doweBack())"));
    assert!(dev.content.contains(
            "setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT))"
        ));
    assert!(dev.content.contains("setAllCaps(false)"));
    assert!(
        dev.content
            .contains("backStack.add(new DoweRouteEntry(currentPath, currentFragment));")
    );
    assert!(dev.content.contains("private boolean doweCanSection"));
    assert!(dev.content.contains("data.getFragment()"));
    assert!(dev.content.contains("doweRegisterBackHandler();"));
    assert!(
        dev.content
            .contains("getOnBackInvokedDispatcher().registerOnBackInvokedCallback(")
    );
    assert!(dev.content.contains("this::doweBack"));
    assert!(
        dev.content
            .contains("public void onBackPressed() {\n        doweBack();")
    );
    assert!(
        dev.content
            .contains("\"/\".equals(path) || \"/signup\".equals(path)")
    );
    assert!(dev.content.contains("doweApplyIntentRoute();"));
    assert!(dev.content.contains("doweScrollToFragment();"));
    assert!(
        dev.content
            .contains("scrollView.scrollTo(0, doweTopRelativeToRoot(target));")
    );
    assert!(dev.content.contains(r#"doweRegisterSection("hero", "#));

    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private data class DoweRouteEntry"));
    assert!(
        views
            .content
            .contains("fun navigate(operation: String, target: String, fragment: String?)")
    );
    assert!(
        views
            .content
            .contains(r#"{ navigate("push", "/signup", "join") }"#)
    );
    assert!(
        views
            .content
            .contains(r#"{ navigate("replace", "", "hero") }"#)
    );
    assert!(views.content.contains("BackHandler(enabled = true)"));
    assert!(views.content.contains("class DoweSectionRegistry"));
    assert!(
        views
            .content
            .contains("scrollState.animateScrollTo(targetSection)")
    );
    assert!(
        views
            .content
            .contains(r#".doweSection(sectionRegistry, "hero")"#)
    );

    let main_activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("MainActivity.kt"))
        .expect("main activity");
    assert!(
        main_activity
            .content
            .contains("override fun onNewIntent(intent: Intent)")
    );
    assert!(
        main_activity
            .content
            .contains("intent?.data?.fragment?.takeIf")
    );
    assert!(main_activity.content.contains("incomingRequest += 1"));
    assert!(views.content.contains("LaunchedEffect(navigationRequest)"));
}

#[test]
fn generates_compose_and_dev_layout_bars() {
    let output = generate_android(
        &[bar_route()],
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
            "Box(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).padding(horizontal = 16.dp, vertical = 8.dp).clip(RoundedCornerShape(DoweDesign.radiusBox)).background(DoweDesign.surface).border(1.dp, DoweDesign.muted, RoundedCornerShape(DoweDesign.radiusBox)), contentAlignment = Alignment.Center)"
        ));
    assert!(
        views
            .content
            .contains("Modifier.fillMaxWidth().widthIn(max = 1152.dp)")
    );
    assert!(
        views
            .content
            .contains("CompositionLocalProvider(LocalContentColor provides DoweDesign.onSurface)")
    );
    assert!(
        views
            .content
            .contains("horizontalArrangement = Arrangement.Center")
    );
    assert!(views.content.contains("Text(\"Brand\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains("doweBackground(DOWE_SURFACE, DOWE_RADIUS_BOX)")
    );
    assert!(
        dev.content
            .contains("setGravity(Gravity.CENTER_VERTICAL | Gravity.START)")
    );
    assert!(
        dev.content
            .contains("setGravity(Gravity.CENTER_VERTICAL | Gravity.CENTER)")
    );
    assert!(
        dev.content
            .contains("setGravity(Gravity.CENTER_VERTICAL | Gravity.END)")
    );
    assert!(
        dev.content
            .contains("new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)")
    );
    assert!(
        dev.content
            .contains("new LinearLayout.LayoutParams(0, 0, 1f)")
    );
    assert!(dev.content.contains("doweText(\"Brand\""));
    assert!(dev.content.contains("doweText(\"Footer\""));
}

#[test]
fn generates_compose_and_dev_side_nav() {
    let output = generate_android(
        &[side_nav_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("DoweSideNavSubmenu(open = true"));
    assert!(
        views
            .content
            .contains("Column(modifier = Modifier.padding(start = 16.dp))")
    );
    assert!(views.content.contains("AnimatedVisibility("));
    assert!(views.content.contains(
        "fadeIn(animationSpec = tween(160)) + expandVertically(animationSpec = tween(180))"
    ));
    assert!(views.content.contains(
        "fadeOut(animationSpec = tween(120)) + shrinkVertically(animationSpec = tween(180))"
    ));
    assert!(views.content.contains(r#"active = activePath == "/bars""#));
    assert!(views.content.contains(r#"Text(text = "Workspace""#));
    assert!(views.content.contains(r#"Text(text = "Blogs""#));
    assert!(
        views
            .content
            .contains("DoweSvg(viewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("setVisibility(View.VISIBLE)"));
    assert!(dev.content.contains("doweToggleSideNavSubmenu"));
    assert!(
        dev.content
            .contains("view.animate().alpha(0f).translationY(-doweDp(4)).setDuration(140)")
    );
    assert!(dev.content.contains("doweText(\"Blogs\""));
    assert!(
        dev.content
            .contains("new DoweSvgView(this, 0f, 0f, 24f, 24f")
    );
}

#[test]
fn generates_compose_and_dev_navigation_shell_components() {
    let output = generate_android(
        &[navigation_shell_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("DoweNavMenu("));
    assert!(views.content.contains("DoweNavMenuItem(active = activePath == \"/\""));
    assert!(views.content.contains("DoweNavMenuItem(active = openIndex == 1"));
    assert!(views.content.contains("Row(modifier = Modifier.fillMaxWidth().weight(1f))"));
    assert!(views.content.contains("Text(\"Resource hub\""));
    assert!(views.content.contains("Text(text = \"Side Home\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("doweText(\"Resource hub\""));
    assert!(dev.content.contains("doweText(\"Side Home\""));
}

#[test]
fn generates_compose_and_dev_tabs() {
    let output = generate_android(
        &[tabs_route()],
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
            .contains("private fun DoweTabs(items: List<DoweTabItem>")
    );
    assert!(views.content.contains("DoweTabs(items = listOf(DoweTabItem(id = \"overview\", label = \"Overview\"), DoweTabItem(id = \"details\", label = \"Details\")), initialId = \"overview\""));
    assert!(views.content.contains("position = \"start\", variant = \"line\""));
    assert!(views.content.contains("backgroundColor = Color.Transparent"));
    assert!(views.content.contains("accentColor = DoweDesign.primary"));
    assert!(views.content.contains("if (activeTab == \"overview\")"));
    assert!(views.content.contains("Text(\"Overview content\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("TextView[] view"));
    assert!(dev.content.contains("View[] view"));
    assert!(dev.content.contains("doweText(\"Overview\""));
    assert!(dev.content.contains("doweText(\"Details\""));
    assert!(dev.content.contains("setVisibility(active ? View.VISIBLE : View.GONE)"));
}

#[test]
fn generates_compose_and_dev_drawer() {
    let output = generate_android(
        &[drawer_route()],
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
            .contains("private fun DoweDrawer(open: Boolean")
    );
    assert!(views.content.contains("DoweDrawer(open = state.bool(\"drawer01\"), onClose = { state.write(\"drawer01\", false) }, position = \"end\""));
    assert!(views.content.contains("radius = 0.dp"));
    assert!(
        views
            .content
            .contains("disableOverlayClose = true, hideCloseButton = false")
    );
    assert!(
        views
            .content
            .contains("Modifier.fillMaxHeight().widthIn(max = 320.dp)")
    );
    assert!(
        views.content.contains(
            "private fun doweDrawerShape(position: String, radius: Dp): RoundedCornerShape"
        )
    );
    assert!(views.content.contains(r#"RoundedCornerShape(topStart = radius, topEnd = 0.dp, bottomEnd = 0.dp, bottomStart = radius)"#));
    let rounded_style = StyleProps {
        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
        ..Default::default()
    };
    assert_eq!(
        super::compose_drawer_radius(&rounded_style),
        "doweResponsive(viewportWidth, xs = 12.dp) ?: 0.dp"
    );
    assert_eq!(
        super::dev_drawer_radius(&rounded_style),
        "doweFloat(doweResponsiveFloat(viewportWidth, 12f, null, null, null, null), 0f)"
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("if (doweBool(\"drawer01\"))"));
    assert!(dev.content.contains("new PopupWindow("));
    assert!(dev.content.contains("doweWrite(\"drawer01\", false)"));
    assert!(
        dev.content
            .contains("private void renderCurrentRoute(boolean scrollToFragment)")
    );
    assert!(
        dev.content
            .contains("if (scrollToFragment) {\n            doweScrollToFragment();\n        }")
    );
    assert!(dev.content.contains("renderCurrentRoute(false);"));
    assert!(
        dev.content
            .contains("scrollView.scrollTo(0, doweTopRelativeToRoot(target));")
    );
    assert!(
        dev.content
            .contains("root.post(() -> { if (root.getWindowToken() != null) { view")
    );
    assert!(!dev.content.contains("smoothScrollTo"));
    assert!(
        dev.content
            .contains(r#"doweDrawerBackground(DOWE_SURFACE, null, "end", 0f)"#)
    );
    assert!(dev.content.contains("TextView"));
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END)"
    ));
    assert!(!dev.content.contains("doweCard(DOWE_SURFACE, null)"));
}

#[test]
fn generates_compose_and_dev_display_overlay_components() {
    let output = generate_android(
        &[display_overlay_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweAvatar("));
    assert!(views.content.contains("DoweAvatar(source = null, name = \"Ada\""));
    assert!(views.content.contains("DoweBadge(text = \"3\", position = \"bottom-right\""));
    assert!(views.content.contains("DoweChip(text = \"Filter\", size = \"sm\""));
    assert!(views.content.contains("DoweSkeleton(variant = \"rounded\", animation = \"pulse\""));
    assert!(views.content.contains("DoweModal(open = state.bool(\"modal01\")"));
    assert!(views.content.contains("DoweAlertDialog(open = state.bool(\"modal01\")"));
    assert!(views.content.contains("DoweTooltip(label = \"More actions\", position = \"end\""));
    assert!(views.content.contains("DoweToast(visible = true, title = \"Saved\""));
    assert!(views.content.contains("DoweDropdown(backgroundColor = DoweDesign.surface"));
    assert!(views.content.contains("DoweCommand(open = state.bool(\"modal01\")"));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("doweText(\"Search\""));
    assert!(dev.content.contains("doweText(\"Profile\""));
    assert!(dev.content.contains("if (doweBool(\"modal01\"))"));
}

#[test]
fn generates_portable_grid_controls_and_variant_colors() {
    let output = generate_android(
        &[parity_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("DoweGrid(modifier ="));
    assert!(
        views
            .content
            .contains("columns = doweResponsive(viewportWidth, xs = 1, md = 2) ?: 1")
    );
    assert!(
        views
            .content
            .contains("horizontalGap = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp")
    );
    assert!(views.content.contains("DoweInput("));
    assert!(views.content.contains("modifier = Modifier.weight(1f)"));
    assert!(views.content.contains("minHeight = 40.dp"));
    assert!(views.content.contains("horizontalPadding = 12.dp"));
    assert!(
        views
            .content
            .contains("contentColor = DoweDesign.secondary")
    );
    assert!(views.content.contains("borderColor = DoweDesign.muted"));
    assert!(
        views
            .content
            .contains("contentColor = DoweDesign.onSoftMuted")
    );
    assert!(views.content.contains(
            "CardDefaults.cardColors(containerColor = DoweDesign.surface, contentColor = DoweDesign.onSurface), border = BorderStroke(1.dp, DoweDesign.surface)"
        ));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("DoweGridLayout"));
    assert!(dev.content.contains(
            "doweGrid(doweResponsiveInt(viewportWidth, 1, null, 2, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null))"
        ));
    assert!(dev.content.contains("setIncludeFontPadding(false)"));
    assert!(dev.content.contains("setMinHeight(doweDp(40))"));
    assert!(
        dev.content
            .contains("setPadding(doweDp(12), 0, doweDp(12), 0)")
    );
    assert!(
        dev.content
            .contains("background.setCornerRadius(doweDp(radius));")
    );
    assert!(dev.content.contains("private float doweDp(float value)"));
    assert!(dev.content.contains(
        "setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f))"
    ));
    assert!(dev.content.contains("doweCard(DOWE_SOFT_MUTED, null)"));
    assert!(dev.content.contains("doweCard(DOWE_SURFACE, DOWE_SURFACE)"));
    assert!(
        dev.content
            .contains("doweText(\"Surface\", DOWE_ON_SURFACE")
    );
}

#[test]
fn generates_labeled_input_and_select_fields() {
    let output = generate_android(
        &[form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweInput("));
    assert!(
        views
            .content
            .contains(r#"label = "Email", placeholder = "Email address", floating = false"#)
    );
    assert!(
        views
            .content
            .contains(r#"label = "Name", placeholder = "Full name", floating = true"#)
    );
    assert!(views.content.contains("private fun DoweSelect("));
    assert!(views.content.contains("private fun DoweSelectPopover("));
    assert!(views.content.contains("popupMounted"));
    assert!(
        views
            .content
            .contains("targetValue = if (visible) 1f else 0f")
    );
    assert!(views.content.contains("Popup("));
    assert!(!views.content.contains("DropdownMenu("));
    assert!(!views.content.contains("DropdownMenuItem("));
    assert!(
        views.content.contains(
            r#"label = "Department", placeholder = "Choose department", floating = false"#
        )
    );
    assert!(
        views
            .content
            .contains(r#"label = "Role", placeholder = "Choose role", floating = true"#)
    );
    assert!(views.content.contains(
        r#"DoweSelectOption(value = "admin", label = "Admin", description = "Manages users")"#
    ));
    assert!(views.content.contains("private val doweSelectArrowPaths"));
    assert!(
        views
            .content
            .contains("DoweSvg(viewBox = doweSelectArrowViewBox")
    );
    assert!(
        views
            .content
            .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4")
    );
    assert!(
        views
            .content
            .contains("val active = expanded || selected != null")
    );
    assert!(
        views
            .content
            .contains("if (selected != null || !floating || expanded)")
    );
    assert!(views.content.contains("Text(text = option.description"));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains(r#"doweControlLabel("Email", DOWE_PRIMARY"#)
    );
    assert!(dev.content.contains(r#".setHint("Email address")"#));
    assert!(dev.content.contains("doweFloatingInput("));
    assert!(dev.content.contains(r#""Name", "Full name", DOWE_PRIMARY"#));
    assert!(dev.content.contains("doweUpdateFloatingInputLabel"));
    assert!(
        dev.content
            .contains(r#"doweControlLabel("Department", DOWE_PRIMARY"#)
    );
    assert!(dev.content.contains("doweFloatingSelect("));
    assert!(dev.content.contains("doweUpdateFloatingSelectLabel"));
    assert!(dev.content.contains("expanded || hasSelection"));
    assert!(
        dev.content
            .contains("label.setTextSize(active ? 12f : baseSize);")
    );
    assert!(dev.content.contains("input.setPadding(input.getPaddingLeft(), active ? doweDp(10) : 0, input.getPaddingRight(), input.getPaddingBottom());"));
    assert!(dev.content.contains("doweSelectFrame("));
    assert!(dev.content.contains("doweSelectPopup("));
    assert!(dev.content.contains("PopupWindow popup = new PopupWindow"));
    assert!(dev.content.contains("doweSelectArrow("));
    assert!(
        dev.content
            .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4")
    );
    assert!(!dev.content.contains("Spinner view"));
    assert!(!dev.content.contains("import android.widget.Spinner;"));
    assert!(dev.content.contains(r#"new String[]{"Admin"}"#));
    assert!(dev.content.contains(r#"new String[]{"Manages users"}"#));
    assert!(!dev.content.contains(r#".setPrompt("Role")"#));
}

#[test]
fn generates_compose_and_dev_media_display_form_components() {
    let output = generate_android(
        &[media_display_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");

    assert!(views.content.contains("private fun DoweAudio("));
    assert!(views.content.contains("DoweAudio(source ="));
    assert!(views.content.contains("private fun DoweImage("));
    assert!(views.content.contains("DoweAccordion("));
    assert!(views.content.contains("DoweCarousel("));
    assert!(views.content.contains("DoweCheckbox("));
    assert!(views.content.contains("DoweColorField("));
    assert!(views.content.contains("DoweDateField("));
    assert!(views.content.contains("DoweDateRangeField("));
    assert!(views.content.contains("DoweRadioGroup("));
    assert!(views.content.contains("DoweToggle("));
    assert!(views.content.contains("CheckboxDefaults.colors"));
    assert!(views.content.contains("DoweInput(value = value"));
    assert!(views.content.contains("doweHexColor(value, backgroundColor)"));
    assert!(views.content.contains("BasicTextField("));
    assert!(views.content.contains("RadioButtonDefaults.colors"));
    assert!(views.content.contains("SwitchDefaults.colors"));
    assert!(dev.content.contains("android.widget.CheckBox"));
    assert!(dev.content.contains("Color.parseColor("));
    assert!(dev.content.contains("doweControlLabel(\"Theme\""));
    assert!(dev.content.contains("doweControlLabel(\"Ship date\""));
    assert!(dev.content.contains("android.widget.RadioGroup"));
    assert!(dev.content.contains("android.widget.Switch"));
    assert!(dev.content.contains("doweText(\"Off\""));
    assert!(dev.content.contains("doweText(\"On\""));
}

#[test]
fn generates_svg_compose_and_dev_views() {
    let output = generate_android(
        &[svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweSvg("));
    assert!(views.content.contains("DoweSvgViewBox(0f, 0f, 24f, 24f)"));
    assert!(views.content.contains("DoweSvgFill.CurrentColor"));
    assert!(views.content.contains(
        "doweResponsive(viewportWidth, xs = DoweDesign.tertiary) ?: LocalContentColor.current"
    ));
    assert!(
        views
            .content
            .contains("PathParser().parsePathString(entry.data).toPath()")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains("private static final class DoweSvgView extends View")
    );
    assert!(
        dev.content
            .contains("private static final class DoweSvgPathParser")
    );
    assert!(
        dev.content
            .contains("Path path = DoweSvgPathParser.parse(entry.data)")
    );
    assert!(dev.content.contains(
        "Integer fill = entry.currentColor ? Integer.valueOf(currentColor) : entry.color;"
    ));
    assert!(!dev.content.contains("import android.graphics.PathParser;"));
    assert!(dev.content.contains("new DoweSvgView(this, 0f, 0f, 24f, 24f, doweColor(doweResponsiveInt(viewportWidth, DOWE_TERTIARY, null, null, null, null), DOWE_ON_BACKGROUND)"));
}

#[test]
fn generates_android_view_motion() {
    let output = generate_android(
        &[motion_route()],
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
            .contains("private enum class DoweAnimationPreset")
    );
    assert!(
        views
            .content
            .contains(".doweAnimation(DoweAnimationPreset.FadeIn)")
    );
    assert!(
        views
            .content
            .contains(".doweAnimation(DoweAnimationPreset.SlideUp)")
    );
    assert!(views.content.contains("animateFloatAsState("));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains(r#"doweAnimate(view0, "fadeIn");"#));
    assert!(dev.content.contains(r#"doweAnimate(view1, "slideUp");"#));
}

fn route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![text("Layout"), ViewNode::Children],
        },
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Card {
                props: VariantProps {
                    color: Some(ColorFamily::Primary),
                    ..Default::default()
                },
                children: vec![text("Login")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn section_route() -> ViewRoute {
    ViewRoute {
        id: "sections".to_string(),
        route_path: "/sections".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Section {
                    props: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::OnBackground)),
                        background: Some(ResponsiveValue::ordered(vec![
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Xs,
                                value: SectionBackground::Aurora,
                            },
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: SectionBackground::Ocean,
                            },
                        ])),
                        ..Default::default()
                    },
                    children: vec![text("Hero")],
                },
                ViewNode::Section {
                    props: StyleProps {
                        cover: Some(ResponsiveValue::scalar(CoverSource(
                            "https://example.com/hero.jpg".to_string(),
                        ))),
                        overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                            "0.35".to_string(),
                        ))),
                        ..Default::default()
                    },
                    children: vec![text("Covered")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn bar_route() -> ViewRoute {
    ViewRoute {
        id: "bars".to_string(),
        route_path: "/bars".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::AppBar {
                    props: bar_props(true),
                    start: vec![text("Menu")],
                    center: vec![text("Brand")],
                    end: vec![text("Account")],
                },
                ViewNode::Children,
                ViewNode::Footer {
                    props: bar_props(false),
                    start: vec![text("Footer")],
                    center: Vec::new(),
                    end: vec![text("Legal")],
                },
            ],
        },
        page_tree: ViewNode::BottomBar {
            props: bar_props(false),
            start: vec![text("Home")],
            center: vec![text("Search")],
            end: vec![text("Profile")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn side_nav_route() -> ViewRoute {
    ViewRoute {
        id: "bars".to_string(),
        route_path: "/bars".to_string(),
        layout_tree: ViewNode::SideNav {
            props: SideNavProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(ColorFamily::Surface),
                    ..Default::default()
                },
                size: SideNavSize::Md,
                wide: true,
            },
            items: vec![
                SideNavItem::Header(side_nav_item("Workspace", None)),
                SideNavItem::Item(side_nav_item(
                    "Home",
                    Some(NavigationAction::Internal {
                        path: "/bars".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                )),
                SideNavItem::Divider,
                SideNavItem::Submenu {
                    props: side_nav_item("Content", None),
                    open: true,
                    items: vec![side_nav_item("Blogs", None)],
                },
            ],
        },
        page_tree: ViewNode::Children,
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn navigation_shell_route() -> ViewRoute {
    ViewRoute {
        id: "shell".to_string(),
        route_path: "/".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scaffold {
            props: ScaffoldProps {
                style: StyleProps::default(),
                boxed: true,
            },
            app_bar: vec![ViewNode::NavMenu {
                props: NavMenuProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Ghost),
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    size: SideNavSize::Md,
                },
                items: vec![
                    NavMenuItem::Item(NavMenuItemProps {
                        label: "Home".to_string(),
                        description: None,
                        icon: None,
                        on_click: None,
                        navigation: Some(NavigationAction::Internal {
                            path: "/".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                    }),
                    NavMenuItem::Submenu {
                        props: NavMenuItemProps {
                            label: "Docs".to_string(),
                            description: None,
                            icon: None,
                            on_click: None,
                            navigation: None,
                        },
                        items: vec![NavMenuItemProps {
                            label: "Guide".to_string(),
                            description: Some("Start here".to_string()),
                            icon: None,
                            on_click: None,
                            navigation: Some(NavigationAction::Internal {
                                path: "/docs".to_string(),
                                fragment: None,
                                operation: NavigationOperation::Push,
                            }),
                        }],
                    },
                    NavMenuItem::Megamenu {
                        props: NavMenuItemProps {
                            label: "Resources".to_string(),
                            description: None,
                            icon: None,
                            on_click: None,
                            navigation: None,
                        },
                        content: vec![text("Resource hub")],
                    },
                ],
            }],
            start: vec![ViewNode::Sidebar {
                props: SideNavProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    size: SideNavSize::Md,
                    wide: true,
                },
                items: vec![SideNavItem::Item(SideNavItemProps {
                    label: "Side Home".to_string(),
                    description: None,
                    status: None,
                    icon: None,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                })],
            }],
            main: vec![text("Main content")],
            end: Vec::new(),
            bottom_bar: vec![text("Bottom")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn tabs_route() -> ViewRoute {
    ViewRoute {
        id: "tabs".to_string(),
        route_path: "/tabs".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Tabs {
            props: TabsProps {
                style: StyleProps::default(),
                variant: TabsVariant::Line,
                color: ColorFamily::Primary,
                position: TabsPosition::Start,
            },
            tabs: vec![
                TabItem {
                    id: "overview".to_string(),
                    label: "Overview".to_string(),
                    children: vec![text("Overview content")],
                },
                TabItem {
                    id: "details".to_string(),
                    label: "Details".to_string(),
                    children: vec![text("Details content")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn drawer_route() -> ViewRoute {
    ViewRoute {
        id: "drawer".to_string(),
        route_path: "/drawer".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            signals: vec![ViewSignal {
                id: "drawer01".to_string(),
                name: "drawerOpen".to_string(),
                initial: ViewSignalValue::Bool(false),
                schema: None,
            }],
            actions: Vec::new(),
            children: vec![ViewNode::Drawer {
                props: DrawerProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    open: "drawerOpen".to_string(),
                    position: DrawerPosition::End,
                    disable_overlay_close: true,
                    hide_close_button: false,
                },
                children: vec![text("Navigation")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn side_nav_item(label: &str, navigation: Option<NavigationAction>) -> SideNavItemProps {
    SideNavItemProps {
        label: label.to_string(),
        description: None,
        status: None,
        icon: (label == "Home").then(|| SideNavIcon {
            props: SvgProps {
                style: Default::default(),
                view_box: SvgViewBox {
                    min_x: "0".to_string(),
                    min_y: "0".to_string(),
                    width: "24".to_string(),
                    height: "24".to_string(),
                },
            },
            paths: vec![SvgPath {
                data: "M3 11l9-8 9 8v10H3z".to_string(),
                fill: SvgPathFill::CurrentColor,
            }],
        }),
        on_click: None,
        navigation,
    }
}

fn index_route_with_signup_link() -> ViewRoute {
    ViewRoute {
        id: "index".to_string(),
        route_path: "/".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                element: ElementProps {
                    id: Some("hero".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Internal {
                            path: "/signup".to_string(),
                            fragment: Some("join".to_string()),
                            operation: NavigationOperation::Push,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Signup")],
                },
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Section {
                            fragment: "hero".to_string(),
                            operation: NavigationOperation::Replace,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Hero")],
                },
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Back),
                        ..Default::default()
                    },
                    children: vec![text("Back")],
                },
            ],
        },
        sections: vec![ViewSection {
            id: "hero".to_string(),
        }],
        navigation_actions: Vec::new(),
    }
}

fn signup_route() -> ViewRoute {
    ViewRoute {
        id: "signup".to_string(),
        route_path: "/signup".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Button {
            props: VariantProps {
                navigation: Some(NavigationAction::Internal {
                    path: "/".to_string(),
                    fragment: None,
                    operation: NavigationOperation::Push,
                }),
                ..Default::default()
            },
            children: vec![text("Signup")],
        },
        sections: vec![ViewSection {
            id: "join".to_string(),
        }],
        navigation_actions: Vec::new(),
    }
}

fn responsive_route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[
                        (Breakpoint::Xs, 4),
                        (Breakpoint::Md, 8),
                    ])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Children],
        },
        page_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[(Breakpoint::Md, 8)])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Text {
                props: TextProps {
                    size: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: TextSize::Lg,
                    }])),
                    weight: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: TextWeight::Thin,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: TextWeight::Extralight,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Lg,
                            value: TextWeight::Black,
                        },
                    ])),
                    ..Default::default()
                },
                value: "Login".to_string(),
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn show_route() -> ViewRoute {
    ViewRoute {
        id: "ready".to_string(),
        route_path: "/ready".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            signals: vec![
                ViewSignal {
                    id: "ready01".to_string(),
                    name: "isReady".to_string(),
                    initial: ViewSignalValue::Bool(false),
                    schema: None,
                },
                ViewSignal {
                    id: "rows01".to_string(),
                    name: "rows".to_string(),
                    initial: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                        ("id".to_string(), ViewSignalValue::String("1".to_string())),
                        ("ready".to_string(), ViewSignalValue::Bool(true)),
                    ])]),
                    schema: None,
                },
            ],
            actions: Vec::new(),
            children: vec![
                ViewNode::Box {
                    props: StyleProps {
                        element: ElementProps {
                            show: Some(VisibilityCondition::Static(responsive_bool(&[
                                (Breakpoint::Xs, false),
                                (Breakpoint::Md, true),
                            ]))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    children: vec![ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                element: ElementProps {
                                    show: Some(VisibilityCondition::Signal("isReady".to_string())),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Ready".to_string(),
                    }],
                },
                ViewNode::Each {
                    item: "row".to_string(),
                    collection: "rows".to_string(),
                    key: "row.id".to_string(),
                    children: vec![ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                element: ElementProps {
                                    show: Some(VisibilityCondition::Signal(
                                        "row.ready".to_string(),
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Row".to_string(),
                    }],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn parity_route() -> ViewRoute {
    let input = || ViewNode::Input {
        props: VariantProps {
            variant: Some(ComponentVariant::Outlined),
            color: Some(ColorFamily::Secondary),
            ..Default::default()
        },
    };
    ViewRoute {
        id: "parity".to_string(),
        route_path: "/parity".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Flex {
                    props: LayoutProps {
                        gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                            ScaleValue::from_half_steps(4),
                        )))),
                        ..Default::default()
                    },
                    children: vec![input(), input()],
                },
                ViewNode::Grid {
                    props: GridProps {
                        columns: Some(ResponsiveValue::ordered(vec![
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Xs,
                                value: GridTracks::Count(1),
                            },
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: GridTracks::Count(2),
                            },
                        ])),
                        gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                            ScaleValue::from_half_steps(8),
                        )))),
                        ..Default::default()
                    },
                    children: vec![
                        ViewNode::Card {
                            props: VariantProps {
                                variant: Some(ComponentVariant::Soft),
                                color: Some(ColorFamily::Muted),
                                ..Default::default()
                            },
                            children: vec![text("Card")],
                        },
                        ViewNode::Card {
                            props: VariantProps {
                                variant: Some(ComponentVariant::Outlined),
                                color: Some(ColorFamily::Surface),
                                ..Default::default()
                            },
                            children: vec![text("Surface")],
                        },
                    ],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn flex_alignment_route() -> ViewRoute {
    ViewRoute {
        id: "flex_alignment".to_string(),
        route_path: "/flex-alignment".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Flex {
            props: LayoutProps {
                justify: Some(ResponsiveValue::scalar(Justify::End)),
                align: Some(ResponsiveValue::scalar(Align::Center)),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(6),
                )))),
                style: StyleProps {
                    sizing: dowe_components::SizingProps {
                        w: Some(ResponsiveValue::scalar(SizeValue::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            children: vec![text("One"), text("Two")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn form_route() -> ViewRoute {
    ViewRoute {
        id: "form".to_string(),
        route_path: "/form".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Input {
                    props: VariantProps {
                        label: Some("Email".to_string()),
                        placeholder: Some("Email address".to_string()),
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                },
                ViewNode::Input {
                    props: VariantProps {
                        label: Some("Name".to_string()),
                        placeholder: Some("Full name".to_string()),
                        label_floating: true,
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                },
                ViewNode::Select {
                    props: VariantProps {
                        label: Some("Department".to_string()),
                        placeholder: Some("Choose department".to_string()),
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                    options: vec![SelectOption {
                        value: "design".to_string(),
                        label: "Design".to_string(),
                        description: None,
                    }],
                },
                ViewNode::Select {
                    props: VariantProps {
                        label: Some("Role".to_string()),
                        placeholder: Some("Choose role".to_string()),
                        label_floating: true,
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                    options: vec![SelectOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                        description: Some("Manages users".to_string()),
                    }],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn media_display_form_route() -> ViewRoute {
    ViewRoute {
        id: "components".to_string(),
        route_path: "/components".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Audio {
                    props: AudioProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Soft),
                            color: Some(ColorFamily::Primary),
                            ..Default::default()
                        },
                        src: "https://cdn.pixabay.com/audio/2022/04/25/audio_5d61b5204f.mp3"
                            .to_string(),
                        subtitle: Some("Preview".to_string()),
                        avatar_src: None,
                    },
                },
                ViewNode::Image {
                    props: ImageProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Secondary),
                            ..Default::default()
                        },
                        src: "https://example.com/photo.jpg".to_string(),
                        alt: "Photo".to_string(),
                        aspect: ImageAspect::Square,
                        object_fit: ImageObjectFit::Cover,
                        loading: ImageLoading::Lazy,
                        hide_controls: true,
                    },
                },
                ViewNode::Accordion {
                    props: AccordionProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Surface),
                            ..Default::default()
                        },
                        multiple: true,
                    },
                    items: vec![AccordionItem {
                        id: "intro".to_string(),
                        label: "Intro".to_string(),
                        disabled: false,
                        default_open: true,
                        children: vec![text("Intro body")],
                    }],
                },
                ViewNode::Carousel {
                    props: CarouselProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Info),
                            ..Default::default()
                        },
                        autoplay: false,
                        autoplay_interval: 3000,
                        disable_loop: false,
                        hide_controls: false,
                        hide_indicators: false,
                        show_navigation: true,
                        show_counter: true,
                        orientation: CarouselOrientation::Horizontal,
                        size: ButtonSize::Md,
                        indicator_type: CarouselIndicatorType::Bar,
                        title: Some("Samples".to_string()),
                        slide_width: None,
                        slide_height: None,
                        slides_per_view: 1,
                        gap: 8,
                    },
                    slides: vec![CarouselSlide {
                        id: "one".to_string(),
                        children: vec![text("First")],
                    }],
                },
                ViewNode::Checkbox {
                    props: CheckboxProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Success),
                            label: Some("Accept".to_string()),
                            element: ElementProps {
                                bind: Some("accepted".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        checked: true,
                        disabled: false,
                        name: Some("accepted".to_string()),
                    },
                },
                ViewNode::Color {
                    props: ColorProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Primary),
                            label: Some("Theme".to_string()),
                            element: ElementProps {
                                bind: Some("themeColor".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "#3366ff".to_string(),
                        size: ButtonSize::Md,
                        name: None,
                        help_text: None,
                        error_text: None,
                        show_hex: true,
                        show_rgb: false,
                        show_cmyk: false,
                        show_oklch: false,
                    },
                },
                ViewNode::Date {
                    props: DateProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Warning),
                            label: Some("Ship date".to_string()),
                            element: ElementProps {
                                bind: Some("shipDate".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: Some("2026-06-05".to_string()),
                        size: ButtonSize::Md,
                        name: None,
                        help_text: None,
                        error_text: None,
                        min: None,
                        max: None,
                    },
                },
                ViewNode::DateRange {
                    props: DateRangeProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Danger),
                            label: Some("Range".to_string()),
                            ..Default::default()
                        },
                        start: Some("startDate".to_string()),
                        end: Some("endDate".to_string()),
                        start_value: Some("2026-06-01".to_string()),
                        end_value: Some("2026-06-08".to_string()),
                        size: ButtonSize::Md,
                        name: None,
                        help_text: None,
                        error_text: None,
                        min: None,
                        max: None,
                    },
                },
                ViewNode::RadioGroup {
                    props: RadioGroupProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Muted),
                            label: Some("Plan".to_string()),
                            element: ElementProps {
                                bind: Some("choice".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        size: ButtonSize::Md,
                        name: Some("plan".to_string()),
                        info: None,
                        error: None,
                    },
                    options: vec![RadioOption {
                        value: "basic".to_string(),
                        label: "Basic".to_string(),
                        disabled: false,
                    }],
                },
                ViewNode::Toggle {
                    props: ToggleProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Secondary),
                            label: Some("Enabled".to_string()),
                            element: ElementProps {
                                bind: Some("accepted".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        checked: true,
                        disabled: false,
                        name: None,
                        label_left: Some("Off".to_string()),
                        label_right: Some("On".to_string()),
                    },
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn svg_route() -> ViewRoute {
    ViewRoute {
        id: "svg".to_string(),
        route_path: "/svg".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: svg_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn motion_route() -> ViewRoute {
    ViewRoute {
        id: "motion".to_string(),
        route_path: "/motion".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                animation: Some(ViewAnimation::FadeIn),
                ..Default::default()
            },
            children: vec![ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        animation: Some(ViewAnimation::SlideUp),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Motion")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn code_route() -> ViewRoute {
    ViewRoute {
        id: "code".to_string(),
        route_path: "/code".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::code_node(
            vec![
                ComponentProp {
                    name: "language".to_string(),
                    value: PropValue::String("dowe".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
            ],
            vec![
                "page docsPage".to_string(),
                "  Card variant:\"soft\" p:4 show:true".to_string(),
                "    Text".to_string(),
                "      Documentation".to_string(),
            ],
        )
        .expect("code"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn video_route() -> ViewRoute {
    ViewRoute {
        id: "video".to_string(),
        route_path: "/video".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::video_node(vec![
            ComponentProp {
                name: "src".to_string(),
                value: PropValue::String(
                    "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string(),
                ),
            },
            ComponentProp {
                name: "poster".to_string(),
                value: PropValue::String("/images/video.jpg".to_string()),
            },
            ComponentProp {
                name: "aspect".to_string(),
                value: PropValue::String("vertical".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
        ])
        .expect("video"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn candlestick_route() -> ViewRoute {
    ViewRoute {
        id: "market".to_string(),
        route_path: "/market".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::candlestick_node(vec![
            ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("candles".to_string()),
            },
            ComponentProp {
                name: "stream".to_string(),
                value: PropValue::String("/api/candles".to_string()),
            },
            ComponentProp {
                name: "variant".to_string(),
                value: PropValue::String("soft".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
            ComponentProp {
                name: "emptyLabel".to_string(),
                value: PropValue::String("Market closed".to_string()),
            },
        ])
        .expect("candlestick"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn table_route() -> ViewRoute {
    ViewRoute {
        id: "users".to_string(),
        route_path: "/users".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::table_node(
            vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("users".to_string()),
                },
                ComponentProp {
                    name: "variant".to_string(),
                    value: PropValue::String("soft".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
                ComponentProp {
                    name: "size".to_string(),
                    value: PropValue::String("lg".to_string()),
                },
                ComponentProp {
                    name: "striped".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "bordered".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "emptyTitle".to_string(),
                    value: PropValue::String("No users".to_string()),
                },
            ],
            vec![
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("name".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Name".to_string()),
                    },
                ])
                .expect("name column"),
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("status".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Status".to_string()),
                    },
                    ComponentProp {
                        name: "align".to_string(),
                        value: PropValue::String("end".to_string()),
                    },
                    ComponentProp {
                        name: "width".to_string(),
                        value: PropValue::String("8rem".to_string()),
                    },
                ])
                .expect("status column"),
            ],
        )
        .expect("table"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn divider_route() -> ViewRoute {
    ViewRoute {
        id: "divider".to_string(),
        route_path: "/divider".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Divider {
            props: DividerProps {
                style: StyleProps::default(),
                orientation: DividerOrientation::Vertical,
                color: ColorFamily::Primary,
            },
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn svg_tree() -> ViewNode {
    ViewNode::Svg {
        props: SvgProps {
            style: StyleProps {
                text: Some(ResponsiveValue::scalar(
                    dowe_components::ColorToken::Tertiary,
                )),
                sizing: dowe_components::SizingProps {
                    w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            view_box: SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
            },
            SvgPath {
                data: "M22 12c0-5.523-4.477-10-10-10".to_string(),
                fill: SvgPathFill::CurrentColor,
            },
        ],
    }
}

fn display_overlay_route() -> ViewRoute {
    ViewRoute {
        id: "overlay".to_string(),
        route_path: "/overlay".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: display_overlay_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn display_overlay_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Avatar {
                props: AvatarProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Ada".to_string()),
                    alt: "Ada Lovelace".to_string(),
                    size: ButtonSize::Lg,
                    status: Some(AvatarStatus::Online),
                    bordered: true,
                },
                icon: None,
            },
            ViewNode::Badge {
                props: BadgeProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    text: "3".to_string(),
                    position: OverlayCornerPosition::BottomRight,
                },
                children: vec![text("Inbox")],
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Info),
                        size: Some(ButtonSize::Sm),
                        ..Default::default()
                    },
                    on_close: Some("close".to_string()),
                },
                value: "Filter".to_string(),
                start: None,
                end: None,
            },
            ViewNode::Skeleton {
                props: SkeletonProps {
                    style: StyleProps::default(),
                    variant: SkeletonVariant::Rounded,
                    animation: SkeletonAnimation::Pulse,
                },
            },
            ViewNode::Modal {
                props: ModalProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    on_close: Some("close".to_string()),
                    disable_overlay_close: false,
                    hide_close_button: false,
                },
                header: vec![text("Settings")],
                body: vec![text("Body")],
                footer: vec![text("Footer")],
            },
            ViewNode::AlertDialog {
                props: AlertDialogProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    title: "Delete?".to_string(),
                    description: "Cannot undo.".to_string(),
                    confirm_text: "Delete".to_string(),
                    cancel_text: "Cancel".to_string(),
                    on_confirm: Some("confirm".to_string()),
                    on_cancel: Some("close".to_string()),
                    loading: false,
                },
            },
            ViewNode::Tooltip {
                props: TooltipProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    label: "More actions".to_string(),
                    position: OverlayPosition::End,
                },
                children: vec![text("Hover")],
            },
            ViewNode::Toast {
                props: ToastProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    source: None,
                    kind: ToastKind::Success,
                    title: Some("Saved".to_string()),
                    description: "Profile updated".to_string(),
                    position: OverlayCornerPosition::TopRight,
                    show_icon: true,
                },
            },
            ViewNode::Dropdown {
                props: DropdownProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                },
                trigger: vec![text("Menu")],
                header: Vec::new(),
                entries: vec![OverlayEntry::Item(OverlayItemProps {
                    label: "Profile".to_string(),
                    description: None,
                    icon: None,
                    on_click: Some("profile".to_string()),
                    navigation: None,
                    disabled: false,
                })],
                footer: Vec::new(),
            },
            ViewNode::Command {
                props: CommandProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    open: Some("modal01".to_string()),
                    placeholder: "Search".to_string(),
                    empty_text: "No results".to_string(),
                    close_text: "to close".to_string(),
                    navigate_text: "Navigate".to_string(),
                    select_text: "Select".to_string(),
                    toggle_text: "Toggle".to_string(),
                    shortcut: "p".to_string(),
                    disable_global_shortcut: false,
                    show_footer: true,
                },
                entries: vec![CommandEntry::Item(OverlayItemProps {
                    label: "Home".to_string(),
                    description: None,
                    icon: None,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                    disabled: false,
                })],
            },
        ],
    }
}

fn text(value: &str) -> ViewNode {
    ViewNode::Text {
        props: Default::default(),
        value: value.to_string(),
    }
}

fn translations() -> TranslationCatalog {
    TranslationCatalog {
        default_locale: Some("en".to_string()),
        locales: vec![
            TranslationLocale {
                locale: "en".to_string(),
                source_path: PathBuf::from("src/i18n/en.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe builds systems.".to_string(),
                }],
            },
            TranslationLocale {
                locale: "es".to_string(),
                source_path: PathBuf::from("src/i18n/es.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe construye sistemas.".to_string(),
                }],
            },
        ],
    }
}

fn bar_props(floating: bool) -> BarProps {
    BarProps {
        style: VariantProps {
            variant: Some(ComponentVariant::Solid),
            color: Some(ColorFamily::Surface),
            ..Default::default()
        },
        bordered: true,
        blurred: true,
        boxed: true,
        floating,
    }
}

fn responsive_scale(entries: &[(Breakpoint, u16)]) -> ResponsiveValue<ScaleValue> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: ScaleValue::from_half_steps(value * 2),
            })
            .collect(),
    )
}

fn responsive_bool(entries: &[(Breakpoint, bool)]) -> ResponsiveValue<bool> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: *value,
            })
            .collect(),
    )
}
