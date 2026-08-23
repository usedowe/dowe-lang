fn android_runtime_media_choice() -> &'static str {
    r##"
private fun doweRadioSize(size: String): Dp {
    return when (size) {
        "sm" -> 16.dp
        "lg" -> 24.dp
        else -> 20.dp
    }
}

private fun doweRadioDotSize(size: String): Dp {
    return when (size) {
        "sm" -> 8.dp
        "lg" -> 14.dp
        else -> 12.dp
    }
}

@Composable
private fun DoweRadioGroup(value: String, onValueChange: (String) -> Unit, options: List<DoweRadioOption>, size: String, orientation: String, name: String?, label: String?, helpText: String?, errorText: String?, modifier: Modifier, accentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) {
            Text(label, fontWeight = FontWeight.SemiBold, color = accentColor)
        }
        if (orientation == "horizontal") {
            Row(modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(16.dp), verticalAlignment = Alignment.CenterVertically) {
                options.forEach { option ->
                    DoweRadioGroupOption(option = option, selected = value == option.value, size = size, accentColor = accentColor) { onValueChange(option.value) }
                }
            }
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                options.forEach { option ->
                    DoweRadioGroupOption(option = option, selected = value == option.value, size = size, accentColor = accentColor) { onValueChange(option.value) }
                }
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = accentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweRadioGroupOption(option: DoweRadioOption, selected: Boolean, size: String, accentColor: Color, onSelect: () -> Unit) {
    Row(modifier = Modifier.clickable(enabled = !option.disabled) { onSelect() }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Box(
            modifier = Modifier
                .width(doweRadioSize(size))
                .height(doweRadioSize(size))
                .clip(RoundedCornerShape(999.dp))
                .border(2.dp, if (selected) accentColor else accentColor.copy(alpha = 0.72f), RoundedCornerShape(999.dp)),
            contentAlignment = Alignment.Center
        ) {
            if (selected) {
                Box(
                    modifier = Modifier
                        .width(doweRadioDotSize(size))
                        .height(doweRadioDotSize(size))
                        .clip(RoundedCornerShape(999.dp))
                        .background(accentColor)
                )
            }
        }
        Text(option.label, color = accentColor)
    }
}

@Composable
private fun DoweToggle(checked: Boolean, onCheckedChange: (Boolean) -> Unit, enabled: Boolean, label: String?, labelLeft: String?, labelRight: String?, name: String?, modifier: Modifier, accentColor: Color) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        if (labelLeft != null) {
            Text(labelLeft, color = accentColor.copy(alpha = if (checked) 0.45f else 1f))
        }
        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
            enabled = enabled,
            colors = SwitchDefaults.colors(checkedThumbColor = Color.White, checkedTrackColor = accentColor, uncheckedThumbColor = Color.White, uncheckedTrackColor = accentColor.copy(alpha = 0.28f))
        )
        if (labelRight != null) {
            Text(labelRight, color = accentColor.copy(alpha = if (checked) 1f else 0.45f))
        }
        if (label != null) {
            Text(label, color = accentColor)
        }
    }
}

@Composable
private fun DoweThemeToggle(modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    val current = DoweDesign.name
    Button(
        modifier = modifier.defaultMinSize(minWidth = 0.dp, minHeight = 0.dp),
        colors = ButtonDefaults.buttonColors(containerColor = backgroundColor, contentColor = contentColor),
        border = borderColor?.let { BorderStroke(1.dp, it) },
        contentPadding = PaddingValues(0.dp),
        onClick = {
            val next = if (current == "dark") "light" else "dark"
            context.getSharedPreferences("dowe", 0).edit().putString("theme-preference", next).apply()
            DoweDesign.applyTheme(next)
        }
    ) {
        Text(if (current == "dark") "sun" else "moon", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
private fun DoweThemeSelect(modifier: Modifier, label: String, placeholder: String, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    val options = DoweThemeModule.names.map { name ->
        DoweSelectOption(name, name.replace("-", " ").split(" ").joinToString(" ") { part -> part.replaceFirstChar { it.uppercase() } }, null)
    }
    DoweSelect(
        value = DoweDesign.name,
        onValueChange = { name ->
            context.getSharedPreferences("dowe", 0).edit().putString("theme-preference", name).apply()
            DoweDesign.applyTheme(name)
        },
        bound = true,
        modifier = modifier,
        label = label,
        placeholder = placeholder,
        floating = false,
        options = options,
        fontFamily = FontFamily.Default,
        fontSize = 14.sp,
        lineHeight = 20.sp,
        minHeight = 40.dp,
        horizontalPadding = 12.dp,
        shape = RoundedCornerShape(DoweDesign.radius),
        backgroundColor = backgroundColor,
        contentColor = contentColor,
        borderColor = borderColor
    )
}

@Composable
private fun DoweSliderField(value: Float, onValueChange: (Float) -> Unit, bound: Boolean, label: String?, hideLabel: Boolean, min: Float, max: Float, size: String, modifier: Modifier, accentColor: Color) {
    var local by remember(value, bound) { mutableStateOf(value) }
    val current = if (bound) value else local
    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (!hideLabel) {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text(label.orEmpty(), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = accentColor)
                Text(current.toInt().toString(), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = accentColor)
            }
        }
        Slider(
            value = current.coerceIn(min, max),
            modifier = Modifier.fillMaxWidth(),
            onValueChange = {
                if (bound) onValueChange(it) else local = it
            },
            valueRange = min..max,
            colors = SliderDefaults.colors(thumbColor = accentColor, activeTrackColor = accentColor, inactiveTrackColor = accentColor.copy(alpha = 0.18f))
        )
    }
}

"##
}
