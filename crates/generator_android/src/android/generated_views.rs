fn generated_views(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
) -> String {
    let mut output = String::from(
        r#"package dev.dowe.generated

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.activity.compose.BackHandler
import android.content.Intent
import android.net.Uri
import android.widget.ImageView
import android.widget.MediaController
import android.widget.VideoView
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.RadioButton
import androidx.compose.material3.RadioButtonDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalDensity
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
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import android.webkit.WebView
import java.time.Instant
import java.net.HttpURLConnection
import java.net.URL
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject

__DOWE_DESIGN__

private enum class DoweJustify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly
}

private enum class DoweAlign {
    Start,
    Center,
    End,
    Stretch,
    Baseline
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

class DoweSectionRegistry {
    val positions = mutableStateMapOf<String, Int>()
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
}

private sealed class DoweSize {
    data class Fixed(val value: Dp) : DoweSize()
    object Full : DoweSize()
}

private sealed class DoweOverlay {
    data class Solid(val color: Color) : DoweOverlay()
    data class Gradient(val start: Color, val end: Color) : DoweOverlay()
}

private enum class DoweSectionBackground {
    Soft,
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
}

private data class DoweSvgPath(val data: String, val fill: DoweSvgFill)

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

@Composable
private fun DoweVideo(source: String, poster: String?, autoplay: Boolean, aspect: String, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, borderColor: Color?) {
    var video by remember(source) { mutableStateOf<VideoView?>(null) }
    var started by remember(source) { mutableStateOf(false) }
    Box(modifier = modifier.aspectRatio(doweVideoAspect(aspect)).clip(shape).background(backgroundColor).then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))) {
        AndroidView(
            modifier = Modifier.matchParentSize(),
            factory = { context ->
                VideoView(context).apply {
                    val controller = MediaController(context)
                    controller.setAnchorView(this)
                    setMediaController(controller)
                    tag = source
                    setVideoURI(Uri.parse(source))
                    setOnPreparedListener {
                        if (autoplay) {
                            started = true
                            start()
                        }
                    }
                    video = this
                }
            },
            update = { view ->
                if (view.tag != source) {
                    view.tag = source
                    view.setVideoURI(Uri.parse(source))
                }
            }
        )
        if (poster != null && !started) {
            DoweCoverBox(modifier = Modifier.matchParentSize().clickable {
                started = true
                video?.start()
            }, source = poster, overlay = null) {}
        }
    }
}

private fun doweVideoAspect(value: String): Float {
    return when (value) {
        "vertical" -> 9f / 16f
        "square" -> 1f
        else -> 16f / 9f
    }
}

@Composable
private fun DoweAudio(source: String, subtitle: String?, avatarSource: String?, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var playing by remember(source) { mutableStateOf(false) }
    Row(
        modifier = modifier
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Button(
            onClick = { playing = !playing },
            colors = ButtonDefaults.buttonColors(containerColor = contentColor.copy(alpha = 0.12f), contentColor = contentColor),
            contentPadding = PaddingValues(horizontal = 10.dp, vertical = 6.dp)
        ) {
            Text(if (playing) "Pause" else "Play")
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(text = subtitle ?: source, color = contentColor, maxLines = 1)
            Row(horizontalArrangement = Arrangement.spacedBy(3.dp)) {
                repeat(24) { index ->
                    Box(
                        modifier = Modifier
                            .width(3.dp)
                            .height(((index % 7) + 4).dp)
                            .background(contentColor.copy(alpha = if (playing) 0.9f else 0.35f), RoundedCornerShape(2.dp))
                    )
                }
            }
        }
        if (avatarSource != null) {
            DoweCoverBox(modifier = Modifier.width(36.dp).height(36.dp).clip(RoundedCornerShape(999.dp)), source = avatarSource, overlay = null) {}
        }
    }
}

@Composable
private fun DoweImage(source: String, alt: String, aspect: String, objectFit: String, loading: String, hideControls: Boolean, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Box(
        modifier = modifier
            .aspectRatio(doweImageAspect(aspect))
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
    ) {
        DoweCoverBox(modifier = Modifier.matchParentSize(), source = source, overlay = null) {}
        if (!hideControls && alt.isNotEmpty()) {
            Text(
                text = alt,
                modifier = Modifier.align(Alignment.BottomStart).background(backgroundColor.copy(alpha = 0.72f)).padding(8.dp),
                color = contentColor,
                maxLines = 1
            )
        }
    }
}

private fun doweImageAspect(value: String): Float {
    return when (value) {
        "vertical" -> 9f / 16f
        "square" -> 1f
        "auto" -> 16f / 9f
        else -> 16f / 9f
    }
}

@Composable
private fun DoweAccordion(multiple: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?, content: @Composable () -> Unit) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(12.dp))),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            content()
        }
    }
}

@Composable
private fun DoweAccordionItem(id: String, label: String, disabled: Boolean, defaultOpen: Boolean, content: @Composable () -> Unit) {
    var open by remember(id) { mutableStateOf(defaultOpen) }
    Column(modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).border(1.dp, LocalContentColor.current.copy(alpha = 0.12f), RoundedCornerShape(10.dp))) {
        Row(
            modifier = Modifier.fillMaxWidth().clickable(enabled = !disabled) { open = !open }.padding(horizontal = 14.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(label, fontWeight = FontWeight.SemiBold)
            Text(if (open) "^" else "v")
        }
        AnimatedVisibility(visible = open, enter = fadeIn() + expandVertically(), exit = fadeOut() + shrinkVertically()) {
            Column(modifier = Modifier.fillMaxWidth().padding(14.dp)) {
                content()
            }
        }
    }
}

@Composable
private fun DoweCarousel(autoplay: Boolean, autoplayInterval: Int, disableLoop: Boolean, hideControls: Boolean, hideIndicators: Boolean, showNavigation: Boolean, showCounter: Boolean, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, modifier: Modifier, accentColor: Color, content: @Composable () -> Unit) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        if (title != null) {
            Text(title, fontWeight = FontWeight.Bold, color = accentColor)
        }
        Row(
            modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(gap.dp)
        ) {
            content()
        }
    }
}

@Composable
private fun DoweCarouselSlide(id: String, content: @Composable () -> Unit) {
    Box(modifier = Modifier.widthIn(min = 220.dp)) {
        content()
    }
}

@Composable
private fun DoweCheckbox(checked: Boolean, onCheckedChange: (Boolean) -> Unit, enabled: Boolean, label: String?, name: String?, modifier: Modifier, accentColor: Color) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Checkbox(
            checked = checked,
            onCheckedChange = onCheckedChange,
            enabled = enabled,
            colors = CheckboxDefaults.colors(checkedColor = accentColor, uncheckedColor = accentColor.copy(alpha = 0.72f), checkmarkColor = Color.White)
        )
        if (label != null) {
            Text(label, color = accentColor)
        }
    }
}

@Composable
private fun DoweColorField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, name: String?, helpText: String?, errorText: String?, showHex: Boolean, showRgb: Boolean, showCmyk: Boolean, showOklch: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = doweControlHeight(size))
                .clip(RoundedCornerShape(10.dp))
                .background(backgroundColor)
                .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)))
                .padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Box(
                modifier = Modifier
                    .width(doweControlSwatchSize(size))
                    .height(doweControlSwatchSize(size))
                    .clip(RoundedCornerShape(6.dp))
                    .background(doweHexColor(value, backgroundColor))
                    .border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(6.dp))
            )
            Text(text = value.ifEmpty { placeholder }.uppercase(), color = contentColor, fontSize = 14.sp, maxLines = 1)
        }
        if (showHex || showRgb || showCmyk || showOklch) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                if (showHex) Text("hex: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
                if (showRgb) Text("rgb: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
                if (showCmyk) Text("cmyk: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
                if (showOklch) Text("oklch: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweDateField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        DoweInput(value = value, onValueChange = onValueChange, modifier = Modifier.fillMaxWidth(), label = label, placeholder = placeholder, floating = floating, fontFamily = FontFamily.Default, fontSize = 14.sp, lineHeight = 20.sp, minHeight = doweControlHeight(size), horizontalPadding = 12.dp, shape = RoundedCornerShape(10.dp), backgroundColor = backgroundColor, contentColor = contentColor, borderColor = borderColor)
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweDateRangeField(startValue: String, endValue: String, onStartChange: (String) -> Unit, onEndChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = doweControlHeight(size))
                .clip(RoundedCornerShape(10.dp))
                .background(backgroundColor)
                .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)))
                .padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            BasicTextField(
                value = startValue,
                onValueChange = onStartChange,
                modifier = Modifier.weight(1f),
                singleLine = true,
                textStyle = TextStyle(fontFamily = FontFamily.Default, fontSize = 14.sp, lineHeight = 20.sp, color = contentColor),
                decorationBox = { innerTextField ->
                    Box {
                        if (startValue.isEmpty()) Text("Start", color = contentColor.copy(alpha = 0.55f), fontSize = 14.sp)
                        innerTextField()
                    }
                }
            )
            Text("-", color = contentColor.copy(alpha = 0.64f))
            BasicTextField(
                value = endValue,
                onValueChange = onEndChange,
                modifier = Modifier.weight(1f),
                singleLine = true,
                textStyle = TextStyle(fontFamily = FontFamily.Default, fontSize = 14.sp, lineHeight = 20.sp, color = contentColor),
                decorationBox = { innerTextField ->
                    Box {
                        if (endValue.isEmpty()) Text("End", color = contentColor.copy(alpha = 0.55f), fontSize = 14.sp)
                        innerTextField()
                    }
                }
            )
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

private data class DoweRadioOption(val value: String, val label: String, val disabled: Boolean)

@Composable
private fun DoweRadioGroup(value: String, onValueChange: (String) -> Unit, options: List<DoweRadioOption>, size: String, name: String?, label: String?, helpText: String?, errorText: String?, modifier: Modifier, accentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) {
            Text(label, fontWeight = FontWeight.SemiBold, color = accentColor)
        }
        options.forEach { option ->
            Row(modifier = Modifier.clickable(enabled = !option.disabled) { onValueChange(option.value) }, verticalAlignment = Alignment.CenterVertically) {
                RadioButton(
                    selected = value == option.value,
                    onClick = { onValueChange(option.value) },
                    enabled = !option.disabled,
                    colors = RadioButtonDefaults.colors(selectedColor = accentColor, unselectedColor = accentColor.copy(alpha = 0.72f))
                )
                Text(option.label, color = accentColor)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = accentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweToggle(checked: Boolean, onCheckedChange: (Boolean) -> Unit, enabled: Boolean, label: String?, labelLeft: String?, labelRight: String?, name: String?, modifier: Modifier, accentColor: Color) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        if (labelLeft != null) {
            Text(labelLeft, color = accentColor.copy(alpha = if (checked) 0.45f else 1f))
        }
        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
            enabled = enabled,
            colors = SwitchDefaults.colors(checkedThumbColor = Color.White, checkedTrackColor = accentColor, uncheckedThumbColor = Color.White, uncheckedTrackColor = accentColor.copy(alpha = 0.28f))
        )
        if (labelRight != null) {
            Text(labelRight, color = accentColor.copy(alpha = if (checked) 1f else 0.45f))
        }
        if (label != null) {
            Text(label, color = accentColor)
        }
    }
}

