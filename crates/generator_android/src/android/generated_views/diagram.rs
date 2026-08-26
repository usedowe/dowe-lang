fn android_runtime_diagram() -> &'static str {
    r#"private data class DoweDiagramNode(val id: String, val x: Float, val y: Float, val width: Float, val height: Float, val label: String, val row: Map<String, Any?>)

private fun doweDiagramNumber(row: Map<String, Any?>, key: String, fallback: Float): Float = (row[key] as? Number)?.toFloat() ?: row[key]?.toString()?.toFloatOrNull() ?: fallback
private fun doweDiagramNodeWidth(row: Map<String, Any?>): Float = max(1f, doweDiagramNumber(row, "width", 160f))
private fun doweDiagramNodeHeight(row: Map<String, Any?>): Float = max(1f, doweDiagramNumber(row, "height", 56f))

@Composable
private fun DoweDiagram(state: DoweReactiveState, nodesPath: String, edgesPath: String, fitView: Boolean, panOnDrag: Boolean, zoomOnScroll: Boolean, controls: Boolean, minimap: Boolean, showGrid: Boolean, emptyLabel: String, onNodeClick: String?, onNodeDrag: String?, onConnect: String?, backgroundColor: Color, contentColor: Color, modifier: Modifier) {
    val actionScope = rememberCoroutineScope()
    var scale by remember { mutableStateOf(1f) }
    var offsetX by remember { mutableStateOf(0f) }
    var offsetY by remember { mutableStateOf(0f) }
    var fitted by remember(nodesPath) { mutableStateOf(!fitView) }
    var selectedKey by remember { mutableStateOf<String?>(null) }
    var dragId by remember { mutableStateOf<String?>(null) }
    var dragX by remember { mutableStateOf(0f) }
    var dragY by remember { mutableStateOf(0f) }
    var dragMoved by remember { mutableStateOf(false) }
    var connectFrom by remember { mutableStateOf<String?>(null) }
    var connectX by remember { mutableStateOf(0f) }
    var connectY by remember { mutableStateOf(0f) }
    var canvasSize by remember { mutableStateOf(IntSize.Zero) }
    val density = LocalDensity.current
    val rows = state.candles(nodesPath)
    val edges = state.candles(edgesPath)
    val nodes = rows.mapNotNull { row ->
        val id = row["id"]?.toString() ?: return@mapNotNull null
        DoweDiagramNode(id, doweDiagramNumber(row, "x", 0f), doweDiagramNumber(row, "y", 0f), doweDiagramNodeWidth(row), doweDiagramNodeHeight(row), row["label"]?.toString() ?: id, row)
    }.distinctBy { it.id }

    fun effectiveX(node: DoweDiagramNode): Float = if (dragId == node.id) dragX else node.x
    fun effectiveY(node: DoweDiagramNode): Float = if (dragId == node.id) dragY else node.y
    fun screenX(x: Float): Float = offsetX + x * scale
    fun screenY(y: Float): Float = offsetY + y * scale
    fun graphX(x: Float): Float = (x - offsetX) / scale
    fun graphY(y: Float): Float = (y - offsetY) / scale
    fun nodeCenterX(node: DoweDiagramNode): Float = effectiveX(node) + node.width / 2f
    fun nodeCenterY(node: DoweDiagramNode): Float = effectiveY(node) + node.height / 2f
    fun borderPoint(node: DoweDiagramNode, towardX: Float, towardY: Float): Offset {
        val centerX = nodeCenterX(node)
        val centerY = nodeCenterY(node)
        val dx = towardX - centerX
        val dy = towardY - centerY
        if (dx == 0f && dy == 0f) return Offset(centerX, effectiveY(node))
        val sx = if (dx == 0f) Float.MAX_VALUE else (node.width / 2f) / abs(dx)
        val sy = if (dy == 0f) Float.MAX_VALUE else (node.height / 2f) / abs(dy)
        val factor = min(sx, sy)
        return Offset(centerX + dx * factor, centerY + dy * factor)
    }

    fun graphBounds(): FloatArray {
        var minX = Float.MAX_VALUE
        var minY = Float.MAX_VALUE
        var maxX = -Float.MAX_VALUE
        var maxY = -Float.MAX_VALUE
        for (node in nodes) {
            minX = min(minX, effectiveX(node))
            minY = min(minY, effectiveY(node))
            maxX = max(maxX, effectiveX(node) + node.width)
            maxY = max(maxY, effectiveY(node) + node.height)
        }
        return floatArrayOf(minX, minY, maxX, maxY)
    }

    fun fitViewport() {
        if (nodes.isEmpty() || canvasSize == IntSize.Zero) return
        val bounds = graphBounds()
        val graphWidth = max(1f, bounds[2] - bounds[0])
        val graphHeight = max(1f, bounds[3] - bounds[1])
        val padding = with(density) { 40.dp.toPx() }
        val next = min(2.5f, max(0.1f, min((canvasSize.width - padding * 2f) / graphWidth, (canvasSize.height - padding * 2f) / graphHeight)))
        scale = next
        offsetX = (canvasSize.width - graphWidth * next) / 2f - bounds[0] * next
        offsetY = (canvasSize.height - graphHeight * next) / 2f - bounds[1] * next
    }

    fun zoomAtCenter(factor: Float) {
        if (canvasSize == IntSize.Zero) return
        val centerX = canvasSize.width / 2f
        val centerY = canvasSize.height / 2f
        val anchorX = graphX(centerX)
        val anchorY = graphY(centerY)
        val next = min(2.5f, max(0.1f, scale * factor))
        scale = next
        offsetX = centerX - anchorX * next
        offsetY = centerY - anchorY * next
    }

    fun updateNode(node: DoweDiagramNode, x: Float, y: Float) {
        val updated = rows.map { row ->
            if (row["id"]?.toString() == node.id) HashMap(row).apply { put("x", x); put("y", y) } else row
        }
        state.write(nodesPath, updated)
    }

    fun persistConnection(source: String, target: String) {
        val updatedEdges = edges.toMutableList()
        if (updatedEdges.none { it["source"]?.toString() == source && it["target"]?.toString() == target }) {
            updatedEdges.add(mapOf("id" to "edge-" + System.currentTimeMillis(), "source" to source, "target" to target, "type" to "default", "label" to ""))
            state.write(edgesPath, updatedEdges)
        }
    }

    fun nodeAt(x: Float, y: Float): DoweDiagramNode? = nodes.lastOrNull { node ->
        val nodeX = effectiveX(node)
        val nodeY = effectiveY(node)
        x >= nodeX && x <= nodeX + node.width && y >= nodeY && y <= nodeY + node.height
    }

    if (!fitted && nodes.isNotEmpty() && canvasSize != IntSize.Zero) {
        fitted = true
        fitViewport()
    }
    val connectSource = if (connectFrom != null) nodes.firstOrNull { it.id == connectFrom } else null
    val connectTarget = if (connectSource != null) nodeAt(connectX, connectY)?.takeIf { it.id != connectSource.id } else null

    Box(modifier = modifier.clip(RoundedCornerShape(12.dp)).background(backgroundColor).onSizeChanged { canvasSize = it }) {
        Canvas(modifier = Modifier.matchParentSize().pointerInput(nodesPath, edgesPath, panOnDrag, zoomOnScroll) {
            detectTransformGestures { centroid, pan, zoom, _ ->
                if (zoomOnScroll && zoom != 1f) {
                    val next = (scale * zoom).coerceIn(0.1f, 2.5f)
                    offsetX = centroid.x - (centroid.x - offsetX) * (next / scale)
                    offsetY = centroid.y - (centroid.y - offsetY) * (next / scale)
                    scale = next
                } else if (panOnDrag) {
                    offsetX += pan.x
                    offsetY += pan.y
                }
            }
        }) {
            if (showGrid) {
                val step = 28.dp.toPx() * scale
                if (step > 6f) {
                    var gridX = offsetX % step
                    while (gridX < size.width) {
                        drawLine(contentColor.copy(alpha = 0.08f), Offset(gridX, 0f), Offset(gridX, size.height), strokeWidth = 1f)
                        gridX += step
                    }
                    var gridY = offsetY % step
                    while (gridY < size.height) {
                        drawLine(contentColor.copy(alpha = 0.08f), Offset(0f, gridY), Offset(size.width, gridY), strokeWidth = 1f)
                        gridY += step
                    }
                }
            }
            val byId = nodes.associateBy { it.id }
            val textPx = 11.sp.toPx()
            for (edge in edges) {
                val source = edge["source"]?.toString()?.let(byId::get) ?: continue
                val target = edge["target"]?.toString()?.let(byId::get) ?: continue
                val type = edge["type"]?.toString() ?: "default"
                val from = borderPoint(source, nodeCenterX(target), nodeCenterY(target))
                val to = borderPoint(target, nodeCenterX(source), nodeCenterY(source))
                val path = Path()
                val labelPoint: Offset
                if (type == "straight") {
                    path.moveTo(screenX(from.x), screenY(from.y))
                    path.lineTo(screenX(to.x), screenY(to.y))
                    labelPoint = Offset((screenX(from.x) + screenX(to.x)) / 2f, (screenY(from.y) + screenY(to.y)) / 2f)
                } else if (type == "step") {
                    val midX = (screenX(from.x) + screenX(to.x)) / 2f
                    path.moveTo(screenX(from.x), screenY(from.y))
                    path.lineTo(midX, screenY(from.y))
                    path.lineTo(midX, screenY(to.y))
                    path.lineTo(screenX(to.x), screenY(to.y))
                    labelPoint = Offset(midX, (screenY(from.y) + screenY(to.y)) / 2f)
                } else {
                    val dx = max(40f * scale, abs(screenX(to.x) - screenX(from.x)) / 2f)
                    val c1X = screenX(from.x) + dx
                    val c2X = screenX(to.x) - dx
                    path.moveTo(screenX(from.x), screenY(from.y))
                    path.cubicTo(c1X, screenY(from.y), c2X, screenY(to.y), screenX(to.x), screenY(to.y))
                    labelPoint = Offset(
                        (screenX(from.x) + 3f * c1X + 3f * c2X + screenX(to.x)) / 8f,
                        (screenY(from.y) + 3f * screenY(from.y) + 3f * screenY(to.y) + screenY(to.y)) / 8f
                    )
                }
                val isSelected = selectedKey == "edge:" + edge["id"]?.toString()
                drawPath(path, contentColor.copy(alpha = if (isSelected) 1f else 0.45f), style = Stroke(width = if (isSelected) 2.5f else 2f))
                val label = edge["label"]?.toString()
                if (!label.isNullOrEmpty()) {
                    drawIntoCanvas { canvas ->
                        val paint = Paint().apply { textSize = textPx; textAlign = Paint.Align.CENTER; isAntiAlias = true; color = contentColor.toArgb() }
                        canvas.nativeCanvas.drawText(label, labelPoint.x, labelPoint.y - 6f * scale, paint)
                    }
                }
            }
            if (connectSource != null) {
                val from = borderPoint(connectSource, connectX, connectY)
                val fromX = screenX(from.x)
                val fromY = screenY(from.y)
                val toX = screenX(connectX)
                val toY = screenY(connectY)
                val dx = max(40f * scale, abs(toX - fromX) / 2f)
                val path = Path()
                path.moveTo(fromX, fromY)
                path.cubicTo(fromX + dx, fromY, toX - dx, toY, toX, toY)
                drawPath(path, contentColor.copy(alpha = 0.9f), style = Stroke(width = 2f, pathEffect = PathEffect.dashPathEffect(floatArrayOf(6f, 4f))))
            }
        }
        for (node in nodes) {
            val left = with(density) { screenX(effectiveX(node)).toDp() }
            val top = with(density) { screenY(effectiveY(node)).toDp() }
            val width = with(density) { (node.width * scale).toDp() }
            val height = with(density) { (node.height * scale).toDp() }
            val isSelected = selectedKey == "node:" + node.id
            val isTarget = connectTarget?.id == node.id
            Box(
                modifier = Modifier
                    .offset(x = left, y = top)
                    .size(width, height)
                    .clip(RoundedCornerShape(10.dp))
                    .background(contentColor.copy(alpha = if (isSelected) 0.16f else 0.08f))
                    .border(if (isTarget) 2.dp else if (isSelected) 2.dp else 1.dp, contentColor.copy(alpha = if (isTarget || isSelected) 1f else 0.35f), RoundedCornerShape(10.dp))
                    .pointerInput(node.id, nodesPath, scale) {
                        detectDragGestures(
                            onDragStart = {
                                if (dragId == null) {
                                    dragId = node.id
                                    dragX = node.x
                                    dragY = node.y
                                    dragMoved = false
                                }
                            },
                            onDrag = { change, amount ->
                                change.consume()
                                if (dragId == node.id) {
                                    dragMoved = true
                                    dragX += amount.x / scale
                                    dragY += amount.y / scale
                                }
                            },
                            onDragEnd = {
                                if (dragId == node.id) {
                                    val x = dragX
                                    val y = dragY
                                    if (dragMoved) {
                                        updateNode(node, x, y)
                                        selectedKey = "node:" + node.id
                                        if (onNodeDrag != null) {
                                            val item = HashMap(node.row)
                                            item["x"] = x
                                            item["y"] = y
                                            actionScope.launch { state.run(onNodeDrag, item) }
                                        }
                                    } else {
                                        selectedKey = "node:" + node.id
                                        if (onNodeClick != null) {
                                            val item = HashMap(node.row)
                                            actionScope.launch { state.run(onNodeClick, item) }
                                        }
                                    }
                                    dragId = null
                                    dragMoved = false
                                }
                            },
                            onDragCancel = {
                                if (dragId == node.id) {
                                    dragId = null
                                    dragMoved = false
                                }
                            }
                        )
                    },
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = node.label,
                    fontSize = (13f * scale).sp,
                    fontWeight = FontWeight.SemiBold,
                    color = contentColor,
                    textAlign = TextAlign.Center,
                    maxLines = 1,
                    modifier = Modifier.padding(horizontal = 8.dp)
                )
                Box(
                    modifier = Modifier
                        .align(Alignment.CenterEnd)
                        .offset(x = 8.dp)
                        .size(16.dp)
                        .pointerInput(node.id, nodesPath, scale) {
                            detectDragGestures(
                                onDragStart = {
                                    connectFrom = node.id
                                    connectX = node.x + node.width
                                    connectY = node.y + node.height / 2f
                                },
                                onDrag = { change, amount ->
                                    change.consume()
                                    connectX += amount.x / scale
                                    connectY += amount.y / scale
                                },
                                onDragEnd = {
                                    val source = connectFrom
                                    val target = nodeAt(connectX, connectY)?.takeIf { it.id != source }
                                    if (source != null && target != null) {
                                        persistConnection(source, target.id)
                                        if (onConnect != null) {
                                            val item = mapOf("source" to source, "target" to target.id)
                                            actionScope.launch { state.run(onConnect, item) }
                                        }
                                    }
                                    connectFrom = null
                                },
                                onDragCancel = { connectFrom = null }
                            )
                        },
                    contentAlignment = Alignment.Center
                ) {
                    Box(modifier = Modifier.size(10.dp).clip(CircleShape).background(contentColor).border(2.dp, backgroundColor, CircleShape))
                }
            }
        }
        if (nodes.isEmpty()) {
            Text(text = emptyLabel, color = contentColor.copy(alpha = 0.64f), fontSize = 13.sp, modifier = Modifier.align(Alignment.Center))
        }
        if (minimap && nodes.isNotEmpty()) {
            Canvas(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(10.dp)
                    .size(120.dp, 80.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(backgroundColor.copy(alpha = 0.9f))
                    .border(1.dp, contentColor.copy(alpha = 0.2f), RoundedCornerShape(8.dp))
                    .pointerInput(nodesPath, edgesPath, scale, offsetX, offsetY, canvasSize) {
                        val minimapWidth = with(density) { 120.dp.toPx() }
                        val minimapHeight = with(density) { 80.dp.toPx() }
                        fun moveViewport(point: Offset) {
                            if (nodes.isEmpty()) return
                            val bounds = graphBounds()
                            val graphWidth = max(1f, bounds[2] - bounds[0])
                            val graphHeight = max(1f, bounds[3] - bounds[1])
                            val padding = 8.dp.toPx()
                            val fit = min((minimapWidth - padding * 2f) / graphWidth, (minimapHeight - padding * 2f) / graphHeight)
                            val graphPointX = (point.x - padding) / fit + bounds[0]
                            val graphPointY = (point.y - padding) / fit + bounds[1]
                            offsetX = canvasSize.width / 2f - graphPointX * scale
                            offsetY = canvasSize.height / 2f - graphPointY * scale
                        }
                        detectDragGestures(
                            onDragStart = { moveViewport(it) },
                            onDrag = { change, _ -> moveViewport(change.position) }
                        )
                    }
            ) {
                if (nodes.isEmpty()) return@Canvas
                val bounds = graphBounds()
                val graphWidth = max(1f, bounds[2] - bounds[0])
                val graphHeight = max(1f, bounds[3] - bounds[1])
                val padding = 8.dp.toPx()
                val fit = min((size.width - padding * 2f) / graphWidth, (size.height - padding * 2f) / graphHeight)
                for (node in nodes) {
                    drawRoundRect(
                        color = contentColor.copy(alpha = 0.45f),
                        topLeft = Offset((effectiveX(node) - bounds[0]) * fit + padding, (effectiveY(node) - bounds[1]) * fit + padding),
                        size = Size(max(3f, node.width * fit), max(2f, node.height * fit)),
                        cornerRadius = CornerRadius(2f, 2f)
                    )
                }
                val viewLeft = (-offsetX / scale - bounds[0]) * fit + padding
                val viewTop = (-offsetY / scale - bounds[1]) * fit + padding
                drawRect(
                    color = contentColor.copy(alpha = 0.12f),
                    topLeft = Offset(viewLeft, viewTop),
                    size = Size(canvasSize.width / scale * fit, canvasSize.height / scale * fit)
                )
                drawRect(
                    color = contentColor.copy(alpha = 0.8f),
                    topLeft = Offset(viewLeft, viewTop),
                    size = Size(canvasSize.width / scale * fit, canvasSize.height / scale * fit),
                    style = Stroke(width = 1f)
                )
            }
        }
        if (controls) {
            Column(modifier = Modifier.align(Alignment.BottomEnd).padding(10.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                DoweDiagramControlButton(label = "+", backgroundColor = backgroundColor, contentColor = contentColor) { zoomAtCenter(1.2f) }
                DoweDiagramControlButton(label = "−", backgroundColor = backgroundColor, contentColor = contentColor) { zoomAtCenter(1f / 1.2f) }
                DoweDiagramControlButton(label = "⤢", backgroundColor = backgroundColor, contentColor = contentColor) { fitViewport() }
            }
        }
    }
}

@Composable
private fun DoweDiagramControlButton(label: String, backgroundColor: Color, contentColor: Color, action: () -> Unit) {
    Box(
        modifier = Modifier
            .size(26.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(backgroundColor.copy(alpha = 0.95f))
            .border(1.dp, contentColor.copy(alpha = 0.15f), RoundedCornerShape(8.dp))
            .clickable { action() },
        contentAlignment = Alignment.Center
    ) {
        Text(text = label, color = contentColor, fontSize = 14.sp, fontWeight = FontWeight.Bold)
    }
}
"#
}
