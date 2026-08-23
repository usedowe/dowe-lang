fn android_runtime_media_color() -> &'static str {
    r##"private fun DoweColorField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, fontSize: TextUnit, lineHeight: TextUnit, name: String?, helpText: String?, errorText: String?, showHex: Boolean, showRgb: Boolean, showCmyk: Boolean, showOklch: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var expanded by remember { mutableStateOf(false) }
    var hsv by remember(value) { mutableStateOf(doweColorHsv(doweColorRgb(value))) }
    val canonical = doweColorHex(doweColorRgb(value))
    val active = expanded || canonical.isNotEmpty()
    val popupOffset = with(LocalDensity.current) { (doweControlHeight(size) + if (floating) 12.dp else 4.dp).roundToPx() }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(doweControlHeight(size) + if (floating) 8.dp else 0.dp)
                    .clip(RoundedCornerShape(DoweDesign.radius))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(DoweDesign.radius)))
                    .clickable { expanded = !expanded }
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                if (label != null && floating) {
                    DoweColorSwatch(canonical, size, contentColor)
                    Box(modifier = Modifier.weight(1f)) {
                        Text(label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else 14.sp, color = contentColor)
                        Text(text = canonical.ifEmpty { placeholder }, modifier = Modifier.align(Alignment.CenterStart).padding(top = if (active) 10.dp else 0.dp), color = contentColor, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, maxLines = 1)
                    }
                } else {
                    DoweColorTriggerContent(canonical, placeholder, size, fontSize, lineHeight, contentColor, Modifier.weight(1f))
                }
            }
            DoweAnchoredPopover(visible = expanded, offset = IntOffset(0, popupOffset), shape = RoundedCornerShape(12.dp), backgroundColor = DoweDesign.background, contentColor = DoweDesign.backgroundText, contentPadding = PaddingValues(16.dp), maxHeight = 480.dp, onDismiss = { expanded = false }) {
                DoweColorPickerPanel(value = canonical, hsv = hsv, onHsvChange = { next -> hsv = next; onValueChange(doweColorHex(doweColorFromHsv(next))) }, showHex = showHex, showRgb = showRgb, showCmyk = showCmyk, showOklch = showOklch)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweColorTriggerContent(value: String, placeholder: String, size: String, fontSize: TextUnit, lineHeight: TextUnit, contentColor: Color, modifier: Modifier) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        DoweColorSwatch(value, size, contentColor)
        Text(text = value.ifEmpty { placeholder }, color = contentColor, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

@Composable
private fun DoweColorSwatch(value: String, size: String, contentColor: Color) {
    Box(modifier = Modifier.size(doweControlSwatchSize(size)).clip(RoundedCornerShape(6.dp)).background(doweHexColor(value, DoweDesign.primary)).border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(6.dp)))
}

private data class DoweColorHsv(val hue: Float, val saturation: Float, val brightness: Float)
private data class DoweColorRgb(val red: Int, val green: Int, val blue: Int)

@Composable
private fun DoweColorPickerPanel(value: String, hsv: DoweColorHsv, onHsvChange: (DoweColorHsv) -> Unit, showHex: Boolean, showRgb: Boolean, showCmyk: Boolean, showOklch: Boolean) {
    val rgb = doweColorRgb(value)
    Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxWidth()
                .height(140.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(Brush.horizontalGradient(listOf(Color.White, Color.hsv(hsv.hue, 1f, 1f))))
                .pointerInput(hsv.hue) {
                    awaitEachGesture {
                        var change = awaitFirstDown()
                        do {
                            val saturation = (change.position.x / size.width).coerceIn(0f, 1f)
                            val brightness = (1f - change.position.y / size.height).coerceIn(0f, 1f)
                            onHsvChange(hsv.copy(saturation = saturation, brightness = brightness))
                            val event = awaitPointerEvent()
                            change = event.changes.first()
                            change.consume()
                        } while (change.pressed)
                    }
                }
                .semantics { contentDescription = "Saturation ${(hsv.saturation * 100).roundToInt()} percent, brightness ${(hsv.brightness * 100).roundToInt()} percent" }
        ) {
            Box(modifier = Modifier.matchParentSize().background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black))))
            Box(modifier = Modifier.offset(x = maxWidth * hsv.saturation - 8.dp, y = maxHeight * (1f - hsv.brightness) - 8.dp).size(16.dp).clip(RoundedCornerShape(999.dp)).background(doweHexColor(value, DoweDesign.primary)).border(2.dp, Color.White, RoundedCornerShape(999.dp)))
        }
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxWidth()
                .height(16.dp)
                .clip(RoundedCornerShape(999.dp))
                .background(Brush.horizontalGradient(listOf(Color.Red, Color.Yellow, Color.Green, Color.Cyan, Color.Blue, Color.Magenta, Color.Red)))
                .pointerInput(Unit) {
                    awaitEachGesture {
                        var change = awaitFirstDown()
                        do {
                            onHsvChange(hsv.copy(hue = (change.position.x / size.width * 360f).coerceIn(0f, 360f)))
                            val event = awaitPointerEvent()
                            change = event.changes.first()
                            change.consume()
                        } while (change.pressed)
                    }
                }
                .semantics { contentDescription = "Hue ${hsv.hue.roundToInt()} degrees" }
        ) {
            Box(modifier = Modifier.offset(x = maxWidth * (hsv.hue / 360f) - 10.dp, y = (-2).dp).size(20.dp).clip(RoundedCornerShape(999.dp)).background(Color.White).border(1.dp, DoweDesign.muted.copy(alpha = 0.3f), RoundedCornerShape(999.dp)))
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Box(modifier = Modifier.size(48.dp).clip(RoundedCornerShape(8.dp)).background(doweHexColor(value, DoweDesign.primary)).border(1.dp, DoweDesign.backgroundText.copy(alpha = 0.22f), RoundedCornerShape(8.dp)))
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(value, color = DoweDesign.backgroundText, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Text("Foreground: ${doweColorForeground(rgb)}", color = DoweDesign.backgroundText.copy(alpha = 0.72f), fontSize = 12.sp)
            }
        }
        if (showHex || showRgb || showCmyk || showOklch) {
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                if (showHex) DoweColorFormatRow("hex: $value")
                if (showRgb) DoweColorFormatRow("rgb: ${doweColorRgbText(rgb)}")
                if (showCmyk) DoweColorFormatRow("cmyk: ${doweColorCmykText(rgb)}")
                if (showOklch) DoweColorFormatRow("oklch: ${doweColorOklchText(rgb)}")
            }
        }
    }
}

