fn android_runtime_rich_controls_map() -> &'static str {
    r#"private data class DoweToggleGroupItem(val id: String, val label: String, val icon: String?)

@Composable
private fun DoweToggleGroup(value: String, onValueChange: (String) -> Unit, items: List<DoweToggleGroupItem>, size: String, wide: Boolean, vertical: Boolean, disabled: Boolean, ariaLabel: String?, backgroundColor: Color, contentColor: Color, borderColor: Color?, onChange: (() -> Unit)?, modifier: Modifier) {
    val container = modifier.then(if (wide) Modifier.fillMaxWidth() else Modifier).clip(RoundedCornerShape(10.dp)).background(backgroundColor).then(if (borderColor != null) Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)) else Modifier).alpha(if (disabled) 0.5f else 1f).padding(4.dp)
    val buttonContent: @Composable RowScope.(DoweToggleGroupItem) -> Unit = { item ->
        Text(text = item.label, fontSize = when (size) { "xs" -> 12.sp; "sm" -> 13.sp; "lg" -> 18.sp; else -> 14.sp }, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
    if (vertical) {
        Column(modifier = container, verticalArrangement = Arrangement.spacedBy(4.dp)) {
            items.forEach { item ->
                Button(enabled = !disabled, onClick = { onValueChange(item.id); onChange?.invoke() }, shape = RoundedCornerShape(8.dp), contentPadding = PaddingValues(horizontal = when (size) { "xs" -> 8.dp; "sm" -> 10.dp; "lg" -> 16.dp; else -> 12.dp }), colors = ButtonDefaults.buttonColors(containerColor = if (value == item.id) contentColor else Color.Transparent, contentColor = if (value == item.id) backgroundColor else contentColor.copy(alpha = 0.72f)), modifier = (if (wide) Modifier.fillMaxWidth() else Modifier).height(when (size) { "xs" -> 24.dp; "sm" -> 32.dp; "lg" -> 48.dp; else -> 40.dp })) { buttonContent(item) }
            }
        }
    } else {
        Row(modifier = container.then(if (wide) Modifier.fillMaxWidth() else Modifier), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            items.forEach { item ->
                Button(enabled = !disabled, onClick = { onValueChange(item.id); onChange?.invoke() }, shape = RoundedCornerShape(8.dp), contentPadding = PaddingValues(horizontal = when (size) { "xs" -> 8.dp; "sm" -> 10.dp; "lg" -> 16.dp; else -> 12.dp }), colors = ButtonDefaults.buttonColors(containerColor = if (value == item.id) contentColor else Color.Transparent, contentColor = if (value == item.id) backgroundColor else contentColor.copy(alpha = 0.72f)), modifier = Modifier.weight(1f).height(when (size) { "xs" -> 24.dp; "sm" -> 32.dp; "lg" -> 48.dp; else -> 40.dp })) { buttonContent(item) }
            }
        }
    }
}

private data class DowePaginationControlContract(
    val controlSize: Int,
    val indicatorGap: Int,
    val indicatorHeight: Int,
    val indicatorInactiveWidth: Int,
    val indicatorActiveWidth: Int,
    val indicatorDotSize: Int,
    val indicatorDotActiveScalePercent: Int,
)

@Composable
private fun DowePagination(value: String, onValueChange: (String) -> Unit, pageCount: Int, size: String, control: DowePaginationControlContract, paginationVariant: String, disabled: Boolean, ariaLabel: String?, backgroundColor: Color, contentColor: Color, accentColor: Color, borderColor: Color?, onChange: (() -> Unit)?, previousIcon: @Composable () -> Unit, nextIcon: @Composable () -> Unit, modifier: Modifier) {
    val current = value.toIntOrNull()?.coerceIn(1, pageCount) ?: 1
    val dimension = control.controlSize.dp
    val fontSize = when (size) { "xs" -> 12.sp; "sm" -> 13.sp; "lg" -> 17.sp; else -> 14.sp }
    val pages = buildList {
        if (pageCount <= 7) {
            addAll(1..pageCount)
        } else {
            add(1)
            if (current > 3) add(0)
            addAll((max(2, current - 1)..min(pageCount - 1, current + 1)))
            if (current < pageCount - 2) add(0)
            add(pageCount)
        }
    }
    Row(modifier = modifier.semantics { contentDescription = ariaLabel ?: "Pagination" }, horizontalArrangement = Arrangement.spacedBy(control.indicatorGap.dp), verticalAlignment = Alignment.CenterVertically) {
        if (paginationVariant == "pages" || paginationVariant == "controls") {
            DoweIconButton(enabled = !disabled && current > 1, dimension = dimension, backgroundColor = backgroundColor, contentColor = contentColor, borderColor = borderColor, label = "Previous page", onClick = { onValueChange((current - 1).toString()); onChange?.invoke() }) { previousIcon() }
        }
        if (paginationVariant == "pages") {
            pages.forEach { page ->
                if (page == 0) {
                    Box(modifier = Modifier.size(dimension), contentAlignment = Alignment.Center) { Text("…", color = DoweDesign.backgroundText.copy(alpha = 0.6f), fontSize = fontSize) }
                } else {
                    DowePaginationButton(enabled = !disabled, selected = page == current, dimension = dimension, backgroundColor = backgroundColor, contentColor = contentColor, borderColor = borderColor, label = "Page $page", onClick = { if (page != current) { onValueChange(page.toString()); onChange?.invoke() } }) {
                        Text(page.toString(), fontSize = fontSize, fontWeight = FontWeight.Medium)
                    }
                }
            }
        } else {
            (1..pageCount).forEach { page ->
                DowePaginationIndicator(active = page == current, dot = paginationVariant == "dots", enabled = !disabled, inactiveWidth = control.indicatorInactiveWidth.dp, activeWidth = control.indicatorActiveWidth.dp, height = control.indicatorHeight.dp, dotSize = control.indicatorDotSize.dp, dotActiveScalePercent = control.indicatorDotActiveScalePercent, color = accentColor, label = "Go to page $page", onClick = { if (page != current) { onValueChange(page.toString()); onChange?.invoke() } })
            }
            if (paginationVariant == "controls") {
                Text("$current / $pageCount", color = accentColor, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
            }
        }
        if (paginationVariant == "pages" || paginationVariant == "controls") {
            DoweIconButton(enabled = !disabled && current < pageCount, dimension = dimension, backgroundColor = backgroundColor, contentColor = contentColor, borderColor = borderColor, label = "Next page", onClick = { onValueChange((current + 1).toString()); onChange?.invoke() }) { nextIcon() }
        }
    }
}

@Composable
private fun DoweIconButton(enabled: Boolean, dimension: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, label: String, onClick: () -> Unit, content: @Composable () -> Unit) {
    Button(
        enabled = enabled,
        onClick = onClick,
        modifier = Modifier.size(dimension).semantics { contentDescription = label },
        shape = CircleShape,
        contentPadding = PaddingValues(0.dp),
        colors = ButtonDefaults.buttonColors(containerColor = backgroundColor, contentColor = contentColor, disabledContainerColor = backgroundColor.copy(alpha = 0.42f), disabledContentColor = contentColor.copy(alpha = 0.42f)),
        border = if (borderColor != null) BorderStroke(1.dp, borderColor) else null
    ) { content() }
}

@Composable
private fun DowePaginationButton(enabled: Boolean, selected: Boolean, dimension: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, label: String, onClick: () -> Unit, content: @Composable () -> Unit) {
    Button(
        enabled = enabled,
        onClick = onClick,
        modifier = Modifier.size(dimension).semantics { contentDescription = label },
        shape = RoundedCornerShape(10.dp),
        contentPadding = PaddingValues(0.dp),
        colors = ButtonDefaults.buttonColors(containerColor = if (selected) backgroundColor else Color.Transparent, contentColor = if (selected) contentColor else DoweDesign.backgroundText, disabledContainerColor = if (selected) backgroundColor.copy(alpha = 0.42f) else Color.Transparent, disabledContentColor = DoweDesign.backgroundText.copy(alpha = 0.42f)),
        border = if (selected && borderColor != null) BorderStroke(1.dp, borderColor) else null
    ) { content() }
}

@Composable
private fun DowePaginationIndicator(active: Boolean, dot: Boolean, enabled: Boolean, inactiveWidth: Dp, activeWidth: Dp, height: Dp, dotSize: Dp, dotActiveScalePercent: Int = 125, color: Color, label: String, onClick: () -> Unit) {
    Box(modifier = Modifier.width(if (dot) dotSize else if (active) activeWidth else inactiveWidth).height(if (dot) dotSize else height).clip(if (dot) CircleShape else RoundedCornerShape(999.dp)).background(color.copy(alpha = if (active) 1f else 0.28f)).graphicsLayer { val scale = if (dot && active) dotActiveScalePercent / 100f else 1f; scaleX = scale; scaleY = scale; alpha = if (enabled) 1f else 0.42f }.clickable(enabled = enabled, onClick = onClick).semantics { contentDescription = label })
}

@Composable
private fun DoweCollapsible(label: String, defaultOpen: Boolean, disabled: Boolean, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp, modifier: Modifier, arrowIcon: @Composable () -> Unit, content: @Composable () -> Unit) {
    var open by remember { mutableStateOf(defaultOpen) }
    Column(modifier = modifier.clip(RoundedCornerShape(radius)).background(backgroundColor).then(if (borderColor != null) Modifier.border(1.dp, borderColor, RoundedCornerShape(radius)) else Modifier).alpha(if (disabled) 0.5f else 1f)) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            Row(modifier = Modifier.fillMaxWidth().clickable(enabled = !disabled) { open = !open }.padding(horizontal = 16.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(text = label, color = contentColor, fontSize = 14.sp, lineHeight = 20.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.weight(1f))
                Box(modifier = Modifier.size(20.dp).graphicsLayer { rotationZ = if (open) 180f else 0f }, contentAlignment = Alignment.Center) {
                    arrowIcon()
                }
            }
            AnimatedVisibility(visible = open, enter = fadeIn(tween(160)) + expandVertically(tween(160)), exit = fadeOut(tween(160)) + shrinkVertically(tween(160))) {
                Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) { content() }
            }
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
        completed = false
        while (!completed) {
            now = System.currentTimeMillis()
            if ((targetMillis - now) <= 0 && !completed) {
                completed = true
                onComplete?.invoke()
            } else {
                delay(1000)
            }
        }
    }
    BoxWithConstraints(modifier = modifier.fillMaxWidth()) {
        val displaySize = if (maxWidth < 480.dp && size != "sm") "sm" else size
        Row(modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.Top) {
            var needsSeparator = false
            if (showDays) {
                DoweCountdownUnit((remaining / 86400).toInt(), daysLabel, displaySize, backgroundColor, contentColor, borderColor)
                needsSeparator = true
            }
            if (showHours) {
                if (needsSeparator) DoweCountdownSeparator(displaySize, contentColor)
                DoweCountdownUnit(((remaining % 86400) / 3600).toInt(), hoursLabel, displaySize, backgroundColor, contentColor, borderColor)
                needsSeparator = true
            }
            if (showMinutes) {
                if (needsSeparator) DoweCountdownSeparator(displaySize, contentColor)
                DoweCountdownUnit(((remaining % 3600) / 60).toInt(), minutesLabel, displaySize, backgroundColor, contentColor, borderColor)
                needsSeparator = true
            }
            if (showSeconds) {
                if (needsSeparator) DoweCountdownSeparator(displaySize, contentColor)
                DoweCountdownUnit((remaining % 60).toInt(), secondsLabel, displaySize, backgroundColor, contentColor, borderColor)
            }
        }
    }
}

@Composable
private fun DoweCountdownUnit(value: Int, label: String, size: String, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val width = when (size) { "sm" -> 40.dp; "lg" -> 80.dp; "xl" -> 112.dp; else -> 56.dp }
    val height = when (size) { "sm" -> 48.dp; "lg" -> 96.dp; "xl" -> 128.dp; else -> 64.dp }
    val font = when (size) { "sm" -> 20.sp; "lg" -> 48.sp; "xl" -> 72.sp; else -> 30.sp }
    Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Box(modifier = Modifier.widthIn(min = width).height(height).clip(RoundedCornerShape(16.dp)).background(backgroundColor).then(if (borderColor != null) Modifier.border(1.dp, borderColor, RoundedCornerShape(16.dp)) else Modifier).padding(horizontal = when (size) { "sm" -> 6.dp; "lg" -> 12.dp; "xl" -> 16.dp; else -> 8.dp }), contentAlignment = Alignment.Center) {
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
        if (showControls) Column(modifier = Modifier.align(Alignment.TopEnd).width(34.dp).clip(RoundedCornerShape(10.dp)).background(DoweDesign.background.copy(alpha = 0.92f))) {
            Text("+", modifier = Modifier.fillMaxWidth().height(34.dp).wrapContentHeight(Alignment.CenterVertically), color = contentColor, textAlign = TextAlign.Center, fontWeight = FontWeight.Bold)
            Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(contentColor.copy(alpha = 0.12f)))
            Text("-", modifier = Modifier.fillMaxWidth().height(34.dp).wrapContentHeight(Alignment.CenterVertically), color = contentColor, textAlign = TextAlign.Center, fontWeight = FontWeight.Bold)
        }
        if (showScale) Text("1 km", modifier = Modifier.align(Alignment.BottomStart).padding(12.dp).clip(RoundedCornerShape(999.dp)).background(DoweDesign.background.copy(alpha = 0.92f)).padding(horizontal = 10.dp, vertical = 4.dp), fontSize = 12.sp, fontWeight = FontWeight.Bold)
        if (showLocationControl) Button(onClick = { onLocation?.invoke() }, modifier = Modifier.align(Alignment.BottomEnd).padding(12.dp)) { Text("⌖") }
    }
}

