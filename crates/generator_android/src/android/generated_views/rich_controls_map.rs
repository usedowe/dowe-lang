fn android_runtime_rich_controls_map() -> &'static str {
    r#"private data class DoweToggleGroupItem(val id: String, val label: String, val icon: String?)

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

"#
}