@Composable
private fun DoweThemeToggle(modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    var dark by remember {
        mutableStateOf(context.getSharedPreferences("dowe", 0).getString("theme-preference", "light") == "dark")
    }
    Button(
        modifier = modifier.defaultMinSize(minWidth = 0.dp, minHeight = 0.dp),
        colors = ButtonDefaults.buttonColors(containerColor = backgroundColor, contentColor = contentColor),
        border = borderColor?.let { BorderStroke(1.dp, it) },
        contentPadding = PaddingValues(0.dp),
        onClick = {
            dark = !dark
            context.getSharedPreferences("dowe", 0).edit().putString("theme-preference", if (dark) "dark" else "light").apply()
        }
    ) {
        Text(if (dark) "sun" else "moon", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
private fun DoweSliderField(value: Float, onValueChange: (Float) -> Unit, bound: Boolean, label: String?, hideLabel: Boolean, min: Float, max: Float, size: String, modifier: Modifier, accentColor: Color) {
    var local by remember(value, bound) { mutableStateOf(value) }
    val current = if (bound) value else local
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(2.dp)) {
        if (!hideLabel) {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text(label.orEmpty(), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = accentColor)
                Text(current.toInt().toString(), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = accentColor)
            }
        }
        Slider(
            value = current.coerceIn(min, max),
            onValueChange = {
                if (bound) onValueChange(it) else local = it
            },
            valueRange = min..max,
            colors = SliderDefaults.colors(thumbColor = accentColor, activeTrackColor = accentColor)
        )
    }
}

@Composable
private fun DoweDropzone(label: String?, placeholder: String, helpText: String?, errorText: String?, size: String, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val height = when (size) {
        "sm" -> 128.dp
        "lg" -> 256.dp
        else -> 192.dp
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(height)
                .clip(RoundedCornerShape(12.dp))
                .background(backgroundColor)
                .border(2.dp, borderColor ?: contentColor.copy(alpha = 0.55f), RoundedCornerShape(12.dp))
                .clickable { },
            contentAlignment = Alignment.Center
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Upload", color = contentColor.copy(alpha = 0.55f), fontWeight = FontWeight.SemiBold)
                Text(placeholder, color = contentColor.copy(alpha = 0.7f), fontSize = 14.sp)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = if (errorText != null) DoweDesign.danger else contentColor.copy(alpha = 0.7f))
        }
    }
}

private fun doweControlHeight(size: String): Dp {
    return when (size) {
        "sm" -> 34.dp
        "lg" -> 48.dp
        else -> 40.dp
    }
}

private fun doweControlSwatchSize(size: String): Dp {
    return when (size) {
        "sm" -> 20.dp
        "lg" -> 32.dp
        else -> 24.dp
    }
}

private fun doweHexColor(value: String, fallback: Color): Color {
    return try {
        Color(android.graphics.Color.parseColor(value))
    } catch (error: IllegalArgumentException) {
        fallback
    }
}

@Composable
private fun DoweCandlestick(state: DoweReactiveState, dataPath: String, stream: String?, upColor: Color, downColor: Color, emptyLabel: String, maxPoints: Int, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val candles = state.candles(dataPath).takeLast(maxPoints).mapIndexedNotNull { index, value -> doweCandlestickCandle(value, index) }
    LaunchedEffect(stream, dataPath, maxPoints) {
        doweConnectCandlestickStream(stream, dataPath, maxPoints, state)
    }
    Box(
        modifier = modifier
            .heightIn(min = 220.dp)
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier.border(1.dp, contentColor.copy(alpha = 0.12f), shape) else Modifier.border(1.dp, borderColor, shape)),
        contentAlignment = Alignment.Center
    ) {
        Canvas(modifier = Modifier.matchParentSize()) {
            if (candles.isEmpty()) {
                return@Canvas
            }
            val top = 12f
            val right = 12f
            val bottom = 18f
            val left = 12f
            val drawingWidth = max(1f, size.width - left - right)
            val drawingHeight = max(1f, size.height - top - bottom)
            val high = candles.maxOf { it.high }
            val low = candles.minOf { it.low }
            val range = max(high - low, 0.000001f)
            val step = drawingWidth / max(candles.size, 1)
            val bodyWidth = max(3f, min(12f, step * 0.56f))
            for (line in 0..3) {
                val y = top + drawingHeight * line / 3f
                drawLine(
                    color = contentColor.copy(alpha = 0.1f),
                    start = Offset(left, y),
                    end = Offset(left + drawingWidth, y),
                    strokeWidth = 1f
                )
            }
            candles.forEachIndexed { index, candle ->
                fun candleY(value: Float): Float = top + drawingHeight * ((high - value) / range)
                val centerX = left + step * (index + 0.5f)
                val highY = candleY(candle.high)
                val lowY = candleY(candle.low)
                val openY = candleY(candle.open)
                val closeY = candleY(candle.close)
                val color = if (candle.close >= candle.open) upColor else downColor
                drawLine(
                    color = color,
                    start = Offset(centerX, highY),
                    end = Offset(centerX, lowY),
                    strokeWidth = 1.4f
                )
                drawRoundRect(
                    color = color,
                    topLeft = Offset(centerX - bodyWidth / 2f, min(openY, closeY)),
                    size = Size(bodyWidth, max(1f, abs(closeY - openY))),
                    cornerRadius = CornerRadius(1.5f, 1.5f)
                )
            }
        }
        if (candles.isEmpty()) {
            Text(text = emptyLabel, color = contentColor.copy(alpha = 0.64f), fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
        }
    }
}

private fun doweCandlestickCandle(source: Map<String, Any?>, index: Int): DoweCandlestickCandle? {
    val time = source["time"]?.toString() ?: return null
    val open = doweCandleNumber(source["open"]) ?: return null
    val high = doweCandleNumber(source["high"]) ?: return null
    val low = doweCandleNumber(source["low"]) ?: return null
    val close = doweCandleNumber(source["close"]) ?: return null
    return DoweCandlestickCandle("$time-$index", time, open, high, low, close)
}

private suspend fun doweConnectCandlestickStream(stream: String?, dataPath: String, maxPoints: Int, state: DoweReactiveState) {
    val address = doweCandlestickStreamUrl(stream) ?: return
    withContext(Dispatchers.IO) {
        try {
            val connection = URL(address).openConnection() as HttpURLConnection
            connection.setRequestProperty("Accept", "text/event-stream")
            connection.inputStream.bufferedReader().use { reader ->
                while (true) {
                    val line = reader.readLine() ?: break
                    val payloadText = doweCandlestickStreamPayload(line)
                    if (payloadText.isEmpty()) {
                        continue
                    }
                    if (payloadText == "[DONE]") {
                        break
                    }
                    val payload = doweCandlestickJson(payloadText) ?: continue
                    withContext(Dispatchers.Main) {
                        state.upsertCandles(dataPath, payload, maxPoints)
                    }
                }
            }
        } catch (error: Exception) {
        }
    }
}

private fun doweCandlestickStreamPayload(line: String): String {
    val text = line.trim()
    return if (text.startsWith("data:")) text.removePrefix("data:").trim() else text
}

private fun doweCandlestickJson(text: String): Any? =
    try {
        when {
            text.startsWith("[") -> doweNativeValue(JSONArray(text))
            text.startsWith("{") -> doweNativeValue(JSONObject(text))
            else -> null
        }
    } catch (error: Exception) {
        null
    }

private fun doweCandlestickStreamUrl(stream: String?): String? {
    if (stream.isNullOrEmpty()) {
        return null
    }
    if (stream.startsWith("https://")) {
        return stream
    }
    if (stream.startsWith("/")) {
        val base = DoweEnvironment.BACKEND_URL.trimEnd('/')
        if (base.isEmpty()) {
            return null
        }
        return base + stream
    }
    return null
}

@Composable
private fun DoweTable(state: DoweReactiveState, dataPath: String, columns: List<DoweTableColumn>, size: DoweTableSize, striped: Boolean, bordered: Boolean, dividers: Boolean, emptyTitle: String, emptyDescription: String, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val rows = state.rows(dataPath)
    val metrics = doweTableMetrics(size)
    Box(
        modifier = modifier
            .fillMaxWidth()
            .clip(shape)
            .background(backgroundColor)
            .then(if (bordered || borderColor != null) Modifier.border(1.dp, borderColor ?: contentColor.copy(alpha = 0.12f), shape) else Modifier)
            .horizontalScroll(rememberScrollState())
    ) {
        Column(modifier = Modifier.widthIn(min = doweTableMinimumWidth(columns, metrics))) {
            Row(modifier = Modifier.fillMaxWidth().background(contentColor.copy(alpha = 0.08f))) {
                columns.forEach { column ->
                    Box(modifier = Modifier.width(doweTableColumnWidth(column.width)).padding(horizontal = metrics.horizontalPadding, vertical = metrics.headerVerticalPadding), contentAlignment = doweTableBoxAlignment(column.align)) {
                        Text(
                            text = column.label,
                            color = contentColor,
                            fontSize = metrics.headerSize,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 1
                        )
                    }
                }
            }
            if (rows.isEmpty()) {
                Column(
                    modifier = Modifier.fillMaxWidth().heightIn(min = 120.dp).padding(16.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center
                ) {
                    Text(text = emptyTitle, color = contentColor, fontSize = metrics.emptyTitleSize, fontWeight = FontWeight.SemiBold)
                    Text(text = emptyDescription, color = contentColor.copy(alpha = 0.68f), fontSize = metrics.emptyDescriptionSize)
                }
            } else {
                rows.forEachIndexed { index, row ->
                    Row(modifier = Modifier.fillMaxWidth().background(if (striped && index % 2 == 1) contentColor.copy(alpha = 0.05f) else Color.Transparent)) {
                        columns.forEachIndexed { columnIndex, column ->
                            Box(
                                modifier = Modifier.width(doweTableColumnWidth(column.width)),
                                contentAlignment = doweTableBoxAlignment(column.align)
                            ) {
                                Text(
                                    text = doweTableValue(row.value, column.field),
                                    modifier = Modifier.padding(horizontal = metrics.horizontalPadding, vertical = metrics.bodyVerticalPadding),
                                    color = contentColor,
                                    fontSize = metrics.bodySize,
                                    maxLines = 1
                                )
                                if (bordered && columnIndex < columns.lastIndex) {
                                    Box(modifier = Modifier.align(Alignment.CenterEnd).width(1.dp).fillMaxHeight().background(contentColor.copy(alpha = 0.12f)))
                                }
                            }
                        }
                    }
                    if (dividers && index < rows.lastIndex) {
                        Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(contentColor.copy(alpha = 0.12f)))
                    }
                }
            }
        }
    }
}

private data class DoweTableMetrics(
    val headerSize: TextUnit,
    val bodySize: TextUnit,
    val emptyTitleSize: TextUnit,
    val emptyDescriptionSize: TextUnit,
    val horizontalPadding: Dp,
    val headerVerticalPadding: Dp,
    val bodyVerticalPadding: Dp
)

private fun doweTableMetrics(size: DoweTableSize): DoweTableMetrics =
    when (size) {
        DoweTableSize.Sm -> DoweTableMetrics(12.sp, 12.sp, 16.sp, 13.sp, 12.dp, 8.dp, 8.dp)
        DoweTableSize.Lg -> DoweTableMetrics(16.sp, 16.sp, 20.sp, 15.sp, 20.dp, 16.dp, 20.dp)
        else -> DoweTableMetrics(14.sp, 14.sp, 18.sp, 14.sp, 16.dp, 12.dp, 16.dp)
    }

private fun doweTableColumnWidth(width: String?): Dp {
    if (width.isNullOrEmpty() || width == "auto" || width == "min-content" || width == "max-content") {
        return 160.dp
    }
    return when {
        width.endsWith("px") -> width.removeSuffix("px").toFloatOrNull()?.dp ?: 160.dp
        width.endsWith("rem") -> ((width.removeSuffix("rem").toFloatOrNull() ?: 10f) * 16f).dp
        else -> 160.dp
    }
}

private fun doweTableMinimumWidth(columns: List<DoweTableColumn>, metrics: DoweTableMetrics): Dp =
    columns.fold(0.dp) { total, column -> total + doweTableColumnWidth(column.width) + metrics.horizontalPadding * 2 }

private fun doweTableBoxAlignment(align: DoweTableColumnAlign): Alignment =
    when (align) {
        DoweTableColumnAlign.Center -> Alignment.Center
        DoweTableColumnAlign.End -> Alignment.CenterEnd
        else -> Alignment.CenterStart
    }

private fun doweTableValue(row: Map<String, Any?>, field: String): String {
    val parts = field.split(".")
    var current: Any? = row[parts.firstOrNull() ?: ""]
    parts.drop(1).forEach { part ->
        current = (current as? Map<*, *>)?.get(part)
    }
    return current?.takeUnless { it === JSONObject.NULL }?.toString() ?: ""
}

