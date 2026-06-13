fn android_runtime_avatar_chat() -> &'static str {
    r#"@Composable
private fun DoweAvatar(source: String?, name: String?, alt: String, size: String, status: String?, bordered: Boolean, backgroundColor: Color, contentColor: Color, borderColor: Color?, onClick: (() -> Unit)?, hasIcon: Boolean, icon: @Composable () -> Unit) {
    val avatarSize = doweAvatarSize(size)
    val indicatorSize = doweAvatarIndicatorSize(size)
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = Modifier
            .width(avatarSize)
            .height(avatarSize)
            .clip(shape)
            .background(backgroundColor)
            .then(if (bordered) Modifier.border(3.dp, borderColor ?: contentColor, shape) else Modifier)
            .then(if (onClick == null) Modifier else Modifier.clickable(onClick = onClick)),
        contentAlignment = Alignment.Center
    ) {
        if (!source.isNullOrBlank()) {
            AndroidView(
                modifier = Modifier.matchParentSize(),
                factory = { context -> ImageView(context).apply { scaleType = ImageView.ScaleType.CENTER_CROP } },
                update = { view -> view.setImageURI(Uri.parse(source)) }
            )
        } else if (hasIcon) {
            CompositionLocalProvider(LocalContentColor provides contentColor) {
                icon()
            }
        } else {
            Text(text = doweAvatarInitial(name, alt), color = contentColor, fontWeight = FontWeight.SemiBold, fontSize = doweAvatarTextSize(size))
        }
        if (status != null) {
            Box(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .width(indicatorSize)
                    .height(indicatorSize)
                    .clip(shape)
                    .background(doweAvatarStatusColor(status))
                    .border(1.dp, DoweDesign.background, shape)
            )
        }
    }
}

private fun doweAvatarSize(size: String): Dp =
    when (size) {
        "xs" -> 24.dp
        "sm" -> 32.dp
        "lg" -> 48.dp
        "xl" -> 64.dp
        else -> 40.dp
    }

private fun doweAvatarIndicatorSize(size: String): Dp =
    when (size) {
        "xs" -> 6.dp
        "sm" -> 8.dp
        "lg" -> 12.dp
        "xl" -> 16.dp
        else -> 10.dp
    }

private fun doweAvatarTextSize(size: String): TextUnit =
    when (size) {
        "xs" -> 12.sp
        "sm" -> 14.sp
        "lg" -> 18.sp
        "xl" -> 24.sp
        else -> 16.sp
    }

private fun doweAvatarInitial(name: String?, alt: String): String =
    (name?.trim()?.takeIf { it.isNotEmpty() } ?: alt).take(1).uppercase()

private fun doweAvatarStatusColor(status: String): Color =
    when (status) {
        "online" -> DoweDesign.success
        "busy" -> DoweDesign.warning
        "away" -> DoweDesign.danger
        else -> DoweDesign.muted
    }

private data class DoweAvatarGroupItem(
    val source: String?,
    val name: String?,
    val alt: String?,
    val onClick: (() -> Unit)?
)

private fun doweAvatarGroupItems(rows: List<Map<String, Any?>>, fallback: List<DoweAvatarGroupItem>): List<DoweAvatarGroupItem> {
    if (rows.isEmpty()) {
        return fallback
    }
    return rows.map { row ->
        DoweAvatarGroupItem(
            source = row["src"]?.toString(),
            name = row["name"]?.toString(),
            alt = row["alt"]?.toString(),
            onClick = null
        )
    }
}

