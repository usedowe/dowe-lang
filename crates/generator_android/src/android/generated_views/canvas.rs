fn android_runtime_canvas() -> &'static str {
    r#"@Composable
private fun DoweCanvas(state: DoweReactiveState, scenePath: String, viewWidth: Float, viewHeight: Float, fit: String, fps: Int, autoplay: Boolean, pixelated: Boolean, backgroundColor: Color, label: String, onPointer: String?, onKey: String?, onMotion: String?, motionRate: Int, modifier: Modifier) {
    val context = LocalContext.current
    val commands = state.candles(scenePath)
    val actionScope = rememberCoroutineScope()
    val focusRequester = remember { FocusRequester() }
    val inputStarted = remember { SystemClock.uptimeMillis() }
    var elapsed by remember(scenePath) { mutableStateOf(0f) }
    val images = remember { mutableStateMapOf<String, ImageBitmap>() }
    val sources = commands.mapNotNull { if (it["type"]?.toString() == "image") it["src"]?.toString() else null }.distinct()
    LaunchedEffect(sources) {
        sources.forEach { source ->
            if (!images.containsKey(source)) {
                val bitmap = withContext(Dispatchers.IO) {
                    try {
                        if (source.startsWith("https://")) BitmapFactory.decodeStream(URL(source).openStream())
                        else context.assets.open(source.trimStart('/')).use(BitmapFactory::decodeStream)
                    } catch (error: Exception) {
                        null
                    }
                }
                if (bitmap != null) images[source] = bitmap.asImageBitmap()
            }
        }
    }
    LaunchedEffect(autoplay, fps, scenePath) {
        elapsed = 0f
        if (autoplay && doweCanvasMotionEnabled(context)) {
            val started = System.nanoTime()
            while (true) {
                elapsed = (System.nanoTime() - started) / 1_000_000_000f
                delay(max(8L, 1000L / max(1, fps)))
            }
        }
    }
    DisposableEffect(onMotion, motionRate) {
        if (onMotion == null) {
            onDispose { }
        } else {
            val manager = context.getSystemService(android.content.Context.SENSOR_SERVICE) as SensorManager
            val acceleration = FloatArray(3)
            val rotation = FloatArray(3)
            var last = 0L
            val listener = object : SensorEventListener {
                override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) { }
                override fun onSensorChanged(event: SensorEvent) {
                    if (event.sensor.type == Sensor.TYPE_ACCELEROMETER) {
                        acceleration[0] = event.values.getOrElse(0) { 0f }
                        acceleration[1] = event.values.getOrElse(1) { 0f }
                        acceleration[2] = event.values.getOrElse(2) { 0f }
                    } else if (event.sensor.type == Sensor.TYPE_ROTATION_VECTOR) {
                        val matrix = FloatArray(9)
                        SensorManager.getRotationMatrixFromVector(matrix, event.values)
                        SensorManager.getOrientation(matrix, rotation)
                    }
                    val now = SystemClock.uptimeMillis()
                    val minimum = max(1L, 1000L / max(1, motionRate))
                    if (now - last < minimum) return
                    val vector = doweCanvasScreenVector(context, acceleration[0], -acceleration[1])
                    last = now
                    actionScope.launch {
                        state.run(onMotion, mapOf(
                            "source" to "motion",
                            "acceleration" to mapOf("x" to vector.first, "y" to vector.second, "z" to acceleration[2]),
                            "rotation" to mapOf("alpha" to Math.toDegrees(rotation[0].toDouble()), "beta" to Math.toDegrees(rotation[1].toDouble()), "gamma" to Math.toDegrees(rotation[2].toDouble())),
                            "interval" to minimum,
                            "timestamp" to now - inputStarted
                        ))
                    }
                }
            }
            val delay = max(1, 1_000_000 / max(1, motionRate))
            manager.getDefaultSensor(Sensor.TYPE_ACCELEROMETER)?.let { manager.registerListener(listener, it, delay) }
            manager.getDefaultSensor(Sensor.TYPE_ROTATION_VECTOR)?.let { manager.registerListener(listener, it, delay) }
            onDispose { manager.unregisterListener(listener) }
        }
    }
    var inputModifier = modifier.clipToBounds().semantics { contentDescription = label }
    if (onPointer != null) {
        inputModifier = inputModifier.pointerInput(onPointer, viewWidth, viewHeight, fit) {
            awaitPointerEventScope {
                while (true) {
                    val event = awaitPointerEvent()
                    for (change in event.changes) {
                        val kind = when {
                            change.changedToDownIgnoreConsumed() -> "down"
                            change.changedToUpIgnoreConsumed() -> "up"
                            change.pressed -> "move"
                            else -> "cancel"
                        }
                        if (kind == "down") focusRequester.requestFocus()
                        val point = doweCanvasLogicalPoint(change.position, size.width.toFloat(), size.height.toFloat(), viewWidth, viewHeight, fit)
                        val previous = doweCanvasLogicalPoint(change.previousPosition, size.width.toFloat(), size.height.toFloat(), viewWidth, viewHeight, fit)
                        val pointerType = when (change.type) { PointerType.Mouse -> "mouse"; PointerType.Touch -> "touch"; PointerType.Stylus -> "pen"; else -> "unknown" }
                        actionScope.launch { state.run(onPointer, mapOf(
                            "source" to "pointer", "kind" to kind, "pointerType" to pointerType,
                            "id" to change.id.value, "x" to point.first, "y" to point.second,
                            "dx" to point.first - previous.first, "dy" to point.second - previous.second,
                            "inside" to point.third, "buttons" to if (change.pressed) 1 else 0,
                            "pressure" to change.pressure.coerceIn(0f, 1f), "primary" to (event.changes.firstOrNull()?.id == change.id),
                            "timestamp" to change.uptimeMillis - inputStarted
                        )) }
                        change.consume()
                    }
                }
            }
        }
    }
    if (onKey != null) {
        inputModifier = inputModifier.focusRequester(focusRequester).focusable().onPreviewKeyEvent { event ->
            val kind = if (event.type == KeyEventType.KeyDown) "down" else if (event.type == KeyEventType.KeyUp) "up" else return@onPreviewKeyEvent false
            val native = event.nativeKeyEvent
            actionScope.launch { state.run(onKey, mapOf(
                "source" to "key", "kind" to kind, "key" to doweCanvasKey(native), "code" to android.view.KeyEvent.keyCodeToString(native.keyCode).removePrefix("KEYCODE_"),
                "repeat" to (native.repeatCount > 0), "alt" to native.isAltPressed, "ctrl" to native.isCtrlPressed,
                "meta" to native.isMetaPressed, "shift" to native.isShiftPressed, "timestamp" to SystemClock.uptimeMillis() - inputStarted
            )) }
            true
        }
    }
    Canvas(modifier = inputModifier) {
        if (backgroundColor != Color.Transparent) drawRect(backgroundColor)
        val scaleX = size.width / max(1f, viewWidth)
        val scaleY = size.height / max(1f, viewHeight)
        val scale = if (fit == "cover") max(scaleX, scaleY) else min(scaleX, scaleY)
        val sx = if (fit == "stretch") scaleX else scale
        val sy = if (fit == "stretch") scaleY else scale
        val left = (size.width - viewWidth * sx) / 2f
        val top = (size.height - viewHeight * sy) / 2f
        withTransform({ translate(left, top); scale(sx, sy, pivot = Offset.Zero) }) {
            commands.forEach { command -> doweDrawCanvasCommand(doweBoundCanvasCommand(command, state), elapsed, viewWidth, viewHeight, images, pixelated) }
        }
    }
}

