fn compose_reactive_runtime() -> &'static str {
    r#"
private data class DoweRow(val id: String, val value: Map<String, Any?>)

private data class DoweRequestAction(
    val method: String,
    val path: String,
    val base: String,
    val body: String?,
    val update: String?,
    val reset: String?,
    val successAlert: String?,
    val successMessage: String?,
    val errorAlert: String?,
    val errorMessage: String?
)

private sealed class DoweAction {
    data class Request(val action: DoweRequestAction) : DoweAction()
    data class Assign(val target: String, val source: String) : DoweAction()
    data class Reset(val target: String) : DoweAction()
}

private class DoweReactiveState(
    private val initial: Map<String, Any?>,
    private val actions: Map<String, DoweAction>
) {
    private val values = mutableStateMapOf<String, Any?>().also { it.putAll(initial) }

    fun text(path: String, item: Map<String, Any?>? = null): String =
        read(path, item)?.takeUnless { it === JSONObject.NULL }?.toString() ?: ""

    fun bool(path: String, item: Map<String, Any?>? = null): Boolean =
        read(path, item) as? Boolean ?: false

    fun rows(path: String): List<DoweRow> =
        (read(path) as? List<*>)?.mapIndexedNotNull { index, value ->
            val row = value as? Map<String, Any?> ?: return@mapIndexedNotNull null
            DoweRow(row["id"]?.toString() ?: index.toString(), row)
        } ?: emptyList()

    fun candles(path: String): List<Map<String, Any?>> =
        (read(path) as? List<*>)?.mapNotNull { it as? Map<String, Any?> } ?: emptyList()

    fun upsertCandles(path: String, payload: Any?, maxPoints: Int) {
        val rows = candles(path).toMutableList()
        doweCandlePayloads(payload).filter(::doweValidCandle).forEach { candle ->
            val key = doweCandleKey(candle)
            val index = rows.indexOfFirst { doweCandleKey(it) == key }
            if (index >= 0) {
                rows[index] = candle
            } else {
                rows.add(candle)
            }
        }
        write(path, if (maxPoints > 0 && rows.size > maxPoints) rows.takeLast(maxPoints) else rows)
    }

    fun write(path: String, value: Any?) {
        val parts = path.split(".")
        val root = parts.firstOrNull() ?: return
        if (parts.size == 1) {
            values[root] = value
        } else {
            val objectValue = (values[root] as? Map<String, Any?>)?.toMutableMap() ?: mutableMapOf()
            objectValue[parts[1]] = value
            values[root] = objectValue
        }
    }

    suspend fun run(id: String, item: Map<String, Any?>? = null) {
        when (val action = actions[id]) {
            is DoweAction.Assign -> read(action.source, item)?.let { write(action.target, it) }
            is DoweAction.Reset -> initial[action.target]?.let { write(action.target, it) }
            is DoweAction.Request -> execute(action.action, item)
            null -> {}
        }
    }

    private fun read(path: String, item: Map<String, Any?>? = null): Any? {
        if (path == "item" && item != null) {
            return item
        }
        if (path.startsWith("item.") && item != null) {
            return readMap(path.removePrefix("item."), item)
        }
        return readMap(path, values)
    }

    private fun readMap(path: String, source: Map<String, Any?>): Any? {
        val parts = path.split(".")
        var current: Any? = source[parts.firstOrNull() ?: return null]
        for (part in parts.drop(1)) {
            current = (current as? Map<*, *>)?.get(part) ?: return null
        }
        return current
    }

    private suspend fun execute(action: DoweRequestAction, item: Map<String, Any?>?) {
        val body = action.body?.let { read(it, item) }
        val result = withContext(Dispatchers.IO) {
            val path = if (action.path.contains(":id") && body is Map<*, *> && body["id"] != null) {
                action.path.replace(":id", body["id"].toString())
            } else {
                action.path
            }
            val base = action.base.trimEnd('/')
            val address = if (base.isEmpty()) path else base + if (path.startsWith("/")) path else "/$path"
            try {
                val connection = URL(address).openConnection() as HttpURLConnection
                connection.requestMethod = action.method
                connection.setRequestProperty("Accept", "application/json")
                if (body != null && action.method != "GET") {
                    connection.doOutput = true
                    connection.setRequestProperty("Content-Type", "application/json")
                    connection.outputStream.bufferedWriter().use { it.write(doweJsonValue(body).toString()) }
                }
                val successful = connection.responseCode in 200..299
                val stream = if (successful) connection.inputStream else connection.errorStream
                val payload = stream?.bufferedReader()?.use { JSONObject(it.readText()) } ?: JSONObject()
                val ok = successful && payload.optBoolean("ok", true)
                Pair(ok, if (payload.has("data")) doweNativeValue(payload.get("data")) else doweNativeValue(payload))
            } catch (error: Exception) {
                Pair(false, null)
            }
        }
        if (result.first) {
            action.update?.let { write(it, result.second) }
            action.reset?.let { initial[it]?.let { value -> write(it, value) } }
            setAlert(action.successAlert, "success", action.successMessage ?: "Request completed")
        } else {
            setAlert(action.errorAlert, "error", action.errorMessage ?: "Request failed")
        }
    }

    private fun setAlert(path: String?, type: String, message: String) {
        path?.let { write(it, mapOf("type" to type, "message" to message, "visible" to true)) }
    }
}

private fun doweCandlePayloads(payload: Any?): List<Map<String, Any?>> {
    if (payload is List<*>) {
        return payload.mapNotNull { (it as? Map<*, *>)?.let(::doweStringMap) }
    }
    val objectValue = payload as? Map<*, *> ?: return emptyList()
    val data = objectValue["data"]
    if (data is List<*>) {
        return data.mapNotNull { (it as? Map<*, *>)?.let(::doweStringMap) }
    }
    if (data is Map<*, *>) {
        return listOf(doweStringMap(data))
    }
    return listOf(doweStringMap(objectValue))
}

private fun doweStringMap(value: Map<*, *>): Map<String, Any?> =
    value.entries.associate { it.key.toString() to it.value }

private fun doweValidCandle(value: Map<String, Any?>): Boolean {
    val open = doweCandleNumber(value["open"]) ?: return false
    val high = doweCandleNumber(value["high"]) ?: return false
    val low = doweCandleNumber(value["low"]) ?: return false
    val close = doweCandleNumber(value["close"]) ?: return false
    return doweCandleKey(value) != null && high >= low && high >= open && high >= close && low <= open && low <= close
}

private fun doweCandleKey(value: Map<String, Any?>): String? =
    value["time"]?.toString()

private fun doweCandleNumber(value: Any?): Float? =
    when (value) {
        is Number -> value.toFloat()
        is String -> value.toFloatOrNull()
        else -> null
    }

private fun doweJsonValue(value: Any?): Any =
    when (value) {
        null -> JSONObject.NULL
        is Map<*, *> -> JSONObject(value.entries.associate { it.key.toString() to doweJsonValue(it.value) })
        is List<*> -> JSONArray(value.map(::doweJsonValue))
        else -> value
    }

private fun doweNativeValue(value: Any?): Any? =
    when (value) {
        is JSONObject -> value.keys().asSequence().associateWith { key -> doweNativeValue(value.get(key)) }
        is JSONArray -> (0 until value.length()).map { index -> doweNativeValue(value.get(index)) }
        JSONObject.NULL -> null
        else -> value
    }
"#
}
