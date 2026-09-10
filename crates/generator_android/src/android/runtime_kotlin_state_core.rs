r#"private class DoweReactiveState(
    private val context: android.content.Context,
    private val constants: Map<String, Any?>,
    private val initial: Map<String, Any?>,
    private val signals: Map<String, DoweSignalMetadata>,
    private val actions: Map<String, DoweAction>,
    private val forms: Map<String, List<DoweFormFieldMetadata>> = emptyMap()
) {
    companion object {
        private val globalValues = mutableMapOf<String, Any?>()
        private val globalStorage = mutableMapOf<String, String>()
    }

    private val preferences = context.getSharedPreferences("dowe_view_state", android.content.Context.MODE_PRIVATE)
    var toast by mutableStateOf<DoweToastState?>(null)
        private set
    var redirectPath by mutableStateOf<String?>(null)
        private set
    private var toastSequence = 0L
    private val formTouched = mutableMapOf<String, Boolean>()
    private val values = mutableStateMapOf<String, Any?>().also { state ->
        state.putAll(initial)
        for ((id, metadata) in signals) {
            if (metadata.scope == "global") {
                globalStorage[metadata.name] = metadata.storage
                if (!globalValues.containsKey(metadata.name)) {
                    val stored = storedSignal(metadata)
                    globalValues[metadata.name] = if (stored != null && compatibleSignalValue(stored, initial[id])) stored else initial[id]
                }
                state[id] = globalValues[metadata.name]
            }
        }
    }

    private fun compatibleSignalValue(value: Any?, initial: Any?): Boolean {
        if (initial == null || initial === JSONObject.NULL) return value == null || value === JSONObject.NULL
        if (initial is Map<*, *>) {
            val actual = value as? Map<*, *> ?: return false
            return initial.all { (key, expected) ->
                actual.containsKey(key) && compatibleSignalValue(actual[key], expected)
            }
        }
        if (initial is List<*>) {
            val actual = value as? List<*> ?: return false
            if (initial.isEmpty()) return true
            return actual.all { item -> initial.any { expected -> compatibleSignalValue(item, expected) } }
        }
        return when (initial) {
            is Boolean -> value is Boolean
            is Number -> value is Number
            is String -> value is String
            else -> value?.javaClass == initial.javaClass
        }
    }

    private fun storageKey(name: String): String = "dowe:signal:$name"

    private fun storedSignal(metadata: DoweSignalMetadata): Any? {
        if (metadata.storage != "local") return null
        val raw = preferences.getString(storageKey(metadata.name), null) ?: return null
        return try {
            doweNativeValue(JSONObject(raw).get("value"))
        } catch (error: Exception) {
            null
        }
    }

    private fun persistRoot(root: String) {
        val metadata = signals[root] ?: return
        if (metadata.scope != "global") return
        val value = values[root]
        for ((id, candidate) in signals) {
            if (candidate.scope == "global" && candidate.name == metadata.name) values[id] = value
        }
        globalValues[metadata.name] = value
        if (metadata.storage == "local") {
            preferences.edit().putString(storageKey(metadata.name), JSONObject().put("value", doweJsonValue(value)).toString()).apply()
        }
    }

    fun text(path: String, item: Map<String, Any?>? = null): String =
        read(path, item)?.takeUnless { it === JSONObject.NULL }?.toString() ?: ""

    fun json(path: String, item: Map<String, Any?>? = null): String {
        val current = read(path, item)?.takeUnless { it === JSONObject.NULL } ?: return ""
        return if (current is String) current else doweJsonValue(current).toString()
    }

    fun bool(path: String, item: Map<String, Any?>? = null): Boolean =
        read(path, item) as? Boolean ?: false

    fun rows(path: String): List<DoweRow> =
        (read(path) as? List<*>)?.mapIndexedNotNull { index, value ->
            val row = value as? Map<String, Any?> ?: return@mapIndexedNotNull null
            DoweRow(row["id"]?.toString() ?: index.toString(), row)
        } ?: emptyList()

    fun treeNodes(path: String): List<DoweTreeNode> = treeChildren(read(path)).mapNotNull(::treeNode)

    private fun treeChildren(value: Any?): List<Any?> {
        val list = value as? List<*>
        if (list != null) return list
        val map = value as? Map<*, *> ?: return emptyList()
        val children = map["children"] as? List<*>
        return children ?: ((map["folders"] as? List<*>).orEmpty() + (map["files"] as? List<*>).orEmpty())
    }

    private fun treeNode(value: Any?): DoweTreeNode? {
        val map = value as? Map<*, *> ?: return null
        val rawId = map["id"] ?: map["path"] ?: map["name"] ?: map["label"] ?: return null
        val id = rawId.toString()
        if (id.isEmpty()) return null
        val rawLabel = map["name"] ?: map["label"] ?: return null
        val label = rawLabel.toString()
        if (label.isEmpty()) return null
        val path = (map["path"] ?: id).toString()
        val childValues = treeChildren(value)
        val children = childValues.mapNotNull(::treeNode)
        val kind = (map["type"] ?: map["kind"] ?: "").toString().lowercase()
        val explicitChildren = map["children"] is List<*> || map["folders"] is List<*> || map["files"] is List<*>
        val branch = children.isNotEmpty() || explicitChildren || kind in setOf("folder", "directory", "branch")
        val typed = map.entries.associate { entry -> entry.key.toString() to entry.value }
        return DoweTreeNode(id, label, path, branch, children, typed)
    }

    fun candles(path: String): List<Map<String, Any?>> =
        (read(path) as? List<*>)?.mapNotNull { it as? Map<String, Any?> } ?: emptyList()

    fun canvasValue(path: String): Any? {
        read(path)?.let { return it }
        val parts = path.split(".")
        val id = signals.entries.firstOrNull { it.value.name == parts.firstOrNull() }?.key ?: return null
        return read(listOf(id).plus(parts.drop(1)).joinToString("."))
    }

    fun text(path: String, fallback: String): String = read(path)?.toString()?.takeIf { it.isNotEmpty() } ?: fallback
    fun bool(path: String, fallback: Boolean): Boolean = read(path) as? Boolean ?: fallback

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

    fun appendChatMessage(path: String, text: String) {
        val next = rows(path).map { it.value }.toMutableList()
        next.add(mapOf("id" to "local-${System.currentTimeMillis()}-${next.size}", "role" to "user", "text" to text, "own" to true, "status" to "sent"))
        write(path, next)
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
            touchFormField(path)
        }
        persistRoot(root)
    }

    private fun touchFormField(path: String) {
        forms.forEach { (form, fields) ->
            if (path.startsWith(form + ".") && fields.any { path.removePrefix(form + ".") == it.path }) {
                formTouched[path] = true
            }
        }
    }

"#
