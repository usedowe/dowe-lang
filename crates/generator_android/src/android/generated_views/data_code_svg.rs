fn android_runtime_data_code_svg() -> &'static str {
    r#"@Composable
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

"#
}
