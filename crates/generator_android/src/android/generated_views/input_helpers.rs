fn android_runtime_input_helpers() -> &'static str {
    r#"private data class DoweValidationRule(val kind: String, val argument: String?, val message: String)

private fun doweValidationError(value: String, rules: List<DoweValidationRule>): String? {
    for (rule in rules) {
        val present = value.isNotEmpty()
        val invalid = when (rule.kind) {
            "required" -> value.trim().isEmpty()
            "email" -> present && !Regex("^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$").matches(value)
            "min" -> present && value.length < (rule.argument?.toIntOrNull() ?: 0)
            "max" -> present && value.length > (rule.argument?.toIntOrNull() ?: Int.MAX_VALUE)
            "url" -> present && !Regex("^https?://(www\\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\\.[a-zA-Z0-9()]{1,6}\\b([-a-zA-Z0-9()@:%_+.~#?&//=]*)$").matches(value)
            "phone" -> present && !Regex("^[+]?[(]?[0-9]{1,4}[)]?[-\\s.]?[(]?[0-9]{1,4}[)]?[-\\s.]?[0-9]{1,9}$").matches(value)
            "pattern" -> present && runCatching { !Regex(rule.argument.orEmpty()).containsMatchIn(value) }.getOrDefault(true)
            "alphanumeric" -> present && !Regex("^[a-zA-Z0-9]+$").matches(value)
            "numeric" -> present && !Regex("^[0-9]+$").matches(value)
            "alpha" -> present && !Regex("^[a-zA-Z]+$").matches(value)
            "matches" -> present && value != rule.argument.orEmpty()
            "strongPassword" -> present && (value.length < 8 || !Regex("[a-z]").containsMatchIn(value) || !Regex("[A-Z]").containsMatchIn(value) || !Regex("[0-9]").containsMatchIn(value) || !Regex("[^a-zA-Z0-9]").containsMatchIn(value))
            "creditCard" -> present && !Regex("^(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|6(?:011|5[0-9]{2})[0-9]{12}|(?:2131|1800|35\\d{3})\\d{11})$").matches(value.replace(Regex("\\s"), ""))
            "date" -> present && !Regex("^\\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$").matches(value)
            "minWords" -> present && value.trim().split(Regex("\\s+")).filter { it.isNotEmpty() }.size < (rule.argument?.toIntOrNull() ?: 0)
            "maxWords" -> present && value.trim().split(Regex("\\s+")).filter { it.isNotEmpty() }.size > (rule.argument?.toIntOrNull() ?: Int.MAX_VALUE)
            else -> false
        }
        if (invalid) return rule.message
    }
    return null
}

private fun doweBooleanValidationError(value: Boolean, rules: List<DoweValidationRule>): String? {
    for (rule in rules) {
        val invalid = when (rule.kind) {
            "required" -> !value
            "matches" -> value && value.toString() != rule.argument.orEmpty()
            else -> value && doweValidationError(value.toString(), listOf(rule)) != null
        }
        if (invalid) return rule.message
    }
    return null
}

@Composable
private fun DoweValidationFeedback(helpText: String?, error: String?, contentColor: Color) {
    val message = error ?: helpText
    if (message != null) Text(message, fontSize = 12.sp, color = if (error != null) DoweDesign.danger else contentColor.copy(alpha = 0.7f))
}

private data class DoweSelectOption(val value: String, val label: String, val description: String?)
private data class DoweComboOption(val value: String, val label: String, val description: String?, val icon: (@Composable () -> Unit)?, val disabled: Boolean)

@Composable
private fun DoweInput(value: String, onValueChange: (String) -> Unit, modifier: Modifier, label: String?, placeholder: String, floating: Boolean, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?, startIcon: (@Composable () -> Unit)? = null, endIcon: (@Composable () -> Unit)? = null, visualTransformation: VisualTransformation = VisualTransformation.None, keyboardOptions: KeyboardOptions = KeyboardOptions.Default, helpText: String? = null, errorText: String? = null, validationRules: List<DoweValidationRule> = emptyList()) {
    var focused by remember { mutableStateOf(false) }
    var hadFocus by remember { mutableStateOf(false) }
    var touched by remember { mutableStateOf(false) }
    val active = focused || value.isNotEmpty()
    val validationError = errorText ?: if (touched) doweValidationError(value, validationRules) else null
    val resolvedBorderColor = if (validationError != null) DoweDesign.danger else borderColor
    val surface = modifier
        .height(minHeight)
        .clip(shape)
        .background(backgroundColor)
        .then(if (resolvedBorderColor == null) Modifier else Modifier.border(1.dp, resolvedBorderColor, shape))
        .padding(horizontal = horizontalPadding)
        .onFocusChanged { state -> focused = state.isFocused; if (state.isFocused) hadFocus = true else if (hadFocus) touched = true }
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
        DoweValidationFeedback(helpText, validationError, contentColor)
    }
}

