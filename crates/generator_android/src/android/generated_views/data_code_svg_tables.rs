r#"private fun DoweTable(state: DoweReactiveState, dataPath: String, columns: List<DoweTableColumn>, size: DoweTableSize, striped: Boolean, bordered: Boolean, dividers: Boolean, emptyTitle: String, emptyDescription: String, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
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
