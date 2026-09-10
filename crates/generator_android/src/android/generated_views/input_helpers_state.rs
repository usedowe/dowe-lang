r#"private fun Modifier.doweReactiveStyle(property: String, value: String): Modifier {
    val number = value.toFloatOrNull()
    return when (property) {
        "p" -> if (number != null) padding(number.dp) else this
        "px" -> if (number != null) padding(horizontal = number.dp) else this
        "py" -> if (number != null) padding(vertical = number.dp) else this
        "pl" -> if (number != null) padding(start = number.dp) else this
        "pr" -> if (number != null) padding(end = number.dp) else this
        "pt" -> if (number != null) padding(top = number.dp) else this
        "pb" -> if (number != null) padding(bottom = number.dp) else this
        "w" -> if (number != null) width(number.dp) else this
        "h" -> if (number != null) height(number.dp) else this
        "minW" -> if (number != null) widthIn(min = number.dp) else this
        "minH" -> if (number != null) heightIn(min = number.dp) else this
        "maxW" -> if (number != null) widthIn(max = number.dp) else this
        "maxH" -> if (number != null) heightIn(max = number.dp) else this
        "border" -> if (number != null) border(number.dp, Color.Transparent) else this
        else -> this
    }
}

private data class DoweValidationRule(val kind: String, val argument: String?, val message: String)

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
"#

