fn android_runtime_diagram() -> &'static str {
    r#"private data class DoweDiagramNode(val id: String, val x: Float, val y: Float, val width: Float, val height: Float, val label: String, val row: Map<String, Any?>)

@Composable
private fun DoweDiagram(state: DoweReactiveState, nodesPath: String, edgesPath: String, fitView: Boolean, panOnDrag: Boolean, zoomOnScroll: Boolean, minimap: Boolean, showGrid: Boolean, emptyLabel: String, onNodeClick: String?, onNodeDrag: String?, onConnect: String?, backgroundColor: Color, contentColor: Color, modifier: Modifier) {
    val actionScope = rememberCoroutineScope()
    var scale by remember { mutableStateOf(1f) }
    var offsetX by remember { mutableStateOf(0f) }
    var offsetY by remember { mutableStateOf(0f) }
    var fitted by remember(nodesPath) { mutableStateOf(!fitView) }
    var dragId by remember { mutableStateOf<String?>(null) }
    var connectFrom by remember { mutableStateOf<String?>(null) }
    val density = LocalDensity.current
    val rows = state.candles(nodesPath)
    val edges = state.candles(edgesPath)
    val nodes = rows.mapNotNull { row ->
        val id = row["id"]?.toString() ?: return@mapNotNull null
        fun number(key: String, fallback: Float): Float = (row[key] as? Number)?.toFloat() ?: fallback
        DoweDiagramNode(id, number("x", 0f), number("y", 0f), number("width", 160f), number("height", 56f), row["label"]?.toString() ?: id, row)
    }.distinctBy { it.id }
    if (!fitted && nodes.isNotEmpty()) {
        fitted = true
        val minX = nodes.minOf { it.x }
        val minY = nodes.minOf { it.y }
        val maxX = nodes.maxOf { it.x + it.width }
        val maxY = nodes.maxOf { it.y + it.height }
        scale = 1f
        offsetX = -minX + 40f
        offsetY = -minY + 40f
        if (maxX - minX > 1200f || maxY - minY > 800f) scale = 0.6f
    }
    fun updateNode(node: DoweDiagramNode, x: Float, y: Float) {
        val updated = rows.map { row ->
            if (row["id"]?.toString() == node.id) HashMap(row).apply { put("x", x); put("y", y) } else row
        }
        state.write(nodesPath, updated)
    }
    Box(modifier = modifier.clip(RoundedCornerShape(12.dp)).background(backgroundColor)) {
        Canvas(
            modifier = Modifier.matchParentSize().pointerInput(nodesPath, edgesPath, panOnDrag, zoomOnScroll) {
                detectTransformGestures { centroid, pan, zoom, _ ->
                    if (zoomOnScroll && zoom != 1f) {
                        val next = (scale * zoom).coerceIn(0.2f, 3f)
                        offsetX = centroid.x - (centroid.x - offsetX) * (next / scale)
                        offsetY = centroid.y - (centroid.y - offsetY) * (next / scale)
                        scale = next
                    } else if (panOnDrag) {
                        offsetX += pan.x
                        offsetY += pan.y
                    }
                }
            }
        ) {
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
            for (edge in state.candles(edgesPath)) {
                val source = edge["source"]?.toString()?.let(byId::get) ?: continue
                val target = edge["target"]?.toString()?.let(byId::get) ?: continue
                fun center(node: DoweDiagramNode): Offset =
                    Offset(offsetX + (node.x + node.width / 2f) * scale, offsetY + (node.y + node.height / 2f) * scale)
                drawLine(contentColor.copy(alpha = 0.45f), center(source), center(target), strokeWidth = 2f)
            }
        }
        for (node in nodes) {
            val left = with(density) { (offsetX + node.x * scale).toDp() }
            val top = with(density) { (offsetY + node.y * scale).toDp() }
            val width = with(density) { (node.width * scale).toDp() }
            val height = with(density) { (node.height * scale).toDp() }
            Text(
                text = node.label,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold,
                color = contentColor,
                textAlign = TextAlign.Center,
                modifier = Modifier
                    .offset(x = left, y = top)
                    .width(width)
                    .height(height)
                    .clip(RoundedCornerShape(10.dp))
                    .background(contentColor.copy(alpha = 0.08f))
                    .border(1.dp, contentColor.copy(alpha = 0.35f), RoundedCornerShape(10.dp))
                    .pointerInput(node.id, nodesPath, scale) {
                        var currentX = node.x
                        var currentY = node.y
                        var connectionX = 0f
                        var connectionY = 0f
                        detectDragGestures(
                            onDragStart = { start ->
                                val portZone = with(density) { 20.dp.toPx() }
                                val localRight = node.width * scale
                                val connecting = start.x >= localRight - portZone
                                dragId = if (connecting) null else node.id
                                connectFrom = if (connecting) node.id else null
                                connectionX = node.x + start.x / scale
                                connectionY = node.y + start.y / scale
                            },
                            onDrag = { change, amount ->
                                change.consume()
                                when {
                                    connectFrom == node.id -> {
                                        connectionX += amount.x / scale
                                        connectionY += amount.y / scale
                                    }
                                    dragId == node.id -> {
                                        currentX += amount.x / scale
                                        currentY += amount.y / scale
                                        updateNode(node, currentX, currentY)
                                    }
                                }
                            },
                            onDragEnd = {
                                if (dragId == node.id && onNodeDrag != null) {
                                    val item = mapOf("id" to node.id, "x" to currentX, "y" to currentY, "label" to node.label)
                                    actionScope.launch { state.run(onNodeDrag, item) }
                                }
                                if (connectFrom == node.id) {
                                    val target = nodes.firstOrNull { candidate ->
                                        val candidateX = candidate.x
                                        val candidateY = candidate.y
                                        val candidateWidth = candidate.width
                                        val candidateHeight = candidate.height
                                        candidate.id != node.id && connectionX >= candidateX && connectionX <= candidateX + candidateWidth && connectionY >= candidateY && connectionY <= candidateY + candidateHeight
                                    }
                                    if (target != null) {
                                        val updatedEdges = edges.toMutableList()
                                        if (updatedEdges.none { it["source"] == node.id && it["target"] == target.id }) {
                                            updatedEdges.add(mapOf("id" to "edge-" + System.currentTimeMillis(), "source" to node.id, "target" to target.id, "type" to "default", "label" to ""))
                                            state.write(edgesPath, updatedEdges)
                                        }
                                        if (onConnect != null) {
                                            val item = mapOf("source" to node.id, "target" to target.id)
                                            actionScope.launch { state.run(onConnect, item) }
                                        }
                                    }
                                }
                                dragId = null
                                connectFrom = null
                            }
                        )
                    }
                    .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) {
                        val action = onNodeClick ?: return@clickable
                        val item = mapOf("id" to node.id, "x" to node.x, "y" to node.y, "label" to node.label)
                        actionScope.launch { state.run(action, item) }
                    }
                    .wrapContentHeight(align = Alignment.CenterVertically)
                    .padding(horizontal = 10.dp)
            )
        }
        if (nodes.isEmpty()) {
            Text(text = emptyLabel, color = contentColor.copy(alpha = 0.64f), fontSize = 13.sp, modifier = Modifier.align(Alignment.Center))
        }
        if (minimap && nodes.isNotEmpty()) {
            Canvas(modifier = Modifier.align(Alignment.TopEnd).padding(8.dp).size(96.dp, 64.dp).clip(RoundedCornerShape(8.dp)).border(1.dp, contentColor.copy(alpha = 0.2f), RoundedCornerShape(8.dp)).background(contentColor.copy(alpha = 0.04f))) {
                if (nodes.isEmpty()) return@Canvas
                val minX = nodes.minOf { it.x }
                val minY = nodes.minOf { it.y }
                val maxX = nodes.maxOf { it.x + it.width }
                val maxY = nodes.maxOf { it.y + it.height }
                val boundsWidth = max(maxX - minX, 1f)
                val boundsHeight = max(maxY - minY, 1f)
                val fit = min(size.width / boundsWidth, size.height / boundsHeight)
                for (node in nodes) {
                    drawRoundRect(
                        color = contentColor.copy(alpha = 0.45f),
                        topLeft = Offset((node.x - minX) * fit + 4f, (node.y - minY) * fit + 4f),
                        size = Size(max(6f, node.width * fit), max(4f, node.height * fit)),
                        cornerRadius = CornerRadius(3f, 3f)
                    )
                }
            }
        }
    }
}
"#
}
