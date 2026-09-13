fn android_runtime_game_socket() -> &'static str {
    r#"private fun doweGameSocketUrl(context: android.content.Context, raw: String?): String? {
    val value = raw?.trim().orEmpty()
    if (value.startsWith("ws://") || value.startsWith("wss://")) return value
    if (!value.startsWith("/") || value.startsWith("//")) return null
    val development = context.getSharedPreferences("dowe-hmr", android.content.Context.MODE_PRIVATE)
        .getString("endpoint", "").orEmpty().trimEnd('/')
    val configured = DoweEnvironment.BACKEND_URL.trimEnd('/')
    val base = listOf(development, configured).firstOrNull { it.startsWith("http://") || it.startsWith("https://") }
        ?: return null
    val socketBase = when {
        base.startsWith("https://") -> "wss://" + base.removePrefix("https://")
        base.startsWith("http://") -> "ws://" + base.removePrefix("http://")
        else -> return null
    }
    return socketBase + value
}

private class DoweGameSocket(
    private val context: android.content.Context,
    private val state: DoweReactiveState,
    private val actionScope: kotlinx.coroutines.CoroutineScope,
    private val sendPath: String?,
    private val statusPath: String?,
    private val onOpen: String?,
    private val onMessage: String?,
    private val onClose: String?,
    private val onError: String?,
    private val reconnect: Boolean,
    private val reconnectDelay: Int,
) {
    private val client = okhttp3.OkHttpClient()
    private val handler = android.os.Handler(android.os.Looper.getMainLooper())
    private var socket: okhttp3.WebSocket? = null
    private var reconnectTask: Runnable? = null
    private var currentUrl: String? = null
    private var stopped = true
    private var lastSent: String? = null

    private fun item(kind: String, data: Any? = null): Map<String, Any?> =
        mapOf("source" to "game", "kind" to kind, "data" to data)

    private fun run(action: String?, value: Map<String, Any?>) {
        action?.let { actionScope.launch { state.run(it, value) } }
    }

    private fun status(value: String) {
        statusPath?.let { state.write(it, value) }
    }

    fun start(raw: String?) {
        stop()
        stopped = false
        val url = doweGameSocketUrl(context, raw)
        if (url == null) {
            status("error")
            run(onError, item("error", "Invalid WebSocket URL"))
            return
        }
        currentUrl = url
        connect()
    }

    private fun connect() {
        val url = currentUrl ?: return
        if (stopped) return
        status("connecting")
        lastSent = null
        val request = okhttp3.Request.Builder().url(url).build()
        socket = client.newWebSocket(request, object : okhttp3.WebSocketListener() {
            override fun onOpen(webSocket: okhttp3.WebSocket, response: okhttp3.Response) {
                if (stopped) return
                socket = webSocket
                status("open")
                sendCurrent()
                run(onOpen, item("open"))
            }

            override fun onMessage(webSocket: okhttp3.WebSocket, text: String) {
                if (!stopped) run(onMessage, item("message", text))
            }

            override fun onFailure(webSocket: okhttp3.WebSocket, error: Throwable, response: okhttp3.Response?) {
                if (stopped) return
                socket = null
                status("error")
                run(onError, item("error", error.message ?: "WebSocket failure"))
                scheduleReconnect()
            }

            override fun onClosed(webSocket: okhttp3.WebSocket, code: Int, reason: String) {
                if (stopped) return
                socket = null
                status("closed")
                run(onClose, item("close", mapOf("code" to code, "reason" to reason)))
                scheduleReconnect()
            }
        })
    }

    private fun scheduleReconnect() {
        if (!reconnect || stopped || currentUrl == null || reconnectTask != null) return
        val task = Runnable {
            reconnectTask = null
            connect()
        }
        reconnectTask = task
        handler.postDelayed(task, reconnectDelay.coerceIn(100, 60000).toLong())
    }

    private fun sendCurrent() {
        send(sendPath?.let { state.json(it) }.orEmpty())
    }

    fun send(payload: String) {
        if (payload.isEmpty() || payload == lastSent) return
        if (socket?.send(payload) == true) lastSent = payload
    }

    fun stop() {
        stopped = true
        reconnectTask?.let(handler::removeCallbacks)
        reconnectTask = null
        socket?.close(1000, "view changed")
        socket = null
        currentUrl = null
        lastSent = null
    }
}

@Composable
private fun DoweGame(
    state: DoweReactiveState,
    scenePath: String?,
    renderer: String,
    worldPath: String?,
    cameraPath: String?,
    controls: String,
    moveSpeed: Int,
    turnSpeed: Int,
    viewWidth: Float,
    viewHeight: Float,
    fit: String,
    fps: Int,
    autoplay: Boolean,
    pixelated: Boolean,
    backgroundColor: Color,
    label: String,
    onPointer: String?,
    onKey: String?,
    onFire: String?,
    onMotion: String?,
    motionRate: Int,
    socketPath: String?,
    socketBinding: Boolean,
    sendPath: String?,
    statusPath: String?,
    onOpen: String?,
    onMessage: String?,
    onClose: String?,
    onError: String?,
    reconnect: Boolean,
    reconnectDelay: Int,
    modifier: Modifier,
) {
    val context = LocalContext.current
    val actionScope = rememberCoroutineScope()
    val socket = remember {
        DoweGameSocket(
            context,
            state,
            actionScope,
            sendPath,
            statusPath,
            onOpen,
            onMessage,
            onClose,
            onError,
            reconnect,
            reconnectDelay,
        )
    }
    val rawSocket = socketPath?.let { if (socketBinding) state.text(it, "") else it }
    DisposableEffect(socket, rawSocket) {
        if (rawSocket != null) socket.start(rawSocket) else socket.stop()
        onDispose { socket.stop() }
    }
    val outbound = sendPath?.let { state.json(it) }.orEmpty()
    LaunchedEffect(outbound) {
        socket.send(outbound)
    }
    if (renderer == "raycast3d") {
        DoweRaycastGame(
            state = state,
            worldPath = worldPath ?: "",
            cameraPath = cameraPath ?: "",
            controls = controls,
            moveSpeed = moveSpeed.toFloat(),
            turnSpeed = turnSpeed.toFloat(),
            viewWidth = viewWidth,
            viewHeight = viewHeight,
            fit = fit,
            fps = fps,
            autoplay = autoplay,
            pixelated = pixelated,
            backgroundColor = backgroundColor,
            label = label,
            onPointer = onPointer,
            onKey = onKey,
            onFire = onFire,
            modifier = modifier,
        )
    } else {
        DoweCanvas(
            state = state,
            scenePath = scenePath ?: "",
            viewWidth = viewWidth,
            viewHeight = viewHeight,
            fit = fit,
            fps = fps,
            autoplay = autoplay,
            pixelated = pixelated,
            backgroundColor = backgroundColor,
            label = label,
            onPointer = onPointer,
            onKey = onKey,
            onMotion = onMotion,
            motionRate = motionRate,
            draw = false,
            drawMode = "pen",
            drawModePath = null,
            layersPath = null,
            selectedPath = null,
            onLayerAdd = null,
            onLayerChange = null,
            onLayerRemove = null,
            onLayerSelect = null,
            modifier = modifier,
        )
    }
}
"#
}
