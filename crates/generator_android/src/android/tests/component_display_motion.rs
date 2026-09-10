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
    assert!(
        views
            .content
            .contains("rotationZ = doweResponsive(viewportWidth, xs = -7f) ?: 0f")
    );
    assert!(
        views
            .content
            .contains("scaleX = doweResponsive(viewportWidth, xs = 1.05f) ?: 1f")
    );
    assert!(
        views
            .content
            .contains("translationX = (doweResponsive(viewportWidth, xs = -6.dp) ?: 0.dp).toPx()")
    );
    assert!(
        views
            .content
            .contains(".doweGesture(DoweGesturePreset.Lift, DoweTransitionPreset.Spring)")
    );
    assert!(
        views
            .content
            .contains("ValueAnimator.areAnimatorsEnabled()")
    );
    assert!(views.content.contains("scaleX = 1f - 0.06f * progress"));
    assert!(
        views
            .content
            .contains("awaitPointerEvent(PointerEventPass.Initial)")
    );
    assert!(
        views
            .content
            .contains("awaitPointerEventScope {\n                while (true)")
    );
    assert!(
        views
            .content
            .contains("pressed = event.changes.any { change ->")
    );
    assert!(
        views
            .content
            .contains("change.position.x <= size.width.toFloat()")
    );
    assert!(
        views
            .content
            .contains("change.position.y <= size.height.toFloat()")
    );
    assert!(
        views
            .content
            .contains("finally {\n                pressed = false")
    );
    assert!(
        views
            .content
            .contains("var routeRevision by remember { mutableIntStateOf(0) }")
    );
    assert!(
        views
            .content
            .contains("if (operation == \"replace\") {\n                pageEntranceSuppressed = false\n                routeRevision += 1")
    );
    assert!(
        views
            .content
            .contains("val pageMotionEnabled = dowePageMotionEnabled(context)")
    );
    assert!(views.content.contains("var pageEntranceSuppressed by remember { mutableStateOf(false) }"));
    assert!(views.content.contains("var pageTransitionSequence by remember { mutableIntStateOf(0) }"));
    assert!(views.content.contains("pageEntranceSuppressed = false\n                routeRevision += 1"));
    assert!(views.content.contains("AnimatedContent("));
    assert!(views.content.contains("targetState = currentEntry.path"));
    assert!(views.content.contains("fadeIn(animationSpec = tween(durationMillis = 280, easing = CubicBezierEasing(0.22f, 0.61f, 0.36f, 1f))) togetherWith ExitTransition.None"));
    assert!(views.content.contains("dowe-page-transition"));
    assert!(views.content.contains("CompositionLocalProvider(LocalDowePageEntranceSuppressed provides pageEntranceSuppressed)"));
    assert!(views.content.contains("val pageEntranceSuppressed = LocalDowePageEntranceSuppressed.current"));
    assert!(views.content.contains("targetValue = if (pageEntranceSuppressed || preset == DoweAnimationPreset.None || active) 1f else 0f"));
    assert!(views.content.contains("key(path, routeRevision)"));
    assert!(!views.content.contains("fadeIn(animationSpec = tween(220)) togetherWith fadeOut(animationSpec = tween(160))"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("baseScaleX * 0.94f"));
    assert!(
        dev.content
            .contains("stateAnimator.addState(new int[]{android.R.attr.state_pressed}")
    );
    assert!(
        dev.content
            .contains("view.setStateListAnimator(stateAnimator);")
    );
    assert!(
        dev.content
            .contains("DOWE_GESTURE_ANIMATORS.put(view, stateAnimator);")
    );
    assert!(
        dev.content
            .contains("if (!DOWE_GESTURE_ANIMATORS.containsKey(view))")
    );
    assert!(dev.content.contains(r#"doweAnimate(view0, "fadeIn");"#));
    assert!(dev.content.contains(r#"doweAnimate(view1, "slideUp");"#));
    assert!(
        dev.content
            .contains("view1.setRotation(doweResponsiveFloat(viewportWidth")
    );
    assert!(
        dev.content
            .contains(r#"doweGesture(view1, "lift", "spring");"#)
    );
    assert!(dev.content.contains("renderCurrentRoute();"));
}

