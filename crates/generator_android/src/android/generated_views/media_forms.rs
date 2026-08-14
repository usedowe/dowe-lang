fn android_runtime_media_forms() -> &'static str {
    r##"private data class DoweVideoIcon(val viewBox: DoweSvgViewBox, val paths: List<DoweSvgPath>)

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

private data class DoweDeviceIcon(val profile: String, val viewBox: DoweSvgViewBox, val paths: List<DoweSvgPath>)

@Composable
private fun DoweDevicePreview(initialProfile: String, source: String, title: String, sandbox: List<String>?, autoplay: Boolean, icons: List<DoweDeviceIcon>, modifier: Modifier) {
    var profile by remember { mutableStateOf(initialProfile) }
    val dimensions = when (profile) {
        "tablet" -> 768f to 1024f
        "laptop" -> 1440f to 900f
        "monitor" -> 1920f to 1080f
        else -> 390f to 844f
    }
    Column(modifier = modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
        Row(modifier = Modifier.padding(4.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            icons.forEach { option ->
                DoweDeviceIconButton(icon = option, selected = profile == option.profile, onClick = { profile = option.profile })
            }
        }
        BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
            val zoom = minOf(1f, maxWidth.value / dimensions.first)
            Box(modifier = Modifier.fillMaxWidth().height((dimensions.second * zoom).dp), contentAlignment = Alignment.TopCenter) {
                DoweIframe(source = source, title = title, sandbox = sandbox, autoplay = autoplay, modifier = Modifier.size(dimensions.first.dp, dimensions.second.dp).graphicsLayer { scaleX = zoom; scaleY = zoom; transformOrigin = TransformOrigin(0.5f, 0f) }, shape = RoundedCornerShape(0.dp))
            }
        }
    }
}

@Composable
private fun DoweDeviceIconButton(icon: DoweDeviceIcon, selected: Boolean, onClick: () -> Unit) {
    Button(
        onClick = onClick,
        modifier = Modifier.size(40.dp).semantics { contentDescription = icon.profile },
        shape = RoundedCornerShape(DoweDesign.radius),
        colors = ButtonDefaults.buttonColors(
            containerColor = if (selected) DoweDesign.softPrimary else Color.Transparent,
            contentColor = if (selected) DoweDesign.primary else DoweDesign.backgroundText
        ),
        border = BorderStroke(1.dp, if (selected) DoweDesign.primary else DoweDesign.backgroundText),
        contentPadding = PaddingValues(0.dp)
    ) {
        DoweSvg(viewBox = icon.viewBox, modifier = Modifier.size(24.dp), color = LocalContentColor.current, paths = icon.paths)
    }
}

@Composable
private fun DoweIframe(source: String, title: String, sandbox: List<String>?, autoplay: Boolean, modifier: Modifier, shape: RoundedCornerShape) {
    val context = LocalContext.current
    val resolvedSource = doweIframeSource(context, source)
    AndroidView(
        modifier = modifier.fillMaxWidth().defaultMinSize(minHeight = 192.dp).clip(shape),
        factory = { context ->
            WebView(context).apply {
                contentDescription = title
                settings.javaScriptEnabled = sandbox == null || sandbox.contains("scripts")
                settings.domStorageEnabled = true
                settings.allowFileAccess = false
                settings.allowContentAccess = false
                settings.mediaPlaybackRequiresUserGesture = !autoplay
                settings.setSupportMultipleWindows(false)
                webViewClient = object : WebViewClient() {
                    override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
                        return !doweIframeUrlAllowed(request.url)
                    }
                }
                tag = resolvedSource
                if (resolvedSource != null) {
                    loadUrl(resolvedSource)
                }
            }
        },
        update = { view ->
            view.contentDescription = title
            if (resolvedSource != null && view.tag != resolvedSource) {
                view.tag = resolvedSource
                view.loadUrl(resolvedSource)
            }
        }
    )
}

