fn compose_reactive_runtime() -> &'static str {
    r#"
private data class DoweRow(val id: String, val value: Map<String, Any?>)

private data class DoweRequestAction(
    val method: String,
    val path: String,
    val base: String,
    val headers: List<Triple<String, String, String>>,
    val body: String?,
    val update: String?,
    val reset: String?,
    val successAlert: String?,
    val successMessage: String?,
    val errorAlert: String?,
    val errorMessage: String?
)

private data class DoweActionMetadata(
    val params: Map<String, String>,
    val returnType: String?
)

private sealed class DoweAction {
    data class Request(val action: DoweRequestAction, val metadata: DoweActionMetadata) : DoweAction()
    data class Assign(val target: String, val source: String, val call: DoweStdlibCall?, val metadata: DoweActionMetadata) : DoweAction()
    data class Reset(val target: String, val metadata: DoweActionMetadata) : DoweAction()
    data class Sequence(val steps: List<DoweStep>, val metadata: DoweActionMetadata) : DoweAction()
}

private sealed class DoweStep {
    data class Validate(val target: String) : DoweStep()
    data class Request(val result: String, val action: DoweRequestAction) : DoweStep()
    data class Branch(val result: String, val success: List<DoweStep>, val error: List<DoweStep>) : DoweStep()
    data class Assign(val target: String, val source: String, val literal: Any?, val hasLiteral: Boolean, val call: DoweStdlibCall?) : DoweStep()
    data class Reset(val target: String) : DoweStep()
    data class Toast(val kind: String, val title: String, val message: String, val duration: Int?, val scheme: String?, val variant: String?, val position: String?) : DoweStep()
    data class Redirect(val path: String) : DoweStep()
}

private data class DoweToastState(val id: Long, val kind: String, val title: String, val message: String, val duration: Int, val scheme: String, val variant: String, val position: String)

private data class DoweStdlibCall(val namespace: String, val function: String, val args: List<DoweStdlibArg>)
private data class DoweStdlibArg(val name: String, val value: DoweStdlibValue)
private data class DoweStdlibValue(val kind: String, val value: Any?)
private data class DoweSignalMetadata(val name: String, val scope: String, val storage: String)
private data class DoweFormFieldMetadata(val path: String, val kind: String, val rules: List<DoweValidationRule>)

private data class DoweSvgImportMatrix(val a: Double, val b: Double, val c: Double, val d: Double, val e: Double, val f: Double) {
    fun multiply(next: DoweSvgImportMatrix) = DoweSvgImportMatrix(
        a * next.a + c * next.b,
        b * next.a + d * next.b,
        a * next.c + c * next.d,
        b * next.c + d * next.d,
        a * next.e + c * next.f + e,
        b * next.e + d * next.f + f
    )
}

private data class DoweSvgImportContext(val matrix: DoweSvgImportMatrix, val fill: String?, val evenOdd: Boolean, val hidden: Boolean)
private data class DoweSvgImportedPath(val data: String, val fill: String, val evenOdd: Boolean, val transform: String?)