@Composable
private fun DoweCode(source: String, language: String, tokens: List<DoweCodeToken>, copyLabel: String, copiedLabel: String, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val clipboard = LocalClipboardManager.current
    var copied by remember { mutableStateOf(false) }
    val highlighted = buildAnnotatedString {
        tokens.forEach { token ->
            withStyle(SpanStyle(color = token.color)) {
                append(token.text)
            }
        }
    }
    LaunchedEffect(copied) {
        if (copied) {
            delay(1500)
            copied = false
        }
    }
    Column(modifier = modifier.clip(shape).background(backgroundColor).then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))) {
        Row(modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 10.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text(text = language.uppercase(), color = contentColor, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            Text(text = if (copied) copiedLabel else copyLabel, modifier = Modifier.clickable {
                clipboard.setText(AnnotatedString(source))
                copied = true
            }, color = contentColor, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        }
        Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(contentColor.copy(alpha = 0.24f)))
        Text(text = highlighted, modifier = Modifier.horizontalScroll(rememberScrollState()).padding(16.dp), fontFamily = FontFamily.Monospace, fontSize = 14.sp, lineHeight = 22.sp)
    }
}

private val doweSelectArrowViewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)
private val doweSelectArrowPaths = listOf(
    DoweSvgPath("M0 0h24v24H0z", DoweSvgFill.None),
    DoweSvgPath("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z", DoweSvgFill.CurrentColor)
)

@Composable
private fun DoweSvg(viewBox: DoweSvgViewBox, modifier: Modifier, color: Color, paths: List<DoweSvgPath>) {
    Canvas(modifier = modifier) {
        val scaleX = size.width / viewBox.width
        val scaleY = size.height / viewBox.height
        withTransform({
            scale(scaleX = scaleX, scaleY = scaleY)
            translate(left = -viewBox.minX, top = -viewBox.minY)
        }) {
            paths.forEach { entry ->
                val fill = when (val value = entry.fill) {
                    DoweSvgFill.None -> null
                    DoweSvgFill.CurrentColor -> color
                    is DoweSvgFill.Solid -> value.color
                }
                if (fill != null) {
                    drawPath(PathParser().parsePathString(entry.data).toPath(), fill)
                }
            }
        }
    }
}

@Composable
private fun DoweAvatar(source: String?, name: String?, alt: String, size: String, status: String?, bordered: Boolean, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, hasIcon: Boolean, icon: @Composable () -> Unit) {
    val avatarSize = doweAvatarSize(size)
    val indicatorSize = doweAvatarIndicatorSize(size)
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = Modifier
            .width(avatarSize)
            .height(avatarSize)
            .clip(shape)
            .background(backgroundColor)
            .then(if (bordered) Modifier.border(3.dp, borderColor ?: contentColor, shape) else Modifier)
            .then(if (onClick == null) Modifier else Modifier.clickable(onClick = onClick)),
        contentAlignment = Alignment.Center
    ) {
        if (!source.isNullOrBlank()) {
            AndroidView(
                modifier = Modifier.matchParentSize(),
                factory = { context -> ImageView(context).apply { scaleType = ImageView.ScaleType.CENTER_CROP } },
                update = { view -> view.setImageURI(Uri.parse(source)) }
            )
        } else if (hasIcon) {
            CompositionLocalProvider(LocalContentColor provides contentColor) {
                icon()
            }
        } else {
            Text(text = doweAvatarInitial(name, alt), color = contentColor, fontWeight = FontWeight.SemiBold, fontSize = doweAvatarTextSize(size))
        }
        if (status != null) {
            Box(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .width(indicatorSize)
                    .height(indicatorSize)
                    .clip(shape)
                    .background(doweAvatarStatusColor(status))
                    .border(1.dp, DoweDesign.background, shape)
            )
        }
    }
}

private fun doweAvatarSize(size: String): Dp =
    when (size) {
        "xs" -> 24.dp
        "sm" -> 32.dp
        "lg" -> 48.dp
        "xl" -> 64.dp
        else -> 40.dp
    }

private fun doweAvatarIndicatorSize(size: String): Dp =
    when (size) {
        "xs" -> 6.dp
        "sm" -> 8.dp
        "lg" -> 12.dp
        "xl" -> 16.dp
        else -> 10.dp
    }

private fun doweAvatarTextSize(size: String): TextUnit =
    when (size) {
        "xs" -> 12.sp
        "sm" -> 14.sp
        "lg" -> 18.sp
        "xl" -> 24.sp
        else -> 16.sp
    }

private fun doweAvatarInitial(name: String?, alt: String): String =
    (name?.trim()?.takeIf { it.isNotEmpty() } ?: alt).take(1).uppercase()

private fun doweAvatarStatusColor(status: String): Color =
    when (status) {
        "online" -> DoweDesign.success
        "busy" -> DoweDesign.warning
        "away" -> DoweDesign.danger
        else -> DoweDesign.muted
    }

private data class DoweAvatarGroupItem(
    val source: String?,
    val name: String?,
    val alt: String?,
    val onClick: (() -> Unit)?
)

private fun doweAvatarGroupItems(rows: List<Map<String, Any?>>, fallback: List<DoweAvatarGroupItem>): List<DoweAvatarGroupItem> {
    if (rows.isEmpty()) {
        return fallback
    }
    return rows.map { row ->
        DoweAvatarGroupItem(
            source = row["src"]?.toString(),
            name = row["name"]?.toString(),
            alt = row["alt"]?.toString(),
            onClick = null
        )
    }
}

@Composable
private fun DoweAvatarGroup(items: List<DoweAvatarGroupItem>, size: String, maxCount: Int?, inline: Boolean, bordered: Boolean, backgroundColor: Color, contentColor: Color, borderColor: Color, modifier: Modifier) {
    val visibleLimit = maxCount?.coerceAtLeast(1)
    val visibleItems = if (visibleLimit != null && items.size > visibleLimit) items.take((visibleLimit - 1).coerceAtLeast(0)) else items
    val hiddenCount = if (visibleLimit != null && items.size > visibleLimit) items.size - visibleItems.size else 0
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(if (inline) 8.dp else (-12).dp)
    ) {
        visibleItems.forEach { item ->
            DoweAvatar(
                source = item.source,
                name = item.name,
                alt = item.alt ?: item.name ?: "",
                size = size,
                status = null,
                bordered = bordered,
                backgroundColor = backgroundColor,
                contentColor = contentColor,
                borderColor = borderColor,
                onClick = item.onClick,
                hasIcon = false
            ) {}
        }
        if (hiddenCount > 0) {
            val counterSize = doweAvatarGroupCounterSize(size)
            Box(
                modifier = Modifier
                    .width(counterSize)
                    .height(counterSize)
                    .clip(RoundedCornerShape(999.dp))
                    .background(backgroundColor)
                    .border(if (bordered) 3.dp else 1.dp, borderColor, RoundedCornerShape(999.dp)),
                contentAlignment = Alignment.Center
            ) {
                Text(text = "+$hiddenCount", color = contentColor, fontSize = doweAvatarGroupCounterTextSize(size), fontWeight = FontWeight.SemiBold, maxLines = 1)
            }
        }
    }
}

private fun doweAvatarGroupCounterSize(size: String): Dp =
    when (size) {
        "xs" -> 20.dp
        "sm" -> 24.dp
        "lg" -> 40.dp
        "xl" -> 56.dp
        else -> 28.dp
    }

private fun doweAvatarGroupCounterTextSize(size: String): TextUnit =
    when (size) {
        "xs", "sm" -> 10.sp
        "lg" -> 14.sp
        "xl" -> 18.sp
        else -> 12.sp
    }

private data class DoweChatMessage(
    val id: String,
    val role: String,
    val userId: String?,
    val name: String?,
    val avatar: String?,
    val text: String,
    val status: String?
)

private fun doweChatMessages(rows: List<Map<String, Any?>>): List<DoweChatMessage> =
    rows.mapIndexed { index, row ->
        DoweChatMessage(
            id = row["id"]?.toString() ?: index.toString(),
            role = row["role"]?.toString() ?: "assistant",
            userId = (row["userId"] ?: row["user_id"])?.toString(),
            name = row["name"]?.toString(),
            avatar = row["avatar"]?.toString(),
            text = (row["text"] ?: row["content"] ?: row["message"])?.toString() ?: "",
            status = row["status"]?.toString()
        )
    }

