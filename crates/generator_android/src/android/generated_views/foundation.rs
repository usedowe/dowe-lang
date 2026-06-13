fn android_runtime_foundation() -> &'static str {
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

"#
}
