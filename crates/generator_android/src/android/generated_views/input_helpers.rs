fn android_runtime_input_helpers() -> &'static str {
    r#"private data class DoweSelectOption(val value: String, val label: String, val description: String?)

@Composable
private fun DoweInput(value: String, onValueChange: (String) -> Unit, modifier: Modifier, label: String?, placeholder: String, floating: Boolean, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var focused by remember { mutableStateOf(false) }
    val active = focused || value.isNotEmpty()
    val surface = modifier
        .heightIn(min = minHeight)
        .clip(shape)
        .background(backgroundColor)
        .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
        .padding(horizontal = horizontalPadding)
        .onFocusChanged { focused = it.isFocused }
    Column {
        if (label != null && !floating) {
            Text(text = label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = surface,
            singleLine = true,
            textStyle = TextStyle(fontFamily = fontFamily, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.Normal, color = contentColor),
            decorationBox = { innerTextField ->
                Box(modifier = Modifier.fillMaxSize()) {
                    if (placeholder.isNotEmpty() && value.isEmpty() && (!floating || active)) {
                        Text(text = placeholder, modifier = Modifier.align(Alignment.CenterStart), fontSize = fontSize, color = contentColor.copy(alpha = 0.55f), fontFamily = fontFamily)
                    }
                    if (label != null && floating) {
                        Text(text = label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else fontSize, color = contentColor, fontFamily = fontFamily)
                    }
                    Box(modifier = Modifier.align(Alignment.CenterStart).padding(top = if (label != null && floating) 10.dp else 0.dp)) {
                        innerTextField()
                    }
                }
            }
        )
    }
}

@Composable
private fun DoweSelect(value: String, onValueChange: (String) -> Unit, bound: Boolean, modifier: Modifier, label: String?, placeholder: String, floating: Boolean, options: List<DoweSelectOption>, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var expanded by remember { mutableStateOf(false) }
    var popupMounted by remember { mutableStateOf(false) }
    var localValue by remember { mutableStateOf("") }
    val selectedValue = if (bound) value else localValue
    val selected = options.firstOrNull { it.value == selectedValue }
    val active = expanded || selected != null
    val popupOffset = with(LocalDensity.current) { IntOffset(0, (minHeight + 4.dp).roundToPx()) }
    LaunchedEffect(expanded) {
        if (expanded) {
            popupMounted = true
        } else if (popupMounted) {
            delay(160)
            popupMounted = false
        }
    }
    Column {
        if (label != null && !floating) {
            Text(text = label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box(modifier = modifier) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(min = minHeight)
                    .clip(shape)
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
                    .clickable { expanded = true }
                    .padding(horizontal = horizontalPadding),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Box(modifier = Modifier.weight(1f)) {
                    if (label != null && floating) {
                        Text(text = label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else fontSize, color = contentColor, fontFamily = fontFamily)
                    }
                    if (selected != null || !floating || expanded) {
                        Text(text = selected?.label ?: placeholder, modifier = Modifier.align(Alignment.CenterStart).padding(top = if (label != null && floating && active) 10.dp else 0.dp), fontSize = fontSize, lineHeight = lineHeight, color = contentColor, fontFamily = fontFamily, maxLines = 1)
                    }
                }
                DoweSvg(viewBox = doweSelectArrowViewBox, modifier = Modifier.width(16.dp).height(16.dp), color = contentColor, paths = doweSelectArrowPaths)
            }
            if (expanded || popupMounted) {
                DoweSelectPopover(
                    visible = expanded,
                    options = options,
                    selectedValue = selectedValue,
                    offset = popupOffset,
                    shape = shape,
                    accentColor = contentColor,
                    fontFamily = fontFamily,
                    fontSize = fontSize,
                    lineHeight = lineHeight,
                    onDismiss = { expanded = false },
                    onSelect = { option ->
                        localValue = option.value
                        onValueChange(option.value)
                        expanded = false
                    }
                )
            }
        }
    }
}

@Composable
private fun DoweSelectPopover(visible: Boolean, options: List<DoweSelectOption>, selectedValue: String, offset: IntOffset, shape: RoundedCornerShape, accentColor: Color, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, onDismiss: () -> Unit, onSelect: (DoweSelectOption) -> Unit) {
    val progress by animateFloatAsState(
        targetValue = if (visible) 1f else 0f,
        animationSpec = tween(durationMillis = 160)
    )
    Popup(alignment = Alignment.TopStart, offset = offset, onDismissRequest = onDismiss, properties = PopupProperties(focusable = true)) {
        Column(
            modifier = Modifier
                .widthIn(min = 220.dp)
                .graphicsLayer {
                    alpha = progress
                    translationY = (1f - progress) * -4f
                    val value = 0.98f + (0.02f * progress)
                    scaleX = value
                    scaleY = value
                }
                .clip(shape)
                .background(DoweDesign.surface)
                .border(1.dp, DoweDesign.onSurface.copy(alpha = 0.08f), shape)
                .padding(vertical = 4.dp)
        ) {
            options.forEach { option ->
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(if (option.value == selectedValue) accentColor.copy(alpha = 0.08f) else Color.Transparent)
                        .clickable { onSelect(option) }
                        .padding(horizontal = 16.dp, vertical = 10.dp)
                ) {
                    Text(text = option.label, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, color = DoweDesign.onSurface, fontFamily = fontFamily)
                    if (option.description != null) {
                        Text(text = option.description, fontSize = 12.sp, color = DoweDesign.onSurface.copy(alpha = 0.68f), fontFamily = fontFamily)
                    }
                }
            }
        }
    }
}

