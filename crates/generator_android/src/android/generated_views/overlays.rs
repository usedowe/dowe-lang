fn android_runtime_overlays() -> &'static str {
    r#"@Composable
private fun DoweModal(open: Boolean, close: () -> Unit, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp, disableOverlayClose: Boolean, hideCloseButton: Boolean, header: (@Composable () -> Unit)?, footer: (@Composable () -> Unit)?, content: @Composable () -> Unit) {
    if (!open) {
        return
    }
    Popup(properties = PopupProperties(focusable = true)) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .background(Color.Black.copy(alpha = 0.48f))
                    .clickable { if (!disableOverlayClose) close() }
            )
            Column(
                modifier = Modifier
                    .widthIn(max = 560.dp)
                    .clip(RoundedCornerShape(radius))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(radius)))
                    .padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                CompositionLocalProvider(LocalContentColor provides contentColor) {
                    if (header != null) {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                            Box(modifier = Modifier.weight(1f)) { header() }
                            if (!hideCloseButton) {
                                Text(text = "x", modifier = Modifier.clickable(onClick = close).padding(4.dp), color = contentColor.copy(alpha = 0.72f), fontWeight = FontWeight.Bold)
                            }
                        }
                    } else if (!hideCloseButton) {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                            Text(text = "x", modifier = Modifier.clickable(onClick = close).padding(4.dp), color = contentColor.copy(alpha = 0.72f), fontWeight = FontWeight.Bold)
                        }
                    }
                    content()
                    footer?.invoke()
                }
            }
        }
    }
}

@Composable
private fun DoweAlertDialog(open: Boolean, close: () -> Unit, title: String, description: String, confirmText: String, cancelText: String, backgroundColor: Color, contentColor: Color, dangerColor: Color, radius: Dp, loading: Boolean, onConfirm: (() -> Unit)?, onCancel: (() -> Unit)?) {
    DoweModal(
        open = open,
        close = close,
        backgroundColor = backgroundColor,
        contentColor = contentColor,
        borderColor = null,
        radius = radius,
        disableOverlayClose = true,
        hideCloseButton = true,
        header = { Text(text = title, color = contentColor, fontSize = 18.sp, fontWeight = FontWeight.SemiBold) },
        footer = {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                Button(
                    enabled = !loading,
                    onClick = { close(); onCancel?.invoke() },
                    colors = ButtonDefaults.buttonColors(containerColor = Color.Transparent, contentColor = contentColor),
                    border = BorderStroke(1.dp, contentColor.copy(alpha = 0.24f))
                ) { Text(cancelText) }
                Button(enabled = !loading, onClick = { onConfirm?.invoke() }, colors = ButtonDefaults.buttonColors(containerColor = dangerColor, contentColor = DoweDesign.onDanger)) { Text(confirmText) }
            }
        }
    ) {
        Text(text = description, color = contentColor.copy(alpha = 0.72f), fontSize = 14.sp)
    }
}

@Composable
private fun DoweTooltip(label: String, position: String, backgroundColor: Color, contentColor: Color, modifier: Modifier, content: @Composable () -> Unit) {
    Box(modifier = modifier) {
        content()
    }
}