private fun doweMapHeight(value: String): Dp =
    value.removeSuffix("px").toFloatOrNull()?.dp ?: 400.dp

@Composable
private fun DoweBadge(text: String, position: String, backgroundColor: Color, contentColor: Color, fontSize: TextUnit, height: Dp, horizontalPadding: Dp, verticalPadding: Dp, modifier: Modifier, content: @Composable () -> Unit) {
    Box(modifier = modifier) {
        content()
        Text(
            text = text,
            modifier = Modifier
                .align(doweBadgeAlignment(position))
                .doweBadgeCornerOffset(position)
                .zIndex(1f)
                .clip(RoundedCornerShape(999.dp))
                .background(backgroundColor)
                .padding(horizontal = horizontalPadding, vertical = verticalPadding)
                .height(height),
            color = contentColor,
            fontSize = fontSize,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1
        )
    }
}

private fun Modifier.doweBadgeCornerOffset(position: String): Modifier =
    layout { measurable, constraints ->
        val placeable = measurable.measure(constraints)
        layout(placeable.width, placeable.height) {
            val x = if (position.endsWith("right")) placeable.width / 2 else -placeable.width / 2
            val y = if (position.startsWith("bottom")) placeable.height / 2 else -placeable.height / 2
            placeable.place(x, y)
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
private fun DoweChip(text: String, size: String, height: Dp, horizontalPadding: Dp, textSize: TextUnit, contentGap: Dp, closeAlpha: Float, backgroundColor: Color, contentColor: Color, borderColor: Color?, modifier: Modifier, compact: Boolean, onClose: (() -> Unit)?, start: (@Composable () -> Unit)?, end: (@Composable () -> Unit)?) {
    val shape = RoundedCornerShape(DoweDesign.radius)
    val surface: @Composable (Modifier) -> Unit = { surfaceModifier ->
        Row(
            modifier = surfaceModifier
                .height(height)
                .clip(shape)
                .background(backgroundColor)
                .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
                .padding(horizontal = horizontalPadding),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(contentGap)
        ) {
            CompositionLocalProvider(LocalContentColor provides contentColor) {
                start?.invoke()
                Text(text = text, color = contentColor, fontSize = textSize, fontWeight = FontWeight.Medium, maxLines = 1)
                end?.invoke()
                if (onClose != null) {
                    Text(text = "x", modifier = Modifier.clickable(onClick = onClose), color = contentColor.copy(alpha = closeAlpha), fontSize = textSize, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
    if (compact) {
        Box(modifier = Modifier.doweGridCompactWidth(), contentAlignment = Alignment.CenterStart) {
            surface(modifier)
        }
    } else {
        surface(modifier)
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
private fun DoweSkeleton(variant: String, animation: String, textHeight: Dp, defaultRadius: Dp, pulseAlpha: Float, pulseDurationMs: Int, modifier: Modifier) {
    val alpha by animateFloatAsState(
        targetValue = if (animation == "pulse") pulseAlpha else 1f,
        animationSpec = tween(durationMillis = pulseDurationMs)
    )
    val shape = when (variant) {
        "circular" -> RoundedCornerShape(999.dp)
        "rectangular" -> RoundedCornerShape(0.dp)
        "rounded" -> RoundedCornerShape(DoweDesign.radius)
        else -> RoundedCornerShape(defaultRadius)
    }
    val base = if (variant == "text") modifier.height(textHeight).fillMaxWidth() else modifier
    Box(modifier = base.clip(shape).background(DoweDesign.muted.copy(alpha = if (animation == "none") 1f else alpha)))
}

"#
}