@Composable
private fun DoweColorFormatRow(value: String) {
    Text(value, modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(DoweDesign.muted).padding(horizontal = 8.dp, vertical = 4.dp), color = DoweDesign.mutedText, fontSize = 12.sp, maxLines = 1)
}

private fun doweColorRgb(value: String): DoweColorRgb {
    val source = value.removePrefix("#")
    val clean = if (source.length == 3) source.map { "${it}${it}" }.joinToString("") else source
    val number = clean.takeIf { it.length == 6 }?.toLongOrNull(16) ?: 0x3B82F6
    return DoweColorRgb(((number shr 16) and 255).toInt(), ((number shr 8) and 255).toInt(), (number and 255).toInt())
}

private fun doweColorHex(rgb: DoweColorRgb): String = String.format(Locale.US, "#%02X%02X%02X", rgb.red, rgb.green, rgb.blue)

private fun doweColorHsv(rgb: DoweColorRgb): DoweColorHsv {
    val result = FloatArray(3)
    AndroidColor.RGBToHSV(rgb.red, rgb.green, rgb.blue, result)
    return DoweColorHsv(result[0], result[1], result[2])
}

private fun doweColorFromHsv(hsv: DoweColorHsv): DoweColorRgb {
    val color = AndroidColor.HSVToColor(floatArrayOf(hsv.hue, hsv.saturation, hsv.brightness))
    return DoweColorRgb(AndroidColor.red(color), AndroidColor.green(color), AndroidColor.blue(color))
}

private fun doweColorRgbText(rgb: DoweColorRgb): String = "rgb(${rgb.red}, ${rgb.green}, ${rgb.blue})"

private fun doweColorCmykText(rgb: DoweColorRgb): String {
    val values = listOf(rgb.red / 255.0, rgb.green / 255.0, rgb.blue / 255.0)
    val black = 1 - (values.maxOrNull() ?: 0.0)
    if (black >= 1) return "cmyk(0%, 0%, 0%, 100%)"
    val channels = values.map { ((1 - it - black) / (1 - black) * 100).roundToInt() }
    return "cmyk(${channels[0]}%, ${channels[1]}%, ${channels[2]}%, ${(black * 100).roundToInt()}%)"
}

private fun doweColorOklchText(rgb: DoweColorRgb): String {
    fun linear(value: Int): Double { val channel = value / 255.0; return if (channel <= 0.04045) channel / 12.92 else ((channel + 0.055) / 1.055).pow(2.4) }
    val red = linear(rgb.red); val green = linear(rgb.green); val blue = linear(rgb.blue)
    val l = (0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue).pow(1.0 / 3.0)
    val m = (0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue).pow(1.0 / 3.0)
    val s = (0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue).pow(1.0 / 3.0)
    val lightness = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s
    val a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s
    val b = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s
    val chroma = sqrt(a * a + b * b)
    var hue = atan2(b, a) * 180 / Math.PI
    if (hue < 0) hue += 360
    return String.format(Locale.US, "oklch(%.2f %.2f %.0f)", lightness, chroma, hue)
}

private fun doweColorForeground(rgb: DoweColorRgb): String = if ((0.299 * rgb.red + 0.587 * rgb.green + 0.114 * rgb.blue) / 255 > 0.5) "#000000" else "#FFFFFF"

@Composable
"##
}
