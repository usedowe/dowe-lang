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
        "Column(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).zIndex(1f).padding(horizontal = 16.dp, vertical = 8.dp).clip(RoundedCornerShape(DoweDesign.radius)).background(DoweDesign.surface).border(1.dp, DoweDesign.muted, RoundedCornerShape(DoweDesign.radius)))"
    ));
    assert!(
        views.content.contains(
            "Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center)"
        )
    );
    assert!(views.content.contains(
        "Box(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).background(DoweDesign.surface).border(1.dp, DoweDesign.muted, RoundedCornerShape(0.dp)), contentAlignment = Alignment.BottomCenter)"
    ));
    assert_eq!(
        views
            .content
            .matches("Modifier.widthIn(max = 1536.dp).fillMaxWidth()")
            .count(),
        3
    );
    assert!(
        views
            .content
            .contains("CompositionLocalProvider(LocalContentColor provides DoweDesign.surfaceText)")
    );
    assert!(
        views
            .content
            .contains("horizontalArrangement = Arrangement.Center")
    );
    assert!(views.content.contains("Text(\"Brand\""));
    assert!(views.content.contains("Text(\"Directory\""));
    assert!(views.content.contains("Text(\"Copyright\""));
    let directory = views
        .content
        .find("Text(\"Directory\"")
        .expect("Footer top");
    let footer_inner = views.content[..directory]
        .rfind("Column(modifier = Modifier.widthIn(max = 1536.dp).fillMaxWidth())")
        .expect("Footer boxed regions");
    let footer_provider = views.content[..footer_inner]
        .rfind("CompositionLocalProvider(LocalContentColor provides DoweDesign.surfaceText)")
        .expect("Footer content color");
    let copyright = views.content[directory..]
        .find("Text(\"Copyright\"")
        .map(|index| directory + index)
        .expect("Footer bottom");
    assert!(footer_provider < footer_inner);
    assert!(footer_inner < directory);
    assert!(directory < copyright);
    assert!(views.content.contains("itemSize = 56.dp"));
    assert!(views.content.contains("featured = true"));
    assert!(
        views
            .content
            .contains("backgroundColor = DoweDesign.primary")
    );

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("doweBackground(DOWE_SURFACE, DOWE_RADIUS)")
    );
    assert!(!dev.content.contains("setElevation(doweDp(4))"));
    assert!(dev.content.contains("doweBackground(DOWE_PRIMARY, 999f)"));
    assert_eq!(
        dev.content
            .matches("doweBoxedContainer(true, 1536)")
            .count(),
        2
    );
    assert_eq!(
        dev.content
            .matches("doweBoxedContainer(false, 1536)")
            .count(),
        1
    );
    assert!(dev.content.contains("dowePinAppBar("));
    assert!(
        dev.content
            .contains("for (int index = 0; index < appBar.getChildCount(); index++)")
    );
    assert!(dev.content.contains(
        "child.measure(childWidthSpec, View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED))"
    ));
    assert!(
        dev.content
            .contains("appBarHeight = Math.max(appBarHeight, child.getMeasuredHeight())")
    );
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, appBarHeight, Gravity.TOP | Gravity.START)"
    ));
    let pin_offset = dev
        .content
        .find("\n        dowePinAppBar(")
        .map(|index| index + 1)
        .expect("persistent AppBar pin");
    let boxed_content = dev.content[..pin_offset]
        .rfind("doweBoxedContainer(true, 1536)")
        .expect("boxed AppBar content");
    let boxed_content_line_start = dev.content[..boxed_content]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let boxed_content_line = dev.content[boxed_content_line_start..]
        .lines()
        .next()
        .expect("boxed AppBar content line");
    let boxed_content_view = boxed_content_line
        .split_whitespace()
        .nth(1)
        .expect("boxed AppBar content view");
    let before_pin = &dev.content[boxed_content..pin_offset];
    assert!(before_pin.contains(&format!(", {boxed_content_view});")));
    let pin_line = dev.content[pin_offset..]
        .lines()
        .next()
        .expect("persistent AppBar pin line");
    assert!(!pin_line.contains(&format!(", {boxed_content_view})")));
    assert!(dev.content.contains("background.addView(appBar, params)"));
    assert!(
        dev.content
            .contains("safeArea.setBackgroundColor(doweSafeAreaTopColor)")
    );
    assert!(dev.content.contains("dowe-pinned-appbar-safe-area"));
    assert!(
        dev.content
            .contains("bottomSafeArea.setBackgroundColor(doweSafeAreaBottomColor)")
    );
    assert!(dev.content.contains("dowe-pinned-appbar-bottom-safe-area"));
    assert!(dev.content.contains("doweRelayoutPinnedAppBar();"));
    assert!(
        dev.content
            .contains("scrollView.post(this::doweRelayoutPinnedAppBar)")
    );
    assert!(dev.content.contains("background.getRootWindowInsets()"));
    assert!(
        dev.content
            .contains("appBarParams.setMargins(leftInset, topInset, rightInset, 0)")
    );
    assert!(dev.content.contains("safeAreaParams.height = topInset"));
    assert!(
        dev.content
            .contains("bottomSafeAreaParams.height = bottomInset")
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
    assert!(dev.content.contains("doweText(\"Directory\""));
    assert!(dev.content.contains("doweText(\"Copyright\""));
    let dev_directory = dev
        .content
        .find("doweText(\"Directory\"")
        .expect("Footer top");
    let dev_footer_inner = dev.content[..dev_directory]
        .rfind("doweBoxedContainer(false, 1536)")
        .expect("Footer boxed regions");
    let dev_copyright = dev.content[dev_directory..]
        .find("doweText(\"Copyright\"")
        .map(|index| dev_directory + index)
        .expect("Footer bottom");
    assert!(dev_footer_inner < dev_directory);
    assert!(dev_directory < dev_copyright);
}

#[test]
fn generates_scroll_docking_appbar_for_compose_and_dev_launcher() {
    let output = generate_android(
        &[docking_appbar_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("doweDockingAppBarModifier("));
    assert!(views.content.contains("scrollState.value > threshold"));
    assert!(views.content.contains("100.dp.roundToPx()"));
    assert!(
        views
            .content
            .contains("CubicBezierEasing(0.4f, 0f, 0.2f, 1f)")
    );
    assert!(views.content.contains("durationMillis = 300"));
    assert!(views.content.contains("16.dp * (1f - progress)"));
    assert!(views.content.contains("8.dp * (1f - progress)"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("dowePinAppBar("));
    assert!(dev.content.contains(", true,"));
    assert!(dev.content.contains("scrollY > doweDp(100)"));
    assert!(dev.content.contains("ValueAnimator.areAnimatorsEnabled()"));
    assert!(dev.content.contains("setDuration(300)"));
    assert!(
        dev.content
            .contains("new PathInterpolator(0.4f, 0f, 0.2f, 1f)")
    );
}

#[test]
fn keeps_drawer_header_visually_flat() {
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

    assert!(views.content.contains("Column(modifier = Modifier.fillMaxWidth().background(DoweDesign.surface))"));
    assert!(!views.content.contains("Column(modifier = Modifier.fillMaxWidth().zIndex(100f).background(DoweDesign.surface))"));
}

#[test]
fn keeps_unbordered_persistent_appbar_visually_flat() {
    let output = generate_android(
        &[unbordered_persistent_appbar_route()],
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
        "Column(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).zIndex(1f).background(DoweDesign.surface))"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("dowePinAppBar("));
    assert!(!dev.content.contains("setElevation(doweDp(4))"));
}


