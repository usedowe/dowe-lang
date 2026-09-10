r#"    LaunchedEffect(expanded) {
        if (expanded) {
            popupMounted = true
        } else if (popupMounted) {
            delay(160)
            popupMounted = false
        }
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        Box {
            Row(modifier = Modifier.fillMaxWidth().heightIn(min = minHeight).onGloballyPositioned { triggerHeight = it.size.height }.clip(RoundedCornerShape(12.dp)).background(backgroundColor).border(1.dp, if (validationError != null) DoweDesign.danger else contentColor.copy(alpha = 0.22f), RoundedCornerShape(12.dp)).padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Row(modifier = Modifier.clickable(enabled = !disabled && countries.isNotEmpty()) { expanded = true }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    if (selected != null) DoweSvg(viewBox = selected.viewBox, modifier = Modifier.size(24.dp).align(Alignment.CenterVertically).clip(RoundedCornerShape(999.dp)), color = contentColor, paths = selected.paths)
                    Text(if (selected == null) "+$country" else "+${selected.dialCode}", modifier = Modifier.align(Alignment.CenterVertically), fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.Bold, color = contentColor)
                    DoweSvg(viewBox = doweSelectArrowViewBox, modifier = Modifier.size(16.dp).align(Alignment.CenterVertically), color = contentColor, paths = doweSelectArrowPaths)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Box(modifier = Modifier.weight(1f).heightIn(min = minHeight), contentAlignment = Alignment.CenterStart) {
                    if (label != null && floating) Text(label, modifier = Modifier.align(Alignment.TopStart), fontSize = if (localValue.isEmpty()) fontSize else 12.sp, color = contentColor, fontWeight = FontWeight.SemiBold)
                    if (localValue.isEmpty() && (!floating || expanded)) Text(placeholder, modifier = Modifier.padding(top = if (label != null && floating) 10.dp else 0.dp), color = contentColor.copy(alpha = 0.55f), fontSize = fontSize, lineHeight = lineHeight)
                    BasicTextField(value = localValue, onValueChange = { next -> if (!disabled) { val filtered = next.filter { char -> char.isDigit() }; localValue = filtered; onValueChange(filtered) } }, modifier = Modifier.fillMaxWidth().heightIn(min = minHeight).padding(top = if (label != null && floating) 10.dp else 0.dp).onFocusChanged { state -> if (state.isFocused) hadFocus = true else if (hadFocus) touched = true }, singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), textStyle = TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight), enabled = !disabled)
                }
            }
            if (triggerHeight > 0 && (expanded || popupMounted)) DoweAnchoredPopover(visible = expanded, offset = popupOffset, shape = RoundedCornerShape(12.dp), backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, contentPadding = PaddingValues(0.dp), minWidth = 280.dp, maxWidth = 384.dp, maxHeight = 380.dp, onDismiss = { expanded = false; query = ""; touched = true }) {
                BasicTextField(value = query, onValueChange = { query = it }, modifier = Modifier.fillMaxWidth().padding(6.dp).clip(RoundedCornerShape(10.dp)).background(DoweDesign.surfaceText.copy(alpha = 0.07f)).padding(horizontal = 12.dp, vertical = 9.dp), singleLine = true, textStyle = TextStyle(color = DoweDesign.surfaceText), decorationBox = { inner -> Box { if (query.isEmpty()) Text(searchPlaceholder, color = DoweDesign.surfaceText.copy(alpha = 0.55f)); inner() } })
                if (countries.isEmpty()) Text(loadingText, modifier = Modifier.padding(16.dp), color = DoweDesign.surfaceText.copy(alpha = 0.68f))
                else if (filtered.isEmpty()) Text(emptyText, modifier = Modifier.padding(16.dp), color = DoweDesign.surfaceText.copy(alpha = 0.68f))
                else filtered.forEach { item ->
                    Row(modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).clickable { selectedCode = item.code; expanded = false; query = ""; touched = true }.background(if (item.code == selectedCode) DoweDesign.surfaceText.copy(alpha = 0.07f) else Color.Transparent).padding(horizontal = 12.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        DoweSvg(viewBox = item.viewBox, modifier = Modifier.size(28.dp).clip(RoundedCornerShape(999.dp)), color = DoweDesign.surfaceText, paths = item.paths)
                        Text(item.name, modifier = Modifier.weight(1f), fontWeight = FontWeight.SemiBold, color = DoweDesign.surfaceText, maxLines = 1)
                        Text("+${item.dialCode}", fontWeight = FontWeight.Bold, color = DoweDesign.surfaceText)
                    }
                }
            }
        }
        DoweValidationFeedback(helpText, validationError, contentColor)
    }
}

