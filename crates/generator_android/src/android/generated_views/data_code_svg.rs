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
private fun DoweChart(state: DoweReactiveState, chartType: String, dataPath: String?, seriesPath: String?, palette: String, legendPosition: String, emptyLabel: String, loading: Boolean, hideLegend: Boolean, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?, donut: Boolean = false, donutWidth: Int = 60, centerLabel: String? = null, centerValue: String? = null, startAngle: Float = -90f, padAngle: Float = 0f, hideLabels: Boolean = false, hideValues: Boolean = false, hidePercentages: Boolean = false, showGlow: Boolean = false, centerText: String? = null, thickness: Int = 16, gap: Int = 8, endAngle: Int = 270, showInlineLabels: Boolean = false, arcHideValues: Boolean = false, arcShowGlow: Boolean = false) {
    val rows = dataPath?.let { state.candles(it) } ?: seriesPath?.let { state.candles(it).flatMap { row -> (row["data"] as? List<*>)?.mapNotNull { it as? Map<String, Any?> } ?: emptyList() } } ?: emptyList()
    val points = rows.mapNotNull(::doweChartPoint)
    val categories = rows.mapIndexedNotNull { index, row -> doweChartCategory(row, index) }
    Column(
        modifier = modifier
            .heightIn(min = if (chartType == "arc" || chartType == "pie") 224.dp else 300.dp)
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier.border(1.dp, contentColor.copy(alpha = 0.12f), shape) else Modifier.border(1.dp, borderColor, shape))
            .padding(12.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        var selectedArc by remember { mutableStateOf<Int?>(null) }
        val chartModifier = if (chartType == "pie" || chartType == "arc") {
            Modifier
                .fillMaxWidth()
                .widthIn(max = 320.dp)
                .aspectRatio(1f)
                .align(Alignment.CenterHorizontally)
        } else {
            Modifier.weight(1f).fillMaxWidth()
        }
        Box(modifier = chartModifier, contentAlignment = Alignment.Center) {
            Canvas(modifier = Modifier.matchParentSize().pointerInput(categories) {
                detectTapGestures { offset ->
                    if (chartType == "arc" && categories.isNotEmpty()) {
                        val center = Offset(size.width / 2f, size.height / 2f)
                        val radius = min(size.width, size.height) / 2f - 18f
                        val ringCount = categories.size.coerceAtLeast(1)
                        val ringGap = min(gap.toFloat().coerceAtLeast(0f), max(1f, radius / (ringCount * 3f)))
                        val stroke = max(6f, min(thickness.toFloat().coerceAtLeast(6f), (radius - ringGap * (ringCount - 1)) / (ringCount + 0.5f)))
                        val distance = sqrt((offset.x - center.x).pow(2f) + (offset.y - center.y).pow(2f))
                        selectedArc = categories.indices.minByOrNull { index -> abs(distance - max(stroke / 2f + 2f, radius - index * (stroke + ringGap))) }
                    }
                }
            }) {
                if (loading || (points.isEmpty() && categories.isEmpty())) {
                    return@Canvas
                }
                when (chartType) {
                    "line", "area" -> doweDrawPointChart(chartType, points, palette, contentColor)
                    "bar" -> doweDrawBarChart(categories, palette, contentColor)
                    "arc" -> doweDrawArcChart(categories, palette, contentColor, backgroundColor, thickness, gap, startAngle, endAngle.toFloat(), showInlineLabels, arcHideValues, arcShowGlow)
                    else -> doweDrawPieChart(categories, palette, contentColor, donut, donutWidth, startAngle, padAngle, hideLabels, hideValues, hidePercentages, showGlow)
                }
            }
            if (chartType == "pie" && !loading && categories.isNotEmpty() && (centerLabel != null || centerValue != null)) {
                Column(
                    modifier = Modifier
                        .background(backgroundColor.copy(alpha = 0.94f), RoundedCornerShape(999.dp))
                        .border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(999.dp))
                        .padding(horizontal = 12.dp, vertical = 8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    centerLabel?.let { Text(text = it, color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold) }
                    Text(text = centerValue ?: categories.sumOf { it.value.toDouble() }.toFloat().toString(), color = contentColor, fontSize = 24.sp, fontWeight = FontWeight.ExtraBold)
                }
            }
            selectedArc?.let { index ->
                if (chartType == "arc" && !loading && index < categories.size) {
                    val item = categories[index]
                    Text(text = item.label + if (arcHideValues) "" else " · " + item.value, modifier = Modifier.align(Alignment.TopCenter).padding(top = 4.dp).background(backgroundColor.copy(alpha = 0.96f), RoundedCornerShape(999.dp)).border(1.dp, contentColor.copy(alpha = 0.24f), RoundedCornerShape(999.dp)).padding(horizontal = 12.dp, vertical = 7.dp), color = contentColor, fontSize = 12.sp, fontWeight = FontWeight.Bold, maxLines = 1)
                }
            }
            if (chartType == "arc" && !loading && categories.isNotEmpty() && (centerText != null || centerValue != null)) {
                Column(
                    modifier = Modifier
                        .background(backgroundColor.copy(alpha = 0.94f), RoundedCornerShape(999.dp))
                        .border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(999.dp))
                        .padding(horizontal = 12.dp, vertical = 8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    centerText?.takeIf { it.isNotEmpty() }?.let { Text(text = it, color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold) }
                    centerValue?.takeIf { it.isNotEmpty() }?.let { Text(text = it, color = contentColor, fontSize = 26.sp, fontWeight = FontWeight.ExtraBold) }
                }
            }
            if (loading || (points.isEmpty() && categories.isEmpty())) {
                Text(text = if (loading) "Loading" else emptyLabel, color = contentColor.copy(alpha = 0.64f), fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
            }
        }
        DoweChartLegend(categories, palette, legendPosition, hideLegend, hideLabels, contentColor)
    }
}

@Composable
private fun DoweChartLegend(categories: List<DoweChartCategory>, palette: String, legendPosition: String, hideLegend: Boolean, hideLabels: Boolean, contentColor: Color) {
    if (hideLegend || legendPosition == "none" || categories.isEmpty()) {
        return
    }
    if (legendPosition == "left" || legendPosition == "right") {
        Column(modifier = Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
            categories.take(8).forEachIndexed { index, item ->
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Box(Modifier.width(10.dp).height(10.dp).background(doweChartColor(palette, index, item.color)))
                    if (!hideLabels) Text(text = item.label, color = contentColor.copy(alpha = 0.82f), fontSize = 12.sp, maxLines = 1)
                }
            }
        }
    } else {
        FlowRow(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterHorizontally), verticalArrangement = Arrangement.spacedBy(6.dp), itemVerticalAlignment = Alignment.CenterVertically) {
            categories.take(8).forEachIndexed { index, item ->
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Box(Modifier.width(10.dp).height(10.dp).background(doweChartColor(palette, index, item.color)))
                    if (!hideLabels) Text(text = item.label, color = contentColor.copy(alpha = 0.82f), fontSize = 12.sp, maxLines = 1)
                }
            }
        }
    }
}

