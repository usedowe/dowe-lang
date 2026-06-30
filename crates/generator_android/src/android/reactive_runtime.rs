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
    data class Assign(val target: String, val source: String, val call: DoweStdlibCall? = null) : DoweAction()
    data class Reset(val target: String) : DoweAction()
}

private data class DoweStdlibCall(val namespace: String, val function: String, val args: List<DoweStdlibArg>)
private data class DoweStdlibArg(val name: String, val value: DoweStdlibValue)
private data class DoweStdlibValue(val kind: String, val value: Any?)

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
            is DoweAction.Assign -> write(action.target, action.call?.let { stdlib(it, item) } ?: read(action.source, item))
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

    private fun stdlib(call: DoweStdlibCall, item: Map<String, Any?>?): Any? {
        val args = call.args.associate { it.name to stdlibValue(it.value, item) }
        fun text(name: String): String = stdlibText(args[name])
        fun number(name: String): Double? = stdlibNumber(args[name])
        fun list(name: String): List<Any?> = args[name] as? List<Any?> ?: emptyList()
        return when (call.namespace + "." + call.function) {
            "str.trim" -> text("value").trim()
            "str.lower" -> text("value").lowercase()
            "str.upper" -> text("value").uppercase()
            "str.length" -> text("value").codePointCount(0, text("value").length)
            "str.contains" -> text("value").contains(text("needle"))
            "str.startsWith" -> text("value").startsWith(text("prefix"))
            "str.endsWith" -> text("value").endsWith(text("suffix"))
            "str.replace" -> text("value").replace(text("from"), text("to"))
            "str.split" -> text("value").split(text("delimiter"))
            "str.join" -> list("values").joinToString(text("delimiter")) { stdlibText(it) }
            "math.add" -> finite(number("left"), number("right")) { left, right -> left + right }
            "math.sub" -> finite(number("left"), number("right")) { left, right -> left - right }
            "math.mul" -> finite(number("left"), number("right")) { left, right -> left * right }
            "math.div" -> finite(number("left"), number("right")) { left, right -> if (right == 0.0) null else left / right }
            "math.round" -> number("value")?.let { kotlin.math.round(it) }
            "math.floor" -> number("value")?.let { kotlin.math.floor(it) }
            "math.ceil" -> number("value")?.let { kotlin.math.ceil(it) }
            "math.abs" -> number("value")?.let { kotlin.math.abs(it) }
            "math.sum" -> list("values").mapNotNull(::stdlibNumber).sum()
            "math.average" -> list("values").mapNotNull(::stdlibNumber).takeIf { it.isNotEmpty() }?.average()
            "math.min" -> list("values").mapNotNull(::stdlibNumber).minOrNull()
            "math.max" -> list("values").mapNotNull(::stdlibNumber).maxOrNull()
            "parse.int" -> text("value").trim().toLongOrNull() ?: args["fallback"]
            "parse.float" -> number("value") ?: args["fallback"]
            "parse.bool" -> stdlibBool(args["value"]) ?: args["fallback"]
            "parse.string" -> stdlibText(args["value"])
            "parse.json", "json.parse" -> runCatching { doweNativeValue(JSONObject(text("value"))) }.getOrDefault(args["fallback"])
            "sort.asc" -> list("values").sortedBy { stdlibText(it) }
            "sort.desc" -> list("values").sortedByDescending { stdlibText(it) }
            "sort.by" -> list("values").sortedBy { stdlibText(stdlibRead(it, text("field"))) }
            "list.take" -> list("values").take(number("count")?.toInt() ?: 0)
            "list.skip" -> list("values").drop(number("count")?.toInt() ?: 0)
            "list.first" -> list("values").firstOrNull()
            "list.last" -> list("values").lastOrNull()
            "list.count" -> list("values").size
            "list.filterEquals" -> list("values").filter { stdlibRead(it, text("field")) == args["value"] }
            "list.filterContains" -> list("values").filter { stdlibText(stdlibRead(it, text("field"))).lowercase().contains(text("value").lowercase()) }
            "list.mapField" -> list("values").map { stdlibRead(it, text("field")) }
            "list.sumBy" -> list("values").mapNotNull { stdlibNumber(stdlibRead(it, text("field"))) }.sum()
            "list.averageBy" -> list("values").mapNotNull { stdlibNumber(stdlibRead(it, text("field"))) }.takeIf { it.isNotEmpty() }?.average()
            "json.get" -> stdlibRead(args["value"], text("path")) ?: args["fallback"]
            "json.stringify" -> doweJsonValue(args["value"]).toString()
            "json.merge" -> (args["left"] as? Map<String, Any?>).orEmpty() + (args["right"] as? Map<String, Any?>).orEmpty()
            "date.now" -> java.time.Instant.now().toString()
            "date.formatIso" -> runCatching { java.time.Instant.parse(text("value")).toString() }.getOrDefault(text("value"))
            "date.addDays" -> runCatching { java.time.Instant.parse(text("value")).plus(java.time.Duration.ofDays(number("days")?.toLong() ?: 0)).toString() }.getOrNull()
            "date.diffDays" -> runCatching { java.time.Duration.between(java.time.Instant.parse(text("start")), java.time.Instant.parse(text("end"))).toDays() }.getOrDefault(0L)
            else -> null
        }
    }

    private fun stdlibValue(value: DoweStdlibValue, item: Map<String, Any?>?): Any? = when (value.kind) {
        "null" -> null
        "bool" -> value.value as? Boolean
        "number" -> stdlibNumber(value.value)
        "string" -> value.value?.toString() ?: ""
        "reference" -> read(value.value?.toString() ?: "", item)
        "array" -> (value.value as? List<DoweStdlibValue>).orEmpty().map { stdlibValue(it, item) }
        "object" -> (value.value as? List<Pair<String, DoweStdlibValue>>).orEmpty().associate { it.first to stdlibValue(it.second, item) }
        else -> null
    }

    private fun stdlibText(value: Any?): String = when (value) {
        null -> ""
        is String -> value
        is JSONObject, is JSONArray -> value.toString()
        else -> value.toString()
    }

    private fun stdlibNumber(value: Any?): Double? = when (value) {
        is Number -> value.toDouble().takeIf { it.isFinite() }
        is String -> value.trim().toDoubleOrNull()?.takeIf { it.isFinite() }
        else -> null
    }

    private fun stdlibBool(value: Any?): Boolean? = when (value) {
        is Boolean -> value
        else -> when (stdlibText(value).trim().lowercase()) {
            "true", "1", "yes", "y" -> true
            "false", "0", "no", "n" -> false
            else -> null
        }
    }

    private fun stdlibRead(value: Any?, path: String): Any? {
        var current = value
        for (part in path.split(".").filter { it.isNotEmpty() }) {
            current = (current as? Map<*, *>)?.get(part) ?: return null
        }
        return current
    }

    private fun finite(left: Double?, right: Double?, op: (Double, Double) -> Double?): Double? {
        val result = if (left == null || right == null) null else op(left, right)
        return result?.takeIf { it.isFinite() }
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
