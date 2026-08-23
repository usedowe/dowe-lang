fn android_runtime_media_video() -> &'static str {
    r##"    r##"private data class DoweVideoIcon(val viewBox: DoweSvgViewBox, val paths: List<DoweSvgPath>)

private data class DoweVideoIcons(val play: DoweVideoIcon, val pause: DoweVideoIcon, val volume: DoweVideoIcon, val muted: DoweVideoIcon, val pictureInPicture: DoweVideoIcon, val fullscreen: DoweVideoIcon)

@Composable
private fun DoweVideo(source: String, poster: String?, autoplay: Boolean, aspect: String, icons: DoweVideoIcons, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    var video by remember(source) { mutableStateOf<VideoView?>(null) }
    var player by remember(source) { mutableStateOf<MediaPlayer?>(null) }
    var started by remember(source) { mutableStateOf(autoplay) }
    var playing by remember(source) { mutableStateOf(autoplay) }
    var muted by remember(source) { mutableStateOf(false) }
    var currentTime by remember(source) { mutableStateOf(0f) }
    var duration by remember(source) { mutableStateOf(0f) }
    var mediaAspect by remember(source) { mutableStateOf(16f / 9f) }
    var fullscreen by remember(source) { mutableStateOf(false) }
    var posterBitmap by remember(poster) { mutableStateOf<android.graphics.Bitmap?>(null) }
    LaunchedEffect(poster) {
        posterBitmap = if (poster == null) null else withContext(Dispatchers.IO) { doweLoadImageBitmap(context, poster) }
    }
    LaunchedEffect(playing, source) {
        while (playing) {
            currentTime = (video?.currentPosition ?: 0) / 1000f
            duration = (video?.duration ?: 0).coerceAtLeast(0) / 1000f
            delay(250)
        }
    }
    val frame: @Composable (Modifier, Boolean) -> Unit = { frameModifier, expanded ->
        val frameShape = if (expanded) RoundedCornerShape(0.dp) else shape
        BoxWithConstraints(modifier = frameModifier.then(if (expanded) Modifier.fillMaxSize() else Modifier.aspectRatio(doweVideoAspect(aspect))).clip(frameShape).background(if (expanded) Color.Black else backgroundColor).then(if (expanded || borderColor == null) Modifier else Modifier.border(1.dp, borderColor, frameShape)), contentAlignment = Alignment.Center) {
        val frameAspect = if (maxHeight.value > 0f) maxWidth.value / maxHeight.value else doweVideoAspect(aspect)
        val videoModifier = if (mediaAspect >= frameAspect) Modifier.fillMaxWidth().aspectRatio(mediaAspect) else Modifier.fillMaxHeight().aspectRatio(mediaAspect)
        Box(modifier = Modifier.matchParentSize().background(Color.Black))
        AndroidView(
            modifier = videoModifier,
            factory = { context ->
                VideoView(context).apply {
                    setMediaController(null)
                    tag = source
                    setVideoURI(Uri.parse(source))
                    setOnPreparedListener { mediaPlayer ->
                        video?.takeIf { it !== this }?.pause()
                        video = this
                        player = mediaPlayer
                        mediaAspect = if (mediaPlayer.videoWidth > 0 && mediaPlayer.videoHeight > 0) mediaPlayer.videoWidth.toFloat() / mediaPlayer.videoHeight.toFloat() else 16f / 9f
                        duration = mediaPlayer.duration.coerceAtLeast(0) / 1000f
                        mediaPlayer.setVolume(if (muted) 0f else 1f, if (muted) 0f else 1f)
                        if (currentTime > 0f) seekTo((currentTime * 1000f).toInt())
                        if (playing) {
                            started = true
                            start()
                            playing = true
                        }
                    }
                    setOnCompletionListener {
                        playing = false
                        currentTime = duration
                    }
                }
            },
            update = { view ->
                if (view.tag != source) {
                    view.tag = source
                    view.setVideoURI(Uri.parse(source))
                }
            }
        )
        posterBitmap?.takeIf { !started }?.let { image ->
            Image(
                bitmap = image.asImageBitmap(),
                contentDescription = null,
                modifier = Modifier.matchParentSize().clickable {
                    started = true
                    playing = true
                    video?.start()
                },
                contentScale = ContentScale.Crop
            )
        }
        Box(modifier = Modifier.matchParentSize().clickable {
            if (playing) {
                video?.pause()
                playing = false
            } else {
                started = true
                playing = true
                video?.start()
            }
        })
        DoweVideoControls(
            playing = playing,
            muted = muted,
            currentTime = currentTime,
            duration = duration,
            icons = icons,
            onPlayPause = {
                if (playing) video?.pause() else {
                    started = true
                    video?.start()
                }
                playing = !playing
            },
            onMute = {
                muted = !muted
                val volume = if (muted) 0f else 1f
                player?.setVolume(volume, volume)
            },
            onSeek = { value ->
                currentTime = value
                video?.seekTo((value * 1000f).toInt())
                if (value > 0f) started = true
            },
            onPictureInPicture = {
                started = true
                playing = true
                video?.pause()
                doweEnterVideoPictureInPicture(context, source, currentTime, muted, mediaAspect) { position, resume ->
                    currentTime = position / 1000f
                    video?.seekTo(position)
                    playing = resume
                    if (resume) video?.start()
                }
            },
            onFullscreen = {
                currentTime = (video?.currentPosition ?: 0) / 1000f
                fullscreen = !fullscreen
            },
            modifier = Modifier.align(Alignment.BottomCenter)
        )
    }
    }
    if (fullscreen) {
        Dialog(onDismissRequest = { fullscreen = false }, properties = DialogProperties(usePlatformDefaultWidth = false, decorFitsSystemWindows = false)) {
            frame(Modifier.fillMaxSize(), true)
        }
    } else {
        frame(modifier, false)
    }
}

