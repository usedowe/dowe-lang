fn android_runtime_overlays() -> &'static str {
    r#"private val doweOverlayCloseViewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)
private val doweOverlayClosePaths = listOf(
    DoweSvgPath("M0 0h24v24H0z", DoweSvgFill.None),
    DoweSvgPath("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z", DoweSvgFill.CurrentColor)
)

@Composable
private fun DoweModal(open: Boolean, close: () -> Unit, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp, disableOverlayClose: Boolean, hideCloseButton: Boolean, header: (@Composable () -> Unit)?, footer: (@Composable () -> Unit)?, content: @Composable () -> Unit) {
    if (!open) {
        return
    }
    Dialog(
        onDismissRequest = { if (!disableOverlayClose) close() },
        properties = DialogProperties(dismissOnClickOutside = !disableOverlayClose, usePlatformDefaultWidth = false, decorFitsSystemWindows = false)
    ) {
        BoxWithConstraints(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            val modalMaxWidth = (maxWidth * 0.95f).coerceAtMost(560.dp)
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .background(Color.Black.copy(alpha = 0.48f))
                    .clickable { if (!disableOverlayClose) close() }
            )
            Box(
                modifier = Modifier
                    .width(modalMaxWidth)
                    .padding(16.dp)
                    .clip(RoundedCornerShape(radius))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(radius)))
            ) {
                CompositionLocalProvider(LocalContentColor provides contentColor) {
                    Column(
                        modifier = Modifier.padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        header?.invoke()
                        content()
                        footer?.invoke()
                    }
                }
                if (!hideCloseButton) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(8.dp)
                            .clip(RoundedCornerShape(999.dp))
                            .background(DoweDesign.softMuted)
                            .clickable(onClick = close)
                            .semantics { contentDescription = "Close modal" }
                            .width(28.dp)
                            .height(28.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        DoweSvg(viewBox = doweOverlayCloseViewBox, modifier = Modifier.width(18.dp).height(18.dp), color = DoweDesign.softMutedText, paths = doweOverlayClosePaths)
                    }
                }
            }
        }
    }
}

@Composable
private fun DoweAlertDialog(open: Boolean, close: () -> Unit, title: String, description: String, confirmText: String, cancelText: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, confirmBackgroundColor: Color, confirmContentColor: Color, radius: Dp, loading: Boolean, onConfirm: (() -> Unit)?) {
    DoweModal(
        open = open,
        close = close,
        backgroundColor = backgroundColor,
        contentColor = contentColor,
        borderColor = borderColor,
        radius = radius,
        disableOverlayClose = true,
        hideCloseButton = true,
        header = { Text(text = title, color = contentColor, fontSize = 18.sp, fontWeight = FontWeight.SemiBold) },
        footer = {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.End), verticalAlignment = Alignment.CenterVertically) {
                Button(
                    enabled = !loading,
                    onClick = close,
                    modifier = Modifier.defaultMinSize(minWidth = 0.dp, minHeight = doweButtonMinHeight("md")),
                    shape = RoundedCornerShape(DoweDesign.radius),
                    colors = ButtonDefaults.buttonColors(containerColor = Color.Transparent, contentColor = DoweDesign.muted),
                    border = BorderStroke(1.dp, DoweDesign.muted),
                    contentPadding = PaddingValues(horizontal = doweButtonHorizontalPadding("md"), vertical = doweButtonVerticalPadding("md"))
                ) { Text(cancelText) }
                Button(
                    enabled = !loading,
                    onClick = { onConfirm?.invoke() },
                    modifier = Modifier.defaultMinSize(minWidth = 0.dp, minHeight = doweButtonMinHeight("md")),
                    shape = RoundedCornerShape(DoweDesign.radius),
                    colors = ButtonDefaults.buttonColors(containerColor = confirmBackgroundColor, contentColor = confirmContentColor),
                    contentPadding = PaddingValues(horizontal = doweButtonHorizontalPadding("md"), vertical = doweButtonVerticalPadding("md"))
                ) { Text(confirmText) }
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
private fun DoweToast(visible: Boolean, title: String, description: String, position: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, showIcon: Boolean, kind: String, close: (() -> Unit)?, viewportWidth: Dp) {
    var dismissed by remember(visible, title, description) { mutableStateOf(false) }
    if (!visible || dismissed) {
        return
    }
    val toastWidth = (viewportWidth - 32.dp).coerceAtLeast(1.dp).coerceAtMost(420.dp)
    Popup(alignment = doweCornerAlignment(position)) {
        Row(
            modifier = Modifier
                .padding(16.dp)
                .width(toastWidth)
                .clip(RoundedCornerShape(DoweDesign.radius))
                .background(backgroundColor)
                .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(DoweDesign.radius)))
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
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
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(DoweDesign.softMuted)
                    .clickable { dismissed = true; close?.invoke() }
                    .semantics { contentDescription = "Close toast" }
                    .width(28.dp)
                    .height(28.dp),
                contentAlignment = Alignment.Center
            ) {
                DoweSvg(viewBox = doweOverlayCloseViewBox, modifier = Modifier.width(18.dp).height(18.dp), color = DoweDesign.softMutedText, paths = doweOverlayClosePaths)
            }
        }
    }
}

@Composable
private fun DoweGlobalToast(toast: DoweToastState?, close: () -> Unit, viewportWidth: Dp) {
    if (toast == null) {
        return
    }
    LaunchedEffect(toast) {
        delay(toast.duration.toLong().coerceAtLeast(500L))
        close()
    }
    DoweToast(
        visible = true,
        title = toast.title,
        description = toast.message,
        position = toast.position,
        backgroundColor = doweCardContainer(toast.variant, toast.scheme),
        contentColor = doweCardContent(toast.variant, toast.scheme),
        borderColor = doweCardBorder(toast.variant, toast.scheme),
        showIcon = false,
        kind = toast.kind,
        close = close,
        viewportWidth = viewportWidth
    )
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