@Composable
private fun DowePin(value: String, onValueChange: (String) -> Unit, label: String?, length: Int, kind: String, size: String, fontSize: TextUnit, lineHeight: TextUnit, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?, helpText: String?, errorText: String?, validationRules: List<DoweValidationRule> = emptyList()) {
    var cells by remember(value, length) { mutableStateOf(value.padEnd(length).take(length).map { if (it == ' ') "" else it.toString() }) }
    var hadFocus by remember { mutableStateOf(false) }
    var touched by remember { mutableStateOf(false) }
    val currentValue = cells.joinToString("")
    val validationError = errorText ?: if (touched) doweValidationError(currentValue, validationRules) else null
    val focusRequesters = remember(length) { List(length) { FocusRequester() } }
    val cellWidth = when (size) {
        "sm" -> 40.dp
        "lg" -> 52.dp
        else -> 44.dp
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
            val cellGap = 8.dp
            val cellCount = length.coerceAtLeast(1)
            val responsiveCellWidth = minOf(cellWidth, ((maxWidth - cellGap * (cellCount - 1)).coerceAtLeast(1.dp) / cellCount.toFloat()))
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(cellGap)) {
                cells.forEachIndexed { index, cell ->
                    val cellModifier = Modifier
                        .width(responsiveCellWidth)
                        .height(doweControlHeight(size))
                        .clip(shape)
                        .background(backgroundColor)
                        .then(if (borderColor == null && validationError == null) Modifier else Modifier.border(1.dp, if (validationError != null) DoweDesign.danger else borderColor!!, shape))
                        .padding(horizontal = if (size == "sm") 8.dp else 12.dp)
                        .focusRequester(focusRequesters[index])
                        .onFocusChanged { state -> if (state.isFocused) hadFocus = true else if (hadFocus) touched = true }
                        .onPreviewKeyEvent { event ->
                            if (event.type == KeyEventType.KeyDown && event.nativeKeyEvent.keyCode == android.view.KeyEvent.KEYCODE_DEL && cell.isEmpty() && index > 0) {
                                focusRequesters[index - 1].requestFocus()
                                true
                            } else {
                                false
                            }
                        }
                    BasicTextField(value = cell, onValueChange = { next ->
                        val filtered = if (kind == "number") next.filter { it.isDigit() } else next
                        val updated = cells.toMutableList()
                        if (filtered.length > 1) {
                            filtered.take(length - index).forEachIndexed { offset, character -> updated[index + offset] = character.toString() }
                        } else {
                            updated[index] = filtered.takeLast(1)
                        }
                        cells = updated
                        onValueChange(updated.joinToString(""))
                        if (filtered.isNotEmpty()) {
                            val focusIndex = if (filtered.length > 1) minOf(index + filtered.length - 1, length - 1) else index + 1
                            if (focusIndex < length) focusRequesters[focusIndex].requestFocus()
                        }
                    }, modifier = cellModifier, singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = if (kind == "number") KeyboardType.Number else KeyboardType.Text), textStyle = TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.Bold, textAlign = TextAlign.Center), visualTransformation = if (kind == "password") PasswordVisualTransformation() else VisualTransformation.None)
                }
            }
        }
        DoweValidationFeedback(helpText, validationError, contentColor)
    }
}