private fun doweBoundCanvasCommand(command: Map<String, Any?>, state: DoweReactiveState): Map<String, Any?> {
    val bindings = command["bind"] as? Map<*, *> ?: return command
    val output = command.toMutableMap()
    bindings.forEach { (field, path) -> if (field is String && path is String) state.canvasValue(path)?.let { output[field] = it } }
    return output
}

private fun doweCanvasLogicalPoint(position: Offset, width: Float, height: Float, viewWidth: Float, viewHeight: Float, fit: String): Triple<Float, Float, Boolean> {
    var sx = width / max(1f, viewWidth)
    var sy = height / max(1f, viewHeight)
    var left = 0f
    var top = 0f
    if (fit != "stretch") {
        val scale = if (fit == "cover") max(sx, sy) else min(sx, sy)
        sx = scale
        sy = scale
        left = (width - viewWidth * scale) / 2f
        top = (height - viewHeight * scale) / 2f
    }
    val rawX = (position.x - left) / max(0.0001f, sx)
    val rawY = (position.y - top) / max(0.0001f, sy)
    return Triple(rawX.coerceIn(0f, viewWidth), rawY.coerceIn(0f, viewHeight), rawX in 0f..viewWidth && rawY in 0f..viewHeight)
}

private fun doweCanvasScreenVector(context: android.content.Context, x: Float, y: Float): Pair<Float, Float> {
    val rotation = (context.getSystemService(android.content.Context.WINDOW_SERVICE) as WindowManager).defaultDisplay.rotation
    return when (rotation) { Surface.ROTATION_90 -> Pair(-y, x); Surface.ROTATION_180 -> Pair(-x, -y); Surface.ROTATION_270 -> Pair(y, -x); else -> Pair(x, y) }
}

