fn android_runtime_media_date() -> &'static str {
    r##"private fun DoweDateField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, fontSize: TextUnit, lineHeight: TextUnit, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?, validationRules: List<DoweValidationRule> = emptyList()) {
    var expanded by remember { mutableStateOf(false) }
    var touched by remember { mutableStateOf(false) }
    var month by remember(value) { mutableStateOf(runCatching { YearMonth.from(LocalDate.parse(value)) }.getOrDefault(YearMonth.now())) }
    val active = expanded || value.isNotEmpty()
    val validationError = errorText ?: if (touched) doweValidationError(value, validationRules) else null
    val popupOffset = with(LocalDensity.current) { (doweControlHeight(size) + if (floating) 12.dp else 4.dp).roundToPx() }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        Box {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(doweControlHeight(size) + if (floating) 8.dp else 0.dp)
                    .clip(RoundedCornerShape(DoweDesign.radius))
                    .background(backgroundColor)
                    .then(if (borderColor == null && validationError == null) Modifier else Modifier.border(1.dp, if (validationError != null) DoweDesign.danger else borderColor!!, RoundedCornerShape(DoweDesign.radius)))
                    .clickable { if (expanded) touched = true; expanded = !expanded }
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Box(modifier = Modifier.weight(1f)) {
                    if (label != null && floating) Text(label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else 14.sp, color = contentColor)
                    Text(if (value.isEmpty()) placeholder else doweDateDisplay(value), modifier = Modifier.align(Alignment.CenterStart).padding(top = if (label != null && floating && active) 10.dp else 0.dp), fontSize = fontSize, lineHeight = lineHeight, color = contentColor, maxLines = 1)
                }
                Text("⌄", fontSize = 20.sp, color = contentColor)
            }
            DoweAnchoredPopover(visible = expanded, offset = IntOffset(0, popupOffset), shape = RoundedCornerShape(12.dp), backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, contentPadding = PaddingValues(8.dp), onDismiss = { expanded = false; touched = true }) {
                DoweDateCalendar(month = month, selected = value, start = "", end = "", min = min, max = max, contentColor = contentColor, accentColor = contentColor, showPrevious = true, showNext = true, onPrevious = { month = month.minusMonths(1) }, onNext = { month = month.plusMonths(1) }, onSelect = { next -> onValueChange(next); month = YearMonth.from(LocalDate.parse(next)); expanded = false; touched = true })
            }
        }
        DoweValidationFeedback(helpText, validationError, contentColor)
    }
}

@Composable
private fun DoweDateRangeField(startValue: String, endValue: String, onStartChange: (String) -> Unit, onEndChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, fontSize: TextUnit, lineHeight: TextUnit, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var expanded by remember { mutableStateOf(false) }
    var selectingEnd by remember { mutableStateOf(false) }
    var month by remember(startValue) { mutableStateOf(runCatching { YearMonth.from(LocalDate.parse(startValue)) }.getOrDefault(YearMonth.now())) }
    val active = expanded || startValue.isNotEmpty() || endValue.isNotEmpty()
    val popupOffset = with(LocalDensity.current) { (doweControlHeight(size) + if (floating) 12.dp else 4.dp).roundToPx() }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(doweControlHeight(size) + if (floating) 8.dp else 0.dp)
                    .clip(RoundedCornerShape(DoweDesign.radius))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(DoweDesign.radius)))
                    .clickable { selectingEnd = false; expanded = !expanded }
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Box(modifier = Modifier.weight(1f)) {
                    if (label != null && floating) Text(label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else 14.sp, color = contentColor)
                    Text(doweDateRangeDisplay(startValue, endValue, placeholder), modifier = Modifier.align(Alignment.CenterStart).padding(top = if (label != null && floating && active) 10.dp else 0.dp), fontSize = fontSize, lineHeight = lineHeight, color = contentColor, maxLines = 1)
                }
                Text("⌄", fontSize = 20.sp, color = contentColor)
            }
            DoweAnchoredPopover(visible = expanded, offset = IntOffset(0, popupOffset), shape = RoundedCornerShape(12.dp), backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, contentPadding = PaddingValues(8.dp), onDismiss = { expanded = false }) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    DoweDateCalendar(month = month, selected = "", start = startValue, end = endValue, min = min, max = max, contentColor = contentColor, accentColor = contentColor, showPrevious = true, showNext = false, onPrevious = { month = month.minusMonths(1) }, onNext = {}, onSelect = { next -> if (!selectingEnd) { onStartChange(next); onEndChange(""); selectingEnd = true; month = YearMonth.from(LocalDate.parse(next)) } else { if (next < startValue) { onEndChange(startValue); onStartChange(next) } else onEndChange(next); selectingEnd = false; expanded = false } }, modifier = Modifier.weight(1f))
                    DoweDateCalendar(month = month.plusMonths(1), selected = "", start = startValue, end = endValue, min = min, max = max, contentColor = contentColor, accentColor = contentColor, showPrevious = false, showNext = true, onPrevious = {}, onNext = { month = month.plusMonths(1) }, onSelect = { next -> if (!selectingEnd) { onStartChange(next); onEndChange(""); selectingEnd = true; month = YearMonth.from(LocalDate.parse(next)) } else { if (next < startValue) { onEndChange(startValue); onStartChange(next) } else onEndChange(next); selectingEnd = false; expanded = false } }, modifier = Modifier.weight(1f))
                }
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

