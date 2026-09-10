r#"@Composable
private fun doweDockingAppBarModifier(modifier: Modifier, scrollState: ScrollState, backgroundColor: Color): Modifier {
    val threshold = with(LocalDensity.current) { 100.dp.roundToPx() }
    val docked by remember(scrollState, threshold) { derivedStateOf { scrollState.value > threshold } }
    val progress by animateFloatAsState(
        targetValue = if (docked) 1f else 0f,
        animationSpec = tween(durationMillis = 300, easing = CubicBezierEasing(0.4f, 0f, 0.2f, 1f)),
        label = "Dowe AppBar docking"
    )
    val radius = DoweDesign.radius * (1f - progress)
    val shape = RoundedCornerShape(radius)
    return modifier
        .padding(horizontal = 16.dp * (1f - progress), vertical = 8.dp * (1f - progress))
        .clip(shape)
        .background(backgroundColor)
        .border(1.dp, DoweDesign.muted.copy(alpha = 1f - progress), shape)
        .drawBehind {
            val stroke = 1.dp.toPx()
            val y = size.height - stroke / 2f
            drawLine(DoweDesign.muted.copy(alpha = progress), Offset(0f, y), Offset(size.width, y), stroke)
        }
}

private fun doweButtonFamily(scheme: String): Color = when (scheme) {
    "background" -> DoweDesign.background
    "surface" -> DoweDesign.surface
    "secondary" -> DoweDesign.secondary
    "accent" -> DoweDesign.accent
    "muted" -> DoweDesign.muted
    "success" -> DoweDesign.success
    "info" -> DoweDesign.info
    "warning" -> DoweDesign.warning
    "danger" -> DoweDesign.danger
    else -> DoweDesign.primary
}

private fun doweButtonTextFamily(scheme: String): Color = when (scheme) {
    "background" -> DoweDesign.backgroundText
    "surface" -> DoweDesign.surfaceText
    "secondary" -> DoweDesign.secondaryText
    "accent" -> DoweDesign.accentText
    "muted" -> DoweDesign.mutedText
    "success" -> DoweDesign.successText
    "info" -> DoweDesign.infoText
    "warning" -> DoweDesign.warningText
    "danger" -> DoweDesign.dangerText
    else -> DoweDesign.primaryText
}

private fun doweButtonTitleFamily(scheme: String): Color = when (scheme) {
    "background" -> DoweDesign.backgroundTitle
    "surface" -> DoweDesign.surfaceTitle
    "secondary" -> DoweDesign.secondaryTitle
    "accent" -> DoweDesign.accentTitle
    "muted" -> DoweDesign.mutedTitle
    "success" -> DoweDesign.successTitle
    "info" -> DoweDesign.infoTitle
    "warning" -> DoweDesign.warningTitle
    "danger" -> DoweDesign.dangerTitle
    else -> DoweDesign.primaryTitle
}

private fun doweSideNavHeaderColor(scheme: String): Color = doweButtonContent("ghost", scheme)

private fun doweButtonContainer(variant: String, scheme: String): Color = when (variant) {
    "solid" -> doweButtonFamily(scheme)
    "outlined", "ghost" -> Color.Transparent
    else -> doweButtonFamily(scheme)
}

private fun doweButtonContent(variant: String, scheme: String): Color =
    if (variant == "solid") doweButtonTextFamily(scheme) else doweButtonFamily(scheme)

private fun doweCardContainer(variant: String, scheme: String): Color = when (variant) {
    "solid" -> doweButtonFamily(scheme)
    "outlined", "ghost" -> Color.Transparent
    else -> doweButtonFamily(scheme)
}

private fun doweCardContent(variant: String, scheme: String): Color = when (variant) {
    "solid" -> doweButtonTextFamily(scheme)
    "outlined", "ghost" -> doweButtonFamily(scheme)
    else -> doweButtonTextFamily(scheme)
}

private fun doweCardTitle(variant: String, scheme: String): Color = when (variant) {
    "solid" -> doweButtonTitleFamily(scheme)
    "outlined", "ghost" -> doweButtonFamily(scheme)
    else -> doweButtonTitleFamily(scheme)
}

private fun doweCardBorder(variant: String, scheme: String): Color? =
    if (variant == "outlined") doweButtonFamily(scheme) else null

private fun doweSideNavMetric(size: String, small: Int, medium: Int, large: Int): Float = when (size) {
    "sm" -> small.toFloat()
    "lg" -> large.toFloat()
    else -> medium.toFloat()
}

private fun doweButtonRadius(value: String): Dp = when (value) {
    "xs" -> 2.dp
    "sm" -> 4.dp
    "lg" -> 12.dp
    "xl" -> 16.dp
    "full" -> 9999.dp
    else -> DoweDesign.radius
}