private fun doweCanvasKey(event: android.view.KeyEvent): String {
    return when (event.keyCode) {
        android.view.KeyEvent.KEYCODE_DPAD_LEFT -> "ArrowLeft"
        android.view.KeyEvent.KEYCODE_DPAD_RIGHT -> "ArrowRight"
        android.view.KeyEvent.KEYCODE_DPAD_UP -> "ArrowUp"
        android.view.KeyEvent.KEYCODE_DPAD_DOWN -> "ArrowDown"
        android.view.KeyEvent.KEYCODE_ENTER -> "Enter"
        android.view.KeyEvent.KEYCODE_SPACE -> " "
        else -> event.unicodeChar.takeIf { it > 0 }?.let { String(Character.toChars(it)) } ?: android.view.KeyEvent.keyCodeToString(event.keyCode).removePrefix("KEYCODE_")
    }
}

private fun doweCanvasMotionEnabled(context: android.content.Context): Boolean =
    try { android.provider.Settings.Global.getFloat(context.contentResolver, android.provider.Settings.Global.ANIMATOR_DURATION_SCALE, 1f) != 0f } catch (error: Exception) { true }

private fun doweCanvasNumber(value: Any?, fallback: Float = 0f): Float =
    when (value) { is Number -> value.toFloat(); else -> value?.toString()?.toFloatOrNull() ?: fallback }

private fun doweCanvasBool(value: Any?): Boolean = value as? Boolean ?: false

