fn android_runtime_navigation_drawer_layout() -> &'static str {
    r#"private data class DoweTabItem(val id: String, val label: String)

@Composable
private fun DoweTabs(items: List<DoweTabItem>, initialId: String, modifier: Modifier, position: String, variant: String, backgroundColor: Color, contentColor: Color, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, borderColor: Color?, radius: Dp, fontFamily: FontFamily, content: @Composable (String) -> Unit) {
    var activeId by remember(initialId) { mutableStateOf(initialId) }
    val vertical = position == "start" || position == "end"
    val listShape = RoundedCornerShape(if (variant == "pills") 999.dp else radius)
    val listModifier = Modifier
        .clip(listShape)
        .background(backgroundColor)
        .then(if (borderColor == null || variant == "line") Modifier else Modifier.border(1.dp, borderColor, listShape))
        .padding(if (variant == "line" || variant == "ghost") 0.dp else 4.dp)
    val tabList: @Composable () -> Unit = {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            if (vertical) {
                Column(modifier = listModifier, verticalArrangement = Arrangement.spacedBy(if (variant == "line") 16.dp else 8.dp)) {
                    items.forEach { item ->
                        DoweTabButton(item = item, active = activeId == item.id, variant = variant, activeBackgroundColor = activeBackgroundColor, activeContentColor = activeContentColor, accentColor = accentColor, radius = radius, fontFamily = fontFamily) {
                            activeId = item.id
                        }
                    }
                }
            } else {
                Row(modifier = listModifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(if (variant == "line") 16.dp else 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    items.forEach { item ->
                        DoweTabButton(item = item, active = activeId == item.id, variant = variant, activeBackgroundColor = activeBackgroundColor, activeContentColor = activeContentColor, accentColor = accentColor, radius = radius, fontFamily = fontFamily) {
                            activeId = item.id
                        }
                    }
                }
            }
        }
    }
    val panel: @Composable () -> Unit = {
        Box(modifier = if (vertical) Modifier else Modifier.fillMaxWidth()) {
            content(activeId)
        }
    }
    when (position) {
        "bottom" -> Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
            panel()
            tabList()
        }
        "start" -> Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            tabList()
            panel()
        }
        "end" -> Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            panel()
            tabList()
        }
        else -> Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
            tabList()
            panel()
        }
    }
}

@Composable
private fun DoweTabButton(item: DoweTabItem, active: Boolean, variant: String, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, radius: Dp, fontFamily: FontFamily, onClick: () -> Unit) {
    val shape = RoundedCornerShape(if (variant == "pills") 999.dp else radius)
    val selectedFill = variant == "solid" || variant == "outlined" || variant == "pills"
    val selectedLine = variant == "line"
    val background = if (active && selectedFill) activeBackgroundColor else Color.Transparent
    val color = if (!active) LocalContentColor.current else if (selectedFill) activeContentColor else accentColor
    val border = if (active && selectedLine) BorderStroke(2.dp, accentColor) else null
    Box(
        modifier = Modifier
            .clip(shape)
            .background(background)
            .then(if (border == null) Modifier else Modifier.border(border, shape))
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(text = item.label, color = color, fontFamily = fontFamily)
    }
}

@Composable
private fun DoweSideNavRow(modifier: Modifier = Modifier, active: Boolean, wide: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, gap: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, content: @Composable RowScope.() -> Unit) {
    val shape = RoundedCornerShape(DoweDesign.radiusUi)
    val surface = modifier
        .then(if (wide) Modifier.fillMaxWidth() else Modifier)
        .clip(shape)
        .background(if (active) backgroundColor else Color.Transparent)
        .then(if (active && borderColor != null) Modifier.border(1.dp, borderColor, shape) else Modifier)
        .then(if (onClick == null) Modifier else Modifier.clickable(onClick = onClick))
        .padding(horizontal = paddingHorizontal, vertical = paddingVertical)
    CompositionLocalProvider(LocalContentColor provides if (active) contentColor else LocalContentColor.current) {
        Row(modifier = surface, horizontalArrangement = Arrangement.spacedBy(gap), verticalAlignment = Alignment.CenterVertically, content = content)
    }
}

