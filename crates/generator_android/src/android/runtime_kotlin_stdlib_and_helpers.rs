r#"        param?.let { builder.appendQueryParameter(name, stdlibText(it)) }
        builder.build().toString()
    }.getOrDefault(value)

    private fun stdlibCsvParse(value: String, delimiter: String, header: Boolean, maxRows: Int, maxColumns: Int): Map<String, Any?> {
        val separator = delimiter.firstOrNull() ?: ','
        val parsed = mutableListOf<List<String>>()
        var row = mutableListOf<String>()
        var cell = StringBuilder()
        var quoted = false
        var index = 0
        var truncated = false
        fun finishCell() {
            row.add(cell.toString())
            cell = StringBuilder()
        }
        fun finishRow() {
            finishCell()
            if (parsed.size < maxOf(0, maxRows)) parsed.add(row.take(maxOf(0, maxColumns))) else truncated = true
            row = mutableListOf()
        }
        while (index < value.length) {
            val character = value[index]
            when {
                character == '"' && quoted && index + 1 < value.length && value[index + 1] == '"' -> {
                    cell.append('"')
                    index += 1
                }
                character == '"' -> quoted = !quoted
                !quoted && character == separator -> finishCell()
                !quoted && (character == '\n' || character == '\r') -> {
                    finishRow()
                    if (character == '\r' && index + 1 < value.length && value[index + 1] == '\n') index += 1
                }
                else -> cell.append(character)
            }
            index += 1
        }
        if (cell.isNotEmpty() || row.isNotEmpty()) finishRow()
        val columns = if (header && parsed.isNotEmpty()) parsed.first() else (0 until (parsed.maxOfOrNull { it.size } ?: 0)).map { "column${it + 1}" }
        val rows = if (header && parsed.isNotEmpty()) {
            parsed.drop(1).map { values -> columns.mapIndexed { position, key -> key to values.getOrNull(position).orEmpty() }.toMap() }
        } else parsed.map { it }
        return mapOf("rows" to rows, "columns" to columns, "errors" to emptyList<Any>(), "truncated" to truncated, "rowCount" to rows.size)
    }

    private fun stdlibCsvStringify(rows: List<Any?>, delimiter: String): String {
        val separator = delimiter.firstOrNull() ?: ','
        val columns = rows.firstOrNull()?.let { (it as? Map<*, *>)?.keys?.map { key -> key.toString() }?.sorted() } ?: emptyList()
        fun escape(value: Any?): String {
            val text = stdlibText(value)
            return if (text.any { it == separator || it == '"' || it == '\n' || it == '\r' }) "\"${text.replace("\"", "\"\"")}\"" else text
        }
        return rows.joinToString("\n") { row ->
            when (row) {
                is Map<*, *> -> columns.joinToString(separator.toString()) { key -> escape(row[key]) }
                is List<*> -> row.joinToString(separator.toString(), transform = ::escape)
                else -> escape(row)
            }
        }
    }

    private fun stdlibSort(values: List<Any?>, field: String?, descending: Boolean, nulls: String): List<Any?> = values.withIndex().sortedWith(Comparator { left, right ->
        val leftValue = field?.let { stdlibRead(left.value, it) } ?: left.value
        val rightValue = field?.let { stdlibRead(right.value, it) } ?: right.value
        val leftNull = leftValue == null
        val rightNull = rightValue == null
        if (leftNull || rightNull) {
            if (leftNull && rightNull) left.index - right.index else if (leftNull == (nulls != "first")) 1 else -1
        } else {
            val order = stdlibText(leftValue).compareTo(stdlibText(rightValue))
            if (order == 0) left.index - right.index else if (descending) -order else order
        }
    }).map { it.value }

    private fun stdlibJsonSet(value: Any?, path: String, next: Any?): Any? {
        val result = (value as? Map<*, *>).orEmpty().entries.associate { it.key.toString() to it.value }.toMutableMap()
        val parts = path.split('.').filter { it.isNotEmpty() }
        if (parts.isEmpty()) return next
        var current = result
        for (part in parts.dropLast(1)) {
            val child = (current[part] as? Map<*, *>).orEmpty().entries.associate { it.key.toString() to it.value }.toMutableMap()
            current[part] = child
            current = child
        }
        current[parts.last()] = next
        return result
    }

    private fun stdlibJsonPick(value: Any?, fields: List<String>): Map<String, Any?> = fields.mapNotNull { field -> ((value as? Map<*, *>)?.get(field))?.let { field to it } }.toMap()

    private fun stdlibJsonOmit(value: Any?, fields: List<String>): Map<String, Any?> = (value as? Map<*, *>).orEmpty().entries.filterNot { it.key.toString() in fields }.associate { it.key.toString() to it.value }

    private fun stdlibJsonStringify(value: Any?, pretty: Boolean): String = when (value) {
        null -> "null"
        is String -> JSONObject.quote(value)
        is Number, is Boolean -> value.toString()
        else -> {
            val json = doweJsonValue(value)
            when {
                pretty && json is JSONObject -> json.toString(2)
                pretty && json is JSONArray -> json.toString(2)
                else -> json.toString()
            }
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

    private fun finiteCompare(left: Double?, right: Double?, op: (Double, Double) -> Boolean): Boolean? {
        return if (left == null || right == null) null else op(left, right)
    }

    private suspend fun execute(action: DoweRequestAction, item: Map<String, Any?>?) {
        val result = request(action, item)
        if (result.first) {
            action.update?.let { write(it, result.second) }
            action.reset?.let { initial[it]?.let { value -> write(it, value) } }
            setAlert(action.successAlert, "success", action.successMessage ?: "Request completed")
        } else {
            setAlert(action.errorAlert, "error", action.errorMessage ?: "Request failed")
        }
    }

    private suspend fun execute(action: DoweInvokeAction, item: Map<String, Any?>?) {
        val result = invoke(action, item)
        if (result.first) {
            action.update?.let { write(it, result.second) }
            action.reset?.let { initial[it]?.let { value -> write(it, value) } }
            setAlert(action.successAlert, "success", action.successMessage ?: "Invocation completed")
        } else {
            setAlert(action.errorAlert, "error", action.errorMessage ?: "Invocation failed")
        }
    }

    private suspend fun invoke(action: DoweInvokeAction, item: Map<String, Any?>?): Pair<Boolean, Any?> = withContext(Dispatchers.IO) {
        try {
            val args = action.args.associate { it.name to stdlibValue(it.value, item) }
            DoweNativeBridge.invoke(action.function, args)
        } catch (error: Exception) {
            Pair(false, null)
        }
    }

    private suspend fun request(action: DoweRequestAction, item: Map<String, Any?>?): Pair<Boolean, Any?> {
        val body = action.body?.let { read(it, item) }
        val path = requestPath(action.path, body, item)
        return withContext(Dispatchers.IO) {
            val base = action.base.trimEnd('/')
            val address = if (base.isEmpty()) path else base + if (path.startsWith("/")) path else "/$path"
            try {
                val connection = URL(address).openConnection() as HttpURLConnection
                connection.requestMethod = action.method
                connection.setRequestProperty("Accept", "application/json")
                for (header in action.headers) {
                    val rawValue = if (header.second == "signal") text(header.third, item) else header.third
                    if (rawValue.isNotEmpty()) {
                        connection.setRequestProperty(header.first, rawValue)
                    }
                }
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
    }

    private fun requestPath(path: String, body: Any?, item: Map<String, Any?>?): String =
        Regex(":([A-Za-z_][A-Za-z0-9_]*)").replace(path) { match ->
            val name = match.groupValues[1]
            val fromBody = (body as? Map<*, *>)?.get(name)
            val signal = signals.entries.lastOrNull { it.value.name == name }?.key
            val value = fromBody ?: signal?.let { read(it, item) } ?: read(name, item)
            java.net.URLEncoder.encode(value?.toString().orEmpty(), Charsets.UTF_8.name())
                .replace("+", "%20")
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