@Composable
private fun DoweToast(visible: Boolean, title: String, description: String, position: String, backgroundColor: Color, contentColor: Color, showIcon: Boolean, kind: String, close: (() -> Unit)?) {
    if (!visible) {
        return
    }
    Popup(alignment = doweCornerAlignment(position)) {
        Row(
            modifier = Modifier
                .padding(16.dp)
                .widthIn(max = 420.dp)
                .clip(RoundedCornerShape(DoweDesign.radius))
                .background(backgroundColor)
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            if (showIcon) {
                Text(text = doweToastIcon(kind), color = contentColor, fontWeight = FontWeight.Bold)
            }
            Column(modifier = Modifier.weight(1f)) {
                if (title.isNotEmpty()) {
                    Text(text = title, color = contentColor, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                }
                Text(text = description, color = contentColor.copy(alpha = 0.9f), fontSize = 14.sp)
            }
            if (close != null) {
                Text(text = "x", modifier = Modifier.clickable(onClick = close), color = contentColor.copy(alpha = 0.72f), fontWeight = FontWeight.Bold)
            }
        }
    }
}

private fun doweCornerAlignment(position: String): Alignment =
    when (position) {
        "top-left" -> Alignment.TopStart
        "top-right" -> Alignment.TopEnd
        "bottom-right" -> Alignment.BottomEnd
        else -> Alignment.BottomStart
    }

private fun doweToastIcon(kind: String): String =
    when (kind) {
        "success" -> "✓"
        "warning" -> "!"
        "danger", "error" -> "x"
        else -> "i"
    }

@Composable
private fun DoweDropdown(backgroundColor: Color, contentColor: Color, modifier: Modifier, trigger: @Composable () -> Unit, content: @Composable (() -> Unit) -> Unit) {
    var open by remember { mutableStateOf(false) }
    var popupMounted by remember { mutableStateOf(false) }
    var triggerHeight by remember { mutableStateOf(0) }
    val popupOffset = with(LocalDensity.current) { IntOffset(0, triggerHeight + 4.dp.roundToPx()) }
    LaunchedEffect(open) {
        if (open) {
            popupMounted = true
        } else if (popupMounted) {
            delay(160)
            popupMounted = false
        }
    }
    Box(modifier = modifier) {
        Box(modifier = Modifier.onGloballyPositioned { triggerHeight = it.size.height }) {
            trigger()
            Box(modifier = Modifier.matchParentSize().clickable { open = !open })
        }
        if (triggerHeight > 0 && (open || popupMounted)) {
            DoweAnchoredPopover(
                visible = open,
                offset = popupOffset,
                shape = RoundedCornerShape(DoweDesign.radius),
                backgroundColor = backgroundColor,
                contentColor = contentColor,
                contentPadding = PaddingValues(8.dp),
                onDismiss = { open = false }
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    content { open = false }
                }
            }
        }
    }
}

@Composable
private fun DoweOverlayItem(label: String, description: String?, disabled: Boolean, backgroundColor: Color, contentColor: Color, onClick: (() -> Unit)?, icon: @Composable () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(DoweDesign.radius))
            .background(backgroundColor.copy(alpha = if (onClick == null) 0f else 0.08f))
            .then(if (onClick == null || disabled) Modifier else Modifier.clickable(onClick = onClick))
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            icon()
            Column(modifier = Modifier.weight(1f)) {
                Text(text = label, color = contentColor.copy(alpha = if (disabled) 0.48f else 1f), fontSize = 14.sp, fontWeight = FontWeight.Medium)
                if (description != null) {
                    Text(text = description, color = contentColor.copy(alpha = 0.68f), fontSize = 12.sp)
                }
            }
        }
    }
}

@Composable
private fun DoweCommand(open: Boolean, close: () -> Unit, placeholder: String, emptyText: String, closeText: String, navigateText: String, selectText: String, toggleText: String, shortcut: String, showFooter: Boolean, backgroundColor: Color, contentColor: Color, accentColor: Color, content: @Composable () -> Unit) {
    if (!open) {
        return
    }
    Popup(properties = PopupProperties(focusable = true)) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            Box(modifier = Modifier.matchParentSize().background(Color.Black.copy(alpha = 0.48f)).clickable(onClick = close))
            Column(
                modifier = Modifier
                    .padding(top = 64.dp)
                    .widthIn(min = 320.dp, max = 560.dp)
                    .clip(RoundedCornerShape(DoweDesign.radius))
                    .background(backgroundColor)
                    .padding(12.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Text(text = placeholder, color = contentColor.copy(alpha = 0.56f), fontSize = 15.sp)
                Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(contentColor.copy(alpha = 0.12f)))
                CompositionLocalProvider(LocalContentColor provides contentColor) {
                    content()
                }
                if (showFooter) {
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                        Text(text = "Esc $closeText", color = contentColor.copy(alpha = 0.6f), fontSize = 12.sp)
                        Text(text = "Ctrl+${shortcut.uppercase()} $toggleText", color = accentColor, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                    }
                }
            }
        }
    }
}

"#
}