@Composable
private fun DoweAvatarGroup(items: List<DoweAvatarGroupItem>, size: String, maxCount: Int?, inline: Boolean, bordered: Boolean, backgroundColor: Color, contentColor: Color, borderColor: Color, modifier: Modifier) {
    val visibleLimit = maxCount?.coerceAtLeast(1)
    val visibleItems = if (visibleLimit != null && items.size > visibleLimit) items.take((visibleLimit - 1).coerceAtLeast(0)) else items
    val hiddenCount = if (visibleLimit != null && items.size > visibleLimit) items.size - visibleItems.size else 0
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(if (inline) 8.dp else (-12).dp)
    ) {
        visibleItems.forEach { item ->
            DoweAvatar(
                source = item.source,
                name = item.name,
                alt = item.alt ?: item.name ?: "",
                size = size,
                status = null,
                bordered = bordered,
                backgroundColor = backgroundColor,
                contentColor = contentColor,
                borderColor = borderColor,
                onClick = item.onClick,
                hasIcon = false
            ) {}
        }
        if (hiddenCount > 0) {
            val counterSize = doweAvatarGroupCounterSize(size)
            Box(
                modifier = Modifier
                    .width(counterSize)
                    .height(counterSize)
                    .clip(RoundedCornerShape(999.dp))
                    .background(backgroundColor)
                    .border(if (bordered) 3.dp else 1.dp, borderColor, RoundedCornerShape(999.dp)),
                contentAlignment = Alignment.Center
            ) {
                Text(text = "+$hiddenCount", color = contentColor, fontSize = doweAvatarGroupCounterTextSize(size), fontWeight = FontWeight.SemiBold, maxLines = 1)
            }
        }
    }
}

private fun doweAvatarGroupCounterSize(size: String): Dp =
    when (size) {
        "xs" -> 20.dp
        "sm" -> 24.dp
        "lg" -> 40.dp
        "xl" -> 56.dp
        else -> 28.dp
    }

private fun doweAvatarGroupCounterTextSize(size: String): TextUnit =
    when (size) {
        "xs", "sm" -> 10.sp
        "lg" -> 14.sp
        "xl" -> 18.sp
        else -> 12.sp
    }

private data class DoweChatMessage(
    val id: String,
    val role: String,
    val userId: String?,
    val name: String?,
    val avatar: String?,
    val text: String,
    val status: String?
)

private fun doweChatMessages(rows: List<Map<String, Any?>>): List<DoweChatMessage> =
    rows.mapIndexed { index, row ->
        DoweChatMessage(
            id = row["id"]?.toString() ?: index.toString(),
            role = row["role"]?.toString() ?: "assistant",
            userId = (row["userId"] ?: row["user_id"])?.toString(),
            name = row["name"]?.toString(),
            avatar = row["avatar"]?.toString(),
            text = (row["text"] ?: row["content"] ?: row["message"])?.toString() ?: "",
            status = row["status"]?.toString()
        )
    }