@Composable
private fun DoweSelect(value: String, onValueChange: (String) -> Unit, bound: Boolean, modifier: Modifier, label: String?, placeholder: String, floating: Boolean, options: List<DoweSelectOption>, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?, helpText: String? = null, errorText: String? = null, validationRules: List<DoweValidationRule> = emptyList()) {
    var expanded by remember { mutableStateOf(false) }
    var popupMounted by remember { mutableStateOf(false) }
    var localValue by remember { mutableStateOf("") }
    var touched by remember { mutableStateOf(false) }
    val selectedValue = if (bound) value else localValue
    val selected = options.firstOrNull { it.value == selectedValue }
    val active = expanded || selected != null
    val validationError = errorText ?: if (touched) doweValidationError(selectedValue, validationRules) else null
    val resolvedBorderColor = if (validationError != null) DoweDesign.danger else borderColor
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
                    .height(minHeight)
                    .clip(shape)
                    .background(backgroundColor)
                    .then(if (resolvedBorderColor == null) Modifier else Modifier.border(1.dp, resolvedBorderColor, shape))
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
                    onDismiss = { expanded = false; touched = true },
                    onSelect = { option ->
                        localValue = option.value
                        onValueChange(option.value)
                        expanded = false
                        touched = true
                    }
                )
            }
        }
        DoweValidationFeedback(helpText, validationError, contentColor)
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
private fun DoweComboBox(value: String, onValueChange: (String) -> Unit, bound: Boolean, label: String?, placeholder: String, floating: Boolean, searchPlaceholder: String, emptyText: String, loadingText: String, clearable: Boolean, disabled: Boolean, options: List<DoweComboOption>, modifier: Modifier, fontFamily: FontFamily, fontSize: TextUnit, lineHeight: TextUnit, minHeight: Dp, horizontalPadding: Dp, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?, helpText: String? = null, errorText: String? = null, validationRules: List<DoweValidationRule> = emptyList()) {
    var expanded by remember { mutableStateOf(false) }
    var query by remember { mutableStateOf("") }
    var localValue by remember(value) { mutableStateOf(value) }
    var touched by remember { mutableStateOf(false) }
    val selectedValue = if (bound) value else localValue
    val selected = options.firstOrNull { it.value == selectedValue }
    val filtered = options.filter { option ->
        query.isBlank() || listOf(option.label, option.value, option.description.orEmpty()).any { it.contains(query, ignoreCase = true) }
    }
    val active = expanded || selected != null
    val validationError = errorText ?: if (touched) doweValidationError(selectedValue, validationRules) else null
    val resolvedBorderColor = if (validationError != null) DoweDesign.danger else borderColor
    val popupOffset = with(LocalDensity.current) { IntOffset(0, (minHeight + 4.dp).roundToPx()) }
    Column {
        if (label != null && !floating) Text(text = label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        Box(modifier = modifier) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(min = minHeight)
                    .clip(shape)
                    .background(backgroundColor)
                    .then(if (resolvedBorderColor == null) Modifier else Modifier.border(1.dp, resolvedBorderColor, shape))
                    .clickable(enabled = !disabled) { expanded = true }
                    .padding(horizontal = horizontalPadding),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Box(modifier = Modifier.weight(1f)) {
                    if (label != null && floating) Text(text = label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else fontSize, color = contentColor, fontFamily = fontFamily)
                    if (selected != null || !floating || expanded) Text(text = selected?.label ?: placeholder, modifier = Modifier.align(Alignment.CenterStart).padding(top = if (label != null && floating && active) 10.dp else 0.dp), fontSize = fontSize, lineHeight = lineHeight, color = contentColor.copy(alpha = if (selected != null) 1f else 0.55f), fontFamily = fontFamily, maxLines = 1)
                }
                if (clearable && selected != null) {
                    Text(text = "×", modifier = Modifier.clickable(enabled = !disabled) { localValue = ""; onValueChange(""); touched = true }.padding(horizontal = 4.dp), color = contentColor.copy(alpha = 0.7f), fontSize = 18.sp)
                }
                DoweSvg(viewBox = doweSelectArrowViewBox, modifier = Modifier.width(16.dp).height(16.dp), color = contentColor, paths = doweSelectArrowPaths)
            }
            if (expanded) {
                DoweAnchoredPopover(visible = true, offset = popupOffset, shape = shape, backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, contentPadding = PaddingValues(vertical = 4.dp), minWidth = 280.dp, maxWidth = 384.dp, maxHeight = 380.dp, onDismiss = { expanded = false; query = ""; touched = true }) {
                    BasicTextField(value = query, onValueChange = { query = it }, modifier = Modifier.fillMaxWidth().padding(6.dp).clip(RoundedCornerShape(10.dp)).background(DoweDesign.surfaceText.copy(alpha = 0.07f)).padding(horizontal = 12.dp, vertical = 9.dp), singleLine = true, textStyle = TextStyle(color = DoweDesign.surfaceText), decorationBox = { inner -> Box { if (query.isEmpty()) Text(searchPlaceholder, color = DoweDesign.surfaceText.copy(alpha = 0.55f)); inner() } })
                    if (options.isEmpty()) Text(loadingText, modifier = Modifier.fillMaxWidth().padding(16.dp), color = DoweDesign.surfaceText.copy(alpha = 0.68f), textAlign = TextAlign.Center)
                    else if (filtered.isEmpty()) Text(emptyText, modifier = Modifier.fillMaxWidth().padding(16.dp), color = DoweDesign.surfaceText.copy(alpha = 0.68f), textAlign = TextAlign.Center)
                    else filtered.forEach { option ->
                        Row(modifier = Modifier.fillMaxWidth().background(if (option.value == selectedValue) contentColor.copy(alpha = 0.1f) else Color.Transparent).clickable(enabled = !option.disabled) { localValue = option.value; onValueChange(option.value); expanded = false; query = ""; touched = true }.padding(horizontal = 12.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                            option.icon?.invoke()
                            Column(modifier = Modifier.weight(1f)) {
                                Text(text = option.label, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, color = DoweDesign.surfaceText.copy(alpha = if (option.disabled) 0.45f else 1f), fontFamily = fontFamily)
                                if (option.description != null) Text(text = option.description, fontSize = 12.sp, color = DoweDesign.surfaceText.copy(alpha = if (option.disabled) 0.35f else 0.68f), fontFamily = fontFamily)
                            }
                        }
                    }
                }
            }
        }
        DoweValidationFeedback(helpText, validationError, contentColor)
    }
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
private fun doweImageCropperSize(size: String): Dp {
    return when (size) {
        "xs" -> 96.dp
        "sm" -> 112.dp
        "lg" -> 160.dp
        "xl" -> 192.dp
        else -> 128.dp
    }
}

private fun doweDataUrlBitmap(value: String): android.graphics.Bitmap? {
    if (!value.startsWith("data:image/")) return null
    val encoded = value.substringAfter(",", "")
    return runCatching {
        val bytes = Base64.decode(encoded, Base64.DEFAULT)
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
    }.getOrNull()
}

private fun doweBitmapDataUrl(bitmap: android.graphics.Bitmap, mime: String): String {
    val jpeg = mime.contains("jpeg") || mime.contains("jpg")
    val output = ByteArrayOutputStream()
    bitmap.compress(if (jpeg) android.graphics.Bitmap.CompressFormat.JPEG else android.graphics.Bitmap.CompressFormat.PNG, 92, output)
    val outputMime = if (jpeg) "image/jpeg" else "image/png"
    return "data:$outputMime;base64," + Base64.encodeToString(output.toByteArray(), Base64.NO_WRAP)
}

private fun doweCropBitmap(bitmap: android.graphics.Bitmap, aspect: Float, zoom: Float, offset: Offset, minWidth: Int, minHeight: Int, maxWidth: Int?, maxHeight: Int?): android.graphics.Bitmap? {
    val frameWidth = 1000f
    val frameHeight = frameWidth / aspect.coerceAtLeast(0.01f)
    val scale = maxOf(frameWidth / bitmap.width, frameHeight / bitmap.height) * zoom
    val imageWidth = bitmap.width * scale
    val imageHeight = bitmap.height * scale
    val left = (frameWidth - imageWidth) / 2f + offset.x
    val top = (frameHeight - imageHeight) / 2f + offset.y
    val sourceX = ((0f - left) / scale).coerceIn(0f, bitmap.width.toFloat())
    val sourceY = ((0f - top) / scale).coerceIn(0f, bitmap.height.toFloat())
    val sourceWidth = (frameWidth / scale).coerceAtMost(bitmap.width - sourceX)
    val sourceHeight = (frameHeight / scale).coerceAtMost(bitmap.height - sourceY)
    if (sourceWidth < minWidth || sourceHeight < minHeight) return null
    var outputWidth = sourceWidth.roundToInt().coerceAtLeast(1)
    var outputHeight = sourceHeight.roundToInt().coerceAtLeast(1)
    val limit = minOf(maxWidth?.toFloat()?.div(outputWidth) ?: 1f, maxHeight?.toFloat()?.div(outputHeight) ?: 1f)
    if (limit < 1f) {
        outputWidth = (outputWidth * limit).roundToInt().coerceAtLeast(1)
        outputHeight = (outputHeight * limit).roundToInt().coerceAtLeast(1)
    }
    return android.graphics.Bitmap.createBitmap(bitmap, sourceX.roundToInt(), sourceY.roundToInt(), sourceWidth.roundToInt().coerceAtLeast(1), sourceHeight.roundToInt().coerceAtLeast(1), null, true).let {
        if (it.width == outputWidth && it.height == outputHeight) it else android.graphics.Bitmap.createScaledBitmap(it, outputWidth, outputHeight, true)
    }
}

@Composable
private fun DoweImageCropper(value: String, onValueChange: (String) -> Unit, bound: Boolean, initialValue: String, label: String?, placeholder: String, alt: String, accept: String, aspectRatio: String?, minWidth: Int, minHeight: Int, maxImageWidth: Int?, maxImageHeight: Int?, shape: String, cropSize: String, disabled: Boolean, helpText: String?, errorText: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    val context = LocalContext.current
    var localValue by remember { mutableStateOf(initialValue) }
    val appliedValue = if (bound && value.isNotEmpty()) value else if (bound && value.isEmpty() && localValue == initialValue && initialValue.isNotEmpty()) initialValue else if (bound) value else localValue
    var appliedBitmap by remember(appliedValue) { mutableStateOf<android.graphics.Bitmap?>(null) }
    var draftBitmap by remember { mutableStateOf<android.graphics.Bitmap?>(null) }
    var draftMime by remember { mutableStateOf("image/png") }
    var pendingUri by remember { mutableStateOf<Uri?>(null) }
    var cropDialog by remember { mutableStateOf(false) }
    var zoom by remember { mutableStateOf(1f) }
    var offset by remember { mutableStateOf(Offset.Zero) }
    var cropError by remember { mutableStateOf<String?>(null) }
    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> pendingUri = uri }
    LaunchedEffect(appliedValue) {
        appliedBitmap = withContext(Dispatchers.IO) {
            doweDataUrlBitmap(appliedValue) ?: doweLoadImageBitmap(context, appliedValue)
        }
    }
    LaunchedEffect(pendingUri) {
        val uri = pendingUri ?: return@LaunchedEffect
        val bitmap = withContext(Dispatchers.IO) { context.contentResolver.openInputStream(uri)?.use(BitmapFactory::decodeStream) }
        if (bitmap != null) {
            draftBitmap = bitmap
            draftMime = context.contentResolver.getType(uri) ?: "image/png"
            zoom = 1f
            offset = Offset.Zero
            cropError = null
            cropDialog = true
        } else {
            cropError = "The selected image could not be decoded."
        }
        pendingUri = null
    }
    fun openExisting() {
        if (disabled) return
        draftBitmap = appliedBitmap
        draftMime = appliedValue.substringAfter("data:", "image/png").substringBefore(";")
        zoom = 1f
        offset = Offset.Zero
        cropError = null
        if (draftBitmap != null) cropDialog = true
    }
    fun remove() {
        localValue = ""
        onValueChange("")
        appliedBitmap = null
    }
    val frameShape = if (shape == "circle") RoundedCornerShape(999.dp) else RoundedCornerShape(18.dp)
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) Text(label, fontWeight = FontWeight.SemiBold, color = contentColor)
        Box(modifier = Modifier.size(doweImageCropperSize(cropSize)).clip(frameShape).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.2f), frameShape).clickable(enabled = !disabled) { if (appliedBitmap == null) picker.launch(doweDropzoneMimeTypes(accept)) else openExisting() }, contentAlignment = Alignment.Center) {
            if (appliedBitmap != null) Image(bitmap = appliedBitmap!!.asImageBitmap(), contentDescription = alt, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop) else Text(placeholder, color = contentColor, fontWeight = FontWeight.Bold)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            TextButton(enabled = !disabled, onClick = { picker.launch(doweDropzoneMimeTypes(accept)) }) { Text(if (appliedBitmap == null) "Upload" else "Change", color = contentColor) }
            if (appliedBitmap != null) TextButton(enabled = !disabled, onClick = ::remove) { Text("Remove", color = contentColor.copy(alpha = 0.72f)) }
        }
        DoweValidationFeedback(helpText, cropError ?: errorText, contentColor)
    }
    if (cropDialog && draftBitmap != null) {
        Dialog(onDismissRequest = { cropDialog = false }, properties = DialogProperties(usePlatformDefaultWidth = false)) {
            BackHandler { cropDialog = false }
            val aspect = aspectRatio?.toFloatOrNull()?.coerceAtLeast(0.01f) ?: 1f
            Column(modifier = Modifier.fillMaxWidth().padding(16.dp).clip(RoundedCornerShape(20.dp)).background(backgroundColor).padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    Text("Adjust image", fontWeight = FontWeight.Bold, color = contentColor)
                    TextButton(onClick = { cropDialog = false }) { Text("Cancel", color = contentColor) }
                }
                BoxWithConstraints(modifier = Modifier.fillMaxWidth().heightIn(max = 420.dp).clip(RoundedCornerShape(12.dp)).background(Color.Black), contentAlignment = Alignment.Center) {
                    val frameWidth = maxWidth
                    val frameHeight = minOf(maxHeight, maxWidth / aspect)
                    Box(modifier = Modifier.width(frameWidth).height(frameHeight).clip(if (shape == "circle") RoundedCornerShape(999.dp) else RoundedCornerShape(0.dp)).pointerInput(Unit) { detectTransformGestures { _, pan, zoomChange, _ -> offset += pan; zoom = (zoom * zoomChange).coerceIn(1f, 3f) } }) {
                        Image(bitmap = draftBitmap!!.asImageBitmap(), contentDescription = alt, modifier = Modifier.fillMaxSize().graphicsLayer { scaleX = zoom; scaleY = zoom; translationX = offset.x; translationY = offset.y }, contentScale = ContentScale.Crop)
                        Canvas(modifier = Modifier.fillMaxSize()) { val canvasSize = size; drawLine(Color.White.copy(alpha = 0.65f), Offset(canvasSize.width / 3f, 0f), Offset(canvasSize.width / 3f, canvasSize.height)); drawLine(Color.White.copy(alpha = 0.65f), Offset(canvasSize.width * 2f / 3f, 0f), Offset(canvasSize.width * 2f / 3f, canvasSize.height)); drawLine(Color.White.copy(alpha = 0.65f), Offset(0f, canvasSize.height / 3f), Offset(canvasSize.width, canvasSize.height / 3f)); drawLine(Color.White.copy(alpha = 0.65f), Offset(0f, canvasSize.height * 2f / 3f), Offset(canvasSize.width, canvasSize.height * 2f / 3f)) }
                    }
                }
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Zoom", color = contentColor, fontSize = 12.sp)
                    androidx.compose.material3.Slider(value = zoom, onValueChange = { zoom = it }, valueRange = 1f..3f, modifier = Modifier.weight(1f))
                }
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TextButton(onClick = { zoom = 1f; offset = Offset.Zero }) { Text("Reset", color = contentColor) }
                    Spacer(modifier = Modifier.weight(1f))
                    TextButton(onClick = { cropDialog = false }) { Text("Cancel", color = contentColor) }
                    Button(onClick = {
                        val result = doweCropBitmap(draftBitmap!!, aspect, zoom, offset, minWidth, minHeight, maxImageWidth, maxImageHeight)
                        if (result == null) cropError = "Image must be at least $minWidth × $minHeight pixels." else {
                            val next = doweBitmapDataUrl(result, draftMime)
                            localValue = next
                            onValueChange(next)
                            appliedBitmap = result
                            cropDialog = false
                        }
                    }) { Text("Apply") }
                }
                cropError?.let { Text(it, color = DoweDesign.danger, fontSize = 12.sp) }
            }
        }
    }
}