private fun doweIframeSource(context: android.content.Context, source: String): String? {
    if (source.startsWith("https://")) return source
    if (!source.startsWith("/") || source.startsWith("//")) return null
    val configured = DoweEnvironment.BACKEND_URL.trimEnd('/')
    val development = context.getSharedPreferences("dowe-hmr", android.content.Context.MODE_PRIVATE).getString("endpoint", "").orEmpty().trimEnd('/')
    val base = listOf(development, configured).firstOrNull { value ->
        runCatching { doweIframeUrlAllowed(Uri.parse(value)) }.getOrDefault(false)
    } ?: return null
    return java.net.URI(base).resolve(source).toString()
}

private fun doweIframeUrlAllowed(url: Uri): Boolean {
    if (url.scheme == "https") return true
    return url.scheme == "http" && (url.host == "localhost" || url.host == "127.0.0.1" || url.host == "::1")
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
private fun DoweImage(source: String, alt: String, aspect: String, objectFit: String, loading: String, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    var bitmap by remember(source) { mutableStateOf<android.graphics.Bitmap?>(null) }
    val imageOpacity by animateFloatAsState(
        targetValue = if (bitmap == null) 0f else 1f,
        animationSpec = tween(durationMillis = 180),
        label = "dowe-image-opacity"
    )
    LaunchedEffect(source, loading) {
        bitmap = withContext(Dispatchers.IO) { doweLoadImageBitmap(context, source) }
    }
    Box(
        modifier = modifier
            .aspectRatio(doweImageAspect(aspect))
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
    ) {
        if (bitmap == null || imageOpacity < 1f) {
            Box(modifier = Modifier.matchParentSize().background(DoweDesign.surface))
        }
        bitmap?.let { image ->
            Image(
                bitmap = image.asImageBitmap(),
                contentDescription = alt.takeIf { it.isNotEmpty() },
                modifier = Modifier.matchParentSize().graphicsLayer { alpha = imageOpacity },
                contentScale = doweImageContentScale(objectFit)
            )
        }
    }
}

private const val DOWE_IMAGE_MEMORY_CACHE_BYTES = 24 * 1024 * 1024
private const val DOWE_IMAGE_DISK_CACHE_BYTES = 64L * 1024L * 1024L
private val doweImageMemoryCache = object : LruCache<String, android.graphics.Bitmap>(DOWE_IMAGE_MEMORY_CACHE_BYTES) {
    override fun sizeOf(key: String, value: android.graphics.Bitmap): Int = value.allocationByteCount
}
private val doweImageLoadLocks = ConcurrentHashMap<String, Mutex>()

private suspend fun doweLoadImageBitmap(context: android.content.Context, source: String): android.graphics.Bitmap? {
    doweImageMemoryCache.get(source)?.let { return it }
    val lock = doweImageLoadLocks.getOrPut(source) { Mutex() }
    return try {
        lock.withLock {
            doweImageMemoryCache.get(source) ?: doweReadImageBitmap(context, source)?.also {
                doweImageMemoryCache.put(source, it)
            }
        }
    } finally {
        doweImageLoadLocks.remove(source, lock)
    }
}

private fun doweReadImageBitmap(context: android.content.Context, source: String): android.graphics.Bitmap? {
    return try {
        if (!source.startsWith("https://") && !source.startsWith("http://")) {
            val assetPath = source.trimStart('/').removePrefix("assets/")
            return context.assets.open(assetPath).use(BitmapFactory::decodeStream)
        }
        val directory = File(context.cacheDir, "dowe-images").apply { mkdirs() }
        val cached = File(directory, doweImageCacheKey(source))
        if (cached.isFile) {
            BitmapFactory.decodeFile(cached.absolutePath)?.let {
                cached.setLastModified(System.currentTimeMillis())
                return it
            }
            cached.delete()
        }
        val temporary = File(directory, "${cached.name}.tmp")
        val connection = URL(source).openConnection() as HttpURLConnection
        connection.connectTimeout = 10_000
        connection.readTimeout = 10_000
        connection.useCaches = true
        connection.instanceFollowRedirects = true
        connection.setRequestProperty("User-Agent", "Dowe/1.0")
        connection.setRequestProperty("Accept", "image/*")
        try {
            connection.inputStream.use { input ->
                FileOutputStream(temporary).use { output -> input.copyTo(output) }
            }
            if (!temporary.renameTo(cached)) {
                temporary.delete()
            }
            BitmapFactory.decodeFile(cached.absolutePath)
        } finally {
            connection.disconnect()
            temporary.delete()
            doweTrimImageDiskCache(directory)
        }
    } catch (error: Exception) {
        null
    }
}

private fun doweImageCacheKey(source: String): String {
    return MessageDigest.getInstance("SHA-256")
        .digest(source.toByteArray(Charsets.UTF_8))
        .joinToString("") { byte -> "%02x".format(byte) }
}

private fun doweTrimImageDiskCache(directory: File) {
    var total = directory.listFiles()?.sumOf { it.length() } ?: 0L
    directory.listFiles()?.sortedBy { it.lastModified() }?.forEach { file ->
        val size = file.length()
        if (total > DOWE_IMAGE_DISK_CACHE_BYTES && file.delete()) {
            total -= size
        }
    }
}

private fun doweImageContentScale(objectFit: String): ContentScale {
    return when (objectFit) {
        "contain" -> ContentScale.Fit
        "fill" -> ContentScale.FillBounds
        "none" -> ContentScale.None
        else -> ContentScale.Crop
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
private fun DoweAccordion(multiple: Boolean, defaultOpenIds: Set<String>, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp, content: @Composable (Set<String>, (String) -> Unit) -> Unit) {
    var openIds by remember(multiple, defaultOpenIds) { mutableStateOf(defaultOpenIds) }
    val toggleItem: (String) -> Unit = { id ->
        openIds = if (id in openIds) {
            openIds - id
        } else if (multiple) {
            openIds + id
        } else {
            setOf(id)
        }
    }
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(radius))
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(radius)))
            .padding(4.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            content(openIds, toggleItem)
        }
    }
}

