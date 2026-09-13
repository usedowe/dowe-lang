fn android_runtime_game_raycast() -> &'static str {
    r##"private data class DoweRaycastCamera(
    val x: Float,
    val y: Float,
    val angle: Float,
    val fov: Float,
    val pitch: Float,
)

private fun doweRaycastNumber(value: Any?, fallback: Float = 0f): Float = when (value) {
    is Number -> value.toFloat()
    is String -> value.toFloatOrNull() ?: fallback
    else -> fallback
}

private fun doweRaycastCamera(value: Any?): DoweRaycastCamera {
    val source = value as? Map<*, *> ?: emptyMap<String, Any?>()
    return DoweRaycastCamera(
        doweRaycastNumber(source["x"], 1.5f),
        doweRaycastNumber(source["y"], 1.5f),
        doweRaycastNumber(source["angle"]),
        doweRaycastNumber(source["fov"], (Math.PI / 3.0).toFloat()).coerceIn(0.35f, 1.8f),
        doweRaycastNumber(source["pitch"]),
    )
}

private fun doweRaycastRows(value: Any?): List<String> = ((value as? Map<*, *>)?.get("map") as? List<*>)
    ?.map { it?.toString().orEmpty() }
    ?: emptyList()

private fun doweRaycastCell(rows: List<String>, x: Int, y: Int): Char {
    if (x < 0 || y < 0 || y >= rows.size) return '1'
    return rows[y].getOrNull(x) ?: '1'
}

private fun doweRaycastSolid(cell: Char): Boolean = cell != '0' && cell != ' ' && cell != '.' && cell != '_'

private fun doweRaycastCanWalk(rows: List<String>, x: Float, y: Float): Boolean {
    val radius = 0.18f
    return listOf(
        x - radius to y - radius,
        x + radius to y - radius,
        x - radius to y + radius,
        x + radius to y + radius,
    ).all { !doweRaycastSolid(doweRaycastCell(rows, floor(it.first).toInt(), floor(it.second).toInt())) }
}

private fun doweRaycastStep(camera: DoweRaycastCamera, rows: List<String>, controls: String, pressed: Set<String>, delta: Float, speed: Float, turnSpeed: Float): DoweRaycastCamera {
    if (controls != "doom" || pressed.isEmpty()) return camera
    var angle = camera.angle
    if ("a" in pressed || "left" in pressed) angle -= turnSpeed * Math.PI.toFloat() / 180f * delta
    if ("d" in pressed || "right" in pressed) angle += turnSpeed * Math.PI.toFloat() / 180f * delta
    val forward = (if ("w" in pressed || "up" in pressed) 1f else 0f) - (if ("s" in pressed || "down" in pressed) 1f else 0f)
    if (forward == 0f) return camera.copy(angle = angle)
    val distance = speed * delta * forward
    val nextX = camera.x + cos(angle) * distance
    val nextY = camera.y + sin(angle) * distance
    return camera.copy(
        x = if (doweRaycastCanWalk(rows, nextX, camera.y)) nextX else camera.x,
        y = if (doweRaycastCanWalk(rows, camera.x, nextY)) nextY else camera.y,
        angle = angle,
    )
}

private fun doweRaycastKey(event: android.view.KeyEvent): String = when (event.keyCode) {
    android.view.KeyEvent.KEYCODE_W -> "w"
    android.view.KeyEvent.KEYCODE_A -> "a"
    android.view.KeyEvent.KEYCODE_S -> "s"
    android.view.KeyEvent.KEYCODE_D -> "d"
    android.view.KeyEvent.KEYCODE_DPAD_UP -> "up"
    android.view.KeyEvent.KEYCODE_DPAD_DOWN -> "down"
    android.view.KeyEvent.KEYCODE_DPAD_LEFT -> "left"
    android.view.KeyEvent.KEYCODE_DPAD_RIGHT -> "right"
    android.view.KeyEvent.KEYCODE_SPACE -> " "
    android.view.KeyEvent.KEYCODE_ENTER -> "Enter"
    else -> event.keyCode.toString()
}

private fun DoweRaycastCamera.wrapAngle(): DoweRaycastCamera {
    var next = angle
    while (next > Math.PI) next -= (Math.PI * 2).toFloat()
    while (next < -Math.PI) next += (Math.PI * 2).toFloat()
    return copy(angle = next)
}