@Composable
private fun DoweSideNavSubmenu(open: Boolean, trigger: @Composable ((() -> Unit) -> Unit), content: @Composable () -> Unit) {
    var expanded by remember { mutableStateOf(open) }
    Column {
        trigger { expanded = !expanded }
        AnimatedVisibility(
            visible = expanded,
            enter = fadeIn(animationSpec = tween(160)) + expandVertically(animationSpec = tween(180)),
            exit = fadeOut(animationSpec = tween(120)) + shrinkVertically(animationSpec = tween(180))
        ) {
            Column(modifier = Modifier.padding(start = 16.dp)) {
                content()
            }
        }
    }
}

@Composable
private fun DoweNavMenu(modifier: Modifier = Modifier, gap: Dp, popoverBackgroundColor: Color, popoverContentColor: Color, content: @Composable RowScope.(Int?, (Int) -> Unit) -> Unit, popover: @Composable (Int?) -> Unit) {
    var openIndex by remember { mutableStateOf<Int?>(null) }
    Column(modifier = modifier) {
        Row(horizontalArrangement = Arrangement.spacedBy(gap), verticalAlignment = Alignment.CenterVertically) {
            content(openIndex) { index -> openIndex = if (openIndex == index) null else index }
        }
        if (openIndex != null) {
            Popup(onDismissRequest = { openIndex = null }, properties = PopupProperties(focusable = true)) {
                Card(
                    colors = CardDefaults.cardColors(containerColor = popoverBackgroundColor, contentColor = popoverContentColor),
                    shape = RoundedCornerShape(DoweDesign.radiusBox),
                    elevation = CardDefaults.cardElevation(defaultElevation = 8.dp)
                ) {
                    Column(modifier = Modifier.widthIn(min = 192.dp, max = 720.dp).heightIn(max = 640.dp).padding(8.dp)) {
                        popover(openIndex)
                    }
                }
            }
        }
    }
}

@Composable
private fun DoweNavMenuItem(active: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, content: @Composable RowScope.() -> Unit) {
    val shape = RoundedCornerShape(DoweDesign.radiusBox)
    val surface = Modifier
        .clip(shape)
        .background(if (active) backgroundColor else Color.Transparent)
        .then(if (active && borderColor != null) Modifier.border(1.dp, borderColor, shape) else Modifier)
        .then(if (onClick == null) Modifier else Modifier.clickable(onClick = onClick))
        .padding(horizontal = paddingHorizontal, vertical = paddingVertical)
    CompositionLocalProvider(LocalContentColor provides if (active) contentColor else LocalContentColor.current) {
        Row(modifier = surface, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically, content = content)
    }
}

@Composable
private fun DoweDrawer(open: Boolean, onClose: () -> Unit, position: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp, disableOverlayClose: Boolean, hideCloseButton: Boolean, content: @Composable () -> Unit) {
    if (!open) {
        return
    }
    Popup(onDismissRequest = onClose, properties = PopupProperties(focusable = true)) {
        Box(modifier = Modifier.fillMaxSize()) {
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .background(Color.Black.copy(alpha = 0.48f))
                    .clickable(enabled = !disableOverlayClose, onClick = onClose)
            )
            val panelModifier = if (position == "start" || position == "end") {
                Modifier.fillMaxHeight().widthIn(max = 320.dp)
            } else {
                Modifier.fillMaxWidth().heightIn(max = 320.dp)
            }
            val shape = doweDrawerShape(position, radius)
            Box(
                modifier = panelModifier
                    .align(doweDrawerAlignment(position))
                    .clip(shape)
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            ) {
                CompositionLocalProvider(LocalContentColor provides contentColor) {
                    content()
                }
                if (!hideCloseButton) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(8.dp)
                            .clip(RoundedCornerShape(999.dp))
                            .background(DoweDesign.softMuted)
                            .clickable(onClick = onClose)
                            .width(28.dp)
                            .height(28.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(text = "x", color = DoweDesign.onSoftMuted)
                    }
                }
            }
        }
    }
}

private fun doweDrawerAlignment(position: String): Alignment =
    when (position) {
        "end" -> Alignment.CenterEnd
        "top" -> Alignment.TopCenter
        "bottom" -> Alignment.BottomCenter
        else -> Alignment.CenterStart
    }