@Composable
private fun DoweAccordionItem(label: String, disabled: Boolean, open: Boolean, radius: Dp, onToggle: () -> Unit, arrowIcon: @Composable () -> Unit, content: @Composable () -> Unit) {
    val itemShape = RoundedCornerShape(radius * 0.85f)
    Column(modifier = Modifier.fillMaxWidth().clip(itemShape).border(1.dp, LocalContentColor.current.copy(alpha = 0.12f), itemShape).alpha(if (disabled) 0.5f else 1f)) {
        Row(
            modifier = Modifier.fillMaxWidth().clickable(enabled = !disabled, onClick = onToggle).padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(label, fontSize = 14.sp, lineHeight = 20.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.weight(1f))
            Box(modifier = Modifier.size(20.dp).graphicsLayer { rotationZ = if (open) 90f else 0f }, contentAlignment = Alignment.Center) {
                arrowIcon()
            }
        }
        AnimatedVisibility(visible = open, enter = fadeIn(tween(160)) + expandVertically(tween(160)), exit = fadeOut(tween(160)) + shrinkVertically(tween(160))) {
            Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                content()
            }
        }
    }
}

private data class DoweCarouselSlideSpec(val id: String, val content: @Composable () -> Unit)

@Composable
private fun DoweCarousel(variant: String, slides: List<DoweCarouselSlideSpec>, autoplay: Boolean, autoplayInterval: Int, disableLoop: Boolean, hideControls: Boolean, hideIndicators: Boolean, showNavigation: Boolean, showCounter: Boolean, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, modifier: Modifier, accentColor: Color) {
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val slideCount = slides.size
    val currentIndex by remember {
        derivedStateOf {
            val layout = listState.layoutInfo
            val center = (layout.viewportStartOffset + layout.viewportEndOffset) / 2
            layout.visibleItemsInfo.minByOrNull { item -> kotlin.math.abs(item.offset + item.size / 2 - center) }?.index ?: 0
        }
    }
    val moveTo: (Int) -> Unit = { requested ->
        val target = when {
            requested < 0 && !disableLoop -> slideCount - 1
            requested >= slideCount && !disableLoop -> 0
            else -> min(slideCount - 1, max(0, requested))
        }
        if (slideCount > 0) scope.launch { listState.animateScrollToItem(target) }
    }
    LaunchedEffect(autoplay, autoplayInterval, currentIndex, slideCount) {
        if (autoplay && slideCount > 1) {
            delay(max(500, autoplayInterval).toLong())
            moveTo(currentIndex + 1)
        }
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        if (title != null) Text(title, fontWeight = FontWeight.Bold, color = accentColor)
        BoxWithConstraints(modifier = Modifier.fillMaxWidth().clipToBounds()) {
            val viewportWidth = maxWidth
            val resolvedWidth = when {
                slideWidth != null -> slideWidth.dp
                slidesPerView > 1 -> (viewportWidth - gap.dp * (slidesPerView - 1)) / slidesPerView
                variant == "simple" || variant == "masonry" || variant == "rtl" || variant == "sticky" -> minOf(280.dp, viewportWidth * 0.84f)
                else -> viewportWidth
            }
            val shouldSnap = variant !in listOf("simple", "masonry", "rtl", "sticky")
            val snapBehavior = rememberSnapFlingBehavior(lazyListState = listState)
            val freeBehavior = ScrollableDefaults.flingBehavior()
            if (orientation == "vertical") {
                LazyColumn(
                    modifier = Modifier.fillMaxWidth().heightIn(max = 560.dp),
                    state = listState,
                    verticalArrangement = Arrangement.spacedBy(gap.dp),
                    flingBehavior = if (shouldSnap) snapBehavior else freeBehavior
                ) {
                    itemsIndexed(slides, key = { _, slide -> slide.id }) { index, slide ->
                        DoweCarouselSlide(variant = variant, index = index, slideWidth = viewportWidth, slideHeight = slideHeight) { slide.content() }
                    }
                }
            } else {
                LazyRow(
                    modifier = Modifier.fillMaxWidth(),
                    state = listState,
                    reverseLayout = variant == "rtl",
                    horizontalArrangement = Arrangement.spacedBy(gap.dp),
                    flingBehavior = if (shouldSnap) snapBehavior else freeBehavior
                ) {
                    itemsIndexed(slides, key = { _, slide -> slide.id }) { index, slide ->
                        DoweCarouselSlide(variant = variant, index = index, slideWidth = resolvedWidth, slideHeight = slideHeight) { slide.content() }
                    }
                }
            }
        }
        if (!hideControls || showNavigation || variant == "controls") {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Button(onClick = { moveTo(currentIndex - 1) }) { Text("Previous") }
                Button(onClick = { moveTo(currentIndex + 1) }) { Text("Next") }
            }
        }
        if (!hideIndicators || variant == "dots" || variant == "thumbnails") {
            Row(modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                repeat(slideCount) { index ->
                    Button(onClick = { moveTo(index) }) { Text(if (variant == "thumbnails") "Slide ${index + 1}" else if (indicatorType == "dot" || variant == "dots") "•" else "${index + 1}") }
                }
            }
        }
        if (showCounter) Text("${currentIndex + 1} / $slideCount", color = accentColor)
    }
}

