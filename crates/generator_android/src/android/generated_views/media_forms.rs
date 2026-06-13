fn android_runtime_media_forms() -> &'static str {
    r#"@Composable
private fun DoweVideo(source: String, poster: String?, autoplay: Boolean, aspect: String, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, borderColor: Color?) {
    var video by remember(source) { mutableStateOf<VideoView?>(null) }
    var started by remember(source) { mutableStateOf(false) }
    Box(modifier = modifier.aspectRatio(doweVideoAspect(aspect)).clip(shape).background(backgroundColor).then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))) {
        AndroidView(
            modifier = Modifier.matchParentSize(),
            factory = { context ->
                VideoView(context).apply {
                    val controller = MediaController(context)
                    controller.setAnchorView(this)
                    setMediaController(controller)
                    tag = source
                    setVideoURI(Uri.parse(source))
                    setOnPreparedListener {
                        if (autoplay) {
                            started = true
                            start()
                        }
                    }
                    video = this
                }
            },
            update = { view ->
                if (view.tag != source) {
                    view.tag = source
                    view.setVideoURI(Uri.parse(source))
                }
            }
        )
        if (poster != null && !started) {
            DoweCoverBox(modifier = Modifier.matchParentSize().clickable {
                started = true
                video?.start()
            }, source = poster, overlay = null) {}
        }
    }
}

private fun doweVideoAspect(value: String): Float {
    return when (value) {
        "vertical" -> 9f / 16f
        "square" -> 1f
        else -> 16f / 9f
    }
}

@Composable
private fun DoweAudio(source: String, subtitle: String?, avatarSource: String?, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var playing by remember(source) { mutableStateOf(false) }
    Row(
        modifier = modifier
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Button(
            onClick = { playing = !playing },
            colors = ButtonDefaults.buttonColors(containerColor = contentColor.copy(alpha = 0.12f), contentColor = contentColor),
            contentPadding = PaddingValues(horizontal = 10.dp, vertical = 6.dp)
        ) {
            Text(if (playing) "Pause" else "Play")
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(text = subtitle ?: source, color = contentColor, maxLines = 1)
            Row(horizontalArrangement = Arrangement.spacedBy(3.dp)) {
                repeat(24) { index ->
                    Box(
                        modifier = Modifier
                            .width(3.dp)
                            .height(((index % 7) + 4).dp)
                            .background(contentColor.copy(alpha = if (playing) 0.9f else 0.35f), RoundedCornerShape(2.dp))
                    )
                }
            }
        }
        if (avatarSource != null) {
            DoweCoverBox(modifier = Modifier.width(36.dp).height(36.dp).clip(RoundedCornerShape(999.dp)), source = avatarSource, overlay = null) {}
        }
    }
}

@Composable
private fun DoweImage(source: String, alt: String, aspect: String, objectFit: String, loading: String, hideControls: Boolean, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Box(
        modifier = modifier
            .aspectRatio(doweImageAspect(aspect))
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
    ) {
        DoweCoverBox(modifier = Modifier.matchParentSize(), source = source, overlay = null) {}
        if (!hideControls && alt.isNotEmpty()) {
            Text(
                text = alt,
                modifier = Modifier.align(Alignment.BottomStart).background(backgroundColor.copy(alpha = 0.72f)).padding(8.dp),
                color = contentColor,
                maxLines = 1
            )
        }
    }
}

private fun doweImageAspect(value: String): Float {
    return when (value) {
        "vertical" -> 9f / 16f
        "square" -> 1f
        "auto" -> 16f / 9f
        else -> 16f / 9f
    }
}

@Composable
private fun DoweAccordion(multiple: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?, content: @Composable () -> Unit) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(12.dp))),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            content()
        }
    }
}

@Composable
private fun DoweAccordionItem(id: String, label: String, disabled: Boolean, defaultOpen: Boolean, content: @Composable () -> Unit) {
    var open by remember(id) { mutableStateOf(defaultOpen) }
    Column(modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).border(1.dp, LocalContentColor.current.copy(alpha = 0.12f), RoundedCornerShape(10.dp))) {
        Row(
            modifier = Modifier.fillMaxWidth().clickable(enabled = !disabled) { open = !open }.padding(horizontal = 14.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(label, fontWeight = FontWeight.SemiBold)
            Text(if (open) "^" else "v")
        }
        AnimatedVisibility(visible = open, enter = fadeIn() + expandVertically(), exit = fadeOut() + shrinkVertically()) {
            Column(modifier = Modifier.fillMaxWidth().padding(14.dp)) {
                content()
            }
        }
    }
}