@Composable
private fun DoweVideoControls(playing: Boolean, muted: Boolean, currentTime: Float, duration: Float, icons: DoweVideoIcons, onPlayPause: () -> Unit, onMute: () -> Unit, onSeek: (Float) -> Unit, onPictureInPicture: () -> Unit, onFullscreen: () -> Unit, modifier: Modifier) {
    Column(modifier = modifier.fillMaxWidth().background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = 0.78f)))).padding(horizontal = 10.dp, vertical = 8.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            DoweVideoControlButton(icon = if (playing) icons.pause else icons.play, label = if (playing) "Pause video" else "Play video", onClick = onPlayPause)
            Text("${doweVideoTime(currentTime)} / ${doweVideoTime(duration)}", color = Color.White, fontSize = 12.sp)
            Spacer(modifier = Modifier.weight(1f))
            DoweVideoControlButton(icon = if (muted) icons.muted else icons.volume, label = if (muted) "Unmute video" else "Mute video", onClick = onMute)
            DoweVideoControlButton(icon = icons.pictureInPicture, label = "Picture in picture", onClick = onPictureInPicture)
            DoweVideoControlButton(icon = icons.fullscreen, label = "Toggle fullscreen", onClick = onFullscreen)
        }
        Slider(
            value = currentTime.coerceIn(0f, duration.coerceAtLeast(0.01f)),
            onValueChange = onSeek,
            valueRange = 0f..duration.coerceAtLeast(0.01f),
            colors = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = Color.White, inactiveTrackColor = Color.White.copy(alpha = 0.4f)),
            modifier = Modifier.fillMaxWidth().height(20.dp)
        )
    }
}

@Composable
private fun DoweVideoControlButton(icon: DoweVideoIcon, label: String, onClick: () -> Unit) {
    Box(modifier = Modifier.size(32.dp).clip(RoundedCornerShape(999.dp)).background(Color.Black.copy(alpha = 0.48f)).clickable(onClick = onClick).semantics { contentDescription = label }, contentAlignment = Alignment.Center) {
        DoweSvg(viewBox = icon.viewBox, modifier = Modifier.size(20.dp), color = Color.White, paths = icon.paths)
    }
}

private fun doweVideoTime(value: Float): String {
    val seconds = value.coerceAtLeast(0f).toInt()
    return "${seconds / 60}:${(seconds % 60).toString().padStart(2, '0')}"
}

private var doweVideoPictureInPictureOverlay: FrameLayout? = null
private var doweVideoPictureInPictureExit: (() -> Unit)? = null

internal fun doweHandleVideoPictureInPictureMode(active: Boolean) {
    if (!active && doweVideoPictureInPictureOverlay != null) doweVideoPictureInPictureExit?.invoke()
}

private fun doweEnterVideoPictureInPicture(context: android.content.Context, source: String, currentTime: Float, muted: Boolean, mediaAspect: Float, onExit: (Int, Boolean) -> Unit) {
    var current = context
    while (current is ContextWrapper && current !is Activity) current = current.baseContext
    val activity = current as? Activity ?: return
    doweVideoPictureInPictureExit?.invoke()
    val decor = activity.window.decorView as? ViewGroup ?: return
    val overlay = FrameLayout(activity).apply { setBackgroundColor(AndroidColor.BLACK) }
    val pictureInPictureVideo = VideoView(activity).apply {
        setMediaController(null)
    }
    overlay.addView(pictureInPictureVideo, FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT))
    decor.addView(overlay, ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT))
    doweVideoPictureInPictureOverlay = overlay
    doweVideoPictureInPictureExit = {
        val position = pictureInPictureVideo.currentPosition.coerceAtLeast(0)
        val resume = pictureInPictureVideo.isPlaying
        pictureInPictureVideo.stopPlayback()
        (overlay.parent as? ViewGroup)?.removeView(overlay)
        doweVideoPictureInPictureOverlay = null
        doweVideoPictureInPictureExit = null
        onExit(position, resume)
    }
    val width = (mediaAspect.coerceAtLeast(0.1f) * 1000f).toInt().coerceAtLeast(1)
    pictureInPictureVideo.setOnPreparedListener { player ->
        player.setVolume(if (muted) 0f else 1f, if (muted) 0f else 1f)
        pictureInPictureVideo.seekTo((currentTime * 1000f).toInt())
        pictureInPictureVideo.start()
        if (!activity.enterPictureInPictureMode(PictureInPictureParams.Builder().setAspectRatio(Rational(width, 1000)).build())) {
            doweHandleVideoPictureInPictureMode(false)
        }
    }
    pictureInPictureVideo.setVideoURI(Uri.parse(source))
}

"##
}