@Composable
private fun DowePassword(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, fontFamily: FontFamily, minHeight: Dp, fontSize: TextUnit, lineHeight: TextUnit, hideStrength: Boolean, weakLabel: String, mediumLabel: String, strongLabel: String, readOnly: Boolean, showIcon: @Composable () -> Unit, hideIcon: @Composable () -> Unit, modifier: Modifier, backgroundColor: Color, contentColor: Color, helpText: String? = null, errorText: String? = null, validationRules: List<DoweValidationRule> = emptyList()) {
    var visible by remember { mutableStateOf(false) }
    var hadFocus by remember { mutableStateOf(false) }
    var touched by remember { mutableStateOf(false) }
    val validationError = errorText ?: if (touched) doweValidationError(value, validationRules) else null
    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        DoweInput(value = value, onValueChange = { if (!readOnly) onValueChange(it) }, modifier = Modifier.fillMaxWidth().onFocusChanged { state -> if (state.isFocused) hadFocus = true else if (hadFocus) touched = true }, label = label, placeholder = placeholder, floating = floating, fontFamily = fontFamily, fontSize = fontSize, lineHeight = lineHeight, minHeight = minHeight, horizontalPadding = 12.dp, shape = RoundedCornerShape(12.dp), backgroundColor = backgroundColor, contentColor = contentColor, borderColor = if (validationError != null) DoweDesign.danger else contentColor.copy(alpha = 0.22f), endIcon = { Box(modifier = Modifier.size(32.dp).semantics { contentDescription = if (visible) "Hide password" else "Show password" }.clickable(enabled = !readOnly) { visible = !visible }, contentAlignment = Alignment.Center) { if (visible) hideIcon() else showIcon() } }, visualTransformation = if (visible) VisualTransformation.None else PasswordVisualTransformation(), keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password))
        if (!hideStrength) {
            val score = listOf(value.length >= 8, value.length >= 12, value.any { it.isDigit() }, value.any { it.isUpperCase() }, value.any { it.isLowerCase() }, value.any { !it.isLetterOrDigit() }).count { it }
            val strengthColor = if (score <= 2) DoweDesign.danger else if (score <= 4) DoweDesign.warning else DoweDesign.success
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) { repeat(6) { index -> Box(Modifier.weight(1f).height(4.dp).clip(RoundedCornerShape(999.dp)).background(if (index < score) strengthColor else contentColor.copy(alpha = 0.18f))) } }
            if (score > 0) Text(if (score <= 2) weakLabel else if (score <= 4) mediumLabel else strongLabel, fontSize = 12.sp, color = strengthColor)
        }
        DoweValidationFeedback(helpText, validationError, contentColor)
    }
}