@Composable
private fun DoweChatBox(state: DoweReactiveState, messagesPath: String, mode: String, currentUserId: String, userName: String, userAvatar: String?, userStatus: String, assistantName: String, assistantAvatar: String?, showHeader: Boolean, placeholder: String, showAttachments: Boolean, showVoiceNote: Boolean, showCamera: Boolean, loading: Boolean, sending: Boolean, streaming: Boolean, hasMore: Boolean, onSend: ((String) -> Unit)?, onLoadMore: (() -> Unit)?, onStop: (() -> Unit)?, onVoiceNote: (() -> Unit)?, onFileAttach: (() -> Unit)?, onCameraCapture: (() -> Unit)?, backgroundColor: Color, contentColor: Color, borderColor: Color?, modifier: Modifier) {
    var draft by remember { mutableStateOf("") }
    val messages = doweChatMessages(state.rows(messagesPath).map { it.value })
    val shape = RoundedCornerShape(DoweDesign.radiusBox)
    Column(
        modifier = modifier
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            .padding(12.dp)
    ) {
        if (showHeader) {
            Row(modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                DoweAvatar(source = assistantAvatar, name = assistantName, alt = assistantName, size = "sm", status = userStatus, bordered = false, backgroundColor = contentColor.copy(alpha = 0.08f), contentColor = contentColor, borderColor = contentColor, onClick = null, hasIcon = false) {}
                Column(modifier = Modifier.weight(1f)) {
                    Text(text = if (mode == "prompt") assistantName else userName, color = contentColor, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                    Text(text = userStatus, color = contentColor.copy(alpha = 0.64f), fontSize = 12.sp)
                }
                Text(text = "Search", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.Medium)
                Text(text = "...", color = contentColor.copy(alpha = 0.72f), fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
        }
        if (hasMore && onLoadMore != null) {
            Text(text = "Load more", modifier = Modifier.align(Alignment.CenterHorizontally).clickable(onClick = onLoadMore).padding(vertical = 6.dp), color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        }
        Column(modifier = Modifier.fillMaxWidth().weight(1f, fill = false), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            messages.forEach { message ->
                val own = message.userId == currentUserId || message.role == "user"
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = if (own) Arrangement.End else Arrangement.Start) {
                    Column(horizontalAlignment = if (own) Alignment.End else Alignment.Start) {
                        Text(
                            text = message.text,
                            modifier = Modifier
                                .widthIn(max = 280.dp)
                                .clip(RoundedCornerShape(16.dp))
                                .background(if (own) contentColor else contentColor.copy(alpha = 0.08f))
                                .padding(horizontal = 12.dp, vertical = 9.dp),
                            color = if (own) backgroundColor else contentColor,
                            fontSize = 14.sp,
                            lineHeight = 20.sp
                        )
                        if (!message.status.isNullOrEmpty()) {
                            Text(text = message.status, color = contentColor.copy(alpha = 0.52f), fontSize = 11.sp, modifier = Modifier.padding(top = 3.dp))
                        }
                    }
                }
            }
            if (loading || streaming) {
                Text(text = if (streaming) "..." else "Typing...", color = contentColor.copy(alpha = 0.64f), fontSize = 13.sp, modifier = Modifier.padding(horizontal = 8.dp, vertical = 6.dp))
            }
        }
        Row(modifier = Modifier.fillMaxWidth().padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (showVoiceNote && onVoiceNote != null) {
                Text(text = "Mic", modifier = Modifier.clickable(onClick = onVoiceNote), color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
            if (showAttachments && onFileAttach != null) {
                Text(text = "+", modifier = Modifier.clickable(onClick = onFileAttach), color = contentColor.copy(alpha = 0.72f), fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
            if (showCamera && onCameraCapture != null) {
                Text(text = "Cam", modifier = Modifier.clickable(onClick = onCameraCapture), color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
            BasicTextField(
                value = draft,
                onValueChange = { draft = it },
                textStyle = TextStyle(color = contentColor, fontSize = 14.sp),
                modifier = Modifier
                    .weight(1f)
                    .heightIn(min = 40.dp)
                    .clip(RoundedCornerShape(20.dp))
                    .background(contentColor.copy(alpha = 0.08f))
                    .padding(horizontal = 14.dp, vertical = 10.dp),
                decorationBox = { inner ->
                    if (draft.isEmpty()) {
                        Text(text = placeholder, color = contentColor.copy(alpha = 0.48f), fontSize = 14.sp)
                    }
                    inner()
                }
            )
            val canSend = draft.trim().isNotEmpty() && onSend != null && !sending
            Text(
                text = if (streaming && onStop != null) "Stop" else "Send",
                modifier = Modifier
                    .clip(RoundedCornerShape(18.dp))
                    .background(if (canSend || streaming) contentColor else contentColor.copy(alpha = 0.16f))
                    .clickable {
                        if (streaming && onStop != null) {
                            onStop()
                        } else if (canSend) {
                            onSend?.invoke(draft)
                            draft = ""
                        }
                    }
                    .padding(horizontal = 12.dp, vertical = 8.dp),
                color = if (canSend || streaming) backgroundColor else contentColor.copy(alpha = 0.48f),
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}

@Composable
private fun DoweEmpty(kind: String, title: String?, description: String?, actionLabel: String, action: (() -> Unit)?, backgroundColor: Color, contentColor: Color, accentColor: Color, modifier: Modifier) {
    Column(
        modifier = modifier.fillMaxWidth().padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        DoweEmptyIcon(kind = kind, color = accentColor)
        Text(text = title ?: doweEmptyTitle(kind), color = contentColor, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
        Text(text = description ?: doweEmptyDescription(kind), color = contentColor.copy(alpha = 0.64f), fontSize = 14.sp, lineHeight = 20.sp)
        if (action != null) {
            Text(
                text = actionLabel,
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(accentColor.copy(alpha = 0.12f))
                    .clickable(onClick = action)
                    .padding(horizontal = 16.dp, vertical = 9.dp),
                color = accentColor,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}

@Composable
private fun DoweEmptyIcon(kind: String, color: Color) {
    Canvas(modifier = Modifier.width(160.dp).height(120.dp)) {
        val sx = size.width / 120f
        val sy = size.height / 100f
        withTransform({ scale(scaleX = sx, scaleY = sy) }) {
            val soft = color.copy(alpha = 0.12f)
            val strong = color.copy(alpha = 0.78f)
            when (kind) {
                "playlist" -> {
                    drawRoundRect(soft, topLeft = Offset(28f, 18f), size = Size(54f, 64f), cornerRadius = CornerRadius(10f, 10f))
                    drawRoundRect(strong, topLeft = Offset(71f, 29f), size = Size(5f, 36f), cornerRadius = CornerRadius(2.5f, 2.5f))
                    drawRoundRect(strong, topLeft = Offset(44f, 29f), size = Size(5f, 36f), cornerRadius = CornerRadius(2.5f, 2.5f))
                    drawRoundRect(strong, topLeft = Offset(49f, 29f), size = Size(27f, 6f), cornerRadius = CornerRadius(3f, 3f))
                    drawCircle(strong, radius = 10f, center = Offset(41f, 63f))
                    drawCircle(strong, radius = 10f, center = Offset(68f, 63f))
                }
                "result" -> {
                    drawCircle(soft, radius = 24f, center = Offset(54f, 45f))
                    drawRoundRect(strong, topLeft = Offset(68f, 61f), size = Size(27f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                    drawRoundRect(strong, topLeft = Offset(45f, 35f), size = Size(18f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                    drawRoundRect(strong, topLeft = Offset(45f, 47f), size = Size(13f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                }
                "template" -> {
                    drawRoundRect(soft, topLeft = Offset(30f, 20f), size = Size(62f, 60f), cornerRadius = CornerRadius(6f, 6f))
                    drawRoundRect(strong, topLeft = Offset(72f, 20f), size = Size(20f, 20f), cornerRadius = CornerRadius(3f, 3f))
                    drawRoundRect(strong, topLeft = Offset(43f, 47f), size = Size(34f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                    drawRoundRect(strong, topLeft = Offset(43f, 61f), size = Size(26f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                }
                else -> {
                    drawRoundRect(soft, topLeft = Offset(24f, 22f), size = Size(72f, 56f), cornerRadius = CornerRadius(10f, 10f))
                    drawRoundRect(strong, topLeft = Offset(38f, 35f), size = Size(44f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                    drawRoundRect(strong, topLeft = Offset(38f, 49f), size = Size(34f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                    drawRoundRect(strong, topLeft = Offset(38f, 63f), size = Size(22f, 7f), cornerRadius = CornerRadius(3.5f, 3.5f))
                }
            }
        }
    }

private fun doweEmptyTitle(kind: String): String =
    when (kind) {
        "playlist" -> "No playlist items"
        "result" -> "No results"
        "template" -> "No templates"
        else -> "No data"
    }

private fun doweEmptyDescription(kind: String): String =
    when (kind) {
        "playlist" -> "Add items to start building this playlist."
        "result" -> "Try changing the search or filters."
        "template" -> "Create a template to reuse this workflow."
        else -> "There is nothing to show yet."
    }

@Composable
private fun DoweMarquee(speed: String, pauseOnHover: Boolean, reverse: Boolean, orientation: String, fade: Boolean, fadeColor: Color, gap: Dp, modifier: Modifier, content: @Composable () -> Unit) {
    var offset by remember { mutableStateOf(0f) }
    val distance = 360f
    LaunchedEffect(speed, reverse, orientation) {
        while (true) {
            delay(16)
            val step = doweMarqueeStep(speed) * if (reverse) 1f else -1f
            offset += step
            if (offset <= -distance || offset >= distance) {
                offset = 0f
            }
        }
    }
    Box(modifier = modifier.clipToBounds()) {
        if (orientation == "vertical") {
            Column(modifier = Modifier.graphicsLayer { translationY = offset }, verticalArrangement = Arrangement.spacedBy(gap)) {
                content()
                Spacer(modifier = Modifier.height(gap))
                content()
            }
        } else {
            Row(modifier = Modifier.graphicsLayer { translationX = offset }, horizontalArrangement = Arrangement.spacedBy(gap), verticalAlignment = Alignment.CenterVertically) {
                content()
                Spacer(modifier = Modifier.width(gap))
                content()
            }
        }
        if (fade) {
            if (orientation == "vertical") {
                Box(modifier = Modifier.align(Alignment.TopCenter).fillMaxWidth().height(32.dp).background(Brush.verticalGradient(listOf(fadeColor, fadeColor.copy(alpha = 0f)))))
                Box(modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(32.dp).background(Brush.verticalGradient(listOf(fadeColor.copy(alpha = 0f), fadeColor))))
            } else {
                Box(modifier = Modifier.align(Alignment.CenterStart).width(32.dp).fillMaxHeight().background(Brush.horizontalGradient(listOf(fadeColor, fadeColor.copy(alpha = 0f)))))
                Box(modifier = Modifier.align(Alignment.CenterEnd).width(32.dp).fillMaxHeight().background(Brush.horizontalGradient(listOf(fadeColor.copy(alpha = 0f), fadeColor))))
            }
        }
    }
}

private fun doweMarqueeStep(speed: String): Float =
    when (speed) {
        "slow" -> 0.45f
        "fast" -> 1.8f
        else -> 0.9f
    }

@Composable
private fun DoweTypeWriter(texts: List<String>, typeSpeed: Long, deleteSpeed: Long, afterTyped: Long, afterDeleted: Long, repeat: Boolean, contentColor: Color, modifier: Modifier) {
    var rendered by remember { mutableStateOf("") }
    LaunchedEffect(texts, typeSpeed, deleteSpeed, afterTyped, afterDeleted, repeat) {
        if (texts.isEmpty()) {
            rendered = ""
            return@LaunchedEffect
        }
        var index = 0
        while (true) {
            val current = texts[index]
            for (length in 1..current.length) {
                rendered = current.take(length)
                delay(typeSpeed)
            }
            delay(afterTyped)
            for (length in current.length downTo 0) {
                rendered = current.take(length)
                delay(deleteSpeed)
            }
            delay(afterDeleted)
            index = (index + 1) % texts.size
            if (!repeat && index == 0) {
                rendered = current
                break
            }
        }
    }
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically) {
        Text(text = rendered, color = contentColor)
        Text(text = "|", color = contentColor.copy(alpha = 0.72f), modifier = Modifier.padding(start = 2.dp))
    }
}

private data class DoweRichTextMark(val text: String, val style: String, val color: Color)

@Composable
private fun DoweRichText(marks: List<DoweRichTextMark>, fontFamily: FontFamily?, fontSize: TextUnit, contentColor: Color, modifier: Modifier) {
    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
        marks.forEach { mark ->
            val accent = if (mark.color == Color.Unspecified) contentColor else mark.color
            val shape = RoundedCornerShape(if (mark.style == "pill") 999.dp else 6.dp)
            val decorated = when (mark.style) {
                "mark" -> Modifier.clip(shape).background(accent.copy(alpha = 0.18f)).padding(horizontal = 4.dp, vertical = 1.dp)
                "pill" -> Modifier.clip(shape).border(2.dp, accent, shape).padding(horizontal = 10.dp, vertical = 2.dp)
                "slant" -> Modifier.clip(shape).background(accent).graphicsLayer(rotationZ = -1f).padding(horizontal = 6.dp, vertical = 1.dp)
                "box" -> Modifier.clip(shape).border(2.dp, accent, shape).padding(horizontal = 12.dp, vertical = 4.dp)
                "tag" -> Modifier.clip(shape).background(accent).padding(horizontal = 12.dp, vertical = 4.dp)
                "pop" -> Modifier.graphicsLayer(rotationZ = -1f, scaleX = 1.04f, scaleY = 1.04f)
                else -> Modifier
            }
            val textColor = if (mark.style == "tag" || mark.style == "slant") DoweDesign.background else accent
            Text(
                text = mark.text,
                modifier = decorated,
                color = textColor,
                fontFamily = fontFamily,
                fontSize = fontSize,
                fontWeight = if (mark.style in setOf("mark", "grad", "pill", "slant", "glow", "neon", "pop", "tag")) FontWeight.Bold else FontWeight.SemiBold,
                fontStyle = if (mark.style == "grad" || mark.style == "slant") FontStyle.Italic else FontStyle.Normal,
                textDecoration = when (mark.style) {
                    "under", "wave" -> TextDecoration.Underline
                    "strike" -> TextDecoration.LineThrough
                    else -> TextDecoration.None
                },
                style = TextStyle(
                    shadow = if (mark.style in setOf("glow", "neon", "pop")) Shadow(color = accent.copy(alpha = 0.7f), offset = Offset.Zero, blurRadius = 8f) else null
                )
            )
        }
    }
}

@Composable
private fun DoweRecord(name: String, url: String?, disabled: Boolean, maxDuration: Int?, backgroundColor: Color, contentColor: Color, borderColor: Color?, onStart: (() -> Unit)?, onPause: (() -> Unit)?, onResume: (() -> Unit)?, onStop: (() -> Unit)?, onDiscard: (() -> Unit)?, onConfirm: (() -> Unit)?, modifier: Modifier) {
    var state by remember(url) { mutableStateOf(if (url != null) "reviewing" else "idle") }
    var elapsed by remember { mutableStateOf(0L) }
    var started by remember { mutableStateOf(0L) }
    var now by remember { mutableStateOf(System.currentTimeMillis()) }
    val seconds = elapsed + if (state == "recording" && started > 0) ((now - started) / 1000L).coerceAtLeast(0L) else 0L
    LaunchedEffect(state, started, elapsed, maxDuration) {
        while (state == "recording") {
            delay(250)
            now = System.currentTimeMillis()
            val max = maxDuration?.toLong()
            val current = elapsed + if (started > 0) ((now - started) / 1000L).coerceAtLeast(0L) else 0L
            if (max != null && current >= max) {
                elapsed = max
                started = 0L
                state = "reviewing"
                onStop?.invoke()
                break
            }
        }
    }
    Row(
        modifier = modifier
            .clip(RoundedCornerShape(16.dp))
            .background(backgroundColor)
            .then(if (borderColor != null) Modifier.border(1.dp, borderColor, RoundedCornerShape(16.dp)) else Modifier)
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Row(modifier = Modifier.weight(1f).height(32.dp), verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
            repeat(50) { index ->
                Box(Modifier.weight(1f).height((((index % 9) + 2) * 2).dp).clip(RoundedCornerShape(2.dp)).background(contentColor.copy(alpha = if (state == "recording") 0.85f else 0.34f)))
            }
        }
        Column {
            Text(text = doweRecordTime(seconds), color = contentColor, fontSize = 12.sp, fontWeight = FontWeight.Bold)
            Text(text = when (state) { "recording" -> "Recording"; "paused" -> "Paused"; "reviewing" -> "Review"; else -> "Ready" }, color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            if (state == "idle" || state == "paused") Button(enabled = !disabled, onClick = { val resume = state == "paused"; now = System.currentTimeMillis(); if (!resume) elapsed = 0L; started = now; state = "recording"; if (resume) onResume?.invoke() else onStart?.invoke() }) { Text(if (state == "paused") "Resume" else "Record", fontSize = 12.sp) }
            if (state == "recording") {
                Button(enabled = !disabled, onClick = { now = System.currentTimeMillis(); elapsed = seconds; started = 0L; state = "paused"; onPause?.invoke() }) { Text("Pause", fontSize = 12.sp) }
                Button(enabled = !disabled, onClick = { now = System.currentTimeMillis(); elapsed = seconds; started = 0L; state = "reviewing"; onStop?.invoke() }) { Text("Stop", fontSize = 12.sp) }
            }
            if (state == "reviewing") {
                Button(enabled = !disabled, onClick = { elapsed = 0L; started = 0L; state = "idle"; onDiscard?.invoke() }) { Text("Discard", fontSize = 12.sp) }
                Button(enabled = !disabled, onClick = { onConfirm?.invoke() }) { Text("Use", fontSize = 12.sp) }
            }
        }
    }
}

private fun doweRecordTime(seconds: Long): String =
    "${seconds / 60}:${(seconds % 60).toString().padStart(2, '0')}"

private data class DoweToggleGroupItem(val id: String, val label: String, val icon: String?)

@Composable
private fun DoweToggleGroup(value: String, onValueChange: (String) -> Unit, items: List<DoweToggleGroupItem>, size: String, wide: Boolean, vertical: Boolean, disabled: Boolean, ariaLabel: String?, backgroundColor: Color, contentColor: Color, borderColor: Color?, onChange: (() -> Unit)?, modifier: Modifier) {
    val container = modifier.clip(RoundedCornerShape(10.dp)).background(backgroundColor).then(if (borderColor != null) Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)) else Modifier).padding(4.dp)
    val buttonContent: @Composable RowScope.(DoweToggleGroupItem) -> Unit = { item ->
        Text(text = item.label, fontSize = when (size) { "xs" -> 12.sp; "sm" -> 13.sp; "lg" -> 17.sp; else -> 14.sp }, fontWeight = FontWeight.SemiBold)
    }
    if (vertical) {
        Column(modifier = container, verticalArrangement = Arrangement.spacedBy(4.dp)) {
            items.forEach { item ->
                Button(enabled = !disabled, onClick = { onValueChange(item.id); onChange?.invoke() }, colors = ButtonDefaults.buttonColors(containerColor = if (value == item.id) contentColor else Color.Transparent, contentColor = if (value == item.id) backgroundColor else contentColor.copy(alpha = 0.72f)), modifier = if (wide) Modifier.fillMaxWidth() else Modifier) { buttonContent(item) }
            }
        }
    } else {
        Row(modifier = container.then(if (wide) Modifier.fillMaxWidth() else Modifier), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            items.forEach { item ->
                Button(enabled = !disabled, onClick = { onValueChange(item.id); onChange?.invoke() }, colors = ButtonDefaults.buttonColors(containerColor = if (value == item.id) contentColor else Color.Transparent, contentColor = if (value == item.id) backgroundColor else contentColor.copy(alpha = 0.72f)), modifier = if (wide) Modifier.weight(1f) else Modifier) { buttonContent(item) }
            }
        }
    }
}

@Composable
private fun DoweCollapsible(label: String, defaultOpen: Boolean, disabled: Boolean, backgroundColor: Color, contentColor: Color, borderColor: Color?, modifier: Modifier, content: @Composable () -> Unit) {
    var open by remember { mutableStateOf(defaultOpen) }
    Column(modifier = modifier.clip(RoundedCornerShape(16.dp)).background(backgroundColor).then(if (borderColor != null) Modifier.border(1.dp, borderColor, RoundedCornerShape(16.dp)) else Modifier)) {
        Row(modifier = Modifier.fillMaxWidth().clickable(enabled = !disabled) { open = !open }.padding(horizontal = 16.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(text = label, color = contentColor, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.weight(1f))
            Text(text = if (open) "⌃" else "⌄", color = contentColor)
        }
        AnimatedVisibility(visible = open, enter = fadeIn(tween(160)) + expandVertically(), exit = fadeOut(tween(160)) + shrinkVertically()) {
            Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) { content() }
        }
    }
}

@Composable
private fun DoweCountdown(target: String, showDays: Boolean, showHours: Boolean, showMinutes: Boolean, showSeconds: Boolean, size: String, daysLabel: String, hoursLabel: String, minutesLabel: String, secondsLabel: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, onComplete: (() -> Unit)?, modifier: Modifier) {
    var now by remember { mutableStateOf(System.currentTimeMillis()) }
    var completed by remember { mutableStateOf(false) }
    val targetMillis = remember(target) { runCatching { Instant.parse(target).toEpochMilli() }.getOrDefault(now) }
    val remaining = max(0L, (targetMillis - now) / 1000L)
    LaunchedEffect(targetMillis) {
        while (true) {
            delay(1000)
            now = System.currentTimeMillis()
            if ((targetMillis - now) <= 0 && !completed) {
                completed = true
                onComplete?.invoke()
            }
        }
    }
    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
        var needsSeparator = false
        if (showDays) {
            DoweCountdownUnit((remaining / 86400).toInt(), daysLabel, size, backgroundColor, contentColor, borderColor)
            needsSeparator = true
        }
        if (showHours) {
            if (needsSeparator) DoweCountdownSeparator(size, contentColor)
            DoweCountdownUnit(((remaining % 86400) / 3600).toInt(), hoursLabel, size, backgroundColor, contentColor, borderColor)
            needsSeparator = true
        }
        if (showMinutes) {
            if (needsSeparator) DoweCountdownSeparator(size, contentColor)
            DoweCountdownUnit(((remaining % 3600) / 60).toInt(), minutesLabel, size, backgroundColor, contentColor, borderColor)
            needsSeparator = true
        }
        if (showSeconds) {
            if (needsSeparator) DoweCountdownSeparator(size, contentColor)
            DoweCountdownUnit((remaining % 60).toInt(), secondsLabel, size, backgroundColor, contentColor, borderColor)
        }
    }
}

@Composable
private fun DoweCountdownUnit(value: Int, label: String, size: String, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val width = when (size) { "sm" -> 40.dp; "lg" -> 80.dp; "xl" -> 112.dp; else -> 56.dp }
    val height = when (size) { "sm" -> 48.dp; "lg" -> 96.dp; "xl" -> 128.dp; else -> 64.dp }
    val font = when (size) { "sm" -> 20.sp; "lg" -> 48.sp; "xl" -> 72.sp; else -> 30.sp }
    Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Box(modifier = Modifier.width(width).height(height).clip(RoundedCornerShape(16.dp)).background(backgroundColor).then(if (borderColor != null) Modifier.border(1.dp, borderColor, RoundedCornerShape(16.dp)) else Modifier), contentAlignment = Alignment.Center) {
            Text(text = value.toString().padStart(2, '0'), color = contentColor, fontSize = font, fontWeight = FontWeight.Bold)
        }
        Text(text = label.uppercase(), color = contentColor.copy(alpha = 0.72f), fontSize = when (size) { "sm" -> 10.sp; "lg" -> 14.sp; "xl" -> 16.sp; else -> 12.sp }, fontWeight = FontWeight.Medium)
    }
}

@Composable
private fun DoweCountdownSeparator(size: String, contentColor: Color) {
    val font = when (size) { "sm" -> 20.sp; "lg" -> 48.sp; "xl" -> 72.sp; else -> 30.sp }
    val top = when (size) { "sm" -> 8.dp; "lg" -> 20.dp; "xl" -> 28.dp; else -> 12.dp }
    Text(text = ":", modifier = Modifier.padding(top = top), color = contentColor.copy(alpha = 0.5f), fontSize = font, fontWeight = FontWeight.Bold)
}

private data class DoweMapMarker(val id: String, val lat: String, val lng: String, val label: String?, val popup: String?, val icon: String, val onClick: (() -> Unit)?)
private data class DoweMapWaypoint(val lat: String, val lng: String)

@Composable
private fun DoweMap(centerLat: String, centerLng: String, zoom: Int, height: String, width: String, showControls: Boolean, showScale: Boolean, showLocationControl: Boolean, interactive: Boolean, markers: List<DoweMapMarker>, waypoints: List<DoweMapWaypoint>, backgroundColor: Color, contentColor: Color, onLocation: (() -> Unit)?, onLocationError: (() -> Unit)?, onRoute: (() -> Unit)?, modifier: Modifier) {
    Box(modifier = modifier.height(doweMapHeight(height)).fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(backgroundColor.copy(alpha = 0.18f)).clipToBounds()) {
        Canvas(Modifier.fillMaxSize()) {
            val step = 32.dp.toPx()
            var x = 0f
            while (x < size.width) { drawLine(contentColor.copy(alpha = 0.16f), Offset(x, 0f), Offset(x, size.height)); x += step }
            var y = 0f
            while (y < size.height) { drawLine(contentColor.copy(alpha = 0.16f), Offset(0f, y), Offset(size.width, y)); y += step }
        }
        Column(modifier = Modifier.align(Alignment.Center), horizontalAlignment = Alignment.CenterHorizontally) {
            markers.forEach { marker ->
                Button(onClick = { marker.onClick?.invoke() }, enabled = interactive, colors = ButtonDefaults.buttonColors(containerColor = Color.Transparent, contentColor = if (marker.icon == "start") DoweDesign.success else if (marker.icon == "end") DoweDesign.danger else contentColor)) {
                    Text(text = "● ${marker.label ?: marker.popup ?: marker.id}", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                }
            }
        }
        if (showControls) Column(modifier = Modifier.align(Alignment.TopEnd).padding(12.dp).clip(RoundedCornerShape(10.dp)).background(DoweDesign.background.copy(alpha = 0.92f))) { Text("+", modifier = Modifier.padding(10.dp), fontWeight = FontWeight.Bold); Text("-", modifier = Modifier.padding(10.dp), fontWeight = FontWeight.Bold) }
        if (showScale) Text("1 km", modifier = Modifier.align(Alignment.BottomStart).padding(12.dp).clip(RoundedCornerShape(999.dp)).background(DoweDesign.background.copy(alpha = 0.92f)).padding(horizontal = 10.dp, vertical = 4.dp), fontSize = 12.sp, fontWeight = FontWeight.Bold)
        if (showLocationControl) Button(onClick = { onLocation?.invoke() }, modifier = Modifier.align(Alignment.BottomEnd).padding(12.dp)) { Text("⌖") }
    }
}

private fun doweMapHeight(value: String): Dp =
    value.removeSuffix("px").toFloatOrNull()?.dp ?: 400.dp

@Composable
private fun DoweBadge(text: String, position: String, backgroundColor: Color, contentColor: Color, modifier: Modifier, content: @Composable () -> Unit) {
    Box(modifier = modifier) {
        content()
        Text(
            text = text,
            modifier = Modifier
                .align(doweBadgeAlignment(position))
                .clip(RoundedCornerShape(999.dp))
                .background(backgroundColor)
                .padding(horizontal = 6.dp, vertical = 2.dp),
            color = contentColor,
            fontSize = 12.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1
        )
    }
}

private fun doweBadgeAlignment(position: String): Alignment =
    when (position) {
        "top-left" -> Alignment.TopStart
        "bottom-left" -> Alignment.BottomStart
        "bottom-right" -> Alignment.BottomEnd
        else -> Alignment.TopEnd
    }

@Composable
private fun DoweChip(text: String, size: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, modifier: Modifier, onClose: (() -> Unit)?, start: (@Composable () -> Unit)?, end: (@Composable () -> Unit)?) {
    val shape = RoundedCornerShape(DoweDesign.radiusUi)
    Row(
        modifier = modifier
            .height(doweChipHeight(size))
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            .padding(horizontal = doweChipPadding(size)),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            start?.invoke()
            Text(text = text, color = contentColor, fontSize = doweChipTextSize(size), fontWeight = FontWeight.Medium, maxLines = 1)
            end?.invoke()
            if (onClose != null) {
                Text(text = "x", modifier = Modifier.clickable(onClick = onClose), color = contentColor.copy(alpha = 0.72f), fontSize = doweChipTextSize(size), fontWeight = FontWeight.Bold)
            }
        }
    }
}

private fun doweChipHeight(size: String): Dp =
    when (size) {
        "xs" -> 20.dp
        "sm" -> 24.dp
        "lg" -> 40.dp
        "xl" -> 48.dp
        else -> 32.dp
    }

private fun doweChipPadding(size: String): Dp =
    when (size) {
        "xs", "sm" -> 12.dp
        "lg" -> 20.dp
        "xl" -> 24.dp
        else -> 16.dp
    }

private fun doweChipTextSize(size: String): TextUnit =
    when (size) {
        "xs", "sm" -> 12.sp
        "lg" -> 18.sp
        "xl" -> 24.sp
        else -> 14.sp
    }

@Composable
private fun DoweSkeleton(variant: String, animation: String, modifier: Modifier) {
    val alpha by animateFloatAsState(
        targetValue = if (animation == "pulse") 0.45f else 1f,
        animationSpec = tween(durationMillis = 900)
    )
    val shape = when (variant) {
        "circular" -> RoundedCornerShape(999.dp)
        "rectangular" -> RoundedCornerShape(0.dp)
        "rounded" -> RoundedCornerShape(DoweDesign.radiusBox)
        else -> RoundedCornerShape(6.dp)
    }
    val base = if (variant == "text") modifier.height(16.dp).fillMaxWidth() else modifier
    Box(modifier = base.clip(shape).background(DoweDesign.muted.copy(alpha = if (animation == "none") 1f else alpha)))
}

@Composable
private fun DoweModal(open: Boolean, close: () -> Unit, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp, disableOverlayClose: Boolean, hideCloseButton: Boolean, header: (@Composable () -> Unit)?, footer: (@Composable () -> Unit)?, content: @Composable () -> Unit) {
    if (!open) {
        return
    }
    Popup(properties = PopupProperties(focusable = true)) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .background(Color.Black.copy(alpha = 0.48f))
                    .clickable { if (!disableOverlayClose) close() }
            )
            Column(
                modifier = Modifier
                    .widthIn(max = 560.dp)
                    .clip(RoundedCornerShape(radius))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(radius)))
                    .padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                CompositionLocalProvider(LocalContentColor provides contentColor) {
                    if (header != null) {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                            Box(modifier = Modifier.weight(1f)) { header() }
                            if (!hideCloseButton) {
                                Text(text = "x", modifier = Modifier.clickable(onClick = close).padding(4.dp), color = contentColor.copy(alpha = 0.72f), fontWeight = FontWeight.Bold)
                            }
                        }
                    } else if (!hideCloseButton) {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                            Text(text = "x", modifier = Modifier.clickable(onClick = close).padding(4.dp), color = contentColor.copy(alpha = 0.72f), fontWeight = FontWeight.Bold)
                        }
                    }
                    content()
                    footer?.invoke()
                }
            }
        }
    }
}

@Composable
private fun DoweAlertDialog(open: Boolean, close: () -> Unit, title: String, description: String, confirmText: String, cancelText: String, backgroundColor: Color, contentColor: Color, dangerColor: Color, radius: Dp, loading: Boolean, onConfirm: (() -> Unit)?, onCancel: (() -> Unit)?) {
    DoweModal(
        open = open,
        close = close,
        backgroundColor = backgroundColor,
        contentColor = contentColor,
        borderColor = null,
        radius = radius,
        disableOverlayClose = true,
        hideCloseButton = true,
        header = { Text(text = title, color = contentColor, fontSize = 18.sp, fontWeight = FontWeight.SemiBold) },
        footer = {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                Button(enabled = !loading, onClick = { close(); onCancel?.invoke() }) { Text(cancelText) }
                Button(enabled = !loading, onClick = { onConfirm?.invoke() }, colors = ButtonDefaults.buttonColors(containerColor = dangerColor, contentColor = DoweDesign.onDanger)) { Text(confirmText) }
            }
        }
    ) {
        Text(text = description, color = contentColor.copy(alpha = 0.72f), fontSize = 14.sp)
    }
}

@Composable
private fun DoweTooltip(label: String, position: String, backgroundColor: Color, contentColor: Color, modifier: Modifier, content: @Composable () -> Unit) {
    var open by remember { mutableStateOf(false) }
    Box(modifier = modifier.clickable { open = !open }) {
        content()
        if (open) {
            Popup(alignment = doweTooltipAlignment(position)) {
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(DoweDesign.radiusBox))
                        .background(backgroundColor)
                        .padding(horizontal = 12.dp, vertical = 8.dp)
                ) {
                    Text(text = label, color = contentColor, fontSize = 13.sp)
                }
            }
        }
    }
}

private fun doweTooltipAlignment(position: String): Alignment =
    when (position) {
        "bottom" -> Alignment.BottomCenter
        "start" -> Alignment.CenterStart
        "end" -> Alignment.CenterEnd
        else -> Alignment.TopCenter
    }

@Composable
private fun DoweToast(visible: Boolean, title: String, description: String, position: String, backgroundColor: Color, contentColor: Color, showIcon: Boolean, kind: String, close: (() -> Unit)?) {
    if (!visible) {
        return
    }
    Popup(alignment = doweCornerAlignment(position)) {
        Row(
            modifier = Modifier
                .padding(16.dp)
                .widthIn(max = 420.dp)
                .clip(RoundedCornerShape(DoweDesign.radiusBox))
                .background(backgroundColor)
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            if (showIcon) {
                Text(text = doweToastIcon(kind), color = contentColor, fontWeight = FontWeight.Bold)
            }
            Column(modifier = Modifier.weight(1f)) {
                if (title.isNotEmpty()) {
                    Text(text = title, color = contentColor, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                }
                Text(text = description, color = contentColor.copy(alpha = 0.9f), fontSize = 14.sp)
            }
            if (close != null) {
                Text(text = "x", modifier = Modifier.clickable(onClick = close), color = contentColor.copy(alpha = 0.72f), fontWeight = FontWeight.Bold)
            }
        }
    }
}

private fun doweCornerAlignment(position: String): Alignment =
    when (position) {
        "top-left" -> Alignment.TopStart
        "top-right" -> Alignment.TopEnd
        "bottom-right" -> Alignment.BottomEnd
        else -> Alignment.BottomStart
    }

private fun doweToastIcon(kind: String): String =
    when (kind) {
        "success" -> "✓"
        "warning" -> "!"
        "danger", "error" -> "x"
        else -> "i"
    }

@Composable
private fun DoweDropdown(backgroundColor: Color, contentColor: Color, modifier: Modifier, trigger: @Composable () -> Unit, content: @Composable () -> Unit) {
    var open by remember { mutableStateOf(false) }
    Box(modifier = modifier) {
        Box(modifier = Modifier.clickable { open = !open }) {
            trigger()
        }
        if (open) {
            Popup(alignment = Alignment.TopStart, properties = PopupProperties(focusable = true, dismissOnClickOutside = true)) {
                Column(
                    modifier = Modifier
                        .widthIn(min = 220.dp, max = 360.dp)
                        .clip(RoundedCornerShape(DoweDesign.radiusBox))
                        .background(backgroundColor)
                        .padding(8.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    CompositionLocalProvider(LocalContentColor provides contentColor) {
                        content()
                    }
                }
            }
        }
    }
}

@Composable
private fun DoweOverlayItem(label: String, description: String?, disabled: Boolean, backgroundColor: Color, contentColor: Color, onClick: (() -> Unit)?, icon: @Composable () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(DoweDesign.radiusUi))
            .background(backgroundColor.copy(alpha = if (onClick == null) 0f else 0.08f))
            .then(if (onClick == null || disabled) Modifier else Modifier.clickable(onClick = onClick))
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            icon()
            Column(modifier = Modifier.weight(1f)) {
                Text(text = label, color = contentColor.copy(alpha = if (disabled) 0.48f else 1f), fontSize = 14.sp, fontWeight = FontWeight.Medium)
                if (description != null) {
                    Text(text = description, color = contentColor.copy(alpha = 0.68f), fontSize = 12.sp)
                }
            }
        }
    }
}

@Composable
private fun DoweCommand(open: Boolean, close: () -> Unit, placeholder: String, emptyText: String, closeText: String, navigateText: String, selectText: String, toggleText: String, shortcut: String, showFooter: Boolean, backgroundColor: Color, contentColor: Color, accentColor: Color, content: @Composable () -> Unit) {
    if (!open) {
        return
    }
    Popup(properties = PopupProperties(focusable = true)) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            Box(modifier = Modifier.matchParentSize().background(Color.Black.copy(alpha = 0.48f)).clickable(onClick = close))
            Column(
                modifier = Modifier
                    .padding(top = 64.dp)
                    .widthIn(min = 320.dp, max = 560.dp)
                    .clip(RoundedCornerShape(DoweDesign.radiusBox))
                    .background(backgroundColor)
                    .padding(12.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Text(text = placeholder, color = contentColor.copy(alpha = 0.56f), fontSize = 15.sp)
                Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(contentColor.copy(alpha = 0.12f)))
                CompositionLocalProvider(LocalContentColor provides contentColor) {
                    content()
                }
                if (showFooter) {
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                        Text(text = "Esc $closeText", color = contentColor.copy(alpha = 0.6f), fontSize = 12.sp)
                        Text(text = "Ctrl+${shortcut.uppercase()} $toggleText", color = accentColor, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                    }
                }
            }
        }
    }
}

private data class DoweTabItem(val id: String, val label: String)

@Composable
private fun DoweTabs(items: List<DoweTabItem>, initialId: String, modifier: Modifier, position: String, variant: String, backgroundColor: Color, contentColor: Color, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, borderColor: Color?, radius: Dp, fontFamily: FontFamily, content: @Composable (String) -> Unit) {
    var activeId by remember(initialId) { mutableStateOf(initialId) }
    val vertical = position == "start" || position == "end"
    val listShape = RoundedCornerShape(if (variant == "pills") 999.dp else radius)
    val listModifier = Modifier
        .clip(listShape)
        .background(backgroundColor)
        .then(if (borderColor == null || variant == "line") Modifier else Modifier.border(1.dp, borderColor, listShape))
        .padding(if (variant == "line" || variant == "ghost") 0.dp else 4.dp)
    val tabList: @Composable () -> Unit = {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            if (vertical) {
                Column(modifier = listModifier, verticalArrangement = Arrangement.spacedBy(if (variant == "line") 16.dp else 8.dp)) {
                    items.forEach { item ->
                        DoweTabButton(item = item, active = activeId == item.id, variant = variant, activeBackgroundColor = activeBackgroundColor, activeContentColor = activeContentColor, accentColor = accentColor, radius = radius, fontFamily = fontFamily) {
                            activeId = item.id
                        }
                    }
                }
            } else {
                Row(modifier = listModifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(if (variant == "line") 16.dp else 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    items.forEach { item ->
                        DoweTabButton(item = item, active = activeId == item.id, variant = variant, activeBackgroundColor = activeBackgroundColor, activeContentColor = activeContentColor, accentColor = accentColor, radius = radius, fontFamily = fontFamily) {
                            activeId = item.id
                        }
                    }
                }
            }
        }
    }
    val panel: @Composable () -> Unit = {
        Box(modifier = if (vertical) Modifier else Modifier.fillMaxWidth()) {
            content(activeId)
        }
    }
    when (position) {
        "bottom" -> Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
            panel()
            tabList()
        }
        "start" -> Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            tabList()
            panel()
        }
        "end" -> Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            panel()
            tabList()
        }
        else -> Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
            tabList()
            panel()
        }
    }
}

@Composable
private fun DoweTabButton(item: DoweTabItem, active: Boolean, variant: String, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, radius: Dp, fontFamily: FontFamily, onClick: () -> Unit) {
    val shape = RoundedCornerShape(if (variant == "pills") 999.dp else radius)
    val selectedFill = variant == "solid" || variant == "outlined" || variant == "pills"
    val selectedLine = variant == "line"
    val background = if (active && selectedFill) activeBackgroundColor else Color.Transparent
    val color = if (!active) LocalContentColor.current else if (selectedFill) activeContentColor else accentColor
    val border = if (active && selectedLine) BorderStroke(2.dp, accentColor) else null
    Box(
        modifier = Modifier
            .clip(shape)
            .background(background)
            .then(if (border == null) Modifier else Modifier.border(border, shape))
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(text = item.label, color = color, fontFamily = fontFamily)
    }
}

@Composable
private fun DoweSideNavRow(modifier: Modifier = Modifier, active: Boolean, wide: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, gap: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, content: @Composable RowScope.() -> Unit) {
    val shape = RoundedCornerShape(DoweDesign.radiusUi)
    val surface = modifier
        .then(if (wide) Modifier.fillMaxWidth() else Modifier)
        .clip(shape)
        .background(if (active) backgroundColor else Color.Transparent)
        .then(if (active && borderColor != null) Modifier.border(1.dp, borderColor, shape) else Modifier)
        .then(if (onClick == null) Modifier else Modifier.clickable(onClick = onClick))
        .padding(horizontal = paddingHorizontal, vertical = paddingVertical)
    CompositionLocalProvider(LocalContentColor provides if (active) contentColor else LocalContentColor.current) {
        Row(modifier = surface, horizontalArrangement = Arrangement.spacedBy(gap), verticalAlignment = Alignment.CenterVertically, content = content)
    }
}

@Composable
private fun DoweSideNavSubmenu(open: Boolean, trigger: @Composable ((() -> Unit) -> Unit), content: @Composable () -> Unit) {
    var expanded by remember { mutableStateOf(open) }
    Column {
        trigger { expanded = !expanded }
        AnimatedVisibility(
            visible = expanded,
            enter = fadeIn(animationSpec = tween(160)) + expandVertically(animationSpec = tween(180)),
            exit = fadeOut(animationSpec = tween(120)) + shrinkVertically(animationSpec = tween(180))
        ) {
            Column(modifier = Modifier.padding(start = 16.dp)) {
                content()
            }
        }
    }
}

@Composable
private fun DoweNavMenu(modifier: Modifier = Modifier, gap: Dp, popoverBackgroundColor: Color, popoverContentColor: Color, content: @Composable RowScope.(Int?, (Int) -> Unit) -> Unit, popover: @Composable (Int?) -> Unit) {
    var openIndex by remember { mutableStateOf<Int?>(null) }
    Column(modifier = modifier) {
        Row(horizontalArrangement = Arrangement.spacedBy(gap), verticalAlignment = Alignment.CenterVertically) {
            content(openIndex) { index -> openIndex = if (openIndex == index) null else index }
        }
        if (openIndex != null) {
            Popup(onDismissRequest = { openIndex = null }, properties = PopupProperties(focusable = true)) {
                Card(
                    colors = CardDefaults.cardColors(containerColor = popoverBackgroundColor, contentColor = popoverContentColor),
                    shape = RoundedCornerShape(DoweDesign.radiusBox),
                    elevation = CardDefaults.cardElevation(defaultElevation = 8.dp)
                ) {
                    Column(modifier = Modifier.widthIn(min = 192.dp, max = 720.dp).heightIn(max = 640.dp).padding(8.dp)) {
                        popover(openIndex)
                    }
                }
            }
        }
    }
}

@Composable
private fun DoweNavMenuItem(active: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, content: @Composable RowScope.() -> Unit) {
    val shape = RoundedCornerShape(DoweDesign.radiusBox)
    val surface = Modifier
        .clip(shape)
        .background(if (active) backgroundColor else Color.Transparent)
        .then(if (active && borderColor != null) Modifier.border(1.dp, borderColor, shape) else Modifier)
        .then(if (onClick == null) Modifier else Modifier.clickable(onClick = onClick))
        .padding(horizontal = paddingHorizontal, vertical = paddingVertical)
    CompositionLocalProvider(LocalContentColor provides if (active) contentColor else LocalContentColor.current) {
        Row(modifier = surface, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically, content = content)
    }
}

@Composable
private fun DoweDrawer(open: Boolean, onClose: () -> Unit, position: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp, disableOverlayClose: Boolean, hideCloseButton: Boolean, content: @Composable () -> Unit) {
    if (!open) {
        return
    }
    Popup(onDismissRequest = onClose, properties = PopupProperties(focusable = true)) {
        Box(modifier = Modifier.fillMaxSize()) {
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .background(Color.Black.copy(alpha = 0.48f))
                    .clickable(enabled = !disableOverlayClose, onClick = onClose)
            )
            val panelModifier = if (position == "start" || position == "end") {
                Modifier.fillMaxHeight().widthIn(max = 320.dp)
            } else {
                Modifier.fillMaxWidth().heightIn(max = 320.dp)
            }
            val shape = doweDrawerShape(position, radius)
            Box(
                modifier = panelModifier
                    .align(doweDrawerAlignment(position))
                    .clip(shape)
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            ) {
                CompositionLocalProvider(LocalContentColor provides contentColor) {
                    content()
                }
                if (!hideCloseButton) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(8.dp)
                            .clip(RoundedCornerShape(999.dp))
                            .background(DoweDesign.softMuted)
                            .clickable(onClick = onClose)
                            .width(28.dp)
                            .height(28.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(text = "x", color = DoweDesign.onSoftMuted)
                    }
                }
            }
        }
    }
}

private fun doweDrawerAlignment(position: String): Alignment =
    when (position) {
        "end" -> Alignment.CenterEnd
        "top" -> Alignment.TopCenter
        "bottom" -> Alignment.BottomCenter
        else -> Alignment.CenterStart
    }

private fun doweDrawerShape(position: String, radius: Dp): RoundedCornerShape =
    when (position) {
        "end" -> RoundedCornerShape(topStart = radius, topEnd = 0.dp, bottomEnd = 0.dp, bottomStart = radius)
        "top" -> RoundedCornerShape(topStart = 0.dp, topEnd = 0.dp, bottomEnd = radius, bottomStart = radius)
        "bottom" -> RoundedCornerShape(topStart = radius, topEnd = radius, bottomEnd = 0.dp, bottomStart = 0.dp)
        else -> RoundedCornerShape(topStart = 0.dp, topEnd = radius, bottomEnd = radius, bottomStart = 0.dp)
    }

@Composable
private fun DoweSectionBackgroundBox(modifier: Modifier = Modifier, background: DoweSectionBackground?, content: @Composable () -> Unit) {
    val backgroundModifier = if (background == null) Modifier else Modifier.background(doweSectionBackgroundBrush(background))
    Box(modifier = modifier.then(backgroundModifier).clipToBounds()) {
        content()
    }
}

private fun doweSectionBackgroundBrush(background: DoweSectionBackground): Brush =
    when (background) {
        DoweSectionBackground.Soft -> Brush.linearGradient(listOf(DoweDesign.surface, DoweDesign.background))
        DoweSectionBackground.Aurora -> Brush.linearGradient(listOf(DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary))
        DoweSectionBackground.Sunrise -> Brush.linearGradient(listOf(DoweDesign.softWarning, DoweDesign.softDanger, DoweDesign.surface))
        DoweSectionBackground.Ocean -> Brush.linearGradient(listOf(DoweDesign.softInfo, DoweDesign.softPrimary, DoweDesign.softTertiary))
        DoweSectionBackground.Meadow -> Brush.linearGradient(listOf(DoweDesign.softSuccess, DoweDesign.softTertiary, DoweDesign.surface))
        DoweSectionBackground.Slate -> Brush.linearGradient(listOf(DoweDesign.softMuted, DoweDesign.surface, DoweDesign.background))
    }

@Composable
private fun DoweCoverBox(modifier: Modifier = Modifier, source: String?, overlay: DoweOverlay?, content: @Composable () -> Unit) {
    Box(modifier = modifier.clipToBounds()) {
        if (source != null) {
            AndroidView(
                modifier = Modifier.matchParentSize(),
                factory = { context ->
                    ImageView(context).apply {
                        scaleType = ImageView.ScaleType.CENTER_CROP
                        setImageURI(Uri.parse(source))
                    }
                },
                update = { view ->
                    view.setImageURI(Uri.parse(source))
                }
            )
        }
        when (overlay) {
            is DoweOverlay.Solid -> Box(modifier = Modifier.matchParentSize().background(overlay.color))
            is DoweOverlay.Gradient -> Box(modifier = Modifier.matchParentSize().background(Brush.verticalGradient(listOf(overlay.start, overlay.end))))
            null -> {}
        }
        content()
    }
}

@Composable
private fun DoweGrid(modifier: Modifier = Modifier, columns: Int, horizontalGap: Dp, verticalGap: Dp, horizontalAlignment: Alignment.Horizontal, content: @Composable () -> Unit) {
    val density = LocalDensity.current
    Layout(content = content, modifier = modifier) { measurables, constraints ->
        val columnCount = columns.coerceAtLeast(1)
        val horizontal = with(density) { horizontalGap.roundToPx() }
        val vertical = with(density) { verticalGap.roundToPx() }
        val cellWidth = ((constraints.maxWidth - horizontal * (columnCount - 1)).coerceAtLeast(0)) / columnCount
        val placeables = measurables.map { it.measure(constraints.copy(minWidth = 0, maxWidth = cellWidth)) }
        val rowHeights = placeables.chunked(columnCount).map { row -> row.maxOfOrNull { it.height } ?: 0 }
        val height = rowHeights.sum() + vertical * (rowHeights.size - 1).coerceAtLeast(0)
        layout(constraints.maxWidth, height.coerceIn(constraints.minHeight, constraints.maxHeight)) {
            var top = 0
            placeables.chunked(columnCount).forEachIndexed { rowIndex, row ->
                row.forEachIndexed { columnIndex, placeable ->
                    val offset = horizontalAlignment.align(placeable.width, cellWidth, layoutDirection)
                    placeable.placeRelative(columnIndex * (cellWidth + horizontal) + offset, top)
                }
                top += rowHeights[rowIndex] + vertical
            }
        }
    }
}

private data class DoweSelectOption(val value: String, val label: String, val description: String?)

@Composable
private fun DoweInput(value: String, onValueChange: (String) -> Unit, modifier: Modifier, label: String?, placeholder: String, floating: Boolean, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var focused by remember { mutableStateOf(false) }
    val active = focused || value.isNotEmpty()
    val surface = modifier
        .heightIn(min = minHeight)
        .clip(shape)
        .background(backgroundColor)
        .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
        .padding(horizontal = horizontalPadding)
        .onFocusChanged { focused = it.isFocused }
    Column {
        if (label != null && !floating) {
            Text(text = label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = surface,
            singleLine = true,
            textStyle = TextStyle(fontFamily = fontFamily, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.Normal, color = contentColor),
            decorationBox = { innerTextField ->
                Box(modifier = Modifier.fillMaxSize()) {
                    if (placeholder.isNotEmpty() && value.isEmpty() && (!floating || active)) {
                        Text(text = placeholder, modifier = Modifier.align(Alignment.CenterStart), fontSize = fontSize, color = contentColor.copy(alpha = 0.55f), fontFamily = fontFamily)
                    }
                    if (label != null && floating) {
                        Text(text = label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else fontSize, color = contentColor, fontFamily = fontFamily)
                    }
                    Box(modifier = Modifier.align(Alignment.CenterStart).padding(top = if (label != null && floating) 10.dp else 0.dp)) {
                        innerTextField()
                    }
                }
            }
        )
    }
}

@Composable
private fun DoweSelect(value: String, onValueChange: (String) -> Unit, bound: Boolean, modifier: Modifier, label: String?, placeholder: String, floating: Boolean, options: List<DoweSelectOption>, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var expanded by remember { mutableStateOf(false) }
    var popupMounted by remember { mutableStateOf(false) }
    var localValue by remember { mutableStateOf("") }
    val selectedValue = if (bound) value else localValue
    val selected = options.firstOrNull { it.value == selectedValue }
    val active = expanded || selected != null
    val popupOffset = with(LocalDensity.current) { IntOffset(0, (minHeight + 4.dp).roundToPx()) }
    LaunchedEffect(expanded) {
        if (expanded) {
            popupMounted = true
        } else if (popupMounted) {
            delay(160)
            popupMounted = false
        }
    }
    Column {
        if (label != null && !floating) {
            Text(text = label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box(modifier = modifier) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(min = minHeight)
                    .clip(shape)
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
                    .clickable { expanded = true }
                    .padding(horizontal = horizontalPadding),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Box(modifier = Modifier.weight(1f)) {
                    if (label != null && floating) {
                        Text(text = label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else fontSize, color = contentColor, fontFamily = fontFamily)
                    }
                    if (selected != null || !floating || expanded) {
                        Text(text = selected?.label ?: placeholder, modifier = Modifier.align(Alignment.CenterStart).padding(top = if (label != null && floating && active) 10.dp else 0.dp), fontSize = fontSize, lineHeight = lineHeight, color = contentColor, fontFamily = fontFamily, maxLines = 1)
                    }
                }
                DoweSvg(viewBox = doweSelectArrowViewBox, modifier = Modifier.width(16.dp).height(16.dp), color = contentColor, paths = doweSelectArrowPaths)
            }
            if (expanded || popupMounted) {
                DoweSelectPopover(
                    visible = expanded,
                    options = options,
                    selectedValue = selectedValue,
                    offset = popupOffset,
                    shape = shape,
                    accentColor = contentColor,
                    fontFamily = fontFamily,
                    fontSize = fontSize,
                    lineHeight = lineHeight,
                    onDismiss = { expanded = false },
                    onSelect = { option ->
                        localValue = option.value
                        onValueChange(option.value)
                        expanded = false
                    }
                )
            }
        }
    }
}

@Composable
private fun DoweSelectPopover(visible: Boolean, options: List<DoweSelectOption>, selectedValue: String, offset: IntOffset, shape: RoundedCornerShape, accentColor: Color, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, onDismiss: () -> Unit, onSelect: (DoweSelectOption) -> Unit) {
    val progress by animateFloatAsState(
        targetValue = if (visible) 1f else 0f,
        animationSpec = tween(durationMillis = 160)
    )
    Popup(alignment = Alignment.TopStart, offset = offset, onDismissRequest = onDismiss, properties = PopupProperties(focusable = true)) {
        Column(
            modifier = Modifier
                .widthIn(min = 220.dp)
                .graphicsLayer {
                    alpha = progress
                    translationY = (1f - progress) * -4f
                    val value = 0.98f + (0.02f * progress)
                    scaleX = value
                    scaleY = value
                }
                .clip(shape)
                .background(DoweDesign.surface)
                .border(1.dp, DoweDesign.onSurface.copy(alpha = 0.08f), shape)
                .padding(vertical = 4.dp)
        ) {
            options.forEach { option ->
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(if (option.value == selectedValue) accentColor.copy(alpha = 0.08f) else Color.Transparent)
                        .clickable { onSelect(option) }
                        .padding(horizontal = 16.dp, vertical = 10.dp)
                ) {
                    Text(text = option.label, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, color = DoweDesign.onSurface, fontFamily = fontFamily)
                    if (option.description != null) {
                        Text(text = option.description, fontSize = 12.sp, color = DoweDesign.onSurface.copy(alpha = 0.68f), fontFamily = fontFamily)
                    }
                }
            }
        }
    }
}

private fun <T> doweResponsive(viewportWidth: Dp, xs: T? = null, sm: T? = null, md: T? = null, lg: T? = null, xl: T? = null): T? {
    var value: T? = null
    if (viewportWidth >= 0.dp && xs != null) {
        value = xs
    }
    if (viewportWidth >= 640.dp && sm != null) {
        value = sm
    }
    if (viewportWidth >= 768.dp && md != null) {
        value = md
    }
    if (viewportWidth >= 1024.dp && lg != null) {
        value = lg
    }
    if (viewportWidth >= 1280.dp && xl != null) {
        value = xl
    }
    return value
}

private fun doweTextSize(viewportWidth: Dp, min: Float, preferredBase: Float, preferredViewport: Float, max: Float): TextUnit {
    return (preferredBase + viewportWidth.value * preferredViewport / 100f).coerceIn(min, max).sp
}

private fun doweTextLineHeight(fontSize: TextUnit, lineHeight: Float): TextUnit {
    return (fontSize.value * lineHeight).sp
}

private fun Modifier.doweBackground(value: Color?): Modifier =
    if (value == null) this else background(value)

private fun Modifier.dowePadding(all: Dp?, horizontal: Dp?, vertical: Dp?, start: Dp?, end: Dp?, top: Dp?, bottom: Dp?): Modifier {
    var modifier = this
    if (all != null) {
        modifier = modifier.padding(all)
    }
    if (horizontal != null) {
        modifier = modifier.padding(horizontal = horizontal)
    }
    if (vertical != null) {
        modifier = modifier.padding(vertical = vertical)
    }
    if (start != null || end != null || top != null || bottom != null) {
        modifier = modifier.padding(start = start ?: 0.dp, end = end ?: 0.dp, top = top ?: 0.dp, bottom = bottom ?: 0.dp)
    }
    return modifier
}

private fun Modifier.doweWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> width(value.value)
        DoweSize.Full -> fillMaxWidth()
        null -> this
    }

private fun Modifier.doweHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> height(value.value)
        DoweSize.Full -> fillMaxHeight()
        null -> this
    }

private fun Modifier.doweMinWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> widthIn(min = value.value)
        else -> this
    }

private fun Modifier.doweMinHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> heightIn(min = value.value)
        else -> this
    }

private fun Modifier.doweRounded(radius: Dp?): Modifier =
    if (radius == null) this else clip(RoundedCornerShape(radius))

private fun Modifier.doweBorder(width: Dp?, radius: Dp?): Modifier =
    if (width == null) this else border(width, DoweDesign.onBackground, RoundedCornerShape(radius ?: DoweDesign.radius))

private fun doweHorizontalAlignment(value: DoweAlign?): Alignment.Horizontal =
    when (value) {
        DoweAlign.Center, DoweAlign.Stretch, DoweAlign.Baseline -> Alignment.CenterHorizontally
        DoweAlign.End -> Alignment.End
        else -> Alignment.Start
    }

private fun doweGridHorizontalAlignment(value: DoweAlign?): Alignment.Horizontal =
    when (value) {
        DoweAlign.Center -> Alignment.CenterHorizontally
        DoweAlign.End -> Alignment.End
        else -> Alignment.Start
    }

private fun doweVerticalAlignment(value: DoweAlign?): Alignment.Vertical =
    when (value) {
        DoweAlign.Center, DoweAlign.Stretch -> Alignment.CenterVertically
        DoweAlign.End -> Alignment.Bottom
        else -> Alignment.Top
    }

private fun doweHorizontalArrangement(value: DoweJustify?, gap: Dp?): Arrangement.Horizontal =
    when (value) {
        DoweJustify.Center -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.CenterHorizontally)
        DoweJustify.End -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.End)
        DoweJustify.Between -> Arrangement.SpaceBetween
        DoweJustify.Around -> Arrangement.SpaceAround
        DoweJustify.Evenly -> Arrangement.SpaceEvenly
        else -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Start)
    }