@Composable
private fun DoweChatBox(state: DoweReactiveState, messagesPath: String, mode: String, currentUserId: String, userName: String, userAvatar: String?, userStatus: String, assistantName: String, assistantAvatar: String?, showHeader: Boolean, placeholder: String, showAttachments: Boolean, showVoiceNote: Boolean, showCamera: Boolean, loading: Boolean, sending: Boolean, streaming: Boolean, hasMore: Boolean, onSend: ((String) -> Unit)?, onLoadMore: (() -> Unit)?, onStop: (() -> Unit)?, onVoiceNote: (() -> Unit)?, onFileAttach: (() -> Unit)?, onCameraCapture: (() -> Unit)?, backgroundColor: Color, contentColor: Color, borderColor: Color?, modifier: Modifier) {
    var draft by remember { mutableStateOf("") }
    val messages = doweChatMessages(state.rows(messagesPath).map { it.value })
    val shape = RoundedCornerShape(DoweDesign.radiusBox)
    Column(
        modifier = modifier
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            .padding(12.dp)
    ) {
        if (showHeader) {
            Row(modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                DoweAvatar(source = assistantAvatar, name = assistantName, alt = assistantName, size = "sm", status = userStatus, bordered = false, backgroundColor = contentColor.copy(alpha = 0.08f), contentColor = contentColor, borderColor = contentColor, onClick = null, hasIcon = false) {}
                Column(modifier = Modifier.weight(1f)) {
                    Text(text = if (mode == "prompt") assistantName else userName, color = contentColor, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                    Text(text = userStatus, color = contentColor.copy(alpha = 0.64f), fontSize = 12.sp)
                }
                Text(text = "Search", color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.Medium)
                Text(text = "...", color = contentColor.copy(alpha = 0.72f), fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
        }
        if (hasMore && onLoadMore != null) {
            Text(text = "Load more", modifier = Modifier.align(Alignment.CenterHorizontally).clickable(onClick = onLoadMore).padding(vertical = 6.dp), color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        }
        Column(modifier = Modifier.fillMaxWidth().weight(1f, fill = false), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            messages.forEach { message ->
                val own = message.userId == currentUserId || message.role == "user"
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = if (own) Arrangement.End else Arrangement.Start) {
                    Column(horizontalAlignment = if (own) Alignment.End else Alignment.Start) {
                        Text(
                            text = message.text,
                            modifier = Modifier
                                .widthIn(max = 280.dp)
                                .clip(RoundedCornerShape(16.dp))
                                .background(if (own) contentColor else contentColor.copy(alpha = 0.08f))
                                .padding(horizontal = 12.dp, vertical = 9.dp),
                            color = if (own) backgroundColor else contentColor,
                            fontSize = 14.sp,
                            lineHeight = 20.sp
                        )
                        if (!message.status.isNullOrEmpty()) {
                            Text(text = message.status, color = contentColor.copy(alpha = 0.52f), fontSize = 11.sp, modifier = Modifier.padding(top = 3.dp))
                        }
                    }
                }
            }
            if (loading || streaming) {
                Text(text = if (streaming) "..." else "Typing...", color = contentColor.copy(alpha = 0.64f), fontSize = 13.sp, modifier = Modifier.padding(horizontal = 8.dp, vertical = 6.dp))
            }
        }
        Row(modifier = Modifier.fillMaxWidth().padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (showVoiceNote && onVoiceNote != null) {
                Text(text = "Mic", modifier = Modifier.clickable(onClick = onVoiceNote), color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
            if (showAttachments && onFileAttach != null) {
                Text(text = "+", modifier = Modifier.clickable(onClick = onFileAttach), color = contentColor.copy(alpha = 0.72f), fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
            if (showCamera && onCameraCapture != null) {
                Text(text = "Cam", modifier = Modifier.clickable(onClick = onCameraCapture), color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
            BasicTextField(
                value = draft,
                onValueChange = { draft = it },
                textStyle = TextStyle(color = contentColor, fontSize = 14.sp),
                modifier = Modifier
                    .weight(1f)
                    .heightIn(min = 40.dp)
                    .clip(RoundedCornerShape(20.dp))
                    .background(contentColor.copy(alpha = 0.08f))
                    .padding(horizontal = 14.dp, vertical = 10.dp),
                decorationBox = { inner ->
                    if (draft.isEmpty()) {
                        Text(text = placeholder, color = contentColor.copy(alpha = 0.48f), fontSize = 14.sp)
                    }
                    inner()
                }
            )
            val canSend = draft.trim().isNotEmpty() && onSend != null && !sending
            Text(
                text = if (streaming && onStop != null) "Stop" else "Send",
                modifier = Modifier
                    .clip(RoundedCornerShape(18.dp))
                    .background(if (canSend || streaming) contentColor else contentColor.copy(alpha = 0.16f))
                    .clickable {
                        if (streaming && onStop != null) {
                            onStop()
                        } else if (canSend) {
                            onSend?.invoke(draft)
                            draft = ""
                        }
                    }
                    .padding(horizontal = 12.dp, vertical = 8.dp),
                color = if (canSend || streaming) backgroundColor else contentColor.copy(alpha = 0.48f),
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}

"#
}
