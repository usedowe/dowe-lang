fn android_runtime_navigation_drawer_layout() -> &'static str {
    r#"private data class DoweTabItem(val id: String, val label: String)

@Composable
private fun DoweTabs(items: List<DoweTabItem>, initialId: String, modifier: Modifier, position: String, variant: String, backgroundColor: Color, contentColor: Color, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, borderColor: Color?, radius: Dp, fontFamily: FontFamily, content: @Composable (String) -> Unit) {
    var activeId by remember(initialId) { mutableStateOf(initialId) }
    val vertical = position == "start" || position == "end"
    val listShape = RoundedCornerShape(if (variant == "pills") 999.dp else radius)
    val listModifier = Modifier
        .wrapContentWidth()
        .clip(listShape)
        .background(backgroundColor)
        .then(if (borderColor == null || variant == "line") Modifier else Modifier.border(1.dp, borderColor, listShape))
        .padding(if (variant == "line" || variant == "ghost") 0.dp else 4.dp)
    val tabList: @Composable () -> Unit = {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            if (vertical) {
                Column(modifier = listModifier, verticalArrangement = Arrangement.spacedBy(if (variant == "stepper") 0.dp else if (variant == "line") 16.dp else 8.dp)) {
                    items.forEachIndexed { index, item ->
                        DoweTabButton(item = item, index = index, active = activeId == item.id, position = position, variant = variant, activeBackgroundColor = activeBackgroundColor, activeContentColor = activeContentColor, accentColor = accentColor, radius = radius, fontFamily = fontFamily) {
                            activeId = item.id
                        }
                        if (variant == "stepper" && index < items.lastIndex) Box(modifier = Modifier.padding(start = 15.dp).width(2.dp).height(20.dp).background(contentColor.copy(alpha = 0.35f)))
                    }
                }
            } else {
                Row(modifier = listModifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(if (variant == "stepper") 0.dp else if (variant == "line") 16.dp else 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    items.forEachIndexed { index, item ->
                        DoweTabButton(item = item, index = index, active = activeId == item.id, position = position, variant = variant, activeBackgroundColor = activeBackgroundColor, activeContentColor = activeContentColor, accentColor = accentColor, radius = radius, fontFamily = fontFamily) {
                            activeId = item.id
                        }
                        if (variant == "stepper" && index < items.lastIndex) Box(modifier = Modifier.padding(horizontal = 8.dp).width(48.dp).height(2.dp).background(contentColor.copy(alpha = 0.35f)))
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
private fun DoweTabButton(item: DoweTabItem, index: Int, active: Boolean, position: String, variant: String, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, radius: Dp, fontFamily: FontFamily, onClick: () -> Unit) {
    val shape = RoundedCornerShape(if (variant == "pills") 999.dp else radius)
    val selectedFill = variant == "solid" || variant == "outlined" || variant == "pills"
    val selectedLine = variant == "line"
    val background = if (active && selectedFill) activeBackgroundColor else Color.Transparent
    val color = if (!active) LocalContentColor.current else if (selectedFill) activeContentColor else accentColor
    Box(
        modifier = Modifier
            .clip(shape)
            .background(background)
            .then(if (!active || !selectedLine) Modifier else Modifier.drawBehind {
                val strokeWidth = 2.dp.toPx()
                val halfStroke = strokeWidth / 2f
                val startAtLeft = layoutDirection == androidx.compose.ui.unit.LayoutDirection.Ltr
                when (position) {
                    "start" -> {
                        val x = if (startAtLeft) halfStroke else size.width - halfStroke
                        drawLine(accentColor, Offset(x, 0f), Offset(x, size.height), strokeWidth)
                    }
                    "end" -> {
                        val x = if (startAtLeft) size.width - halfStroke else halfStroke
                        drawLine(accentColor, Offset(x, 0f), Offset(x, size.height), strokeWidth)
                    }
                    else -> drawLine(accentColor, Offset(0f, size.height - halfStroke), Offset(size.width, size.height - halfStroke), strokeWidth)
                }
            })
            .clickable(onClick = onClick)
            .padding(horizontal = if (variant == "stepper") 0.dp else 16.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center
    ) {
        if (variant == "stepper") {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(modifier = Modifier.size(32.dp).clip(CircleShape).background(if (active) activeBackgroundColor else Color.Transparent).border(2.dp, if (active) accentColor else LocalContentColor.current.copy(alpha = 0.45f), CircleShape), contentAlignment = Alignment.Center) {
                    Text(text = (index + 1).toString(), color = if (active) activeContentColor else LocalContentColor.current, fontFamily = fontFamily, fontWeight = FontWeight.Bold)
                }
                Text(text = item.label, color = color, fontFamily = fontFamily, maxLines = 1)
            }
        } else {
            Text(text = item.label, color = color, fontFamily = fontFamily)
        }
    }
}

@Composable
private fun DoweSideNavRow(modifier: Modifier = Modifier, active: Boolean, wide: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, gap: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, content: @Composable RowScope.() -> Unit) {
    val shape = RoundedCornerShape(DoweDesign.radius)
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
private fun DoweRailNavItem(label: String, showLabel: Boolean, active: Boolean, itemSize: Dp, labelSize: Float, backgroundColor: Color, contentColor: Color, borderColor: Color?, featured: Boolean = false, onClick: (() -> Unit)?, content: @Composable () -> Unit) {
    val selected = active || featured
    val shape = RoundedCornerShape(if (featured) itemSize / 2 else DoweDesign.radius)
    val surface = Modifier
        .width(itemSize)
        .heightIn(min = itemSize)
        .clip(shape)
        .background(if (selected) backgroundColor else Color.Transparent)
        .then(if (selected && borderColor != null) Modifier.border(1.dp, borderColor, shape) else Modifier)
        .then(if (onClick == null) Modifier else Modifier.clickable(onClick = onClick))
        .semantics { contentDescription = label }
        .padding(6.dp)
    CompositionLocalProvider(LocalContentColor provides if (selected) contentColor else LocalContentColor.current) {
        Column(modifier = surface, horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(4.dp)) {
            content()
            if (showLabel) {
                Text(text = label, modifier = Modifier.fillMaxWidth(), fontSize = labelSize.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, textAlign = TextAlign.Center)
            }
        }
    }
}

@Composable
private fun DoweSideNavStatus(text: String, descriptionSize: Float, fontFamily: FontFamily) {
    Text(
        text = text,
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(DoweDesign.muted)
            .padding(horizontal = 8.dp, vertical = 2.dp),
        color = DoweDesign.mutedText,
        fontSize = descriptionSize.sp,
        fontFamily = fontFamily,
        fontWeight = FontWeight.SemiBold
    )
}

private data class DoweSideNavEntry(val id: String, val kind: String, val label: String, val description: String?, val status: String?, val operation: String?, val path: String?, val fragment: String?, val open: Boolean = false, val bordered: Boolean = true, val children: List<DoweSideNavEntry> = emptyList())

@Composable
private fun DoweSideNav(items: List<DoweSideNavEntry>, stateKey: String, modifier: Modifier = Modifier, activePath: String, wide: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, gap: Dp, labelSize: Float, descriptionSize: Float, fontFamily: FontFamily, backgroundColor: Color, contentColor: Color, titleColor: Color, activeContentColor: Color, borderColor: Color?, navigate: (String, String, String?) -> Unit) {
    Column(modifier = modifier.then(if (wide) Modifier.fillMaxWidth() else Modifier), verticalArrangement = Arrangement.spacedBy(2.dp)) {
        items.forEach { item ->
            DoweSideNavEntryView(item = item, stateKey = "$stateKey:${item.id}", activePath = activePath, wide = wide, paddingHorizontal = paddingHorizontal, paddingVertical = paddingVertical, gap = gap, labelSize = labelSize, descriptionSize = descriptionSize, fontFamily = fontFamily, backgroundColor = backgroundColor, contentColor = contentColor, titleColor = titleColor, activeContentColor = activeContentColor, borderColor = borderColor, navigate = navigate)
        }
    }
}

@Composable
private fun DoweSideNavEntryView(item: DoweSideNavEntry, stateKey: String, activePath: String, wide: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, gap: Dp, labelSize: Float, descriptionSize: Float, fontFamily: FontFamily, backgroundColor: Color, contentColor: Color, titleColor: Color, activeContentColor: Color, borderColor: Color?, navigate: (String, String, String?) -> Unit) {
    when (item.kind) {
        "divider" -> Box(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp).height(1.dp).background(DoweDesign.muted))
        "submenu" -> DoweSideNavSubmenu(stateKey = stateKey, open = item.open, bordered = item.bordered, wide = wide, trigger = { expanded, toggle ->
            DoweSideNavEntryRow(item = item, header = false, activePath = activePath, wide = wide, paddingHorizontal = paddingHorizontal, paddingVertical = paddingVertical, gap = gap, labelSize = labelSize, descriptionSize = descriptionSize, fontFamily = fontFamily, backgroundColor = backgroundColor, contentColor = contentColor, titleColor = titleColor, borderColor = borderColor, onClick = toggle, submenuExpanded = expanded)
        }) {
            item.children.forEach { child ->
                DoweSideNavEntryRow(item = child, header = false, activePath = activePath, wide = wide, paddingHorizontal = paddingHorizontal, paddingVertical = paddingVertical, gap = gap, labelSize = labelSize, descriptionSize = descriptionSize, fontFamily = fontFamily, backgroundColor = backgroundColor, contentColor = contentColor, titleColor = titleColor, borderColor = borderColor, onClick = sideNavAction(child, navigate))
            }
        }
        "header" -> DoweSideNavEntryRow(item = item, header = true, activePath = activePath, wide = wide, paddingHorizontal = paddingHorizontal, paddingVertical = paddingVertical, gap = gap, labelSize = labelSize, descriptionSize = descriptionSize, fontFamily = fontFamily, backgroundColor = backgroundColor, contentColor = contentColor, titleColor = titleColor, borderColor = borderColor, onClick = sideNavAction(item, navigate))
        else -> DoweSideNavEntryRow(item = item, header = false, activePath = activePath, wide = wide, paddingHorizontal = paddingHorizontal, paddingVertical = paddingVertical, gap = gap, labelSize = labelSize, descriptionSize = descriptionSize, fontFamily = fontFamily, backgroundColor = backgroundColor, contentColor = contentColor, titleColor = titleColor, borderColor = borderColor, onClick = sideNavAction(item, navigate))
    }
}

@Composable
private fun DoweSideNavEntryRow(item: DoweSideNavEntry, header: Boolean, activePath: String, wide: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, gap: Dp, labelSize: Float, descriptionSize: Float, fontFamily: FontFamily, backgroundColor: Color, contentColor: Color, titleColor: Color, borderColor: Color?, onClick: (() -> Unit)?, submenuExpanded: Boolean? = null) {
    DoweSideNavRow(active = item.path == activePath, wide = wide, paddingHorizontal = paddingHorizontal, paddingVertical = paddingVertical, gap = gap, backgroundColor = backgroundColor, contentColor = contentColor, borderColor = borderColor, onClick = onClick) {
        Column(modifier = Modifier.weight(1f)) {
            Text(text = item.label, fontSize = labelSize.sp, fontFamily = fontFamily, fontWeight = if (header) FontWeight.SemiBold else FontWeight.Normal, color = if (header) titleColor else LocalContentColor.current)
            item.description?.let { description ->
                Text(text = description, fontSize = descriptionSize.sp, fontFamily = fontFamily, color = LocalContentColor.current.copy(alpha = 0.72f))
            }
        }
        if (item.status != null || submenuExpanded != null) {
            Row(horizontalArrangement = Arrangement.spacedBy(gap), verticalAlignment = Alignment.CenterVertically) {
                item.status?.let { status ->
                    DoweSideNavStatus(text = status, descriptionSize = descriptionSize, fontFamily = fontFamily)
                }
                submenuExpanded?.let { expanded ->
                    DoweSideNavArrow(expanded = expanded)
                }
            }
        }
    }
}

private fun sideNavAction(item: DoweSideNavEntry, navigate: (String, String, String?) -> Unit): (() -> Unit)? {
    val path = item.path ?: return null
    return { navigate(item.operation ?: "push", path, item.fragment) }
}

private val doweSideNavArrowViewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)
private val doweSideNavArrowPaths = listOf(
    DoweSvgPath("M0 0h24v24H0z", DoweSvgFill.None),
    DoweSvgPath("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__", DoweSvgFill.CurrentColor)
)

@Composable
private fun DoweSideNavArrow(expanded: Boolean) {
    val rotation by animateFloatAsState(targetValue = if (expanded) 90f else 0f, animationSpec = tween(160))
    DoweSvg(viewBox = doweSideNavArrowViewBox, modifier = Modifier.width(16.dp).height(16.dp).graphicsLayer { rotationZ = rotation }, color = LocalContentColor.current, paths = doweSideNavArrowPaths)
}

private val doweSideNavExpandedMemory = mutableStateMapOf<String, Boolean>()

@Composable
private fun DoweSideNavSubmenu(stateKey: String, open: Boolean, bordered: Boolean, wide: Boolean, trigger: @Composable ((Boolean, () -> Unit) -> Unit), content: @Composable () -> Unit) {
    val expanded = doweSideNavExpandedMemory.getOrPut(stateKey) { open }
    Column(modifier = Modifier.then(if (wide) Modifier.fillMaxWidth() else Modifier)) {
        trigger(expanded) { doweSideNavExpandedMemory[stateKey] = !expanded }
        AnimatedVisibility(
            visible = expanded,
            enter = fadeIn(animationSpec = tween(160)) + expandVertically(animationSpec = tween(180)),
            exit = fadeOut(animationSpec = tween(120)) + shrinkVertically(animationSpec = tween(180))
        ) {
            Column(
                modifier = Modifier
                    .padding(start = 16.dp)
                    .then(if (bordered) Modifier.drawBehind { drawLine(DoweDesign.muted, Offset(0f, 0f), Offset(0f, size.height), strokeWidth = 1.dp.toPx()) } else Modifier)
                    .padding(start = if (bordered) 8.dp else 0.dp)
                    .then(if (wide) Modifier.fillMaxWidth() else Modifier)
            ) {
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
                    shape = RoundedCornerShape(DoweDesign.radius),
                    elevation = CardDefaults.cardElevation(defaultElevation = 8.dp)
                ) {
                    DoweNavMenuPopoverSurface(onDismiss = { openIndex = null }) {
                        Column(modifier = Modifier.widthIn(min = 192.dp, max = 720.dp).heightIn(max = 640.dp).padding(8.dp)) {
                            popover(openIndex)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun DoweNavMenuPopoverSurface(onDismiss: () -> Unit, content: @Composable () -> Unit) {
    Box(
        modifier = Modifier.pointerInput(onDismiss) {
            awaitPointerEventScope {
                while (true) {
                    val event = awaitPointerEvent(PointerEventPass.Final)
                    if (event.type == PointerEventType.Release) {
                        onDismiss()
                        break
                    }
                }
            }
        }
    ) {
        content()
    }
}

@Composable
private fun DoweNavMenuItem(active: Boolean, paddingHorizontal: Dp, paddingVertical: Dp, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, content: @Composable RowScope.() -> Unit) {
    val shape = RoundedCornerShape(DoweDesign.radius)
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
            }
            if (!hideCloseButton) {
                val closeAlignment = when (position) {
                    "end" -> Alignment.TopStart
                    "top" -> Alignment.BottomEnd
                    else -> Alignment.TopEnd
                }
                Box(
                    modifier = Modifier
                        .align(closeAlignment)
                        .padding(8.dp)
                        .clip(RoundedCornerShape(999.dp))
                        .background(DoweDesign.muted)
                        .clickable(onClick = onClose)
                        .width(28.dp)
                        .height(28.dp),
                    contentAlignment = Alignment.Center
                ) {
                    DoweSvg(viewBox = doweOverlayCloseViewBox, modifier = Modifier.width(18.dp).height(18.dp), color = DoweDesign.mutedText, paths = doweOverlayClosePaths)
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
        DoweSectionBackground.Aurora -> Brush.linearGradient(listOf(DoweDesign.primary, DoweDesign.secondary, DoweDesign.accent))
        DoweSectionBackground.Sunrise -> Brush.linearGradient(listOf(DoweDesign.warning, DoweDesign.danger, DoweDesign.surface))
        DoweSectionBackground.Ocean -> Brush.linearGradient(listOf(DoweDesign.info, DoweDesign.primary, DoweDesign.accent))
        DoweSectionBackground.Meadow -> Brush.linearGradient(listOf(DoweDesign.success, DoweDesign.accent, DoweDesign.surface))
        DoweSectionBackground.Slate -> Brush.linearGradient(listOf(DoweDesign.muted, DoweDesign.surface, DoweDesign.background))
    }

@Composable
private fun DoweCoverBox(modifier: Modifier = Modifier, source: String?, overlay: DoweOverlay?, content: @Composable BoxScope.() -> Unit) {
    val context = LocalContext.current
    var bitmap by remember(source) { mutableStateOf<android.graphics.Bitmap?>(null) }
    LaunchedEffect(source) {
        bitmap = if (source == null) null else withContext(Dispatchers.IO) { doweLoadImageBitmap(context, source) }
    }
    Box(modifier = modifier.clipToBounds()) {
        bitmap?.let { image ->
            Image(
                bitmap = image.asImageBitmap(),
                contentDescription = null,
                modifier = Modifier.matchParentSize(),
                contentScale = ContentScale.Crop
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
private fun DoweGrid(modifier: Modifier = Modifier, tracks: List<Float>, horizontalGap: Dp, verticalGap: Dp, horizontalAlignment: Alignment.Horizontal, verticalAlignment: Alignment.Vertical, horizontalStretch: Boolean, fillHeight: Boolean, verticalStretch: Boolean, content: @Composable () -> Unit) {
    val density = LocalDensity.current
    Layout(content = content, modifier = modifier) { measurables, constraints ->
        val weights = tracks.ifEmpty { listOf(1f) }
        val columnCount = weights.size
        val horizontal = with(density) { horizontalGap.roundToPx() }
        val vertical = with(density) { verticalGap.roundToPx() }
        val availableWidth = (constraints.maxWidth - horizontal * (columnCount - 1)).coerceAtLeast(0)
        val totalWeight = weights.sum().coerceAtLeast(1f)
        val cellWidths = weights.map { (availableWidth * it / totalWeight).toInt() }
        val placeables = measurables.mapIndexed { index, measurable ->
            measurable.measure(constraints.copy(minWidth = if (horizontalStretch) cellWidths[index % columnCount] else 0, maxWidth = cellWidths[index % columnCount]))
        }
        val intrinsicRowHeights = placeables.chunked(columnCount).map { row -> row.maxOfOrNull { it.height } ?: 0 }
        val intrinsicHeight = intrinsicRowHeights.sum() + vertical * (intrinsicRowHeights.size - 1).coerceAtLeast(0)
        val height = if (fillHeight) {
            constraints.maxHeight.coerceAtLeast(constraints.minHeight)
        } else {
            intrinsicHeight.coerceIn(constraints.minHeight, constraints.maxHeight)
        }
        val extraHeight = (height - intrinsicHeight).coerceAtLeast(0)
        val rowExtra = if (intrinsicRowHeights.isEmpty()) 0 else extraHeight / intrinsicRowHeights.size
        val rowRemainder = if (intrinsicRowHeights.isEmpty()) 0 else extraHeight % intrinsicRowHeights.size
        val rowHeights = intrinsicRowHeights.mapIndexed { index, rowHeight ->
            rowHeight + rowExtra + if (index < rowRemainder) 1 else 0
        }
        val laidOutPlaceables = if (verticalStretch) {
            measurables.mapIndexed { index, measurable ->
                val rowHeight = rowHeights[index / columnCount]
                val stretched = measurable.measure(constraints.copy(
                    minWidth = if (horizontalStretch) cellWidths[index % columnCount] else 0,
                    maxWidth = cellWidths[index % columnCount],
                    minHeight = 0,
                    maxHeight = rowHeight
                ))
                if (stretched.height >= rowHeight) stretched else placeables[index]
            }
        } else {
            placeables
        }
        layout(constraints.maxWidth, height) {
            var top = 0
            laidOutPlaceables.chunked(columnCount).forEachIndexed { rowIndex, row ->
                var left = 0
                row.forEachIndexed { columnIndex, placeable ->
                    val cellWidth = cellWidths[columnIndex]
                    val horizontalOffset = horizontalAlignment.align(placeable.width, cellWidth, layoutDirection)
                    val verticalOffset = verticalAlignment.align(placeable.height, rowHeights[rowIndex])
                    placeable.placeRelative(left + horizontalOffset, top + verticalOffset)
                    left += cellWidth + horizontal
                }
                top += rowHeights[rowIndex] + vertical
            }
        }
    }
}

"#
}