@Composable
private fun DoweTextarea(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, rows: Int, maxLength: Int?, fontSize: TextUnit, lineHeight: TextUnit, readOnly: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    var focused by remember { mutableStateOf(false) }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        BasicTextField(value = value, onValueChange = { next -> if (!readOnly) onValueChange(maxLength?.let { next.take(it) } ?: next) }, modifier = Modifier.fillMaxWidth().heightIn(min = (rows * 28).dp).clip(RoundedCornerShape(12.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(12.dp)).padding(12.dp).onFocusChanged { focused = it.isFocused }, textStyle = TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight), decorationBox = { inner -> Box(modifier = Modifier.fillMaxSize()) { if (value.isEmpty() && placeholder.isNotEmpty() && (!floating || focused)) Text(placeholder, modifier = Modifier.align(Alignment.TopStart).padding(top = if (floating) 18.dp else 0.dp), color = contentColor.copy(alpha = 0.55f), fontSize = fontSize, lineHeight = lineHeight); if (label != null && floating) Text(label, modifier = Modifier.align(Alignment.TopStart), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = contentColor.copy(alpha = 0.72f)); Box(modifier = Modifier.align(Alignment.TopStart).padding(top = if (label != null && floating) 18.dp else 0.dp)) { inner() } } })
    }
}

private fun <T> doweResponsive(viewportWidth: Dp, xs: T? = null, sm: T? = null, md: T? = null, lg: T? = null, xl: T? = null): T? {
    var value: T? = null
    if (viewportWidth >= 0.dp && xs != null) {
        value = xs
    }
    if (viewportWidth >= 640.dp && sm != null) {
        value = sm
    }
    if (viewportWidth >= 768.dp && md != null) {
        value = md
    }
    if (viewportWidth >= 1024.dp && lg != null) {
        value = lg
    }
    if (viewportWidth >= 1280.dp && xl != null) {
        value = xl
    }
    return value
}

private fun doweTextSize(viewportWidth: Dp, min: Float, preferredBase: Float, preferredViewport: Float, max: Float): TextUnit {
    return (preferredBase + viewportWidth.value * preferredViewport / 100f).coerceIn(min, max).sp
}

private fun doweTextLineHeight(fontSize: TextUnit, lineHeight: Float): TextUnit {
    return (fontSize.value * lineHeight).sp
}

private fun Modifier.doweBackground(value: Color?): Modifier =
    if (value == null) this else background(value)

private fun Modifier.dowePadding(all: Dp?, horizontal: Dp?, vertical: Dp?, start: Dp?, end: Dp?, top: Dp?, bottom: Dp?): Modifier {
    return padding(
        start = start ?: horizontal ?: all ?: 0.dp,
        end = end ?: horizontal ?: all ?: 0.dp,
        top = top ?: vertical ?: all ?: 0.dp,
        bottom = bottom ?: vertical ?: all ?: 0.dp
    )
}

@Composable
private fun Modifier.doweWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> width(value.value)
        is DoweSize.Percent -> fillMaxWidth(value.fraction)
        is DoweSize.ViewportMinus -> width((LocalConfiguration.current.screenWidthDp.dp - value.inset).coerceAtLeast(0.dp))
        DoweSize.Full -> fillMaxWidth()
        DoweSize.Auto -> this
        null -> this
    }

@Composable
private fun doweViewportHeight(inset: Dp): Dp {
    return (LocalConfiguration.current.screenHeightDp.dp - inset).coerceAtLeast(0.dp)
}

@Composable
private fun Modifier.doweHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> height(value.value)
        is DoweSize.Percent -> this
        is DoweSize.ViewportMinus -> height(doweViewportHeight(value.inset))
        DoweSize.Full -> fillMaxHeight()
        DoweSize.Auto -> this
        null -> this
    }

private fun Modifier.doweMinWidthFraction(fraction: Float): Modifier =
    layout { measurable, constraints ->
        if (!constraints.hasBoundedWidth) {
            val placeable = measurable.measure(constraints)
            layout(placeable.width, placeable.height) {
                placeable.placeRelative(0, 0)
            }
        } else {
            val minimumWidth = (constraints.maxWidth * fraction)
                .toInt()
                .coerceIn(constraints.minWidth, constraints.maxWidth)
            val placeable = measurable.measure(constraints.copy(minWidth = minimumWidth))
            layout(placeable.width, placeable.height) {
                placeable.placeRelative(0, 0)
            }
        }
    }