private fun doweVerticalArrangement(value: DoweJustify?, gap: Dp?): Arrangement.Vertical =
    when (value) {
        DoweJustify.Center -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.CenterVertically)
        DoweJustify.End -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Bottom)
        DoweJustify.Between -> Arrangement.SpaceBetween
        DoweJustify.Around -> Arrangement.SpaceAround
        DoweJustify.Evenly -> Arrangement.SpaceEvenly
        else -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Top)
    }

private data class DoweRouteEntry(val path: String, val fragment: String?)

@Composable
fun DoweApp(startPath: String = DoweRoutes.initialPath, startFragment: String? = null, navigationRequest: Int = 0) {
"#,
    );
    output = output.replace(
        "__DOWE_DESIGN__",
        &android_design_block(design_config.default_theme()),
    );
    replace_android_font_support(&mut output, font_config, font_families);

    if routes.first().is_some() {
        output.push_str(
            r#"    val initialPath = if (DoweRoutes.paths.contains(startPath)) startPath else DoweRoutes.initialPath
    val initialFragment = startFragment?.takeIf { DoweRoutes.sections[initialPath]?.contains(it) == true }
    var currentEntry by remember { mutableStateOf(DoweRouteEntry(initialPath, initialFragment)) }
    var externalUrl by remember { mutableStateOf<String?>(null) }
    val backStack = remember { mutableStateListOf<DoweRouteEntry>() }
    val context = LocalContext.current
    val scrollState = rememberScrollState()
    val sectionRegistry = remember(currentEntry.path) { DoweSectionRegistry() }
    val targetSection = currentEntry.fragment?.let { sectionRegistry.positions[it] }
    fun navigate(operation: String, target: String, fragment: String?) {
        val path = target.ifEmpty { currentEntry.path }
        if (!DoweRoutes.paths.contains(path)) {
            return
        }
        val destination = DoweRouteEntry(path, fragment?.takeIf { DoweRoutes.sections[path]?.contains(it) == true })
        if (destination == currentEntry) {
            return
        }
        if (operation == "replace") {
            currentEntry = destination
        } else {
            backStack.add(currentEntry)
            currentEntry = destination
        }
    }
    fun goBack() {
        if (externalUrl != null) {
            externalUrl = null
        } else if (backStack.isNotEmpty()) {
            currentEntry = backStack.removeAt(backStack.lastIndex)
        } else if (currentEntry.path != DoweRoutes.initialPath || currentEntry.fragment != null) {
            currentEntry = DoweRouteEntry(DoweRoutes.initialPath, null)
        }
    }
    fun openExternal(mode: String, target: String) {
        if (mode == "webview") {
            externalUrl = target
        } else {
            context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(target)))
        }
    }
    LaunchedEffect(navigationRequest) {
        navigate("replace", initialPath, initialFragment)
    }
    LaunchedEffect(currentEntry.path, currentEntry.fragment, targetSection) {
        if (targetSection != null) {
            scrollState.animateScrollTo(targetSection)
        }
    }
    BackHandler(enabled = true) {
        goBack()
    }
    Box(modifier = Modifier.fillMaxSize().background(DoweDesign.background)) {
        if (externalUrl != null) {
            AndroidView(
                modifier = Modifier.fillMaxSize().safeDrawingPadding(),
                factory = { WebView(it).apply { loadUrl(externalUrl ?: "") } },
                update = {
                    if (it.url != externalUrl) {
                        it.loadUrl(externalUrl ?: "")
                    }
                }
"#,
        );
        output.push_str(
            r#"            )
        } else {
            BoxWithConstraints(modifier = Modifier.fillMaxSize().safeDrawingPadding().verticalScroll(scrollState), contentAlignment = Alignment.TopStart) {
                when (currentEntry.path) {
"#,
        );
        for route in routes {
            output.push_str(&format!(
                "                    \"{}\" -> {}(maxWidth, sectionRegistry, ::navigate, ::goBack, ::openExternal)\n",
                route.route_path,
                compose_screen_name(&route.route_path)
            ));
        }
        if let Some(route) = routes.first() {
            output.push_str(&format!(
                "                    else -> {}(maxWidth, sectionRegistry, ::navigate, ::goBack, ::openExternal)\n",
                compose_screen_name(&route.route_path)
            ));
        }
        output.push_str("                }\n            }\n        }\n    }\n");
    } else {
        output.push_str("    Column {\n    }\n");
    }

    output.push_str("}\n");
    output.push_str(compose_reactive_runtime());

    for route in routes {
        output.push('\n');
        output.push_str("@Composable\n");
        output.push_str(&format!(
            "fun {}(viewportWidth: Dp, sectionRegistry: DoweSectionRegistry, navigate: (String, String, String?) -> Unit, goBack: () -> Unit, openExternal: (String, String) -> Unit) {{\n",
            compose_screen_name(&route.route_path)
        ));
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let reactive = compose_reactive_route(&tree);
        output.push_str(&format!(
            "    val activePath = \"{}\"\n    val state = remember {{ DoweReactiveState(initial = {}, actions = {}) }}\n    val actionScope = rememberCoroutineScope()\n",
            escape_kotlin(&route.route_path),
            reactive.initial,
            reactive.actions
        ));
        for id in &reactive.autoload {
            output.push_str(&format!(
                "    LaunchedEffect(\"{}\") {{ state.run(\"{}\") }}\n",
                escape_kotlin(id),
                escape_kotlin(id)
            ));
        }
        render_compose_node(&tree, 4, &mut output, font_config.default_family);
        output.push_str("}\n");
    }

    output
}