@Composable
private fun DoweCarouselSlide(variant: String, index: Int, slideWidth: Dp, slideHeight: Int?, content: @Composable () -> Unit) {
    val effect = when (variant) {
        "coverFlow" -> Modifier.graphicsLayer { rotationY = if (index % 2 == 0) -8f else 8f; scaleX = 0.96f; scaleY = 0.96f }
        "stories" -> Modifier.graphicsLayer { rotationY = if (index % 2 == 0) -14f else 14f; cameraDistance = 24f * density }
        "smartStack" -> Modifier.graphicsLayer { rotationZ = (index % 3 - 1) * 0.8f; translationY = (index % 3) * 4f }
        "cardStack" -> Modifier.graphicsLayer { scaleX = 1f - (index % 3) * 0.012f; scaleY = scaleX; translationY = (index % 3) * 4f }
        "flipbook" -> Modifier.graphicsLayer { rotationY = if (index % 2 == 0) -18f else 18f; cameraDistance = 24f * density }
        "slideshow" -> Modifier.graphicsLayer { translationX = if (index % 2 == 0) -6f else 6f }
        else -> Modifier
    }
    Box(modifier = Modifier.width(slideWidth).then(if (slideHeight == null) Modifier else Modifier.height(slideHeight.dp)).then(effect)) { content() }
}

