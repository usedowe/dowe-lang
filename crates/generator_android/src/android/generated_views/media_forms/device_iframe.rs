fn android_runtime_media_device_iframe() -> &'static str {
    r##"private data class DoweDeviceIcon(val profile: String, val viewBox: DoweSvgViewBox, val paths: List<DoweSvgPath>)

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
            containerColor = if (selected) DoweDesign.primary else Color.Transparent,
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

"##
}
