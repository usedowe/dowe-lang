fn android_runtime_empty_motion_text() -> &'static str {
    r#"@Composable
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

"#
}