private fun doweRaycastAngleDelta(value: Float): Float {
    var next = value
    while (next > Math.PI) next -= (Math.PI * 2).toFloat()
    while (next < -Math.PI) next += (Math.PI * 2).toFloat()
    return next
}

private fun doweRaycastDistance(rows: List<String>, camera: DoweRaycastCamera, angle: Float): Float {
    val rayX = cos(angle)
    val rayY = sin(angle)
    var mapX = floor(camera.x).toInt()
    var mapY = floor(camera.y).toInt()
    val deltaX = if (abs(rayX) < 0.00001f) 1e30f else abs(1f / rayX)
    val deltaY = if (abs(rayY) < 0.00001f) 1e30f else abs(1f / rayY)
    val stepX = if (rayX < 0f) -1 else 1
    val stepY = if (rayY < 0f) -1 else 1
    var sideX = if (rayX < 0f) (camera.x - mapX) * deltaX else (mapX + 1 - camera.x) * deltaX
    var sideY = if (rayY < 0f) (camera.y - mapY) * deltaY else (mapY + 1 - camera.y) * deltaY
    var side = 0
    repeat(128) {
        if (sideX < sideY) { sideX += deltaX; mapX += stepX; side = 0 } else { sideY += deltaY; mapY += stepY; side = 1 }
        if (doweRaycastSolid(doweRaycastCell(rows, mapX, mapY))) {
            val distance = if (side == 0) (mapX - camera.x + (1 - stepX) / 2f) / (rayX.takeUnless { abs(it) < 0.00001f } ?: 0.00001f) else (mapY - camera.y + (1 - stepY) / 2f) / (rayY.takeUnless { abs(it) < 0.00001f } ?: 0.00001f)
            return max(0.01f, abs(distance))
        }
    }
    return 64f
}

