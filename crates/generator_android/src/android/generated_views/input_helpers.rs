fn android_runtime_input_helpers() -> &'static str {
    r#"private data class DoweSelectOption(val value: String, val label: String, val description: String?)

@Composable
private fun DoweInput(value: String, onValueChange: (String) -> Unit, modifier: Modifier, label: String?, placeholder: String, floating: Boolean, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?, startIcon: (@Composable () -> Unit)? = null, endIcon: (@Composable () -> Unit)? = null, visualTransformation: VisualTransformation = VisualTransformation.None, keyboardOptions: KeyboardOptions = KeyboardOptions.Default) {
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
            visualTransformation = visualTransformation,
            keyboardOptions = keyboardOptions,
            textStyle = TextStyle(fontFamily = fontFamily, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.Normal, color = contentColor),
            decorationBox = { innerTextField ->
                Row(modifier = Modifier.fillMaxSize(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    if (!floating || active) {
                        startIcon?.invoke()
                    }
                    Box(modifier = Modifier.weight(1f).fillMaxHeight()) {
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
                    if (!floating || active) {
                        endIcon?.invoke()
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
    DoweAnchoredPopover(
        visible = visible,
        offset = offset,
        shape = shape,
        backgroundColor = DoweDesign.surface,
        contentColor = DoweDesign.surfaceText,
        contentPadding = PaddingValues(vertical = 4.dp),
        onDismiss = onDismiss
    ) {
        options.forEach { option ->
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(if (option.value == selectedValue) accentColor.copy(alpha = 0.08f) else Color.Transparent)
                    .clickable { onSelect(option) }
                    .padding(horizontal = 16.dp, vertical = 10.dp)
            ) {
                Text(text = option.label, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, color = DoweDesign.surfaceText, fontFamily = fontFamily)
                if (option.description != null) {
                    Text(text = option.description, fontSize = 12.sp, color = DoweDesign.surfaceText.copy(alpha = 0.68f), fontFamily = fontFamily)
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
private fun DowePassword(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, minHeight: Dp, fontSize: TextUnit, lineHeight: TextUnit, hideStrength: Boolean, weakLabel: String, mediumLabel: String, strongLabel: String, readOnly: Boolean, showIcon: @Composable () -> Unit, hideIcon: @Composable () -> Unit, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    var visible by remember { mutableStateOf(false) }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        DoweInput(value = value, onValueChange = { if (!readOnly) onValueChange(it) }, modifier = Modifier.fillMaxWidth(), label = label, placeholder = placeholder, floating = floating, fontFamily = FontFamily.Default, fontSize = fontSize, lineHeight = lineHeight, minHeight = minHeight, horizontalPadding = 12.dp, shape = RoundedCornerShape(12.dp), backgroundColor = backgroundColor, contentColor = contentColor, borderColor = contentColor.copy(alpha = 0.22f), endIcon = { Box(modifier = Modifier.size(32.dp).semantics { contentDescription = if (visible) "Hide password" else "Show password" }.clickable(enabled = !readOnly) { visible = !visible }, contentAlignment = Alignment.Center) { if (visible) hideIcon() else showIcon() } }, visualTransformation = if (visible) VisualTransformation.None else PasswordVisualTransformation(), keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password))
        if (!hideStrength) {
            val score = listOf(value.length >= 8, value.length >= 12, value.any { it.isDigit() }, value.any { it.isUpperCase() }, value.any { it.isLowerCase() }, value.any { !it.isLetterOrDigit() }).count { it }
            val strengthColor = if (score <= 2) DoweDesign.danger else if (score <= 4) DoweDesign.warning else DoweDesign.success
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) { repeat(6) { index -> Box(Modifier.weight(1f).height(4.dp).clip(RoundedCornerShape(999.dp)).background(if (index < score) strengthColor else contentColor.copy(alpha = 0.18f))) } }
            if (score > 0) Text(if (score <= 2) weakLabel else if (score <= 4) mediumLabel else strongLabel, fontSize = 12.sp, color = strengthColor)
        }
    }
}

private data class DowePhoneCountry(val code: String, val name: String, val dialCode: String, val viewBox: DoweSvgViewBox, val paths: List<DoweSvgPath>)

__DOWE_PHONE_COUNTRIES__

@Composable
private fun DowePhone(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, country: String, countries: List<DowePhoneCountry>, priorityCountries: List<String>, searchPlaceholder: String, emptyText: String, loadingText: String, floating: Boolean, minHeight: Dp, fontSize: TextUnit, lineHeight: TextUnit, disabled: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    var expanded by remember { mutableStateOf(false) }
    var popupMounted by remember { mutableStateOf(false) }
    var triggerHeight by remember { mutableStateOf(0) }
    var selectedCode by remember(country) { mutableStateOf(country) }
    var query by remember { mutableStateOf("") }
    var localValue by remember(value) { mutableStateOf(value.filter { it.isDigit() }) }
    val selected = countries.firstOrNull { it.code.equals(selectedCode, ignoreCase = true) } ?: countries.firstOrNull()
    val ordered = remember(selectedCode, countries, priorityCountries) {
        buildList {
            selected?.let { add(it) }
            priorityCountries.forEach { code -> countries.firstOrNull { it.code.equals(code, ignoreCase = true) }?.let { item -> if (none { it.code == item.code }) add(item) } }
            countries.forEach { item -> if (none { it.code == item.code }) add(item) }
        }
    }
    val normalizedQuery = query.trim().lowercase()
    val filtered = if (normalizedQuery.isEmpty()) ordered else ordered.filter { it.name.lowercase().contains(normalizedQuery) || it.code.lowercase().contains(normalizedQuery) || it.dialCode.contains(normalizedQuery) }
    val popupOffset = with(LocalDensity.current) { IntOffset(0, triggerHeight + 4.dp.roundToPx()) }
    LaunchedEffect(expanded) {
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
            Row(modifier = Modifier.fillMaxWidth().heightIn(min = minHeight).onGloballyPositioned { triggerHeight = it.size.height }.clip(RoundedCornerShape(12.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(12.dp)).padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Row(modifier = Modifier.clickable(enabled = !disabled && countries.isNotEmpty()) { expanded = true }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    if (selected != null) DoweSvg(viewBox = selected.viewBox, modifier = Modifier.size(24.dp).align(Alignment.CenterVertically).clip(RoundedCornerShape(999.dp)), color = contentColor, paths = selected.paths)
                    Text(if (selected == null) "+$country" else "+${selected.dialCode}", modifier = Modifier.align(Alignment.CenterVertically), fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.Bold, color = contentColor)
                    DoweSvg(viewBox = doweSelectArrowViewBox, modifier = Modifier.size(16.dp).align(Alignment.CenterVertically), color = contentColor, paths = doweSelectArrowPaths)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Box(modifier = Modifier.weight(1f).heightIn(min = minHeight), contentAlignment = Alignment.CenterStart) {
                    if (label != null && floating) Text(label, modifier = Modifier.align(Alignment.TopStart), fontSize = if (localValue.isEmpty()) fontSize else 12.sp, color = contentColor, fontWeight = FontWeight.SemiBold)
                    if (localValue.isEmpty() && (!floating || expanded)) Text(placeholder, modifier = Modifier.padding(top = if (label != null && floating) 10.dp else 0.dp), color = contentColor.copy(alpha = 0.55f), fontSize = fontSize, lineHeight = lineHeight)
                    BasicTextField(value = localValue, onValueChange = { next -> if (!disabled) { val filtered = next.filter { char -> char.isDigit() }; localValue = filtered; onValueChange(filtered) } }, modifier = Modifier.fillMaxWidth().heightIn(min = minHeight).padding(top = if (label != null && floating) 10.dp else 0.dp), singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), textStyle = TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight), enabled = !disabled)
                }
            }
            if (triggerHeight > 0 && (expanded || popupMounted)) DoweAnchoredPopover(visible = expanded, offset = popupOffset, shape = RoundedCornerShape(12.dp), backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, contentPadding = PaddingValues(0.dp), minWidth = 280.dp, maxWidth = 384.dp, maxHeight = 380.dp, onDismiss = { expanded = false; query = "" }) {
                BasicTextField(value = query, onValueChange = { query = it }, modifier = Modifier.fillMaxWidth().padding(6.dp).clip(RoundedCornerShape(10.dp)).background(DoweDesign.surfaceText.copy(alpha = 0.07f)).padding(horizontal = 12.dp, vertical = 9.dp), singleLine = true, textStyle = TextStyle(color = DoweDesign.surfaceText), decorationBox = { inner -> Box { if (query.isEmpty()) Text(searchPlaceholder, color = DoweDesign.surfaceText.copy(alpha = 0.55f)); inner() } })
                if (countries.isEmpty()) Text(loadingText, modifier = Modifier.padding(16.dp), color = DoweDesign.surfaceText.copy(alpha = 0.68f))
                else if (filtered.isEmpty()) Text(emptyText, modifier = Modifier.padding(16.dp), color = DoweDesign.surfaceText.copy(alpha = 0.68f))
                else filtered.forEach { item ->
                    Row(modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).clickable { selectedCode = item.code; expanded = false; query = "" }.background(if (item.code == selectedCode) DoweDesign.surfaceText.copy(alpha = 0.07f) else Color.Transparent).padding(horizontal = 12.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        DoweSvg(viewBox = item.viewBox, modifier = Modifier.size(28.dp).clip(RoundedCornerShape(999.dp)), color = DoweDesign.surfaceText, paths = item.paths)
                        Text(item.name, modifier = Modifier.weight(1f), fontWeight = FontWeight.SemiBold, color = DoweDesign.surfaceText, maxLines = 1)
                        Text("+${item.dialCode}", fontWeight = FontWeight.Bold, color = DoweDesign.surfaceText)
                    }
                }
            }
        }
    }
}

@Composable
private fun DowePin(value: String, onValueChange: (String) -> Unit, label: String?, length: Int, kind: String, size: String, fontSize: TextUnit, lineHeight: TextUnit, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?, helpText: String?, errorText: String?) {
    var cells by remember(value, length) { mutableStateOf(value.padEnd(length).take(length).map { if (it == ' ') "" else it.toString() }) }
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
                        .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
                        .padding(horizontal = if (size == "sm") 8.dp else 12.dp)
                        .focusRequester(focusRequesters[index])
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
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = if (errorText != null) DoweDesign.danger else contentColor.copy(alpha = 0.7f))
        }
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
        is DoweSize.ViewportMinus -> width((LocalConfiguration.current.screenWidthDp.dp - value.inset).coerceAtLeast(0.dp))
        DoweSize.Full -> fillMaxWidth()
        null -> this
    }

@Composable
private fun Modifier.doweHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> height(value.value)
        is DoweSize.ViewportMinus -> height((LocalConfiguration.current.screenHeightDp.dp - value.inset).coerceAtLeast(0.dp))
        DoweSize.Full -> fillMaxHeight()
        null -> this
    }

private fun Modifier.doweMinWidth(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> widthIn(min = value.value)
        else -> this
    }

@Composable
private fun Modifier.doweMinHeight(value: DoweSize?): Modifier =
    when (value) {
        is DoweSize.Fixed -> heightIn(min = value.value)
        is DoweSize.ViewportMinus -> heightIn(min = (LocalConfiguration.current.screenHeightDp.dp - value.inset).coerceAtLeast(0.dp))
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
        is DoweSize.ViewportMinus -> heightIn(max = (LocalConfiguration.current.screenHeightDp.dp - value.inset).coerceAtLeast(0.dp))
        else -> this
    }

private fun Modifier.doweRounded(radius: Dp?): Modifier =
    if (radius == null) this else clip(RoundedCornerShape(radius))

private fun Modifier.doweBorder(width: Dp?, radius: Dp?): Modifier =
    if (width == null) this else border(width, DoweDesign.backgroundText, RoundedCornerShape(radius ?: DoweDesign.radius))

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