@Composable
private fun DoweCheckbox(checked: Boolean, onCheckedChange: (Boolean) -> Unit, enabled: Boolean, label: String?, name: String?, modifier: Modifier, accentColor: Color) {
    Row(modifier = modifier.clickable(enabled = enabled) { onCheckedChange(!checked) }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Box(
            modifier = Modifier
                .width(20.dp)
                .height(20.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(if (checked) accentColor else Color.Transparent)
                .border(2.dp, if (checked) accentColor else accentColor.copy(alpha = 0.72f), RoundedCornerShape(4.dp))
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
}

@Composable
private fun DoweColorField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, fontSize: TextUnit, lineHeight: TextUnit, name: String?, helpText: String?, errorText: String?, showHex: Boolean, showRgb: Boolean, showCmyk: Boolean, showOklch: Boolean, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var expanded by remember { mutableStateOf(false) }
    var hsv by remember(value) { mutableStateOf(doweColorHsv(doweColorRgb(value))) }
    val canonical = doweColorHex(doweColorRgb(value))
    val active = expanded || canonical.isNotEmpty()
    val popupOffset = with(LocalDensity.current) { (doweControlHeight(size) + if (floating) 12.dp else 4.dp).roundToPx() }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(min = doweControlHeight(size) + if (floating) 8.dp else 0.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)))
                    .clickable { expanded = !expanded }
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                if (label != null && floating) {
                    DoweColorSwatch(canonical, size, contentColor)
                    Box(modifier = Modifier.weight(1f)) {
                        Text(label, modifier = Modifier.align(if (active) Alignment.TopStart else Alignment.CenterStart), fontSize = if (active) 12.sp else 14.sp, color = contentColor)
                        Text(text = canonical.ifEmpty { placeholder }, modifier = Modifier.align(Alignment.CenterStart).padding(top = if (active) 10.dp else 0.dp), color = contentColor, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, maxLines = 1)
                    }
                } else {
                    DoweColorTriggerContent(canonical, placeholder, size, fontSize, lineHeight, contentColor, Modifier.weight(1f))
                }
            }
            DoweAnchoredPopover(visible = expanded, offset = IntOffset(0, popupOffset), shape = RoundedCornerShape(12.dp), backgroundColor = DoweDesign.background, contentColor = DoweDesign.backgroundText, contentPadding = PaddingValues(16.dp), maxHeight = 480.dp, onDismiss = { expanded = false }) {
                DoweColorPickerPanel(value = canonical, hsv = hsv, onHsvChange = { next -> hsv = next; onValueChange(doweColorHex(doweColorFromHsv(next))) }, showHex = showHex, showRgb = showRgb, showCmyk = showCmyk, showOklch = showOklch)
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
    }
}