@Composable
private fun DoweRaycastGame(
    state: DoweReactiveState,
    worldPath: String,
    cameraPath: String,
    controls: String,
    moveSpeed: Float,
    turnSpeed: Float,
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
    modifier: Modifier,
) {
    val actionScope = rememberCoroutineScope()
    val focusRequester = remember { FocusRequester() }
    val pressed = remember { mutableStateMapOf<String, Boolean>() }
    var camera by remember(cameraPath) { mutableStateOf(doweRaycastCamera(state.canvasValue(cameraPath))) }
    var fireAt by remember { mutableStateOf(0L) }
    var defeated by remember(cameraPath) { mutableStateOf(emptySet<Int>()) }
    val currentCamera by androidx.compose.runtime.rememberUpdatedState(camera)
    LaunchedEffect(worldPath, cameraPath, controls, fps, autoplay) {
        var previous = System.nanoTime()
        while (true) {
            val now = System.nanoTime()
            val delta = ((now - previous).coerceAtLeast(0L) / 1_000_000_000f).coerceAtMost(0.08f)
            previous = now
            if (autoplay || pressed.isNotEmpty()) {
                val rows = doweRaycastRows(state.canvasValue(worldPath))
                camera = doweRaycastStep(camera, rows, controls, pressed.filterValues { it }.keys, delta, moveSpeed, turnSpeed).wrapAngle()
            }
            delay(max(8L, 1000L / max(1, fps)))
        }
    }
    val fire: () -> Unit = {
        fireAt = System.nanoTime()
        val world = state.canvasValue(worldPath) as? Map<*, *> ?: emptyMap<String, Any?>()
        val rows = doweRaycastRows(world)
        val wallDistance = doweRaycastDistance(rows, currentCamera, currentCamera.angle)
        val sprites = (world["sprites"] as? List<*>)?.mapNotNull { it as? Map<*, *> } ?: emptyList()
        var target: Any? = null
        var targetIndex = -1
        var nearest = Float.MAX_VALUE
        sprites.forEachIndexed { index, sprite ->
            if (index in defeated) return@forEachIndexed
            val dx = doweRaycastNumber(sprite["x"]) - currentCamera.x
            val dy = doweRaycastNumber(sprite["y"]) - currentCamera.y
            val distance = hypot(dx, dy)
            val relative = abs(doweRaycastAngleDelta(atan2(dy, dx) - currentCamera.angle))
            if (distance < wallDistance && distance < nearest && relative < 0.1f) {
                target = sprite["id"] ?: index
                targetIndex = index
                nearest = distance
            }
        }
        if (targetIndex >= 0) defeated = defeated + targetIndex
        onFire?.let { action ->
            actionScope.launch {
                state.run(action, mapOf("source" to "game", "kind" to "fire", "data" to mapOf("x" to currentCamera.x, "y" to currentCamera.y, "angle" to currentCamera.angle, "target" to target, "hit" to (target != null))))
            }
        }
    }
    val inputModifier = modifier
        .clipToBounds()
        .semantics { contentDescription = label }
        .focusRequester(focusRequester)
        .focusable()
        .onPreviewKeyEvent { event ->
            val kind = if (event.type == KeyEventType.KeyDown) "down" else if (event.type == KeyEventType.KeyUp) "up" else return@onPreviewKeyEvent false
            val native = event.nativeKeyEvent
            val key = doweRaycastKey(native)
            if (kind == "down") pressed[key] = true else pressed.remove(key)
            onKey?.let { action ->
                actionScope.launch {
                    state.run(action, mapOf("source" to "key", "kind" to kind, "key" to key, "code" to native.keyCode.toString(), "repeat" to (native.repeatCount > 0), "timestamp" to SystemClock.uptimeMillis()))
                }
            }
            if (kind == "down" && (key == " " || key == "Enter")) fire()
            true
        }
        .pointerInput(worldPath, cameraPath, onPointer, onFire) {
            awaitPointerEventScope {
                while (true) {
                    val event = awaitPointerEvent()
                    event.changes.forEach { change ->
                        val down = change.changedToDownIgnoreConsumed()
                        val up = change.changedToUpIgnoreConsumed()
                        if (down) {
                            focusRequester.requestFocus()
                            fire()
                        }
                        if (!down && !up && controls == "doom") {
                            val dx = change.position.x - change.previousPosition.x
                            val dy = change.position.y - change.previousPosition.y
                            val nextAngle = camera.angle + dx * 0.006f
                            val amount = -dy * 0.012f * max(0.1f, moveSpeed / 2f)
                            val rows = doweRaycastRows(state.canvasValue(worldPath))
                            val nextX = camera.x + cos(nextAngle) * amount
                            val nextY = camera.y + sin(nextAngle) * amount
                            camera = camera.copy(
                                x = if (doweRaycastCanWalk(rows, nextX, camera.y)) nextX else camera.x,
                                y = if (doweRaycastCanWalk(rows, camera.x, nextY)) nextY else camera.y,
                                angle = nextAngle,
                            ).wrapAngle()
                        }
                        val kind = if (down) "down" else if (up) "up" else "move"
                        onPointer?.let { action ->
                            actionScope.launch {
                                state.run(action, mapOf("source" to "pointer", "kind" to kind, "pointerType" to "touch", "id" to change.id.value, "x" to change.position.x, "y" to change.position.y, "dx" to change.position.x - change.previousPosition.x, "dy" to change.position.y - change.previousPosition.y, "inside" to true, "buttons" to if (change.pressed) 1 else 0, "timestamp" to change.uptimeMillis))
                            }
                        }
                        change.consume()
                    }
                }
            }
        }
    Canvas(modifier = inputModifier) {
        if (backgroundColor != Color.Transparent) drawRect(backgroundColor)
        val world = state.canvasValue(worldPath) as? Map<*, *> ?: emptyMap<String, Any?>()
        doweRaycastScene(this, world, camera, viewWidth, viewHeight, fit, fireAt, defeated)
    }
}

