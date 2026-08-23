fn android_runtime_media_checkbox() -> &'static str {
    r##"@Composable
private fun DoweCheckbox(checked: Boolean, onCheckedChange: (Boolean) -> Unit, enabled: Boolean, label: String?, name: String?, modifier: Modifier, accentColor: Color, helpText: String? = null, errorText: String? = null, validationRules: List<DoweValidationRule> = emptyList()) {
    var touched by remember { mutableStateOf(false) }
    val validationError = errorText ?: if (touched) doweBooleanValidationError(checked, validationRules) else null
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
    Row(modifier = Modifier.clickable(enabled = enabled) { touched = true; onCheckedChange(!checked) }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Box(
            modifier = Modifier
                .width(20.dp)
                .height(20.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(if (checked) accentColor else Color.Transparent)
                .border(2.dp, if (validationError != null) DoweDesign.danger else if (checked) accentColor else accentColor.copy(alpha = 0.72f), RoundedCornerShape(4.dp))
        ) {
            if (checked) {
                Canvas(modifier = Modifier.fillMaxSize().padding(4.dp)) {
                    drawLine(Color.White, Offset(size.width * 0.12f, size.height * 0.52f), Offset(size.width * 0.38f, size.height * 0.78f), strokeWidth = 3f)
                    drawLine(Color.White, Offset(size.width * 0.38f, size.height * 0.78f), Offset(size.width * 0.88f, size.height * 0.18f), strokeWidth = 3f)
                }
            }
        }
        if (label != null) {
            Text(label, color = accentColor)
        }
    }
    DoweValidationFeedback(helpText, validationError, accentColor)
    }
}

@Composable
"##
}