@Composable
private fun DoweColorTriggerContent(value: String, placeholder: String, size: String, fontSize: TextUnit, lineHeight: TextUnit, contentColor: Color, modifier: Modifier) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        DoweColorSwatch(value, size, contentColor)
        Text(text = value.ifEmpty { placeholder }, color = contentColor, fontSize = fontSize, lineHeight = lineHeight, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

@Composable
private fun DoweColorSwatch(value: String, size: String, contentColor: Color) {
    Box(modifier = Modifier.size(doweControlSwatchSize(size)).clip(RoundedCornerShape(6.dp)).background(doweHexColor(value, DoweDesign.primary)).border(1.dp, contentColor.copy(alpha = 0.22f), RoundedCornerShape(6.dp)))
}

private data class DoweColorHsv(val hue: Float, val saturation: Float, val brightness: Float)
private data class DoweColorRgb(val red: Int, val green: Int, val blue: Int)

@Composable
private fun DoweColorPickerPanel(value: String, hsv: DoweColorHsv, onHsvChange: (DoweColorHsv) -> Unit, showHex: Boolean, showRgb: Boolean, showCmyk: Boolean, showOklch: Boolean) {
    val rgb = doweColorRgb(value)
    Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxWidth()
                .height(140.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(Brush.horizontalGradient(listOf(Color.White, Color.hsv(hsv.hue, 1f, 1f))))
                .pointerInput(hsv.hue) {
                    awaitEachGesture {
                        var change = awaitFirstDown()
                        do {
                            val saturation = (change.position.x / size.width).coerceIn(0f, 1f)
                            val brightness = (1f - change.position.y / size.height).coerceIn(0f, 1f)
                            onHsvChange(hsv.copy(saturation = saturation, brightness = brightness))
                            val event = awaitPointerEvent()
                            change = event.changes.first()
                            change.consume()
                        } while (change.pressed)
                    }
                }
                .semantics { contentDescription = "Saturation ${(hsv.saturation * 100).roundToInt()} percent, brightness ${(hsv.brightness * 100).roundToInt()} percent" }
        ) {
            Box(modifier = Modifier.matchParentSize().background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black))))
            Box(modifier = Modifier.offset(x = maxWidth * hsv.saturation - 8.dp, y = maxHeight * (1f - hsv.brightness) - 8.dp).size(16.dp).clip(RoundedCornerShape(999.dp)).background(doweHexColor(value, DoweDesign.primary)).border(2.dp, Color.White, RoundedCornerShape(999.dp)))
        }
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxWidth()
                .height(16.dp)
                .clip(RoundedCornerShape(999.dp))
                .background(Brush.horizontalGradient(listOf(Color.Red, Color.Yellow, Color.Green, Color.Cyan, Color.Blue, Color.Magenta, Color.Red)))
                .pointerInput(Unit) {
                    awaitEachGesture {
                        var change = awaitFirstDown()
                        do {
                            onHsvChange(hsv.copy(hue = (change.position.x / size.width * 360f).coerceIn(0f, 360f)))
                            val event = awaitPointerEvent()
                            change = event.changes.first()
                            change.consume()
                        } while (change.pressed)
                    }
                }
                .semantics { contentDescription = "Hue ${hsv.hue.roundToInt()} degrees" }
        ) {
            Box(modifier = Modifier.offset(x = maxWidth * (hsv.hue / 360f) - 10.dp, y = (-2).dp).size(20.dp).clip(RoundedCornerShape(999.dp)).background(Color.White).border(1.dp, DoweDesign.muted.copy(alpha = 0.3f), RoundedCornerShape(999.dp)))
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Box(modifier = Modifier.size(48.dp).clip(RoundedCornerShape(8.dp)).background(doweHexColor(value, DoweDesign.primary)).border(1.dp, DoweDesign.backgroundText.copy(alpha = 0.22f), RoundedCornerShape(8.dp)))
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(value, color = DoweDesign.backgroundText, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Text("Foreground: ${doweColorForeground(rgb)}", color = DoweDesign.backgroundText.copy(alpha = 0.72f), fontSize = 12.sp)
            }
        }
        if (showHex || showRgb || showCmyk || showOklch) {
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                if (showHex) DoweColorFormatRow("hex: $value")
                if (showRgb) DoweColorFormatRow("rgb: ${doweColorRgbText(rgb)}")
                if (showCmyk) DoweColorFormatRow("cmyk: ${doweColorCmykText(rgb)}")
                if (showOklch) DoweColorFormatRow("oklch: ${doweColorOklchText(rgb)}")
            }
        }
    }
}

@Composable
private fun DoweColorFormatRow(value: String) {
    Text(value, modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(DoweDesign.softMuted).padding(horizontal = 8.dp, vertical = 4.dp), color = DoweDesign.softMutedText, fontSize = 12.sp, maxLines = 1)
}

private fun doweColorRgb(value: String): DoweColorRgb {
    val source = value.removePrefix("#")
    val clean = if (source.length == 3) source.map { "${it}${it}" }.joinToString("") else source
    val number = clean.takeIf { it.length == 6 }?.toLongOrNull(16) ?: 0x3B82F6
    return DoweColorRgb(((number shr 16) and 255).toInt(), ((number shr 8) and 255).toInt(), (number and 255).toInt())
}