private fun doweCanvasColor(value: Any?, fallback: Color = Color.Transparent): Color = when (value?.toString()) {
    "primary" -> DoweDesign.primary
    "onPrimary" -> DoweDesign.onPrimary
    "secondary" -> DoweDesign.secondary
    "onSecondary" -> DoweDesign.onSecondary
    "tertiary" -> DoweDesign.tertiary
    "onTertiary" -> DoweDesign.onTertiary
    "muted" -> DoweDesign.muted
    "onMuted" -> DoweDesign.onMuted
    "background" -> DoweDesign.background
    "onBackground", "foreground", "currentColor" -> DoweDesign.onBackground
    "surface" -> DoweDesign.surface
    "onSurface" -> DoweDesign.onSurface
    "success" -> DoweDesign.success
    "onSuccess" -> DoweDesign.onSuccess
    "info" -> DoweDesign.info
    "onInfo" -> DoweDesign.onInfo
    "warning" -> DoweDesign.warning
    "onWarning" -> DoweDesign.onWarning
    "danger" -> DoweDesign.danger
    "onDanger" -> DoweDesign.onDanger
    "softPrimary" -> DoweDesign.softPrimary
    "onSoftPrimary" -> DoweDesign.onSoftPrimary
    "softSecondary" -> DoweDesign.softSecondary
    "onSoftSecondary" -> DoweDesign.onSoftSecondary
    "softTertiary" -> DoweDesign.softTertiary
    "onSoftTertiary" -> DoweDesign.onSoftTertiary
    "softMuted" -> DoweDesign.softMuted
    "onSoftMuted" -> DoweDesign.onSoftMuted
    "softSuccess" -> DoweDesign.softSuccess
    "onSoftSuccess" -> DoweDesign.onSoftSuccess
    "softInfo" -> DoweDesign.softInfo
    "onSoftInfo" -> DoweDesign.onSoftInfo
    "softWarning" -> DoweDesign.softWarning
    "onSoftWarning" -> DoweDesign.onSoftWarning
    "softDanger" -> DoweDesign.softDanger
    "onSoftDanger" -> DoweDesign.onSoftDanger
    "transparent" -> Color.Transparent
    else -> value?.toString()?.let { doweHexColor(it, fallback) } ?: fallback
}

private data class DoweCanvasMotion(val dx: Float, val dy: Float, val rotation: Float, val alpha: Float)

