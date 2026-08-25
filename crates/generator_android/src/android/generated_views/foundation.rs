fn android_runtime_foundation() -> &'static str {
    r#"package dev.dowe.generated

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.Image
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.gestures.ScrollableDefaults
import androidx.compose.foundation.gestures.snapping.rememberSnapFlingBehavior
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.shape.CircleShape
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import android.app.Activity
import android.Manifest
import android.animation.ValueAnimator
import android.app.PictureInPictureParams
import android.content.ContextWrapper
import android.content.Intent
import android.graphics.Color as AndroidColor
import android.media.MediaPlayer
import android.media.MediaRecorder
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.net.Uri
import android.provider.OpenableColumns
import android.provider.MediaStore
import android.util.Rational
import android.widget.ImageView
import android.widget.FrameLayout
import android.widget.VideoView
import android.view.ViewGroup
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import java.time.LocalDate
import java.time.YearMonth
import java.io.File
import java.io.FileOutputStream
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlin.math.atan2
import kotlin.math.pow
import kotlin.math.roundToInt
import kotlin.math.sqrt
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.TextButton
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.key
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.zIndex
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.layout
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.foundation.focusable
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.shadow.Shadow as DoweDropShadow
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.PointerType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.changedToDownIgnoreConsumed
import androidx.compose.ui.input.pointer.changedToUpIgnoreConsumed

import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.res.stringResource
import androidx.compose.material3.LocalContentColor
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.constrainHeight
import androidx.compose.ui.unit.constrainWidth
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import android.graphics.BitmapFactory
import android.util.Base64
import android.graphics.Paint
import android.hardware.Sensor
import android.hardware.SensorEvent
import android.hardware.SensorEventListener
import android.hardware.SensorManager
import android.os.SystemClock
import android.util.LruCache
import android.view.Surface
import android.view.WindowManager
import java.io.ByteArrayOutputStream
import java.time.Instant
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import java.util.concurrent.ConcurrentHashMap
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject

__DOWE_DESIGN__

val LocalDoweTitleColor = compositionLocalOf { DoweDesign.backgroundTitle }

@Composable
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
    "outlined" -> if (scheme == "background") DoweDesign.background else DoweDesign.surface
    "ghost" -> Color.Transparent
    else -> doweButtonFamily(scheme)
}

private fun doweCardContent(variant: String, scheme: String): Color = when (variant) {
    "solid" -> doweButtonTextFamily(scheme)
    "solid" -> doweButtonTextFamily(scheme)
    "outlined" -> if (scheme == "background") DoweDesign.backgroundText else DoweDesign.surfaceText
    "ghost" -> if (scheme == "background" || scheme == "surface") doweButtonTextFamily(scheme) else doweButtonFamily(scheme)
    else -> doweButtonTextFamily(scheme)
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
    var active by remember(preset) { mutableStateOf(preset == DoweAnimationPreset.None) }
    LaunchedEffect(preset) {
        active = true
    }
    val alpha by animateFloatAsState(
        targetValue = if (preset == DoweAnimationPreset.None || active) 1f else 0f,
        animationSpec = tween(durationMillis = 220)
    )
    val progress by animateFloatAsState(
        targetValue = if (preset == DoweAnimationPreset.None || active) 1f else 0f,
        animationSpec = tween(durationMillis = 220)
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
private fun Modifier.doweGesture(
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
}