private object DoweSvgImporter {
    private val identity = DoweSvgImportMatrix(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    private val tokens = listOf("primary", "secondary", "tertiary", "muted", "success", "info", "warning", "danger")
    private val tagPattern = Regex("<(/?)([A-Za-z][A-Za-z0-9:_-]*)([^>]*)>")
    private val attrPattern = Regex("([A-Za-z_:][A-Za-z0-9_.:-]*)\\s*=\\s*([\\\"'])(.*?)\\2")
    private val pathPattern = Regex("[0-9\\sMmZzLlHhVvCcSsQqTtAa+.,eE-]+")

    fun convert(source: String, colorsMode: String = "tokens", format: String = "source"): String? = runCatching {
        require(colorsMode in setOf("tokens", "original"))
        require(format in setOf("source", "data"))
        require(format != "data" || colorsMode == "original")
        require(source.toByteArray(Charsets.UTF_8).size <= 262144)
        var viewBox: String? = null
        val colors = mutableListOf<String>()
        val paths = mutableListOf<DoweSvgImportedPath>()
        val stack = mutableListOf(DoweSvgImportContext(identity, null, false, false))
        for (match in tagPattern.findAll(source)) {
            val closing = match.groupValues[1].isNotEmpty()
            if (closing) {
                if (stack.size > 1) stack.removeAt(stack.lastIndex)
                continue
            }
            val name = match.groupValues[2].lowercase()
            val tail = match.groupValues[3]
            val attrs = attrPattern.findAll(tail).associate { entry ->
                entry.groupValues[1].lowercase() to decode(entry.groupValues[3])
            }
            val parent = stack.last()
            val local = attrs["transform"]?.let(::matrix) ?: identity
            val combined = parent.matrix.multiply(local)
            val styleFill = attrs["style"]?.split(";")?.firstNotNullOfOrNull { entry ->
                val pair = entry.split(":", limit = 2)
                pair.takeIf { it.size == 2 && it[0].trim().equals("fill", true) }?.get(1)?.trim()
            }
            val fill = attrs["fill"] ?: styleFill ?: parent.fill
            val styleFillRule = attrs["style"]?.split(";")?.firstNotNullOfOrNull { entry ->
                val pair = entry.split(":", limit = 2)
                pair.takeIf { it.size == 2 && it[0].trim().equals("fill-rule", true) }?.get(1)?.trim()
            }
            val fillRule = attrs["fill-rule"] ?: styleFillRule
            val evenOdd = when (fillRule?.trim()?.lowercase()) {
                null -> parent.evenOdd
                "nonzero" -> false
                "evenodd" -> true
                else -> error("fill-rule")
            }
            val hidden = parent.hidden || name in setOf("defs", "clippath", "mask", "symbol", "script", "style")
            if (name == "svg" && viewBox == null) {
                val raw = attrs["viewbox"] ?: "0 0 " + dimension(attrs["width"]) + " " + dimension(attrs["height"])
                val values = raw.trim().split(Regex("[\\s,]+")).map { it.toDouble() }
                require(values.size == 4 && values.all(Double::isFinite) && values[2] > 0 && values[3] > 0)
                viewBox = values.joinToString(" ", transform = ::number)
            }
            val drawable = name == "path" || (name == "rect" && "rx" !in attrs && "ry" !in attrs)
            if (drawable && !hidden) {
                require(paths.size < 1024)
                val data = if (name == "path") attrs["d"]?.trim().orEmpty() else rectangle(attrs)
                require(data.isNotEmpty() && pathPattern.matches(data))
                val transform = if (same(combined, identity)) null else matrixSource(combined)
                val portableFill = if (colorsMode == "original") originalFill(fill) else fill(fill, colors)
                paths += DoweSvgImportedPath(data, portableFill, evenOdd, transform)
            }
            if (!match.value.trimEnd().endsWith("/>")) stack += DoweSvgImportContext(combined, fill, evenOdd, hidden)
        }
        require(viewBox != null && paths.isNotEmpty())
        if (format == "data") {
            val values = org.json.JSONArray()
            paths.forEach { path ->
                val value = org.json.JSONObject().put("d", path.data).put(
                    "paint",
                    when (path.fill) { "none" -> "none"; "currentColor" -> "currentColor"; else -> "fill" }
                )
                if (path.fill != "none" && path.fill != "currentColor") value.put("color", path.fill)
                if (path.evenOdd) value.put("evenOdd", true)
                path.transform?.let { value.put("transform", it) }
                values.put(value)
            }
            org.json.JSONObject().put("viewBox", viewBox).put("paths", values).toString()
        } else {
            "Svg viewBox:\"" + viewBox + "\" w:\"full\" h:\"full\"\n" + paths.joinToString("\n") { path ->
                "  Path d:\"" + path.data + "\" fill:\"" + path.fill + "\"" + (if (path.evenOdd) " fillRule:\"evenodd\"" else "") + (path.transform?.let { " transform:\"" + it + "\"" } ?: "")
            }
        }
    }.getOrNull()

    private fun matrix(source: String): DoweSvgImportMatrix {
        var rest = source.trim()
        var output = identity
        while (rest.isNotEmpty()) {
            val match = Regex("^matrix\\s*\\(([^)]*)\\)").find(rest) ?: error("matrix")
            val values = match.groupValues[1].trim().split(Regex("[\\s,]+")).map { it.toDouble() }
            require(values.size == 6 && values.all(Double::isFinite))
            output = output.multiply(DoweSvgImportMatrix(values[0], values[1], values[2], values[3], values[4], values[5]))
            rest = rest.drop(match.value.length).trim()
        }
        return output
    }

    private fun dimension(value: String?): String {
        val number = value?.trim()?.removeSuffix("px")?.toDoubleOrNull()
        require(number != null && number.isFinite() && number > 0)
        return number(number)
    }

    private fun rectangle(attrs: Map<String, String>): String {
        val x = attrs["x"]?.trim()?.toDoubleOrNull() ?: 0.0
        val y = attrs["y"]?.trim()?.toDoubleOrNull() ?: 0.0
        val width = attrs["width"]?.trim()?.toDoubleOrNull()
        val height = attrs["height"]?.trim()?.toDoubleOrNull()
        require(x.isFinite() && y.isFinite() && width != null && height != null && width.isFinite() && height.isFinite() && width > 0 && height > 0)
        val right = x + width
        val bottom = y + height
        require(right.isFinite() && bottom.isFinite())
        return "M" + number(x) + " " + number(y) + "H" + number(right) + "V" + number(bottom) + "H" + number(x) + "Z"
    }

    private fun fill(value: String?, colors: MutableList<String>): String {
        val source = value?.trim().orEmpty()
        if (source.equals("none", true)) return "none"
        if (source.isEmpty() || source.equals("currentColor", true)) return "currentColor"
        val key = source.lowercase()
        var index = colors.indexOfFirst { color -> sameColor(color, key) }
        if (index < 0) {
            colors += key
            index = colors.lastIndex
        }
        return tokens[index % tokens.size]
    }

    private fun originalFill(value: String?): String {
        val source = value?.trim().orEmpty()
        if (source.equals("none", true)) return "none"
        if (source.isEmpty() || source.equals("currentColor", true)) return "currentColor"
        val normalized = source.lowercase()
        require(Regex("^#[0-9a-f]{3,4}$|^#[0-9a-f]{6}([0-9a-f]{2})?$").matches(normalized) || rgb(normalized) != null)
        return if (normalized.startsWith(35.toChar())) normalized else 35.toChar().toString() + rgb(normalized)!!.joinToString("") { it.toString(16).padStart(2, '0') }
    }

    private fun sameColor(left: String, right: String): Boolean {
        if (left == right) return true
        val leftChannels = rgb(left) ?: return false
        val rightChannels = rgb(right) ?: return false
        return leftChannels.indices.all { index -> kotlin.math.abs(leftChannels[index] - rightChannels[index]) <= 1 }
    }

    private fun rgb(value: String): List<Int>? {
        val match = Regex("^rgb\\s*\\(\\s*(\\d+)\\s*,\\s*(\\d+)\\s*,\\s*(\\d+)\\s*\\)$", RegexOption.IGNORE_CASE).matchEntire(value) ?: return null
        val channels = match.groupValues.drop(1).map(String::toInt)
        return channels.takeIf { values -> values.all { channel -> channel in 0..255 } }
    }

    private fun matrixSource(value: DoweSvgImportMatrix) = "matrix(" + listOf(value.a, value.b, value.c, value.d, value.e, value.f).joinToString(" ", transform = ::number) + ")"
    private fun same(left: DoweSvgImportMatrix, right: DoweSvgImportMatrix) = listOf(left.a - right.a, left.b - right.b, left.c - right.c, left.d - right.d, left.e - right.e, left.f - right.f).all { kotlin.math.abs(it) < 0.0000001 }
    private fun number(value: Double): String = if (kotlin.math.abs(value) < 0.0000001) "0" else java.math.BigDecimal.valueOf(value).setScale(6, java.math.RoundingMode.HALF_UP).stripTrailingZeros().toPlainString()
    private fun decode(value: String) = value.replace("&quot;", "\"").replace("&apos;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

private class DoweReactiveState(
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

    suspend fun run(id: String, item: Map<String, Any?>? = null) {
        when (val action = actions[id]) {
            is DoweAction.Assign -> {
                val value = action.call?.let { stdlib(it, item) } ?: when (action.source) {
                    "\$dowe:bool:true" -> true
                    "\$dowe:bool:false" -> false
                    else -> if (action.source.startsWith("\$dowe:string:")) action.source.removePrefix("\$dowe:string:") else if (action.source.startsWith("!")) !(read(action.source.drop(1), item) as? Boolean ?: false) else read(action.source, item)
                }
                write(action.target, value)
            }
            is DoweAction.Reset -> initial[action.target]?.let { write(action.target, it) }
            is DoweAction.Request -> execute(action.action, item)
            is DoweAction.Sequence -> runSteps(action.steps, item, mutableMapOf())
            null -> {}
        }
    }

    suspend fun load(ids: List<String>) {
        for (id in ids) run(id)
    }

    private suspend fun runSteps(steps: List<DoweStep>, item: Map<String, Any?>?, results: MutableMap<String, Any?>): Boolean {
        for (step in steps) {
            when (step) {
                is DoweStep.Validate -> if (!validateForm(step.target, item)) return true
                is DoweStep.Request -> {
                    val result = request(step.action, item)
                    results[step.result] = mapOf("ok" to result.first, "data" to result.second)
                }
                is DoweStep.Branch -> {
                    val ok = readResult("${step.result}.ok", item, results) as? Boolean ?: false
                    if (runSteps(if (ok) step.success else step.error, item, results)) return true
                }
                is DoweStep.Assign -> {
                    val value = if (step.hasLiteral) step.literal else step.call?.let { stdlib(it, item) }
                        ?: valueFor(step.source, item, results)
                    write(step.target, value)
                }
                is DoweStep.Reset -> initial[step.target]?.let { write(step.target, it) }
                is DoweStep.Toast -> showToast(step)
                is DoweStep.Redirect -> {
                    redirectPath = step.path
                    return true
                }
            }
        }
        return false
    }

    private fun valueFor(path: String, item: Map<String, Any?>?, results: Map<String, Any?>): Any? = when (path) {
        "\$dowe:bool:true" -> true
        "\$dowe:bool:false" -> false
        else -> if (path.startsWith("\$dowe:string:")) path.removePrefix("\$dowe:string:")
        else if (path.startsWith("!")) !(readResult(path.drop(1), item, results) as? Boolean ?: false)
        else readResult(path, item, results)
    }

    private fun readResult(path: String, item: Map<String, Any?>?, results: Map<String, Any?>): Any? {
        val root = path.substringBefore('.')
        val result = results[root]
        if (result != null) {
            val suffix = path.removePrefix(root).removePrefix(".")
            return if (suffix.isEmpty()) result else readMap(suffix, result as? Map<String, Any?> ?: emptyMap())
        }
        return read(path, item)
    }

    private fun showToast(action: DoweStep.Toast) {
        val scheme = action.scheme ?: if (action.kind == "error") "danger" else action.kind
        toastSequence += 1
        toast = DoweToastState(toastSequence, action.kind, action.title, action.message, action.duration ?: 4000, scheme, action.variant ?: "solid", action.position ?: "top-right")
    }

    fun closeToast() {
        toast = null
    }

    fun consumeRedirect() {
        redirectPath = null
    }

    private fun read(path: String, item: Map<String, Any?>? = null): Any? {
        formValue(path, item)?.let { return it }
        if (path == "item" && item != null) {
            return item
        }
        if (path.startsWith("item.") && item != null) {
            return readMap(path.removePrefix("item."), item)
        }
        return readMap(path, values) ?: readMap(path, constants)
    }

    private fun formError(form: String, field: DoweFormFieldMetadata, item: Map<String, Any?>?): String? {
        val value = readMap(form + "." + field.path, values)
        val rules = field.rules.map { rule ->
            if (rule.kind == "matches" && rule.argument != null) rule.copy(argument = read(rule.argument, item)?.toString() ?: "") else rule
        }
        return if (field.kind == "boolean") doweBooleanValidationError(value as? Boolean ?: false, rules) else doweValidationError(value?.toString() ?: "", rules)
    }

    private fun formValue(path: String, item: Map<String, Any?>?): Any? {
        val parts = path.split(".")
        val fields = forms[parts.firstOrNull() ?: return null] ?: return null
        val form = parts.first()
        if (parts.getOrNull(1) == "isValid" && parts.size == 2) return fields.all { formError(form, it, item) == null }
        if (parts.getOrNull(1) == "isInvalid" && parts.size == 2) return fields.any { formError(form, it, item) != null }
        if (parts.getOrNull(1) == "errors") {
            val errors = fields.mapNotNull { field -> formError(form, field, item)?.let { field.path to it } }.toMap()
            return if (parts.size == 2) errors else errors[parts.drop(2).joinToString(".")]
        }
        if (parts.getOrNull(1) == "touched") {
            val touched = fields.associate { it.path to (formTouched[form + "." + it.path] ?: false) }
            return if (parts.size == 2) touched else touched[parts.drop(2).joinToString(".")] ?: false
        }
        return null
    }

    private fun validateForm(target: String, item: Map<String, Any?>?): Boolean {
        val fields = forms[target] ?: return true
        fields.forEach { formTouched[target + "." + it.path] = true }
        return formValue(target + ".isValid", item) as? Boolean ?: true
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
            "parse.svg" -> DoweSvgImporter.convert(text("value"), text("colors").ifEmpty { "tokens" }, text("format").ifEmpty { "source" }) ?: args["fallback"]
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
        val result = request(action, item)
        if (result.first) {
            action.update?.let { write(it, result.second) }
            action.reset?.let { initial[it]?.let { value -> write(it, value) } }
            setAlert(action.successAlert, "success", action.successMessage ?: "Request completed")
        } else {
            setAlert(action.errorAlert, "error", action.errorMessage ?: "Request failed")
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
}