private fun doweColorHex(rgb: DoweColorRgb): String = String.format(Locale.US, "#%02X%02X%02X", rgb.red, rgb.green, rgb.blue)

private fun doweColorHsv(rgb: DoweColorRgb): DoweColorHsv {
    val result = FloatArray(3)
    AndroidColor.RGBToHSV(rgb.red, rgb.green, rgb.blue, result)
    return DoweColorHsv(result[0], result[1], result[2])
}

private fun doweColorFromHsv(hsv: DoweColorHsv): DoweColorRgb {
    val color = AndroidColor.HSVToColor(floatArrayOf(hsv.hue, hsv.saturation, hsv.brightness))
    return DoweColorRgb(AndroidColor.red(color), AndroidColor.green(color), AndroidColor.blue(color))
}

private fun doweColorRgbText(rgb: DoweColorRgb): String = "rgb(${rgb.red}, ${rgb.green}, ${rgb.blue})"

private fun doweColorCmykText(rgb: DoweColorRgb): String {
    val values = listOf(rgb.red / 255.0, rgb.green / 255.0, rgb.blue / 255.0)
    val black = 1 - (values.maxOrNull() ?: 0.0)
    if (black >= 1) return "cmyk(0%, 0%, 0%, 100%)"
    val channels = values.map { ((1 - it - black) / (1 - black) * 100).roundToInt() }
    return "cmyk(${channels[0]}%, ${channels[1]}%, ${channels[2]}%, ${(black * 100).roundToInt()}%)"
}

private fun doweColorOklchText(rgb: DoweColorRgb): String {
    fun linear(value: Int): Double { val channel = value / 255.0; return if (channel <= 0.04045) channel / 12.92 else ((channel + 0.055) / 1.055).pow(2.4) }
    val red = linear(rgb.red); val green = linear(rgb.green); val blue = linear(rgb.blue)
    val l = (0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue).pow(1.0 / 3.0)
    val m = (0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue).pow(1.0 / 3.0)
    val s = (0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue).pow(1.0 / 3.0)
    val lightness = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s
    val a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s
    val b = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s
    val chroma = sqrt(a * a + b * b)
    var hue = atan2(b, a) * 180 / Math.PI
    if (hue < 0) hue += 360
    return String.format(Locale.US, "oklch(%.2f %.2f %.0f)", lightness, chroma, hue)
}

private fun doweColorForeground(rgb: DoweColorRgb): String = if ((0.299 * rgb.red + 0.587 * rgb.green + 0.114 * rgb.blue) / 255 > 0.5) "#000000" else "#FFFFFF"

@Composable
private fun DoweDateField(value: String, onValueChange: (String) -> Unit, label: String?, placeholder: String, floating: Boolean, size: String, fontSize: TextUnit, lineHeight: TextUnit, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    var expanded by remember { mutableStateOf(false) }
    var month by remember(value) { mutableStateOf(runCatching { YearMonth.from(LocalDate.parse(value)) }.getOrDefault(YearMonth.now())) }
    val active = expanded || value.isNotEmpty()
    val popupOffset = with(LocalDensity.current) { (doweControlHeight(size) + if (floating) 12.dp else 4.dp).roundToPx() }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null && !floating) Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        Box {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(min = doweControlHeight(size) + if (floating) 8.dp else 0.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)))
                    .clickable { expanded = !expanded }
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
            DoweAnchoredPopover(visible = expanded, offset = IntOffset(0, popupOffset), shape = RoundedCornerShape(12.dp), backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, contentPadding = PaddingValues(8.dp), onDismiss = { expanded = false }) {
                DoweDateCalendar(month = month, selected = value, start = "", end = "", min = min, max = max, contentColor = contentColor, accentColor = contentColor, showPrevious = true, showNext = true, onPrevious = { month = month.minusMonths(1) }, onNext = { month = month.plusMonths(1) }, onSelect = { next -> onValueChange(next); month = YearMonth.from(LocalDate.parse(next)); expanded = false })
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = contentColor.copy(alpha = 0.7f))
        }
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
                    .heightIn(min = doweControlHeight(size) + if (floating) 8.dp else 0.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(backgroundColor)
                    .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(10.dp)))
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