private data class DowePhoneCountry(val code: String, val name: String, val dialCode: String, val viewBox: DoweSvgViewBox, val paths: List<DoweSvgPath>)

__DOWE_PHONE_COUNTRIES__

@Composable
private fun DowePhone(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, country: String, countries: List<DowePhoneCountry>, priorityCountries: List<String>, searchPlaceholder: String, emptyText: String, loadingText: String, floating: Boolean, minHeight: Dp, fontSize: TextUnit, lineHeight: TextUnit, disabled: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color, helpText: String? = null, errorText: String? = null, validationRules: List<DoweValidationRule> = emptyList()) {
    var expanded by remember { mutableStateOf(false) }
    var popupMounted by remember { mutableStateOf(false) }
    var triggerHeight by remember { mutableStateOf(0) }
    var selectedCode by remember(country) { mutableStateOf(country) }
    var query by remember { mutableStateOf("") }
    var localValue by remember(value) { mutableStateOf(value.filter { it.isDigit() }) }
    var hadFocus by remember { mutableStateOf(false) }
    var touched by remember { mutableStateOf(false) }
    val validationError = errorText ?: if (touched) doweValidationError(localValue, validationRules) else null
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

private fun doweGridHorizontalStretch(value: DoweAlign?): Boolean = value == DoweAlign.Stretch

private fun doweGridVerticalAlignment(value: DoweAlign?): Alignment.Vertical =
    when (value) {
        DoweAlign.Center -> Alignment.CenterVertically
        DoweAlign.End -> Alignment.Bottom
        else -> Alignment.Top
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