private fun doweDrawerShape(position: String, radius: Dp): RoundedCornerShape =
    when (position) {
        "end" -> RoundedCornerShape(topStart = radius, topEnd = 0.dp, bottomEnd = 0.dp, bottomStart = radius)
        "top" -> RoundedCornerShape(topStart = 0.dp, topEnd = 0.dp, bottomEnd = radius, bottomStart = radius)
        "bottom" -> RoundedCornerShape(topStart = radius, topEnd = radius, bottomEnd = 0.dp, bottomStart = 0.dp)
        else -> RoundedCornerShape(topStart = 0.dp, topEnd = radius, bottomEnd = radius, bottomStart = 0.dp)
    }

@Composable
private fun DoweSectionBackgroundBox(modifier: Modifier = Modifier, background: DoweSectionBackground?, content: @Composable () -> Unit) {
    val backgroundModifier = if (background == null) Modifier else Modifier.background(doweSectionBackgroundBrush(background))
    Box(modifier = modifier.then(backgroundModifier).clipToBounds()) {
        content()
    }
}

private fun doweSectionBackgroundBrush(background: DoweSectionBackground): Brush =
    when (background) {
        DoweSectionBackground.Soft -> Brush.linearGradient(listOf(DoweDesign.surface, DoweDesign.background))
        DoweSectionBackground.Aurora -> Brush.linearGradient(listOf(DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary))
        DoweSectionBackground.Sunrise -> Brush.linearGradient(listOf(DoweDesign.softWarning, DoweDesign.softDanger, DoweDesign.surface))
        DoweSectionBackground.Ocean -> Brush.linearGradient(listOf(DoweDesign.softInfo, DoweDesign.softPrimary, DoweDesign.softTertiary))
        DoweSectionBackground.Meadow -> Brush.linearGradient(listOf(DoweDesign.softSuccess, DoweDesign.softTertiary, DoweDesign.surface))
        DoweSectionBackground.Slate -> Brush.linearGradient(listOf(DoweDesign.softMuted, DoweDesign.surface, DoweDesign.background))
    }

@Composable
private fun DoweCoverBox(modifier: Modifier = Modifier, source: String?, overlay: DoweOverlay?, content: @Composable () -> Unit) {
    Box(modifier = modifier.clipToBounds()) {
        if (source != null) {
            AndroidView(
                modifier = Modifier.matchParentSize(),
                factory = { context ->
                    ImageView(context).apply {
                        scaleType = ImageView.ScaleType.CENTER_CROP
                        setImageURI(Uri.parse(source))
                    }
                },
                update = { view ->
                    view.setImageURI(Uri.parse(source))
                }
            )
        }
        when (overlay) {
            is DoweOverlay.Solid -> Box(modifier = Modifier.matchParentSize().background(overlay.color))
            is DoweOverlay.Gradient -> Box(modifier = Modifier.matchParentSize().background(Brush.verticalGradient(listOf(overlay.start, overlay.end))))
            null -> {}
        }
        content()
    }
}

@Composable
private fun DoweGrid(modifier: Modifier = Modifier, columns: Int, horizontalGap: Dp, verticalGap: Dp, horizontalAlignment: Alignment.Horizontal, content: @Composable () -> Unit) {
    val density = LocalDensity.current
    Layout(content = content, modifier = modifier) { measurables, constraints ->
        val columnCount = columns.coerceAtLeast(1)
        val horizontal = with(density) { horizontalGap.roundToPx() }
        val vertical = with(density) { verticalGap.roundToPx() }
        val cellWidth = ((constraints.maxWidth - horizontal * (columnCount - 1)).coerceAtLeast(0)) / columnCount
        val placeables = measurables.map { it.measure(constraints.copy(minWidth = 0, maxWidth = cellWidth)) }
        val rowHeights = placeables.chunked(columnCount).map { row -> row.maxOfOrNull { it.height } ?: 0 }
        val height = rowHeights.sum() + vertical * (rowHeights.size - 1).coerceAtLeast(0)
        layout(constraints.maxWidth, height.coerceIn(constraints.minHeight, constraints.maxHeight)) {
            var top = 0
            placeables.chunked(columnCount).forEachIndexed { rowIndex, row ->
                row.forEachIndexed { columnIndex, placeable ->
                    val offset = horizontalAlignment.align(placeable.width, cellWidth, layoutDirection)
                    placeable.placeRelative(columnIndex * (cellWidth + horizontal) + offset, top)
                }
                top += rowHeights[rowIndex] + vertical
            }
        }
    }
}

"#
}