private data class DowePickedFile(val uri: Uri, val name: String, val size: Long?)

private fun doweDropzoneMimeTypes(accept: String?): Array<String> {
    val values = accept
        ?.split(",")
        ?.map { it.trim() }
        ?.filter { it.isNotEmpty() }
        ?.toTypedArray()
    return if (values.isNullOrEmpty()) arrayOf("*/*") else values
}

private fun dowePickedFile(context: android.content.Context, uri: Uri, maxSize: Long?): DowePickedFile? {
    var name = uri.lastPathSegment ?: "Selected file"
    var size: Long? = null
    context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)?.use { cursor ->
        if (cursor.moveToFirst()) {
            val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
            if (nameIndex >= 0) name = cursor.getString(nameIndex) ?: name
            if (sizeIndex >= 0 && !cursor.isNull(sizeIndex)) size = cursor.getLong(sizeIndex)
        }
    }
    if (maxSize != null && size != null && size > maxSize) return null
    return DowePickedFile(uri, name, size)
}

private fun dowePickedFileSize(size: Long?): String {
    if (size == null || size < 0) return ""
    val units = arrayOf("Bytes", "KB", "MB", "GB")
    var value = size.toDouble()
    var index = 0
    while (value >= 1024 && index < units.lastIndex) {
        value /= 1024
        index += 1
    }
    return "%.1f %s".format(java.util.Locale.US, value, units[index])
}

@Composable
private fun DoweDropzone(label: String?, placeholder: String, accept: String?, multiple: Boolean, maxSize: Long?, disabled: Boolean, helpText: String?, errorText: String?, size: String, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    val selectedFiles = remember { mutableStateListOf<DowePickedFile>() }
    val multiplePicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        val known = selectedFiles.map { it.uri }.toHashSet()
        uris.mapNotNull { uri -> dowePickedFile(context, uri, maxSize) }
            .filter { it.uri !in known }
            .forEach { selectedFiles.add(it) }
    }
    val singlePicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        uri?.let { pickedUri ->
            selectedFiles.clear()
            dowePickedFile(context, pickedUri, maxSize)?.let { selectedFiles.add(it) }
        }
    }
    val height = when (size) {
        "sm" -> 128.dp
        "lg" -> 256.dp
        else -> 192.dp
    }
    val launchPicker = {
        if (!disabled) {
            if (multiple) multiplePicker.launch(doweDropzoneMimeTypes(accept))
            else singlePicker.launch(doweDropzoneMimeTypes(accept))
        }
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
                .clickable(enabled = !disabled, onClick = launchPicker)
                .then(if (disabled) Modifier.graphicsLayer { alpha = 0.5f } else Modifier),
            contentAlignment = Alignment.Center
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(if (selectedFiles.isEmpty()) "Upload" else "Selected files", color = contentColor.copy(alpha = 0.55f), fontWeight = FontWeight.SemiBold)
                if (selectedFiles.isEmpty()) {
                    Text(placeholder, color = contentColor.copy(alpha = 0.7f), fontSize = 14.sp)
                } else {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        selectedFiles.take(3).forEach { file ->
                            Text(file.name, color = contentColor, fontSize = 14.sp, maxLines = 1)
                            val formattedSize = dowePickedFileSize(file.size)
                            if (formattedSize.isNotEmpty()) Text(formattedSize, color = contentColor.copy(alpha = 0.7f), fontSize = 12.sp)
                        }
                        if (selectedFiles.size > 3) Text("+${selectedFiles.size - 3} more", color = contentColor.copy(alpha = 0.7f), fontSize = 12.sp)
                    }
                }
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = if (errorText != null) DoweDesign.danger else contentColor.copy(alpha = 0.7f))
        }
    }
}
private fun doweControlHeight(size: String): Dp {
    return when (size) {
        "sm" -> 32.dp
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

"##
}