private data class DoweCsvColumn(val name: String, val label: String?)
private data class DoweDragItem(val id: String, val label: String?, val description: String?, val disabled: Boolean)
private data class DoweDragGroup(val id: String, val title: String?, val items: List<DoweDragItem>)

@Composable
private fun DoweComboBox(value: String, onValueChange: (String) -> Unit, bound: Boolean, label: String?, placeholder: String, floating: Boolean, searchPlaceholder: String, emptyText: String, clearable: Boolean, options: List<DoweSelectOption>, modifier: Modifier, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    DoweSelect(value = value, onValueChange = onValueChange, bound = bound, modifier = modifier, label = label, placeholder = placeholder, floating = floating, options = options, fontFamily = fontFamily, fontSize = fontSize, lineHeight = lineHeight, minHeight = minHeight, horizontalPadding = horizontalPadding, shape = shape, backgroundColor = backgroundColor, contentColor = contentColor, borderColor = borderColor)
}

@Composable
private fun DoweCsvField(label: String?, buttonText: String, modalTitle: String, instructions: String, columns: List<DoweCsvColumn>, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) Text(text = label, fontWeight = FontWeight.SemiBold, color = contentColor)
        Button(onClick = {}, colors = ButtonDefaults.buttonColors(containerColor = backgroundColor, contentColor = contentColor)) {
            Text(buttonText)
        }
        Column(modifier = Modifier.fillMaxWidth().border(1.dp, contentColor.copy(alpha = 0.18f), RoundedCornerShape(12.dp)).padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(text = modalTitle, fontWeight = FontWeight.Bold, color = contentColor)
            Text(text = instructions, fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
            columns.forEach { column ->
                Text(text = column.label ?: column.name, fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
            }
        }
    }
}

@Composable
private fun DoweDragDrop(label: String?, emptyText: String, direction: String, items: List<DoweDragItem>, groups: List<DoweDragGroup>, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) Text(text = label, fontWeight = FontWeight.SemiBold, color = contentColor)
        val surface = Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(backgroundColor).padding(8.dp)
        if (groups.isNotEmpty()) {
            Row(modifier = surface.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                groups.forEach { group -> DoweDragGroupView(group.title ?: group.id, group.items, emptyText, contentColor) }
            }
        } else {
            Column(modifier = surface, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (items.isEmpty()) Text(emptyText, color = contentColor.copy(alpha = 0.65f))
                items.forEach { item -> DoweDragItemView(item, contentColor) }
            }
        }
    }
}

