r#"    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
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
private fun DoweEditorTokens(source: String, language: String, contentColor: Color): AnnotatedString {
    val keywords = if (language == "dowe") "action|set|component|config|each|else|handler|if|import|layout|page|return|route|server|signal|type|views" else "as|async|await|class|const|else|export|from|function|if|import|interface|let|new|return|type|var"
    val pattern = Regex("(//[^\\n]*|#[^\\n]*|/\\*[\\s\\S]*?\\*/|\\\"(?:\\\\.|[^\\\"])*\\\"|'(?:\\\\.|[^'])*'|\\b(?:$keywords)\\b|\\b\\d+(?:\\.\\d+)?\\b)")
    return buildAnnotatedString {
        var end = 0
        pattern.findAll(source).forEach { match ->
            append(source.substring(end, match.range.first))
            val token = match.value
            val color = when {
                token.startsWith("//") || token.startsWith(35.toChar().toString()) || token.startsWith("/*") -> contentColor.copy(alpha = 0.55f)
                token.startsWith("\"") || token.startsWith("'") -> DoweDesign.success
                token.firstOrNull()?.isDigit() == true -> DoweDesign.warning
                else -> DoweDesign.primary
            }
            withStyle(SpanStyle(color = color)) { append(token) }
            end = match.range.last + 1
        }
        append(source.substring(end))
    }
}

@Composable
private fun DoweEditorField(value: String, onValueChange: (String) -> Unit, language: String, label: String?, placeholder: String, minHeight: Dp, hideToolbar: Boolean, readOnly: Boolean, onSave: (() -> Unit)?, modifier: Modifier, backgroundColor: Color, contentColor: Color) {
    Column(modifier = modifier.onPreviewKeyEvent { event ->
        if (onSave != null && event.type == KeyEventType.KeyDown && event.key == Key.S && (event.nativeKeyEvent.isCtrlPressed || event.nativeKeyEvent.isMetaPressed)) {
            onSave()
            true
        } else false
    }.clip(RoundedCornerShape(16.dp)).background(backgroundColor).border(1.dp, contentColor.copy(alpha = 0.18f), RoundedCornerShape(16.dp)), verticalArrangement = Arrangement.spacedBy(0.dp)) {
        if (label != null) Text(text = label, modifier = Modifier.padding(12.dp, 10.dp, 12.dp, 0.dp), fontWeight = FontWeight.SemiBold, color = contentColor)
        if (!hideToolbar) Row(modifier = Modifier.fillMaxWidth().background(contentColor.copy(alpha = 0.08f)).padding(6.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            listOf("B", "I", "U", "List").forEach { Text(it, modifier = Modifier.clip(RoundedCornerShape(8.dp)).background(contentColor.copy(alpha = 0.08f)).padding(horizontal = 8.dp, vertical = 5.dp), fontWeight = FontWeight.Bold, color = contentColor) }
        }
        BasicTextField(value = value, onValueChange = { if (!readOnly) onValueChange(it) }, modifier = Modifier.fillMaxWidth().heightIn(min = minHeight).padding(12.dp), textStyle = TextStyle(color = Color.Transparent, fontFamily = FontFamily.Monospace), decorationBox = { inner -> Box { if (value.isEmpty() && placeholder.isNotEmpty()) Text(placeholder, color = contentColor.copy(alpha = 0.52f)); Text(DoweEditorTokens(value, language, contentColor), fontFamily = FontFamily.Monospace); inner() } })
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
"#

