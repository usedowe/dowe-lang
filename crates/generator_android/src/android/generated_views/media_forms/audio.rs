fn android_runtime_media_audio() -> &'static str {
    r##"private val doweAudioWaveform = listOf(
    0.48f, 0.62f, 0.38f, 0.54f, 0.76f, 0.44f, 0.30f, 0.52f, 0.68f, 0.84f,
    0.58f, 0.42f, 0.65f, 0.92f, 0.72f, 0.49f, 0.35f, 0.61f, 0.80f, 0.55f,
    0.41f, 0.71f, 0.96f, 0.64f, 0.46f, 0.32f, 0.57f, 0.75f, 0.88f, 0.60f,
    0.37f, 0.51f, 0.69f, 0.83f, 0.47f, 0.29f, 0.55f, 0.73f, 0.63f, 0.40f,
    0.67f, 0.89f, 0.58f, 0.34f, 0.50f, 0.77f, 0.68f, 0.43f, 0.60f, 0.82f
)

@Composable
private fun DoweAudio(source: String, subtitle: String?, avatarSource: String?, playIconViewBox: DoweSvgViewBox, playIconPaths: List<DoweSvgPath>, pauseIconViewBox: DoweSvgViewBox, pauseIconPaths: List<DoweSvgPath>, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, buttonBackgroundColor: Color, buttonContentColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    var player by remember(source) { mutableStateOf<MediaPlayer?>(null) }
    var playing by remember(source) { mutableStateOf(false) }
    var currentTime by remember(source) { mutableStateOf(0f) }
    var duration by remember(source) { mutableStateOf(0f) }
    var prepared by remember(source) { mutableStateOf(false) }
    LaunchedEffect(source) {
        val created = MediaPlayer()
        player = created
        runCatching {
            created.setDataSource(context, doweAudioUri(context, source))
            created.setOnPreparedListener { mediaPlayer ->
                duration = mediaPlayer.duration.coerceAtLeast(0) / 1000f
                prepared = true
            }
            created.setOnCompletionListener {
                playing = false
                currentTime = duration
            }
            created.setOnErrorListener { _, _, _ ->
                playing = false
                prepared = false
                true
            }
            created.prepareAsync()
        }.onFailure {
            prepared = false
            player = null
            created.release()
        }
    }
    DisposableEffect(source) {
        onDispose {
            player?.release()
            player = null
            prepared = false
            playing = false
        }
    }
    LaunchedEffect(playing, player) {
        while (playing) {
            val active = player
            if (active == null || !prepared) {
                playing = false
                break
            }
            currentTime = (active.currentPosition.coerceAtLeast(0) / 1000f).coerceAtMost(duration)
            delay(250)
        }
    }
    fun seek(value: Float) {
        val next = value.coerceIn(0f, duration.coerceAtLeast(0f))
        currentTime = next
        player?.seekTo((next * 1000f).toInt())
    }
    val waveformModifier = Modifier
        .fillMaxWidth()
        .height(32.dp)
        .padding(top = 12.dp)
        .pointerInput(duration) {
            awaitEachGesture {
                val down = awaitFirstDown()
                fun seekAt(x: Float) {
                    if (size.width > 0) seek((x / size.width.toFloat()).coerceIn(0f, 1f) * duration)
                }
                seekAt(down.position.x)
                while (true) {
                    val event = awaitPointerEvent()
                    val change = event.changes.firstOrNull() ?: break
                    seekAt(change.position.x)
                    if (change.changedToUpIgnoreConsumed()) break
                }
            }
        }
        .focusable()
        .semantics { contentDescription = "Audio progress" }
        .onPreviewKeyEvent { event ->
            if (event.type != KeyEventType.KeyDown || duration <= 0f) return@onPreviewKeyEvent false
            val step = minOf(5f, duration / 20f)
            val next = when (event.key) {
                Key.DirectionLeft -> currentTime - step
                Key.DirectionRight -> currentTime + step
                Key.MoveHome -> 0f
                Key.MoveEnd -> duration
                else -> return@onPreviewKeyEvent false
            }
            seek(next)
            true
        }
    Row(
        modifier = modifier
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            .padding(horizontal = 12.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Button(
            onClick = {
                if (prepared) {
                    if (playing) {
                        player?.pause()
                        playing = false
                    } else {
                        player?.start()
                        playing = true
                    }
                }
            },
            modifier = Modifier.size(40.dp).semantics { contentDescription = if (playing) "Pause audio" else "Play audio" },
            shape = RoundedCornerShape(999.dp),
            colors = ButtonDefaults.buttonColors(containerColor = buttonBackgroundColor, contentColor = buttonContentColor),
            contentPadding = PaddingValues(0.dp)
        ) {
            DoweSvg(
                viewBox = if (playing) pauseIconViewBox else playIconViewBox,
                modifier = Modifier.size(20.dp),
                color = buttonContentColor,
                paths = if (playing) pauseIconPaths else playIconPaths
            )
        }
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Row(modifier = waveformModifier, horizontalArrangement = Arrangement.spacedBy(2.dp), verticalAlignment = Alignment.CenterVertically) {
                val progress = if (duration > 0f) currentTime / duration else 0f
                repeat(50) { index ->
                    val active = (index + 0.5f) / 50f <= progress
                    val opacity by animateFloatAsState(
                        targetValue = if (active) 1f else 0.3f,
                        animationSpec = tween(durationMillis = 300),
                        label = "dowe-audio-bar-$index"
                    )
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .height((doweAudioWaveform[index] * 20f).dp)
                            .background(contentColor.copy(alpha = opacity), RoundedCornerShape(2.dp))
                    )
                }
            }
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text(text = doweAudioTime((duration - currentTime).coerceAtLeast(0f)), color = contentColor, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                subtitle?.let { Text(text = it, color = contentColor.copy(alpha = 0.72f), fontSize = 12.sp, maxLines = 1, modifier = Modifier.weight(1f).padding(start = 12.dp)) }
            }
        }
        if (avatarSource != null) {
            DoweCoverBox(modifier = Modifier.size(48.dp).clip(RoundedCornerShape(999.dp)), source = avatarSource, overlay = null) {}
        }
    }
}

private fun doweAudioUri(context: android.content.Context, source: String): Uri {
    if (source.startsWith("https://") || source.startsWith("http://") || source.startsWith("content:") || source.startsWith("file:")) return Uri.parse(source)
    val path = source.trimStart('/').removePrefix("assets/")
    return Uri.parse("file:///android_asset/$path")
}

private fun doweAudioTime(value: Float): String {
    val seconds = value.coerceAtLeast(0f).toInt()
    return "${seconds / 60}:${(seconds % 60).toString().padStart(2, '0')}"
}

"##
}