private data class DoweChartPoint(val x: Float, val y: Float)

private data class DoweChartCategory(val label: String, val value: Float, val max: Float?, val color: String?)

private fun doweChartPoint(source: Map<String, Any?>): DoweChartPoint? {
    val x = doweCandleNumber(source["x"]) ?: return null
    val y = doweCandleNumber(source["y"]) ?: return null
    return DoweChartPoint(x, y)
}

private fun doweChartCategory(source: Map<String, Any?>, index: Int): DoweChartCategory? {
    val value = doweCandleNumber(source["value"]) ?: return null
    if (value < 0f) {
        return null
    }
    val max = doweCandleNumber(source["max"])?.takeIf { it > 0f }
    return DoweChartCategory(source["label"]?.toString() ?: (index + 1).toString(), value, max, source["color"]?.toString())
}

private fun doweChartColor(palette: String, index: Int, explicit: String?): Color {
    val colors = when (palette) {
        "rainbow" -> listOf("danger", "warning", "success", "info", "primary", "secondary", "muted")
        "ocean" -> listOf("info", "primary", "secondary", "success", "muted", "warning", "danger")
        "sunset" -> listOf("warning", "danger", "secondary", "primary", "info", "success", "muted")
        "forest" -> listOf("success", "primary", "info", "secondary", "muted", "warning", "danger")
        "neon" -> listOf("secondary", "primary", "success", "warning", "danger", "info", "muted")
        else -> listOf("primary", "secondary", "success", "info", "warning", "danger", "muted")
    }
    return when (explicit ?: colors[index % colors.size]) {
        "secondary" -> DoweDesign.secondary
        "success" -> DoweDesign.success
        "info" -> DoweDesign.info
        "warning" -> DoweDesign.warning
        "danger" -> DoweDesign.danger
        "muted" -> DoweDesign.muted
        else -> DoweDesign.primary
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.doweDrawPointChart(chartType: String, points: List<DoweChartPoint>, palette: String, contentColor: Color) {
    if (points.isEmpty()) {
        return
    }
    val left = 36f
    val top = 12f
    val right = 12f
    val bottom = 28f
    val width = max(1f, size.width - left - right)
    val height = max(1f, size.height - top - bottom)
    val minX = points.minOf { it.x }
    val maxX = points.maxOf { it.x }.let { if (it == minX) it + 1f else it }
    val minY = min(0f, points.minOf { it.y })
    val maxY = points.maxOf { it.y }.let { if (it == minY) it + 1f else it }
    for (line in 0..4) {
        val y = top + height * line / 4f
        drawLine(contentColor.copy(alpha = 0.14f), Offset(left, y), Offset(left + width, y), 1f)
    }
    val mapped = points.map {
        Offset(left + ((it.x - minX) / (maxX - minX)) * width, top + ((maxY - it.y) / (maxY - minY)) * height)
    }
    if (chartType == "area" && mapped.size > 1) {
        val path = androidx.compose.ui.graphics.Path()
        path.moveTo(mapped.first().x, top + height)
        mapped.forEachIndexed { index, point -> if (index == 0) path.lineTo(point.x, point.y) else path.lineTo(point.x, point.y) }
        path.lineTo(mapped.last().x, top + height)
        path.close()
        drawPath(path, doweChartColor(palette, 0, null).copy(alpha = 0.28f))
    }
    for (index in 1 until mapped.size) {
        drawLine(doweChartColor(palette, 0, null), mapped[index - 1], mapped[index], 2.5f)
    }
    mapped.forEach { drawCircle(doweChartColor(palette, 0, null), 3.5f, it) }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.doweDrawBarChart(items: List<DoweChartCategory>, palette: String, contentColor: Color) {
    if (items.isEmpty()) {
        return
    }
    val left = 36f
    val top = 12f
    val bottom = 28f
    val width = max(1f, size.width - left - 12f)
    val height = max(1f, size.height - top - bottom)
    val maxValue = max(1f, items.maxOf { it.value })
    for (line in 0..4) {
        val y = top + height * line / 4f
        drawLine(contentColor.copy(alpha = 0.14f), Offset(left, y), Offset(left + width, y), 1f)
    }
    val step = width / max(1, items.size)
    items.forEachIndexed { index, item ->
        val barHeight = height * (item.value / maxValue)
        drawRoundRect(
            color = doweChartColor(palette, index, item.color),
            topLeft = Offset(left + index * step + step * 0.18f, top + height - barHeight),
            size = Size(max(2f, step * 0.64f), max(1f, barHeight)),
            cornerRadius = CornerRadius(4f, 4f)
        )
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.doweDrawPieChart(items: List<DoweChartCategory>, palette: String, contentColor: Color, donut: Boolean, donutWidth: Int, startAngle: Float, padAngle: Float, hideLabels: Boolean, hideValues: Boolean, hidePercentages: Boolean, showGlow: Boolean) {
    val total = items.sumOf { it.value.toDouble() }.toFloat().takeIf { it > 0f } ?: return
    val diameter = min(size.width, size.height) - 24f
    val topLeft = Offset((size.width - diameter) / 2f, (size.height - diameter) / 2f)
    val ringWidth = min(donutWidth.toFloat(), diameter / 2f - 4f).coerceAtLeast(4f)
    var start = startAngle
    items.forEachIndexed { index, item ->
        val sweep = 360f * item.value / total
        val gap = min(padAngle, sweep * 0.45f)
        val color = doweChartColor(palette, index, item.color)
        if (showGlow) {
            if (donut) {
                drawArc(color.copy(alpha = 0.14f), start + gap / 2f, sweep - gap, false, topLeft, Size(diameter, diameter), style = androidx.compose.ui.graphics.drawscope.Stroke(width = ringWidth + 8f))
            } else {
                drawCircle(color.copy(alpha = 0.1f), radius = diameter / 2f + 4f, style = androidx.compose.ui.graphics.drawscope.Stroke(width = 8f))
            }
        }
        if (donut) {
            drawArc(color, start + gap / 2f, sweep - gap, false, topLeft, Size(diameter, diameter), style = androidx.compose.ui.graphics.drawscope.Stroke(width = ringWidth))
        } else {
            drawArc(color, start + gap / 2f, sweep - gap, true, topLeft, Size(diameter, diameter))
        }
        start += sweep
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.doweDrawArcChart(items: List<DoweChartCategory>, palette: String, contentColor: Color, backgroundColor: Color, thickness: Int, gap: Int, startAngle: Float, endAngle: Float, showInlineLabels: Boolean, hideValues: Boolean, showGlow: Boolean) {
    val total = items.sumOf { it.value.toDouble() }.toFloat().takeIf { it > 0f } ?: return
    val radius = min(size.width, size.height) / 2f - 18f
    val ringCount = items.size.coerceAtLeast(1)
    val ringGap = min(gap.toFloat().coerceAtLeast(0f), max(1f, radius / (ringCount * 3f)))
    val stroke = max(6f, min(thickness.toFloat().coerceAtLeast(6f), (radius - ringGap * (ringCount - 1)) / (ringCount + 0.5f)))
    val range = endAngle - startAngle
    val center = Offset(size.width / 2f, size.height / 2f)
    items.forEachIndexed { index, item ->
        val currentRadius = max(stroke / 2f + 2f, radius - index * (stroke + ringGap))
        val diameter = currentRadius * 2f
        val topLeft = Offset(center.x - currentRadius, center.y - currentRadius)
        val maxValue = item.max ?: total
        val progress = (item.value / maxValue).coerceIn(0f, 1f)
        val color = doweChartColor(palette, index, item.color)
        if (showGlow) {
            drawArc(color.copy(alpha = 0.14f), startAngle, range * progress, false, topLeft, Size(diameter, diameter), style = androidx.compose.ui.graphics.drawscope.Stroke(width = stroke + 8f))
        }
        drawArc(contentColor.copy(alpha = 0.16f), startAngle, range, false, topLeft, Size(diameter, diameter), style = androidx.compose.ui.graphics.drawscope.Stroke(width = stroke))
        drawArc(color, startAngle, range * progress, false, topLeft, Size(diameter, diameter), style = androidx.compose.ui.graphics.drawscope.Stroke(width = stroke))
        if (showInlineLabels) {
            val angle = Math.toRadians((startAngle + range * progress - 90f).toDouble())
            val labelRadius = currentRadius + stroke / 2f + 12f
            val x = center.x + labelRadius * kotlin.math.cos(angle).toFloat()
            val y = center.y + labelRadius * kotlin.math.sin(angle).toFloat()
            drawIntoCanvas { canvas ->
                val label = item.label + if (hideValues) "" else " ${item.value}"
                val paint = Paint().apply { textSize = 11f; textAlign = if (x < center.x) Paint.Align.RIGHT else if (x > center.x) Paint.Align.LEFT else Paint.Align.CENTER; isAntiAlias = true; typeface = android.graphics.Typeface.DEFAULT_BOLD }
                val textWidth = paint.measureText(label)
                val horizontalPadding = 6f
                val verticalPadding = 4f
                val textLeft = when (paint.textAlign) {
                    Paint.Align.RIGHT -> x - textWidth
                    Paint.Align.CENTER -> x - textWidth / 2f
                    else -> x
                }
                val clampedLeft = textLeft.coerceIn(horizontalPadding, size.width - textWidth - horizontalPadding)
                val textX = when (paint.textAlign) {
                    Paint.Align.RIGHT -> clampedLeft + textWidth
                    Paint.Align.CENTER -> clampedLeft + textWidth / 2f
                    else -> clampedLeft
                }
                val textY = y.coerceIn(paint.textSize + verticalPadding, size.height - verticalPadding)
                paint.style = Paint.Style.FILL
                paint.color = backgroundColor.copy(alpha = 0.94f).toArgb()
                canvas.nativeCanvas.drawRoundRect(clampedLeft - horizontalPadding, textY - paint.textSize - verticalPadding, clampedLeft + textWidth + horizontalPadding, textY + verticalPadding, 6f, 6f, paint)
                paint.color = contentColor.toArgb()
                canvas.nativeCanvas.drawText(label, textX, textY, paint)
            }
        }
    }
}

@Composable
private fun DoweTable(state: DoweReactiveState, dataPath: String, columns: List<DoweTableColumn>, size: DoweTableSize, striped: Boolean, bordered: Boolean, dividers: Boolean, emptyTitle: String, emptyDescription: String, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val rows = state.rows(dataPath)
    val metrics = doweTableMetrics(size)
    BoxWithConstraints(
        modifier = modifier
            .fillMaxWidth()
            .clip(shape)
            .background(backgroundColor)
            .then(if (bordered || borderColor != null) Modifier.border(1.dp, borderColor ?: DoweDesign.surfaceText.copy(alpha = 0.28f), shape) else Modifier)
    ) {
        val minimumWidth = doweTableMinimumWidth(columns)
        val tableWidth = maxOf(maxWidth, minimumWidth)
        val columnExpansion = (tableWidth - minimumWidth) / columns.size.coerceAtLeast(1).toFloat()
        Box(modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState())) {
            Column(modifier = Modifier.width(tableWidth)) {
                Row(modifier = Modifier.fillMaxWidth().background(DoweDesign.muted)) {
                    columns.forEach { column ->
                        Box(modifier = Modifier.width(doweTableColumnWidth(column.width) + columnExpansion).padding(horizontal = metrics.horizontalPadding, vertical = metrics.headerVerticalPadding), contentAlignment = doweTableBoxAlignment(column.align)) {
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
                        Row(modifier = Modifier.fillMaxWidth().background(if (striped && index % 2 == 1) DoweDesign.surfaceText.copy(alpha = 0.12f) else Color.Transparent)) {
                            columns.forEachIndexed { columnIndex, column ->
                                Box(
                                    modifier = Modifier.width(doweTableColumnWidth(column.width) + columnExpansion),
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
                                        Box(modifier = Modifier.align(Alignment.CenterEnd).width(1.dp).fillMaxHeight().background(DoweDesign.surfaceText.copy(alpha = 0.28f)))
                                    }
                                }
                            }
                        }
                        if (dividers && index < rows.lastIndex) {
                            Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(DoweDesign.surfaceText.copy(alpha = 0.28f)))
                        }
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

private fun doweTableMinimumWidth(columns: List<DoweTableColumn>): Dp =
    columns.fold(0.dp) { total, column -> total + doweTableColumnWidth(column.width) }

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
    Column(modifier = modifier.clip(shape).background(backgroundColor).then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape)).clipToBounds()) {
        Row(modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 10.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text(text = language.uppercase(), color = contentColor, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            Text(text = if (copied) copiedLabel else copyLabel, modifier = Modifier.clickable {
                clipboard.setText(AnnotatedString(source))
                copied = true
            }, color = contentColor, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        }
        Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(contentColor.copy(alpha = 0.24f)))
        Box(modifier = Modifier.fillMaxWidth().clipToBounds().horizontalScroll(rememberScrollState())) {
            Text(text = highlighted, modifier = Modifier.padding(16.dp), fontFamily = FontFamily.Monospace, fontSize = 14.sp, lineHeight = 22.sp)
        }
    }
}

private val doweSelectArrowViewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)
private val doweSelectArrowPaths = listOf(
    DoweSvgPath("M0 0h24v24H0z", DoweSvgFill.None),
    DoweSvgPath("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z", DoweSvgFill.CurrentColor)
)

private data class DoweRuntimeSvgRecord(val viewBox: DoweSvgViewBox, val paths: List<DoweSvgPath>)

private fun doweRuntimeSvgColor(value: String): Color? {
    if (value == "currentColor") return null
    if (!Regex("^#[0-9a-fA-F]{3}([0-9a-fA-F]{3}|[0-9a-fA-F]{5})?$").matches(value)) return null
    return runCatching { Color(android.graphics.Color.parseColor(value)) }.getOrNull()
}

private fun doweRuntimeSvgTransform(value: String): DoweSvgTransform? {
    if (!value.startsWith("matrix(") || !value.endsWith(")")) return null
    val values = value.substring(7, value.length - 1).trim().split(Regex("[\\s,]+")).mapNotNull(String::toFloatOrNull)
    if (values.size != 6 || values.any { !it.isFinite() }) return null
    return DoweSvgTransform(values[0], values[1], values[2], values[3], values[4], values[5])
}

private fun doweRuntimeSvgInteger(source: JSONObject, name: String, fallback: Int, range: IntRange): Int? {
    if (!source.has(name) || source.isNull(name)) return fallback
    val value = source.get(name) as? Number ?: return null
    val integer = value.toInt()
    return integer.takeIf { value.toDouble() == integer.toDouble() && it in range }
}

private fun doweRuntimeSvgRecord(payload: String): DoweRuntimeSvgRecord? {
    if (payload.isEmpty() || payload.length > 131072) return null
    return runCatching {
        val source = JSONObject(payload)
        val values = source.optString("viewBox").trim().split(Regex("[\\s,]+")).mapNotNull(String::toFloatOrNull)
        if (values.size != 4 || values.any { !it.isFinite() } || values[2] <= 0f || values[3] <= 0f) return null
        val sourcePaths = source.optJSONArray("paths") ?: return null
        if (sourcePaths.length() !in 1..64) return null
        val paths = mutableListOf<DoweSvgPath>()
        for (index in 0 until sourcePaths.length()) {
            val sourcePath = sourcePaths.optJSONObject(index) ?: return null
            val data = sourcePath.optString("d")
            if (data.isEmpty() || data.length > 32768 || !Regex("^[MmZzLlHhVvCcSsQqTtAa0-9eE.,+\\-\\s]+$").matches(data)) return null
            val paint = sourcePath.optString("paint", "currentColor")
            if (paint !in setOf("fill", "stroke", "none", "currentColor")) return null
            val colorSource = sourcePath.optString("color", "currentColor")
            if (colorSource != "currentColor" && doweRuntimeSvgColor(colorSource) == null) return null
            val color = doweRuntimeSvgColor(colorSource)
            val opacity = doweRuntimeSvgInteger(sourcePath, "opacity", 255, 0..255) ?: return null
            val width = doweRuntimeSvgInteger(sourcePath, "width", 100, 1..10000) ?: return null
            val cap = sourcePath.optString("lineCap", "butt")
            val join = sourcePath.optString("lineJoin", "miter")
            if (cap !in setOf("butt", "round", "square") || join !in setOf("miter", "round", "bevel")) return null
            val transform = if (sourcePath.has("transform") && !sourcePath.isNull("transform")) {
                doweRuntimeSvgTransform(sourcePath.getString("transform")) ?: return null
            } else {
                null
            }
            val fill = when (paint) {
                "none" -> DoweSvgFill.None
                "currentColor" -> DoweSvgFill.CurrentColor
                "stroke" -> DoweSvgFill.Stroke(color, opacity / 255f, width / 100f, cap, join)
                else -> DoweSvgFill.Fill(color, opacity / 255f, sourcePath.optBoolean("evenOdd", false))
            }
            paths += DoweSvgPath(data, fill, transform)
        }
        DoweRuntimeSvgRecord(DoweSvgViewBox(values[0], values[1], values[2], values[3]), paths)
    }.getOrNull()
}

@Composable
private fun DoweRuntimeSvg(payload: String, modifier: Modifier, color: Color, animated: Boolean = false) {
    val record = remember(payload) { doweRuntimeSvgRecord(payload) }
    if (record != null) {
        DoweSvg(viewBox = record.viewBox, modifier = modifier, color = color, paths = record.paths, animated = animated)
    }
}

@Composable
private fun DoweSvg(viewBox: DoweSvgViewBox, modifier: Modifier, color: Color, paths: List<DoweSvgPath>, animated: Boolean = false) {
    val rotation = if (animated) {
        val transition = rememberInfiniteTransition(label = "dowe-svg-spinner")
        transition.animateFloat(
            initialValue = 0f,
            targetValue = 360f,
            animationSpec = infiniteRepeatable(tween(durationMillis = 900, easing = LinearEasing)),
            label = "dowe-svg-spinner-rotation"
        ).value
    } else {
        0f
    }
    Canvas(modifier = modifier.rotate(rotation)) {
        val scale = minOf(size.width / viewBox.width, size.height / viewBox.height)
        val renderedWidth = viewBox.width * scale
        val renderedHeight = viewBox.height * scale
        withTransform({
            scale(scaleX = scale, scaleY = scale)
            translate(
                left = (size.width - renderedWidth) / (2f * scale) - viewBox.minX,
                top = (size.height - renderedHeight) / (2f * scale) - viewBox.minY
            )
        }) {
            paths.forEach { entry ->
                val fill = when (val value = entry.fill) {
                    DoweSvgFill.None -> null
                    DoweSvgFill.CurrentColor -> color
                    is DoweSvgFill.Solid -> value.color
                    is DoweSvgFill.Fill -> (value.color ?: color).copy(alpha = value.opacity)
                    is DoweSvgFill.Stroke -> (value.color ?: color).copy(alpha = value.opacity)
                }
                if (fill != null) {
                    val parsed = PathParser().parsePathString(entry.data).toPath()
                    entry.transform?.let { value ->
                        val matrix = androidx.compose.ui.graphics.Matrix()
                        matrix[0, 0] = value.a
                        matrix[0, 1] = value.c
                        matrix[0, 3] = value.e
                        matrix[1, 0] = value.b
                        matrix[1, 1] = value.d
                        matrix[1, 3] = value.f
                        parsed.transform(matrix)
                    }
                    when (val value = entry.fill) {
                        is DoweSvgFill.Stroke -> drawPath(parsed, fill, style = androidx.compose.ui.graphics.drawscope.Stroke(width = value.width, cap = when (value.cap) { "round" -> androidx.compose.ui.graphics.StrokeCap.Round; "square" -> androidx.compose.ui.graphics.StrokeCap.Square; else -> androidx.compose.ui.graphics.StrokeCap.Butt }, join = when (value.join) { "round" -> androidx.compose.ui.graphics.StrokeJoin.Round; "bevel" -> androidx.compose.ui.graphics.StrokeJoin.Bevel; else -> androidx.compose.ui.graphics.StrokeJoin.Miter }))
                        else -> drawPath(parsed, fill)
                    }
                }
            }
        }
    }
}

"#
}
