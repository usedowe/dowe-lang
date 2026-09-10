r#"private fun Modifier.doweGesture(
    preset: DoweGesturePreset,
    transition: DoweTransitionPreset
): Modifier {
    var pressed by remember(preset) { mutableStateOf(false) }
    val motionEnabled = ValueAnimator.areAnimatorsEnabled()
    val target = if (pressed && motionEnabled) 1f else 0f
    val spec: FiniteAnimationSpec<Float> = when (transition) {
        DoweTransitionPreset.None -> snap()
        DoweTransitionPreset.Quick -> tween(durationMillis = 120)
        DoweTransitionPreset.Smooth -> tween(durationMillis = 220)
        DoweTransitionPreset.Spring -> spring(dampingRatio = 0.72f, stiffness = 600f)
    }
    val progress by animateFloatAsState(targetValue = target, animationSpec = spec)
    return this
        .pointerInput(preset, motionEnabled) {
            try {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent(PointerEventPass.Initial)
                        pressed = event.changes.any { change ->
                            change.pressed &&
                                change.position.x >= 0f &&
                                change.position.y >= 0f &&
                                change.position.x <= size.width.toFloat() &&
                                change.position.y <= size.height.toFloat()
                        }
                    }
                }
            } finally {
                pressed = false
            }
        }
        .graphicsLayer {
            when (preset) {
                DoweGesturePreset.Lift -> {
                    translationY = -4f * progress
                    scaleX = 1f - 0.02f * progress
                    scaleY = scaleX
                }
                DoweGesturePreset.Press -> {
                    scaleX = 1f - 0.06f * progress
                    scaleY = scaleX
                }
                DoweGesturePreset.Grow -> {
                    scaleX = 1f + 0.04f * progress
                    scaleY = scaleX
                }
                DoweGesturePreset.Tilt -> rotationZ = 3f * progress
                DoweGesturePreset.None -> Unit
            }
        }
}

private enum class DoweFont {
    System,
    Inter,
    Roboto,
    Montserrat,
    Lato,
    Poppins,
    Manrope,
    Quicksand,
    Lora
}

