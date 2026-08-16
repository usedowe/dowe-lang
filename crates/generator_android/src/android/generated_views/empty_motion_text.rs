fn android_runtime_empty_motion_text() -> &'static str {
    r#"@Composable
private fun DoweEmpty(kind: String, title: String?, description: String?, actionLabel: String, action: (() -> Unit)?, iconViewBox: DoweSvgViewBox, iconPaths: List<DoweSvgPath>, backgroundColor: Color, contentColor: Color, accentColor: Color, modifier: Modifier) {
    Column(
        modifier = modifier.fillMaxWidth().padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        DoweSvg(viewBox = iconViewBox, modifier = Modifier.width(112.dp).height(112.dp), color = accentColor, paths = iconPaths)
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

private data class DoweRichTextMark(val text: String, val style: String, val scheme: String)
private data class DoweRichTextLine(val start: Int, val end: Int, val width: Int, val height: Int)

@Composable
private fun DoweRichText(marks: List<DoweRichTextMark>, fontFamily: FontFamily?, fontSize: TextUnit, contentColor: Color, modifier: Modifier) {
    Layout(modifier = modifier, content = {
        marks.forEach { mark ->
            DoweRichTextRun(mark = mark, fontFamily = fontFamily, fontSize = fontSize, contentColor = contentColor)
        }
    }) { measurables, constraints ->
        val gap = 4.dp.roundToPx()
        val childConstraints = constraints.copy(minWidth = 0, minHeight = 0)
        val placeables = measurables.map { it.measure(childConstraints) }
        val lines = mutableListOf<DoweRichTextLine>()
        var start = 0
        var width = 0
        var height = 0
        placeables.forEachIndexed { index, placeable ->
            val nextWidth = if (index == start) placeable.width else width + gap + placeable.width
            if (index > start && nextWidth > constraints.maxWidth) {
                lines += DoweRichTextLine(start, index, width, height)
                start = index
                width = placeable.width
                height = placeable.height
            } else {
                width = nextWidth
                height = maxOf(height, placeable.height)
            }
        }
        if (placeables.isNotEmpty()) {
            lines += DoweRichTextLine(start, placeables.size, width, height)
        }
        val contentWidth = lines.maxOfOrNull { it.width } ?: 0
        val contentHeight = lines.sumOf { it.height } + gap * maxOf(lines.size - 1, 0)
        val layoutWidth = constraints.constrainWidth(contentWidth)
        val layoutHeight = constraints.constrainHeight(contentHeight)
        layout(layoutWidth, layoutHeight) {
            var lineTop = 0
            lines.forEach { line ->
                var lineLeft = ((layoutWidth - line.width) / 2).coerceAtLeast(0)
                for (index in line.start until line.end) {
                    val placeable = placeables[index]
                    placeable.placeRelative(lineLeft, lineTop + (line.height - placeable.height) / 2)
                    lineLeft += placeable.width + gap
                }
                lineTop += line.height + gap
            }
        }
    }
}

@Composable
private fun DoweRichTextRun(mark: DoweRichTextMark, fontFamily: FontFamily?, fontSize: TextUnit, contentColor: Color) {
    val accent = doweButtonFamily(mark.scheme)
    val onAccent = doweButtonTextFamily(mark.scheme)
    val softAccent = doweButtonSoftFamily(mark.scheme)
    val inheritedColor = if (contentColor == Color.Unspecified) DoweDesign.backgroundText else contentColor
    val density = LocalDensity.current
    var measuredTextWidth by remember(mark.text, fontFamily, fontSize) { mutableStateOf<Dp?>(null) }
    val shape = when (mark.style) {
        "mark" -> RoundedCornerShape(2.dp)
        "pill" -> RoundedCornerShape(999.dp)
        else -> RoundedCornerShape(DoweDesign.radius)
    }
    val neonTransition = rememberInfiniteTransition(label = "dowe-rich-neon")
    val neonAlpha by neonTransition.animateFloat(
        initialValue = 1f,
        targetValue = if (mark.style == "neon") 0.9f else 1f,
        animationSpec = infiniteRepeatable(tween(1000, easing = LinearEasing), repeatMode = RepeatMode.Reverse),
        label = "dowe-rich-neon-alpha"
    )
    val decoration = when (mark.style) {
        "mark" -> Modifier.clip(shape).background(accent).padding(horizontal = 8.dp, vertical = 2.dp)
        "pill" -> Modifier.border(2.dp, accent, shape).padding(horizontal = 10.dp, vertical = 2.dp)
        "slant" -> Modifier.padding(horizontal = 6.dp, vertical = 1.dp).drawBehind {
            val slant = 4.dp.toPx()
            val path = Path().apply {
                moveTo(slant, 0f)
                lineTo(size.width, 0f)
                lineTo(size.width - slant, size.height)
                lineTo(0f, size.height)
                close()
            }
            drawPath(path, accent)
        }
        "under" -> Modifier.padding(bottom = 2.dp).drawBehind {
            drawLine(accent, Offset(0f, size.height), Offset(size.width, size.height), strokeWidth = 3.dp.toPx())
        }
        "strike" -> Modifier.drawBehind {
            drawLine(accent, Offset(0f, size.height / 2f), Offset(size.width, size.height / 2f), strokeWidth = 3.dp.toPx())
        }
        "box" -> Modifier.border(2.dp, accent, shape).padding(horizontal = 12.dp, vertical = 4.dp)
        "wave" -> Modifier.padding(bottom = 4.dp).drawBehind {
            val baseline = size.height - 1.dp.toPx()
            val amplitude = 1.5.dp.toPx()
            val wavelength = 6.dp.toPx()
            val path = Path()
            var x = 0f
            path.moveTo(0f, baseline)
            while (x <= size.width) {
                path.lineTo(x, baseline + sin(x / wavelength * 2f * Math.PI.toFloat()) * amplitude)
                x += 1.dp.toPx()
            }
            drawPath(path, accent, style = Stroke(width = 2.dp.toPx()))
        }
        "tag" -> Modifier.doweShadow(radius = 8.dp, shape = shape, color = inheritedColor, alpha = 0.1f).clip(shape).background(softAccent).padding(horizontal = 12.dp, vertical = 4.dp)
        else -> Modifier
    }
    val resolvedText = if (mark.style == "mark" || mark.style == "neon") mark.text.uppercase(Locale.ROOT) else mark.text
    val textColor = when (mark.style) {
        "mark", "slant" -> onAccent
        "under", "strike", "wave" -> inheritedColor
        else -> accent
    }
    Text(
        text = resolvedText,
        modifier = decoration.then(measuredTextWidth?.let { Modifier.width(it) } ?: Modifier).graphicsLayer(alpha = neonAlpha),
        color = textColor,
        fontFamily = fontFamily,
        fontSize = fontSize,
        textAlign = TextAlign.Center,
        onTextLayout = { layout ->
            val lineWidth = (0 until layout.lineCount).maxOfOrNull { index ->
                layout.getLineRight(index) - layout.getLineLeft(index)
            } ?: 0f
            val nextWidth = with(density) { ceil(lineWidth).toDp() }
            if (measuredTextWidth != nextWidth) measuredTextWidth = nextWidth
        },
        fontWeight = when (mark.style) {
            "strike" -> FontWeight.Medium
            "pill", "under", "box", "wave" -> FontWeight.SemiBold
            else -> FontWeight.Bold
        },
        fontStyle = if (mark.style == "grad") FontStyle.Italic else FontStyle.Normal,
        letterSpacing = when (mark.style) {
            "mark" -> 0.025.em
            "neon" -> 0.05.em
            else -> TextUnit.Unspecified
        },
        style = TextStyle(
            brush = if (mark.style == "grad") Brush.horizontalGradient(listOf(accent, Color.White.copy(alpha = 0.6f))) else null,
            shadow = when (mark.style) {
                "glow" -> Shadow(color = accent.copy(alpha = 0.7f), offset = Offset.Zero, blurRadius = 15f)
                "neon" -> Shadow(color = accent, offset = Offset.Zero, blurRadius = 20f)
                "pop" -> Shadow(color = accent.copy(alpha = 0.6f), offset = Offset(3f, 3f), blurRadius = 0f)
                else -> null
            }
        )
    )
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

"#
}