@Composable
private fun DoweDragGroupView(title: String, items: List<DoweDragItem>, emptyText: String, contentColor: Color) {
    Column(modifier = Modifier.widthIn(min = 220.dp).border(1.dp, contentColor.copy(alpha = 0.18f), RoundedCornerShape(12.dp)).padding(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(text = title, fontWeight = FontWeight.Bold, color = contentColor)
        if (items.isEmpty()) Text(emptyText, color = contentColor.copy(alpha = 0.65f))
        items.forEach { item -> DoweDragItemView(item, contentColor) }
    }
}

@Composable
private fun DoweDragItemView(item: DoweDragItem, contentColor: Color) {
    Row(modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).background(contentColor.copy(alpha = 0.08f)).padding(10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Text("::", fontWeight = FontWeight.Bold, color = contentColor.copy(alpha = 0.55f))
        Column {
            Text(item.label ?: item.id, fontWeight = FontWeight.SemiBold, color = contentColor)
            if (item.description != null) Text(item.description, fontSize = 12.sp, color = contentColor.copy(alpha = 0.68f))
        }
    }
}

@Composable
private fun DoweEditorField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, minHeight: Dp, hideToolbar: Boolean, readOnly: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    Column(modifier = modifier.clip(RoundedCornerShape(16.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.18f), RoundedCornerShape(16.dp)), verticalArrangement = Arrangement.spacedBy(0.dp)) {
        if (label != null) Text(text = label, modifier = Modifier.padding(12.dp, 10.dp, 12.dp, 0.dp), fontWeight = FontWeight.SemiBold, color = contentColor)
        if (!hideToolbar) Row(modifier = Modifier.fillMaxWidth().background(contentColor.copy(alpha = 0.08f)).padding(6.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            listOf("B", "I", "U", "List").forEach { Text(it, modifier = Modifier.clip(RoundedCornerShape(8.dp)).background(contentColor.copy(alpha = 0.08f)).padding(horizontal = 8.dp, vertical = 5.dp), fontWeight = FontWeight.Bold, color = contentColor) }
        }
        BasicTextField(value = value, onValueChange = { if (!readOnly) onValueChange(it) }, modifier = Modifier.fillMaxWidth().heightIn(min = minHeight).padding(12.dp), textStyle = TextStyle(color = contentColor), decorationBox = { inner -> Box { if (value.isEmpty() && placeholder.isNotEmpty()) Text(placeholder, color = contentColor.copy(alpha = 0.52f)); inner() } })
    }
}

@Composable
private fun DoweImageCropper(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, shape: String, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        Box(modifier = Modifier.width(128.dp).height(128.dp).clip(if (shape == "circle") RoundedCornerShape(999.dp) else RoundedCornerShape(18.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.2f), if (shape == "circle") RoundedCornerShape(999.dp) else RoundedCornerShape(18.dp)), contentAlignment = Alignment.Center) {
            Text(if (value.isEmpty()) placeholder else "Image", color = contentColor, fontWeight = FontWeight.Bold)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Edit", color = contentColor, fontWeight = FontWeight.SemiBold)
            Text("Remove", color = contentColor.copy(alpha = 0.72f), fontWeight = FontWeight.SemiBold)
        }
    }
}

@Composable
private fun DowePasswordField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, hideStrength: Boolean, weakLabel: String, mediumLabel: String, strongLabel: String, readOnly: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    var visible by remember { mutableStateOf(false) }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        DoweInput(value = value, onValueChange = { if (!readOnly) onValueChange(it) }, modifier = Modifier.fillMaxWidth(), label = label, placeholder = placeholder, floating = floating, fontFamily = FontFamily.Default, fontSize = 16.sp, lineHeight = 20.sp, minHeight = 48.dp, horizontalPadding = 12.dp, shape = RoundedCornerShape(12.dp), backgroundColor = backgroundColor, contentColor = contentColor, borderColor = contentColor.copy(alpha = 0.22f))
        if (!hideStrength) {
            val score = listOf(value.length >= 8, value.length >= 12, value.any { it.isDigit() }, value.any { it.isUpperCase() }, value.any { !it.isLetterOrDigit() }).count { it }
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) { repeat(6) { index -> Box(Modifier.weight(1f).height(4.dp).clip(RoundedCornerShape(999.dp)).background(if (index < score) contentColor else contentColor.copy(alpha = 0.18f))) } }
            Text(if (score <= 2) weakLabel else if (score <= 4) mediumLabel else strongLabel, fontSize = 12.sp, color = contentColor.copy(alpha = 0.75f))
        }
    }
}

@Composable
private fun DowePhoneField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, country: String, floating: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        Row(modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(12.dp)).padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(country, fontWeight = FontWeight.Bold, color = contentColor)
            Text("+", modifier = Modifier.padding(horizontal = 8.dp), color = contentColor.copy(alpha = 0.55f))
            BasicTextField(value = value, onValueChange = onValueChange, modifier = Modifier.weight(1f).heightIn(min = 48.dp), textStyle = TextStyle(color = contentColor), decorationBox = { inner -> Box(contentAlignment = Alignment.CenterStart) { if (value.isEmpty()) Text(placeholder, color = contentColor.copy(alpha = 0.55f)); inner() } })
        }
    }
}