private fun doweCanvasMotion(command: Map<String, Any?>, elapsed: Float, width: Float, height: Float): DoweCanvasMotion {
    val motion = command["motion"] as? Map<String, Any?> ?: emptyMap()
    var dx = doweCanvasNumber(motion["vx"]) * elapsed
    var dy = doweCanvasNumber(motion["vy"]) * elapsed
    if (doweCanvasBool(motion["wrap"])) {
        val x = doweCanvasNumber(command["x"])
        val y = doweCanvasNumber(command["y"])
        dx = ((x + dx) % width + width) % width - x
        dy = ((y + dy) % height + height) % height - y
    }
    val pulse = doweCanvasNumber(motion["pulse"])
    val opacity = doweCanvasNumber(command["opacity"], 1f)
    val alpha = (opacity * if (pulse == 0f) 1f else 0.55f + 0.45f * sin(elapsed * pulse * Math.PI.toFloat() * 2f)).coerceIn(0f, 1f)
    return DoweCanvasMotion(dx, dy, doweCanvasNumber(command["rotation"]) + doweCanvasNumber(motion["rotation"]) * elapsed, alpha)
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.doweDrawCanvasCommand(command: Map<String, Any?>, elapsed: Float, viewWidth: Float, viewHeight: Float, images: Map<String, ImageBitmap>, pixelated: Boolean) {
    val type = command["type"]?.toString() ?: return
    val motion = doweCanvasMotion(command, elapsed, viewWidth, viewHeight)
    val x = doweCanvasNumber(command["x"])
    val y = doweCanvasNumber(command["y"])
    withTransform({ translate(motion.dx, motion.dy); rotate(motion.rotation, Offset(x, y)) }) {
        val fill = doweCanvasColor(command["fill"])
        val stroke = doweCanvasColor(command["stroke"])
        val strokeWidth = max(0f, doweCanvasNumber(command["strokeWidth"], 1f))
        when (type) {
            "rect" -> {
                val width = max(0f, doweCanvasNumber(command["width"]))
                val height = max(0f, doweCanvasNumber(command["height"]))
                val radius = max(0f, min(doweCanvasNumber(command["radius"]), min(width, height) / 2f))
                if (fill != Color.Transparent) drawRoundRect(fill.copy(alpha = motion.alpha), Offset(x, y), Size(width, height), CornerRadius(radius, radius))
                if (stroke != Color.Transparent) drawRoundRect(stroke.copy(alpha = motion.alpha), Offset(x, y), Size(width, height), CornerRadius(radius, radius), style = androidx.compose.ui.graphics.drawscope.Stroke(strokeWidth))
            }
            "circle" -> {
                val radius = max(0f, doweCanvasNumber(command["radius"]))
                if (fill != Color.Transparent) drawCircle(fill.copy(alpha = motion.alpha), radius, Offset(x, y))
                if (stroke != Color.Transparent) drawCircle(stroke.copy(alpha = motion.alpha), radius, Offset(x, y), style = androidx.compose.ui.graphics.drawscope.Stroke(strokeWidth))
            }
            "line" -> drawLine(stroke.copy(alpha = motion.alpha), Offset(doweCanvasNumber(command["x1"]), doweCanvasNumber(command["y1"])), Offset(doweCanvasNumber(command["x2"]), doweCanvasNumber(command["y2"])), strokeWidth)
            "polyline" -> {
                val points = (command["points"] as? List<*>)?.mapNotNull { it as? Map<String, Any?> } ?: emptyList()
                if (points.isNotEmpty()) {
                    val path = Path().apply {
                        moveTo(doweCanvasNumber(points[0]["x"]), doweCanvasNumber(points[0]["y"]))
                        points.drop(1).forEach { lineTo(doweCanvasNumber(it["x"]), doweCanvasNumber(it["y"])) }
                        if (doweCanvasBool(command["closed"])) close()
                    }
                    if (fill != Color.Transparent) drawPath(path, fill.copy(alpha = motion.alpha))
                    if (stroke != Color.Transparent) drawPath(path, stroke.copy(alpha = motion.alpha), style = androidx.compose.ui.graphics.drawscope.Stroke(strokeWidth))
                }
            }
            "text" -> drawIntoCanvas { canvas ->
                val paint = Paint().apply { color = doweCanvasColor(command["fill"], DoweDesign.onBackground).copy(alpha = motion.alpha).toArgb(); textSize = max(1f, doweCanvasNumber(command["size"], 16f)); textAlign = when (command["align"]?.toString()) { "center" -> Paint.Align.CENTER; "end" -> Paint.Align.RIGHT; else -> Paint.Align.LEFT }; isAntiAlias = true }
                canvas.nativeCanvas.drawText(command["text"]?.toString() ?: "", x, y, paint)
            }
            "image" -> images[command["src"]?.toString()]?.let { image ->
                val width = max(0f, doweCanvasNumber(command["width"]))
                val height = max(0f, doweCanvasNumber(command["height"]))
                if (width > 0f && height > 0f) {
                    val fit = command["fit"]?.toString() ?: "contain"
                    if (fit == "stretch") {
                        drawImage(image, dstOffset = IntOffset(x.toInt(), y.toInt()), dstSize = IntSize(width.toInt(), height.toInt()), alpha = motion.alpha, filterQuality = if (pixelated) FilterQuality.None else FilterQuality.Low)
                    } else {
                        val scale = if (fit == "cover") max(width / image.width, height / image.height) else min(width / image.width, height / image.height)
                        val drawWidth = image.width * scale
                        val drawHeight = image.height * scale
                        val left = x + (width - drawWidth) / 2f
                        val top = y + (height - drawHeight) / 2f
                        clipRect(x, y, x + width, y + height) {
                            drawImage(image, dstOffset = IntOffset(left.toInt(), top.toInt()), dstSize = IntSize(drawWidth.toInt(), drawHeight.toInt()), alpha = motion.alpha, filterQuality = if (pixelated) FilterQuality.None else FilterQuality.Low)
                        }
                    }
                }
            }
        }
    }
}
"#
}