private fun doweDateDisplay(value: String): String {
    return runCatching { LocalDate.parse(value).format(DateTimeFormatter.ofPattern("MMM d, yyyy", Locale.getDefault())) }.getOrDefault(value)
}

private fun doweDateRangeDisplay(start: String, end: String, placeholder: String): String {
    return when {
        start.isNotEmpty() && end.isNotEmpty() -> "${doweDateDisplay(start)} – ${doweDateDisplay(end)}"
        start.isNotEmpty() -> "${doweDateDisplay(start)} – …"
        else -> placeholder
    }
}

private fun doweDateAllowed(value: String, min: String?, max: String?): Boolean {
    val date = runCatching { LocalDate.parse(value) }.getOrNull() ?: return false
    val minimum = min?.let { runCatching { LocalDate.parse(it) }.getOrNull() }
    val maximum = max?.let { runCatching { LocalDate.parse(it) }.getOrNull() }
    return (minimum == null || !date.isBefore(minimum)) && (maximum == null || !date.isAfter(maximum))
}

private fun doweDateMonthDays(month: YearMonth): List<LocalDate?> {
    val leading = month.atDay(1).dayOfWeek.value - 1
    return List(leading) { null } + (1..month.lengthOfMonth()).map { month.atDay(it) }
}

@Composable
private fun DoweDateCalendar(month: YearMonth, selected: String, start: String, end: String, min: String?, max: String?, contentColor: Color, accentColor: Color, showPrevious: Boolean, showNext: Boolean, onPrevious: () -> Unit, onNext: () -> Unit, onSelect: (String) -> Unit, modifier: Modifier = Modifier) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            Text("‹", modifier = Modifier.size(32.dp).clickable(enabled = showPrevious) { onPrevious() }, fontSize = 24.sp, color = contentColor, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
            Text(month.format(DateTimeFormatter.ofPattern("MMMM yyyy", Locale.getDefault())), fontWeight = FontWeight.SemiBold, color = contentColor)
            Text("›", modifier = Modifier.size(32.dp).clickable(enabled = showNext) { onNext() }, fontSize = 24.sp, color = contentColor, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
        }
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(3.dp)) {
            listOf("M", "T", "W", "T", "F", "S", "S").forEach { Text(it, modifier = Modifier.weight(1f), fontSize = 11.sp, color = contentColor.copy(alpha = 0.68f), textAlign = androidx.compose.ui.text.style.TextAlign.Center) }
        }
        doweDateMonthDays(month).chunked(7).forEach { week ->
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(3.dp)) {
                week.forEach { date ->
                    if (date == null) Spacer(modifier = Modifier.weight(1f).height(34.dp)) else {
                        val value = date.toString()
                        val isStart = value == start
                        val isEnd = value == end
                        val isSelected = value == selected || isStart || isEnd
                        val isToday = date == LocalDate.now()
                        val inRange = start.isNotEmpty() && end.isNotEmpty() && value > start && value < end
                        val enabled = doweDateAllowed(value, min, max)
                        Box(modifier = Modifier.weight(1f).height(34.dp).clip(RoundedCornerShape(8.dp)).background(if (isSelected) accentColor else if (inRange) accentColor.copy(alpha = 0.16f) else Color.Transparent).then(if (isToday && !isSelected) Modifier.border(1.dp, accentColor, RoundedCornerShape(8.dp)) else Modifier).then(if (enabled) Modifier.clickable { onSelect(value) } else Modifier), contentAlignment = Alignment.Center) {
                            Text(date.dayOfMonth.toString(), fontSize = 12.sp, fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Normal, color = if (isSelected) Color.White else contentColor.copy(alpha = if (enabled) 1f else 0.35f))
                        }
                    }
                }
                repeat(7 - week.size) { Spacer(modifier = Modifier.weight(1f).height(34.dp)) }
            }
        }
    }
}

private data class DoweRadioOption(val value: String, val label: String, val disabled: Boolean)
"##
}