private object DoweFonts {
    val inter = FontFamily(
        Font(R.font.inter_light, FontWeight.Light),
        Font(R.font.inter_regular, FontWeight.Normal),
        Font(R.font.inter_medium, FontWeight.Medium),
        Font(R.font.inter_semibold, FontWeight.SemiBold),
        Font(R.font.inter_bold, FontWeight.Bold),
        Font(R.font.inter_extrabold, FontWeight.ExtraBold)
    )
    val roboto = FontFamily(
        Font(R.font.roboto_light, FontWeight.Light),
        Font(R.font.roboto_regular, FontWeight.Normal),
        Font(R.font.roboto_medium, FontWeight.Medium),
        Font(R.font.roboto_semibold, FontWeight.SemiBold),
        Font(R.font.roboto_bold, FontWeight.Bold),
        Font(R.font.roboto_extrabold, FontWeight.ExtraBold)
    )
    val montserrat = FontFamily(
        Font(R.font.montserrat_light, FontWeight.Light),
        Font(R.font.montserrat_regular, FontWeight.Normal),
        Font(R.font.montserrat_medium, FontWeight.Medium),
        Font(R.font.montserrat_semibold, FontWeight.SemiBold),
        Font(R.font.montserrat_bold, FontWeight.Bold),
        Font(R.font.montserrat_extrabold, FontWeight.ExtraBold)
    )
    val lato = FontFamily(
        Font(R.font.lato_light, FontWeight.Light),
        Font(R.font.lato_regular, FontWeight.Normal),
        Font(R.font.lato_medium, FontWeight.Medium),
        Font(R.font.lato_semibold, FontWeight.SemiBold),
        Font(R.font.lato_bold, FontWeight.Bold),
        Font(R.font.lato_extrabold, FontWeight.ExtraBold)
    )
    val poppins = FontFamily(
        Font(R.font.poppins_light, FontWeight.Light),
        Font(R.font.poppins_regular, FontWeight.Normal),
        Font(R.font.poppins_medium, FontWeight.Medium),
        Font(R.font.poppins_semibold, FontWeight.SemiBold),
        Font(R.font.poppins_bold, FontWeight.Bold),
        Font(R.font.poppins_extrabold, FontWeight.ExtraBold)
    )
    val manrope = FontFamily(
        Font(R.font.manrope_light, FontWeight.Light),
        Font(R.font.manrope_regular, FontWeight.Normal),
        Font(R.font.manrope_medium, FontWeight.Medium),
        Font(R.font.manrope_semibold, FontWeight.SemiBold),
        Font(R.font.manrope_bold, FontWeight.Bold),
        Font(R.font.manrope_extrabold, FontWeight.ExtraBold)
    )
    val quicksand = FontFamily(
        Font(R.font.quicksand_light, FontWeight.Light),
        Font(R.font.quicksand_regular, FontWeight.Normal),
        Font(R.font.quicksand_medium, FontWeight.Medium),
        Font(R.font.quicksand_semibold, FontWeight.SemiBold),
        Font(R.font.quicksand_bold, FontWeight.Bold),
        Font(R.font.quicksand_extrabold, FontWeight.ExtraBold)
    )
    val lora = FontFamily(
        Font(R.font.lora_light, FontWeight.Light),
        Font(R.font.lora_regular, FontWeight.Normal),
        Font(R.font.lora_medium, FontWeight.Medium),
        Font(R.font.lora_semibold, FontWeight.SemiBold),
        Font(R.font.lora_bold, FontWeight.Bold),
        Font(R.font.lora_extrabold, FontWeight.ExtraBold)
    )
    val syne = FontFamily(
        Font(R.font.syne_variable, FontWeight.Light),
        Font(R.font.syne_variable, FontWeight.Normal),
        Font(R.font.syne_variable, FontWeight.Medium),
        Font(R.font.syne_variable, FontWeight.SemiBold),
        Font(R.font.syne_variable, FontWeight.Bold),
        Font(R.font.syne_variable, FontWeight.ExtraBold)
    )
    val jost = FontFamily(
        Font(R.font.jost_variable, FontWeight.Light),
        Font(R.font.jost_variable, FontWeight.Normal),
        Font(R.font.jost_variable, FontWeight.Medium),
        Font(R.font.jost_variable, FontWeight.SemiBold),
        Font(R.font.jost_variable, FontWeight.Bold),
        Font(R.font.jost_variable, FontWeight.ExtraBold)
    )
    val puritan = FontFamily(
        Font(R.font.puritan_regular, FontWeight.Light),
        Font(R.font.puritan_regular, FontWeight.Normal),
        Font(R.font.puritan_regular, FontWeight.Medium),
        Font(R.font.puritan_bold, FontWeight.SemiBold),
        Font(R.font.puritan_bold, FontWeight.Bold),
        Font(R.font.puritan_bold, FontWeight.ExtraBold)
    )
}

private sealed class DoweSize {
    data class Fixed(val value: Dp) : DoweSize()
    data class Percent(val fraction: Float) : DoweSize()
    data class ViewportMinus(val inset: Dp) : DoweSize()
    object Full : DoweSize()
    object Auto : DoweSize()
}

private sealed class DoweOverlay {
    data class Solid(val color: Color) : DoweOverlay()
    data class Gradient(val start: Color, val end: Color) : DoweOverlay()
}

private enum class DoweSectionBackground {
    Aurora,
    Sunrise,
    Ocean,
    Meadow,
    Slate
}


private data class DoweSvgViewBox(val minX: Float, val minY: Float, val width: Float, val height: Float)

private sealed class DoweSvgFill {
    object None : DoweSvgFill()
    object CurrentColor : DoweSvgFill()
    data class Solid(val color: Color) : DoweSvgFill()
    data class Fill(val color: Color?, val opacity: Float, val evenOdd: Boolean) : DoweSvgFill()
    data class Stroke(val color: Color?, val opacity: Float, val width: Float, val cap: String, val join: String) : DoweSvgFill()
}

private data class DoweSvgTransform(val a: Float, val b: Float, val c: Float, val d: Float, val e: Float, val f: Float)

private data class DoweSvgPath(val data: String, val fill: DoweSvgFill, val transform: DoweSvgTransform? = null)

private data class DoweCodeToken(val text: String, val color: Color)

private data class DoweCandlestickCandle(
    val id: String,
    val time: String,
    val open: Float,
    val high: Float,
    val low: Float,
    val close: Float
)

private enum class DoweTableColumnAlign {
    Start,
    Center,
    End
}

private enum class DoweTableSize {
    Sm,
    Md,
    Lg
}

private data class DoweTableColumn(val field: String, val label: String, val align: DoweTableColumnAlign, val width: String?)

"#