private fun doweButtonHorizontalPadding(value: String): Dp = when (value) {
    "xs" -> 10.dp
    "sm" -> 12.dp
    "lg" -> 20.dp
    "xl" -> 24.dp
    else -> 16.dp
}

private fun doweButtonVerticalPadding(value: String): Dp = when (value) {
    "xs" -> 6.dp
    "sm" -> 8.dp
    "lg" -> 12.dp
    "xl" -> 14.dp
    else -> 10.dp
}

private fun doweButtonMinHeight(value: String): Dp = when (value) {
    "xs" -> 28.dp
    "sm" -> 32.dp
    "lg" -> 44.dp
    "xl" -> 48.dp
    else -> 40.dp
}

private enum class DoweJustify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
    Stretch,
    Normal,
    EndSafe,
    CenterSafe
}

private enum class DoweFlexDirection {
    Row,
    Column
}

private enum class DoweAlign {
    Start,
    End,
    EndSafe,
    Center,
    CenterSafe,
    Between,
    Around,
    Evenly,
    Stretch,
    Baseline,
    BaselineLast,
    Normal
}

private enum class DoweAnimationPreset {
    None,
    FadeIn,
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    ScaleIn
}

private enum class DoweGesturePreset {
    None,
    Lift,
    Press,
    Grow,
    Tilt
}

private fun dowePageMotionEnabled(context: android.content.Context): Boolean =
    try { android.provider.Settings.Global.getFloat(context.contentResolver, android.provider.Settings.Global.ANIMATOR_DURATION_SCALE, 1f) != 0f } catch (error: Exception) { true }

private enum class DoweTransitionPreset {
    None,
    Quick,
    Smooth,
    Spring
}

class DoweSectionRegistry {
    val positions = mutableStateMapOf<String, Int>()
}

private fun doweShadowAlpha(radius: Dp): Float = when {
    radius <= 2.dp -> 0.12f
    radius <= 12.dp -> 0.14f
    radius <= 24.dp -> 0.16f
    radius <= 44.dp -> 0.18f
    else -> 0.22f
}

private fun doweShadowOffset(radius: Dp): Dp = when {
    radius <= 2.dp -> 1.dp
    radius <= 12.dp -> 4.dp
    radius <= 24.dp -> 10.dp
    radius <= 44.dp -> 18.dp
    else -> 28.dp
}

private fun Modifier.doweShadow(radius: Dp, shape: Shape, color: Color, alpha: Float?): Modifier =
    if (radius <= 0.dp) {
        this
    } else {
        this.dropShadow(
            shape = shape,
            shadow = DoweDropShadow(
                radius = radius,
                color = color,
                spread = 0.dp,
                offset = DpOffset(0.dp, doweShadowOffset(radius)),
                alpha = alpha ?: doweShadowAlpha(radius)
            )
        )
    }

private fun Modifier.doweSection(registry: DoweSectionRegistry, id: String): Modifier =
    this.onGloballyPositioned {
        registry.positions[id] = it.positionInParent().y.toInt().coerceAtLeast(0)
    }

@Composable
private fun Modifier.doweAnimation(preset: DoweAnimationPreset): Modifier {
    val pageEntranceSuppressed = LocalDowePageEntranceSuppressed.current
    var active by remember(preset) { mutableStateOf(preset == DoweAnimationPreset.None) }
    LaunchedEffect(preset) {
        active = true
    }
    val alpha by animateFloatAsState(
        targetValue = if (pageEntranceSuppressed || preset == DoweAnimationPreset.None || active) 1f else 0f,
        animationSpec = if (pageEntranceSuppressed) snap() else tween(durationMillis = 220)
    )
    val progress by animateFloatAsState(
        targetValue = if (pageEntranceSuppressed || preset == DoweAnimationPreset.None || active) 1f else 0f,
        animationSpec = if (pageEntranceSuppressed) snap() else tween(durationMillis = 220)
    )
    return this.graphicsLayer {
        this.alpha = alpha
        when (preset) {
            DoweAnimationPreset.SlideUp -> translationY = (1f - progress) * 16f
            DoweAnimationPreset.SlideDown -> translationY = (progress - 1f) * 16f
            DoweAnimationPreset.SlideLeft -> translationX = (1f - progress) * 16f
            DoweAnimationPreset.SlideRight -> translationX = (progress - 1f) * 16f
            else -> Unit
        }
        if (preset == DoweAnimationPreset.ScaleIn) {
            val value = 0.96f + (0.04f * progress)
            scaleX = value
            scaleY = value
        }
    }
}

@Composable
"#
