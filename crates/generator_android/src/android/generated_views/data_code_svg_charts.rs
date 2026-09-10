r#"
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
"#
