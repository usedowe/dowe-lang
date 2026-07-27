fn android_runtime_anchored_popover() -> &'static str {
    r#"@Composable
private fun DoweAnchoredPopover(visible: Boolean, offset: IntOffset, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, contentPadding: PaddingValues, onDismiss: () -> Unit, content: @Composable () -> Unit) {
    val progress by animateFloatAsState(
        targetValue = if (visible) 1f else 0f,
        animationSpec = tween(durationMillis = 160)
    )
    Popup(alignment = Alignment.TopStart, offset = offset, onDismissRequest = onDismiss, properties = PopupProperties(focusable = true)) {
        Column(
            modifier = Modifier
                .widthIn(min = 220.dp, max = 360.dp)
                .graphicsLayer {
                    alpha = progress
                    translationY = (1f - progress) * -4f
                    val value = 0.98f + (0.02f * progress)
                    scaleX = value
                    scaleY = value
                }
                .clip(shape)
                .background(backgroundColor)
                .border(1.dp, contentColor.copy(alpha = 0.08f), shape)
                .heightIn(max = 260.dp)
                .verticalScroll(rememberScrollState())
                .padding(contentPadding)
        ) {
            CompositionLocalProvider(LocalContentColor provides contentColor) {
                content()
            }
        }
    }
}

"#
}