@Composable
private fun DoweCarousel(autoplay: Boolean, autoplayInterval: Int, disableLoop: Boolean, hideControls: Boolean, hideIndicators: Boolean, showNavigation: Boolean, showCounter: Boolean, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, modifier: Modifier, accentColor: Color, content: @Composable () -> Unit) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        if (title != null) {
            Text(title, fontWeight = FontWeight.Bold, color = accentColor)
        }
        Row(
            modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(gap.dp)
        ) {
            content()
        }
    }
}

@Composable
private fun DoweCarouselSlide(id: String, content: @Composable () -> Unit) {
    Box(modifier = Modifier.widthIn(min = 220.dp)) {
        content()
    }
}

@Composable
private fun DoweCheckbox(checked: Boolean, onCheckedChange: (Boolean) -> Unit, enabled: Boolean, label: String?, name: String?, modifier: Modifier, accentColor: Color) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Checkbox(
            checked = checked,
            onCheckedChange = onCheckedChange,
            enabled = enabled,
            colors = CheckboxDefaults.colors(checkedColor = accentColor, uncheckedColor = accentColor.copy(alpha = 0.72f), checkmarkColor = Color.White)
        )
        if (label != null) {
            Text(label, color = accentColor)
        }
    }
}

@Composable
private fun DoweColorField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, name: String?, helpText: String?, errorText: String?, showHex: Boolean, showRgb: Boolean, showCmyk: Boolean, showOklch: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = doweControlHeight(size))
                .clip(RoundedCornerShape(10.dp))
                .background(backgroundColor)
                .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)))
                .padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Box(
                modifier = Modifier
                    .width(doweControlSwatchSize(size))
                    .height(doweControlSwatchSize(size))
                    .clip(RoundedCornerShape(6.dp))
                    .background(doweHexColor(value, backgroundColor))
                    .border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(6.dp))
            )
            Text(text = value.ifEmpty { placeholder }.uppercase(), color = contentColor, fontSize = 14.sp, maxLines = 1)
        }
        if (showHex || showRgb || showCmyk || showOklch) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                if (showHex) Text("hex: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
                if (showRgb) Text("rgb: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
                if (showCmyk) Text("cmyk: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
                if (showOklch) Text("oklch: $value", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweDateField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        DoweInput(value = value, onValueChange = onValueChange, modifier = Modifier.fillMaxWidth(), label = label, placeholder = placeholder, floating = floating, fontFamily = FontFamily.Default, fontSize = 14.sp, lineHeight = 20.sp, minHeight = doweControlHeight(size), horizontalPadding = 12.dp, shape = RoundedCornerShape(10.dp), backgroundColor = backgroundColor, contentColor = contentColor, borderColor = borderColor)
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweDateRangeField(startValue: String, endValue: String, onStartChange: (String) -> Unit, onEndChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = doweControlHeight(size))
                .clip(RoundedCornerShape(10.dp))
                .background(backgroundColor)
                .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)))
                .padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            BasicTextField(
                value = startValue,
                onValueChange = onStartChange,
                modifier = Modifier.weight(1f),
                singleLine = true,
                textStyle = TextStyle(fontFamily = FontFamily.Default, fontSize = 14.sp, lineHeight = 20.sp, color = contentColor),
                decorationBox = { innerTextField ->
                    Box {
                        if (startValue.isEmpty()) Text("Start", color = contentColor.copy(alpha = 0.55f), fontSize = 14.sp)
                        innerTextField()
                    }
                }
            )
            Text("-", color = contentColor.copy(alpha = 0.64f))
            BasicTextField(
                value = endValue,
                onValueChange = onEndChange,
                modifier = Modifier.weight(1f),
                singleLine = true,
                textStyle = TextStyle(fontFamily = FontFamily.Default, fontSize = 14.sp, lineHeight = 20.sp, color = contentColor),
                decorationBox = { innerTextField ->
                    Box {
                        if (endValue.isEmpty()) Text("End", color = contentColor.copy(alpha = 0.55f), fontSize = 14.sp)
                        innerTextField()
                    }
                }
            )
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

private data class DoweRadioOption(val value: String, val label: String, val disabled: Boolean)

@Composable
private fun DoweRadioGroup(value: String, onValueChange: (String) -> Unit, options: List<DoweRadioOption>, size: String, name: String?, label: String?, helpText: String?, errorText: String?, modifier: Modifier, accentColor: Color) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) {
            Text(label, fontWeight = FontWeight.SemiBold, color = accentColor)
        }
        options.forEach { option ->
            Row(modifier = Modifier.clickable(enabled = !option.disabled) { onValueChange(option.value) }, verticalAlignment = Alignment.CenterVertically) {
                RadioButton(
                    selected = value == option.value,
                    onClick = { onValueChange(option.value) },
                    enabled = !option.disabled,
                    colors = RadioButtonDefaults.colors(selectedColor = accentColor, unselectedColor = accentColor.copy(alpha = 0.72f))
                )
                Text(option.label, color = accentColor)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = accentColor.copy(alpha = 0.7f))
        }
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
    var dark by remember {
        mutableStateOf(context.getSharedPreferences("dowe", 0).getString("theme-preference", "light") == "dark")
    }
    Button(
        modifier = modifier.defaultMinSize(minWidth = 0.dp, minHeight = 0.dp),
        colors = ButtonDefaults.buttonColors(containerColor = backgroundColor, contentColor = contentColor),
        border = borderColor?.let { BorderStroke(1.dp, it) },
        contentPadding = PaddingValues(0.dp),
        onClick = {
            dark = !dark
            context.getSharedPreferences("dowe", 0).edit().putString("theme-preference", if (dark) "dark" else "light").apply()
        }
    ) {
        Text(if (dark) "sun" else "moon", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
private fun DoweSliderField(value: Float, onValueChange: (Float) -> Unit, bound: Boolean, label: String?, hideLabel: Boolean, min: Float, max: Float, size: String, modifier: Modifier, accentColor: Color) {
    var local by remember(value, bound) { mutableStateOf(value) }
    val current = if (bound) value else local
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(2.dp)) {
        if (!hideLabel) {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text(label.orEmpty(), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = accentColor)
                Text(current.toInt().toString(), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = accentColor)
            }
        }
        Slider(
            value = current.coerceIn(min, max),
            onValueChange = {
                if (bound) onValueChange(it) else local = it
            },
            valueRange = min..max,
            colors = SliderDefaults.colors(thumbColor = accentColor, activeTrackColor = accentColor)
        )
    }
}