private fun Modifier.doweMinWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> widthIn(min = value.value)
        is DoweSize.Percent -> doweMinWidthFraction(value.fraction)
        else -> this
    }

@Composable
private fun Modifier.doweMinHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> heightIn(min = value.value)
        is DoweSize.ViewportMinus -> heightIn(min = doweViewportHeight(value.inset))
        DoweSize.Full -> fillMaxHeight()
        else -> this
    }

@Composable
private fun Modifier.doweMaxWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> widthIn(max = value.value)
        is DoweSize.ViewportMinus -> widthIn(max = (LocalConfiguration.current.screenWidthDp.dp - value.inset).coerceAtLeast(0.dp))
        else -> this
    }

@Composable
private fun Modifier.doweMaxHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> heightIn(max = value.value)
        is DoweSize.ViewportMinus -> heightIn(max = doweViewportHeight(value.inset))
        DoweSize.Full -> doweMaxParentHeight()
        else -> this
    }

private fun Modifier.doweMaxParentHeight(): Modifier =
    layout { measurable, constraints ->
        val placeable = measurable.measure(constraints)
        val height = if (constraints.hasBoundedHeight) {
            minOf(placeable.height, constraints.maxHeight)
        } else {
            placeable.height
        }
        layout(placeable.width, height) {
            placeable.placeRelative(0, 0)
        }
    }

private fun Modifier.doweRounded(radius: Dp?): Modifier =
    if (radius == null) this else clip(RoundedCornerShape(radius))

private fun Modifier.doweBorder(width: Dp?, radius: Dp?): Modifier =
    if (width == null) this else border(width, DoweDesign.backgroundText, RoundedCornerShape(radius ?: DoweDesign.radius))

private fun doweHorizontalAlignment(value: DoweAlign?): Alignment.Horizontal =
    when (value) {
        DoweAlign.Center, DoweAlign.Stretch, DoweAlign.Baseline, DoweAlign.BaselineLast, DoweAlign.CenterSafe -> Alignment.CenterHorizontally
        DoweAlign.End, DoweAlign.EndSafe -> Alignment.End
        else -> Alignment.Start
    }

private fun doweGridHorizontalAlignment(value: DoweAlign?): Alignment.Horizontal =
    when (value) {
        DoweAlign.Center, DoweAlign.CenterSafe -> Alignment.CenterHorizontally
        DoweAlign.End, DoweAlign.EndSafe -> Alignment.End
        else -> Alignment.Start
    }

private fun doweGridHorizontalStretch(value: DoweAlign?): Boolean =
    value == DoweAlign.Stretch || value == DoweAlign.Normal

private fun doweGridVerticalAlignment(value: DoweAlign?): Alignment.Vertical =
    when (value) {
        DoweAlign.Center, DoweAlign.CenterSafe -> Alignment.CenterVertically
        DoweAlign.End, DoweAlign.EndSafe -> Alignment.Bottom
        else -> Alignment.Top
    }

private fun doweVerticalAlignment(value: DoweAlign?): Alignment.Vertical =
    when (value) {
        DoweAlign.Center, DoweAlign.Stretch, DoweAlign.Baseline, DoweAlign.BaselineLast, DoweAlign.CenterSafe -> Alignment.CenterVertically
        DoweAlign.End, DoweAlign.EndSafe -> Alignment.Bottom
        else -> Alignment.Top
    }

private fun doweHorizontalArrangement(value: DoweJustify?, gap: Dp?): Arrangement.Horizontal =
    when (value) {
        DoweJustify.Center, DoweJustify.CenterSafe -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.CenterHorizontally)
        DoweJustify.End, DoweJustify.EndSafe -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.End)
        DoweJustify.Between -> Arrangement.SpaceBetween
        DoweJustify.Around -> Arrangement.SpaceAround
        DoweJustify.Evenly -> Arrangement.SpaceEvenly
        else -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Start)
    }

private fun doweVerticalArrangement(value: DoweJustify?, gap: Dp?): Arrangement.Vertical =
    when (value) {
"#

