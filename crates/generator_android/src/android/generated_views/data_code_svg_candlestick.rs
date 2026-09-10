r#"@Composable
private fun DoweCandlestick(state: DoweReactiveState, dataPath: String, stream: String?, upColor: Color, downColor: Color, emptyLabel: String, maxPoints: Int, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val candles = state.candles(dataPath).takeLast(maxPoints).mapIndexedNotNull { index, value -> doweCandlestickCandle(value, index) }
    LaunchedEffect(stream, dataPath, maxPoints) {
        doweConnectCandlestickStream(stream, dataPath, maxPoints, state)
    }
    Box(
        modifier = modifier
            .heightIn(min = 220.dp)
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier.border(1.dp, contentColor.copy(alpha = 0.12f), shape) else Modifier.border(1.dp, borderColor, shape)),
        contentAlignment = Alignment.Center
    ) {
        Canvas(modifier = Modifier.matchParentSize()) {
            if (candles.isEmpty()) {
                return@Canvas
            }
            val top = 12f
            val right = 12f
            val bottom = 18f
            val left = 12f
            val drawingWidth = max(1f, size.width - left - right)
            val drawingHeight = max(1f, size.height - top - bottom)
            val high = candles.maxOf { it.high }
            val low = candles.minOf { it.low }
            val range = max(high - low, 0.000001f)
            val step = drawingWidth / max(candles.size, 1)
            val bodyWidth = max(3f, min(12f, step * 0.56f))
            for (line in 0..3) {
                val y = top + drawingHeight * line / 3f
                drawLine(
                    color = contentColor.copy(alpha = 0.1f),
                    start = Offset(left, y),
                    end = Offset(left + drawingWidth, y),
                    strokeWidth = 1f
                )
            }
            candles.forEachIndexed { index, candle ->
                fun candleY(value: Float): Float = top + drawingHeight * ((high - value) / range)
                val centerX = left + step * (index + 0.5f)
                val highY = candleY(candle.high)
                val lowY = candleY(candle.low)
                val openY = candleY(candle.open)
                val closeY = candleY(candle.close)
                val color = if (candle.close >= candle.open) upColor else downColor
                drawLine(
                    color = color,
                    start = Offset(centerX, highY),
                    end = Offset(centerX, lowY),
                    strokeWidth = 1.4f
                )
                drawRoundRect(
                    color = color,
                    topLeft = Offset(centerX - bodyWidth / 2f, min(openY, closeY)),
                    size = Size(bodyWidth, max(1f, abs(closeY - openY))),
                    cornerRadius = CornerRadius(1.5f, 1.5f)
                )
            }
        }
        if (candles.isEmpty()) {
            Text(text = emptyLabel, color = contentColor.copy(alpha = 0.64f), fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
        }
    }
}

private fun doweCandlestickCandle(source: Map<String, Any?>, index: Int): DoweCandlestickCandle? {
    val time = source["time"]?.toString() ?: return null
    val open = doweCandleNumber(source["open"]) ?: return null
    val high = doweCandleNumber(source["high"]) ?: return null
    val low = doweCandleNumber(source["low"]) ?: return null
    val close = doweCandleNumber(source["close"]) ?: return null
    return DoweCandlestickCandle("$time-$index", time, open, high, low, close)
}

private suspend fun doweConnectCandlestickStream(stream: String?, dataPath: String, maxPoints: Int, state: DoweReactiveState) {
    val address = doweCandlestickStreamUrl(stream) ?: return
    withContext(Dispatchers.IO) {
        try {
            val connection = URL(address).openConnection() as HttpURLConnection
            connection.setRequestProperty("Accept", "text/event-stream")
            connection.inputStream.bufferedReader().use { reader ->
                while (true) {
                    val line = reader.readLine() ?: break
                    val payloadText = doweCandlestickStreamPayload(line)
                    if (payloadText.isEmpty()) {
                        continue
                    }
                    if (payloadText == "[DONE]") {
                        break
                    }
                    val payload = doweCandlestickJson(payloadText) ?: continue
                    withContext(Dispatchers.Main) {
                        state.upsertCandles(dataPath, payload, maxPoints)
                    }
                }
            }
        } catch (error: Exception) {
        }
    }
}

private fun doweCandlestickStreamPayload(line: String): String {
    val text = line.trim()
    return if (text.startsWith("data:")) text.removePrefix("data:").trim() else text
}

private fun doweCandlestickJson(text: String): Any? =
    try {
        when {
            text.startsWith("[") -> doweNativeValue(JSONArray(text))
            text.startsWith("{") -> doweNativeValue(JSONObject(text))
            else -> null
        }
    } catch (error: Exception) {
        null
    }

private fun doweCandlestickStreamUrl(stream: String?): String? {
    if (stream.isNullOrEmpty()) {
        return null
    }
    if (stream.startsWith("https://")) {
        return stream
    }
    if (stream.startsWith("/")) {
        val base = DoweEnvironment.BACKEND_URL.trimEnd('/')
        if (base.isEmpty()) {
            return null
        }
        return base + stream
    }
    return null
}
"#