@Composable
private fun DoweDropzone(label: String?, placeholder: String, helpText: String?, errorText: String?, size: String, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val height = when (size) {
        "sm" -> 128.dp
        "lg" -> 256.dp
        else -> 192.dp
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(height)
                .clip(RoundedCornerShape(12.dp))
                .background(backgroundColor)
                .border(2.dp, borderColor ?: contentColor.copy(alpha = 0.55f), RoundedCornerShape(12.dp))
                .clickable { },
            contentAlignment = Alignment.Center
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Upload", color = contentColor.copy(alpha = 0.55f), fontWeight = FontWeight.SemiBold)
                Text(placeholder, color = contentColor.copy(alpha = 0.7f), fontSize = 14.sp)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = if (errorText != null) DoweDesign.danger else contentColor.copy(alpha = 0.7f))
        }
    }
}

private fun doweControlHeight(size: String): Dp {
    return when (size) {
        "sm" -> 34.dp
        "lg" -> 48.dp
        else -> 40.dp
    }
}

private fun doweControlSwatchSize(size: String): Dp {
    return when (size) {
        "sm" -> 20.dp
        "lg" -> 32.dp
        else -> 24.dp
    }
}

private fun doweHexColor(value: String, fallback: Color): Color {
    return try {
        Color(android.graphics.Color.parseColor(value))
    } catch (error: IllegalArgumentException) {
        fallback
    }
}

"#
}