private fun doweRaycastScene(scope: androidx.compose.ui.graphics.drawscope.DrawScope, world: Map<*, *>, camera: DoweRaycastCamera, viewWidth: Float, viewHeight: Float, fit: String, fireAt: Long, defeated: Set<Int>) {
    val rows = doweRaycastRows(world)
    val width = scope.size.width
    val height = scope.size.height
    val horizon = height * (0.5f + camera.pitch * 0.2f)
    scope.drawRect(doweCanvasColor(world["ceiling"] ?: "#101827"), topLeft = Offset.Zero, size = Size(width, horizon.coerceAtLeast(0f)))
    scope.drawRect(doweCanvasColor(world["floor"] ?: "#283244"), topLeft = Offset(0f, horizon), size = Size(width, (height - horizon).coerceAtLeast(0f)))
    val columns = width.toInt().coerceIn(120, 480)
    val step = width / columns.toFloat()
    val depth = FloatArray(columns) { 64f }
    for (column in 0 until columns) {
        val cameraX = (column + 0.5f) / columns * 2f - 1f
        val rayAngle = camera.angle + atan(cameraX * tan(camera.fov / 2f))
        val rayX = cos(rayAngle)
        val rayY = sin(rayAngle)
        var mapX = floor(camera.x).toInt()
        var mapY = floor(camera.y).toInt()
        val deltaX = if (abs(rayX) < 0.00001f) 1e30f else abs(1f / rayX)
        val deltaY = if (abs(rayY) < 0.00001f) 1e30f else abs(1f / rayY)
        val stepX = if (rayX < 0f) -1 else 1
        val stepY = if (rayY < 0f) -1 else 1
        var sideX = if (rayX < 0f) (camera.x - mapX) * deltaX else (mapX + 1 - camera.x) * deltaX
        var sideY = if (rayY < 0f) (camera.y - mapY) * deltaY else (mapY + 1 - camera.y) * deltaY
        var side = 0
        var cell = '1'
        var depth = 0
        while (depth < 128 && !doweRaycastSolid(cell)) {
            if (sideX < sideY) { sideX += deltaX; mapX += stepX; side = 0 } else { sideY += deltaY; mapY += stepY; side = 1 }
            cell = doweRaycastCell(rows, mapX, mapY)
            depth += 1
        }
        val distance = if (side == 0) (mapX - camera.x + (1 - stepX) / 2f) / (rayX.takeUnless { abs(it) < 0.00001f } ?: 0.00001f) else (mapY - camera.y + (1 - stepY) / 2f) / (rayY.takeUnless { abs(it) < 0.00001f } ?: 0.00001f)
        val corrected = max(0.05f, abs(distance) * cos(rayAngle - camera.angle))
        depth[column] = corrected
        val wallHeight = min(height * 3f, height / corrected)
        val walls = world["walls"] as? Map<*, *>
        val color = walls?.get(cell.toString()) ?: world["wall"] ?: "#6b7280"
        scope.drawRect(doweCanvasColor(color), topLeft = Offset(column * step, horizon - wallHeight / 2f), size = Size(step + 1f, wallHeight))
    }
    val sprites = (world["sprites"] as? List<*>)?.mapNotNull { it as? Map<*, *> } ?: emptyList()
    sprites.mapIndexed { index, sprite -> index to sprite }.filter { it.first !in defeated }.sortedByDescending { hypot(doweRaycastNumber(it.second["x"]) - camera.x, doweRaycastNumber(it.second["y"]) - camera.y) }.forEach { (_, sprite) ->
        val dx = doweRaycastNumber(sprite["x"]) - camera.x
        val dy = doweRaycastNumber(sprite["y"]) - camera.y
        val distance = hypot(dx, dy)
        val relative = atan2(dy, dx) - camera.angle
        if (abs(relative) > camera.fov * 0.7f || distance < 0.2f) return@forEach
        val projected = distance * cos(relative)
        val screenX = width / 2f + tan(relative) / tan(camera.fov / 2f) * width / 2f
        val column = (screenX / width * columns).toInt().coerceIn(0, columns - 1)
        if (depth[column] < projected) return@forEach
        val size = max(5f, height / max(0.1f, projected) * doweRaycastNumber(sprite["size"], 0.75f))
        scope.drawRect(doweCanvasColor(sprite["color"] ?: "#ff4d6d"), topLeft = Offset(screenX - size * 0.32f, height / 2f - size * 0.33f), size = Size(size * 0.64f, size * 0.64f))
        scope.drawCircle(doweCanvasColor(sprite["eye"] ?: "#fff4b8"), radius = size * 0.07f, center = Offset(screenX - size * 0.15f, height / 2f - size * 0.08f))
        scope.drawCircle(doweCanvasColor(sprite["eye"] ?: "#fff4b8"), radius = size * 0.07f, center = Offset(screenX + size * 0.15f, height / 2f - size * 0.08f))
    }
    scope.drawLine(doweCanvasColor("#fff4b8"), Offset(width / 2f - 8f, height / 2f), Offset(width / 2f - 2f, height / 2f), strokeWidth = max(1f, width / 480f))
    scope.drawLine(doweCanvasColor("#fff4b8"), Offset(width / 2f + 2f, height / 2f), Offset(width / 2f + 8f, height / 2f), strokeWidth = max(1f, width / 480f))
    if (fireAt > 0 && System.nanoTime() - fireAt < 140_000_000L) scope.drawRect(doweCanvasColor("#fff4b8").copy(alpha = 0.22f))
}
"##
}