@Composable
private fun DowePinField(value: String, onValueChange: (String) -> Unit, label: String?, length: Int, kind: String, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    var cells by remember(value) { mutableStateOf(value.padEnd(length).take(length).map { if (it == ' ') "" else it.toString() }) }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            cells.forEachIndexed { index, cell ->
                BasicTextField(value = cell, onValueChange = { next ->
                    cells = cells.toMutableList().also { it[index] = next.takeLast(1) }
                    onValueChange(cells.joinToString(""))
                }, modifier = Modifier.width(44.dp).height(48.dp).clip(RoundedCornerShape(10.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.25f), RoundedCornerShape(10.dp)).padding(12.dp), textStyle = TextStyle(color = contentColor, fontWeight = FontWeight.Bold))
            }
        }
    }
}

@Composable
private fun DoweTextarea(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, rows: Int, maxLength: Int?, readOnly: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        BasicTextField(value = value, onValueChange = { next -> if (!readOnly) onValueChange(maxLength?.let { next.take(it) } ?: next) }, modifier = Modifier.fillMaxWidth().heightIn(min = (rows * 28).dp).clip(RoundedCornerShape(12.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(12.dp)).padding(12.dp), textStyle = TextStyle(color = contentColor), decorationBox = { inner -> Box { if (value.isEmpty() && placeholder.isNotEmpty()) Text(placeholder, color = contentColor.copy(alpha = 0.55f)); inner() } })
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
    var modifier = this
    if (all != null) {
        modifier = modifier.padding(all)
    }
    if (horizontal != null) {
        modifier = modifier.padding(horizontal = horizontal)
    }
    if (vertical != null) {
        modifier = modifier.padding(vertical = vertical)
    }
    if (start != null || end != null || top != null || bottom != null) {
        modifier = modifier.padding(start = start ?: 0.dp, end = end ?: 0.dp, top = top ?: 0.dp, bottom = bottom ?: 0.dp)
    }
    return modifier
}

private fun Modifier.doweWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> width(value.value)
        DoweSize.Full -> fillMaxWidth()
        null -> this
    }

private fun Modifier.doweHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> height(value.value)
        DoweSize.Full -> fillMaxHeight()
        null -> this
    }

private fun Modifier.doweMinWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> widthIn(min = value.value)
        else -> this
    }

private fun Modifier.doweMinHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> heightIn(min = value.value)
        else -> this
    }

private fun Modifier.doweRounded(radius: Dp?): Modifier =
    if (radius == null) this else clip(RoundedCornerShape(radius))

private fun Modifier.doweBorder(width: Dp?, radius: Dp?): Modifier =
    if (width == null) this else border(width, DoweDesign.onBackground, RoundedCornerShape(radius ?: DoweDesign.radius))

private fun doweHorizontalAlignment(value: DoweAlign?): Alignment.Horizontal =
    when (value) {
        DoweAlign.Center, DoweAlign.Stretch, DoweAlign.Baseline -> Alignment.CenterHorizontally
        DoweAlign.End -> Alignment.End
        else -> Alignment.Start
    }

private fun doweGridHorizontalAlignment(value: DoweAlign?): Alignment.Horizontal =
    when (value) {
        DoweAlign.Center -> Alignment.CenterHorizontally
        DoweAlign.End -> Alignment.End
        else -> Alignment.Start
    }

private fun doweVerticalAlignment(value: DoweAlign?): Alignment.Vertical =
    when (value) {
        DoweAlign.Center, DoweAlign.Stretch -> Alignment.CenterVertically
        DoweAlign.End -> Alignment.Bottom
        else -> Alignment.Top
    }

private fun doweHorizontalArrangement(value: DoweJustify?, gap: Dp?): Arrangement.Horizontal =
    when (value) {
        DoweJustify.Center -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.CenterHorizontally)
        DoweJustify.End -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.End)
        DoweJustify.Between -> Arrangement.SpaceBetween
        DoweJustify.Around -> Arrangement.SpaceAround
        DoweJustify.Evenly -> Arrangement.SpaceEvenly
        else -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Start)
    }

private fun doweVerticalArrangement(value: DoweJustify?, gap: Dp?): Arrangement.Vertical =
    when (value) {
        DoweJustify.Center -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.CenterVertically)
        DoweJustify.End -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Bottom)
        DoweJustify.Between -> Arrangement.SpaceBetween
        DoweJustify.Around -> Arrangement.SpaceAround
        DoweJustify.Evenly -> Arrangement.SpaceEvenly
        else -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Top)
    }

"#
}
