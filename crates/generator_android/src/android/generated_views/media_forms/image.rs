fn android_runtime_media_image() -> &'static str {
    r##"@Composable
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
            .then(if (aspect == "auto") Modifier else Modifier.aspectRatio(doweImageAspect(aspect)))
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
            if (connection.responseCode !in 200..299) return null
            val bytes = connection.inputStream.use { it.readBytes() }
            val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size) ?: return null
            FileOutputStream(temporary).use { output -> output.write(bytes) }
            if (!temporary.renameTo(cached)) {
                temporary.delete()
            }
            bitmap
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

"##
}
