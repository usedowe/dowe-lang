r#"    suspend fun run(id: String, item: Map<String, Any?>? = null) {
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
            is DoweAction.Invoke -> execute(action.action, item)
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
                is DoweStep.Invoke -> {
                    val result = invoke(step.action, item)
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
            "str.equals" -> text("value") == text("other")
            "str.startsWith" -> text("value").startsWith(text("prefix"))
            "str.endsWith" -> text("value").endsWith(text("suffix"))
            "str.replace" -> text("value").replace(text("from"), text("to"))
            "str.truncate" -> text("value").take(maxOf(0, number("max")?.toInt() ?: 0))
            "str.split" -> text("value").split(text("delimiter")).let { values -> args["limit"]?.let { values.take(maxOf(0, stdlibNumber(it)?.toInt() ?: 0)) } ?: values }
            "str.join" -> list("values").joinToString(text("delimiter")) { stdlibText(it) }
            "math.add" -> finite(number("left"), number("right")) { left, right -> left + right }
            "math.sub" -> finite(number("left"), number("right")) { left, right -> left - right }
            "math.mul" -> finite(number("left"), number("right")) { left, right -> left * right }
            "math.div" -> finite(number("left"), number("right")) { left, right -> if (right == 0.0) null else left / right }
            "math.gt" -> finiteCompare(number("left"), number("right")) { left, right -> left > right }
            "math.gte" -> finiteCompare(number("left"), number("right")) { left, right -> left >= right }
            "math.lt" -> finiteCompare(number("left"), number("right")) { left, right -> left < right }
            "math.lte" -> finiteCompare(number("left"), number("right")) { left, right -> left <= right }
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
            "parse.json", "json.parse" -> runCatching { doweNativeValue(org.json.JSONTokener(text("value")).nextValue()) }.getOrDefault(args["fallback"])
            "url.encode" -> java.net.URLEncoder.encode(text("value"), Charsets.UTF_8.name()).replace("+", "%20")
            "url.decode" -> runCatching { java.net.URLDecoder.decode(text("value"), Charsets.UTF_8.name()) }.getOrDefault(args["fallback"])
            "url.parse" -> stdlibUrlParse(text("value"))
            "url.queryGet" -> android.net.Uri.parse(text("value")).getQueryParameter(text("name"))
            "url.querySet" -> stdlibUrlQuerySet(text("value"), text("name"), args["param"])
            "csv.parse" -> stdlibCsvParse(text("value"), text("delimiter").ifEmpty { "," }, args["header"] as? Boolean ?: false, number("maxRows")?.toInt() ?: 1000, number("maxColumns")?.toInt() ?: 100)
            "csv.stringify" -> stdlibCsvStringify(list("rows"), text("delimiter").ifEmpty { "," })
            "sort.asc" -> stdlibSort(list("values"), null, false, text("nulls"))
            "sort.desc" -> stdlibSort(list("values"), null, true, text("nulls"))
            "sort.by" -> stdlibSort(list("values"), text("field"), text("direction") == "desc", text("nulls"))
            "list.take" -> list("values").take(maxOf(0, number("count")?.toInt() ?: 0))
            "list.skip" -> list("values").drop(maxOf(0, number("count")?.toInt() ?: 0))
            "list.first" -> list("values").firstOrNull()
            "list.last" -> list("values").lastOrNull()
            "list.count" -> list("values").size
            "list.filterEquals" -> list("values").filter { stdlibRead(it, text("field")) == args["value"] }
            "list.filterContains" -> list("values").filter { stdlibText(stdlibRead(it, text("field"))).lowercase().contains(text("value").lowercase()) }
            "list.filterContainsAny" -> {
                val needles = list("needles").map(::stdlibText).filter { it.isNotEmpty() }.map { it.lowercase() }
                list("values").filter { item ->
                    val value = stdlibText(stdlibRead(item, text("field"))).lowercase()
                    needles.any { needle -> value.contains(needle) }
                }
            }
            "list.concat" -> list("values") + list("other")
            "list.mapField" -> list("values").map { stdlibRead(it, text("field")) }
            "list.sumBy" -> list("values").mapNotNull { stdlibNumber(stdlibRead(it, text("field"))) }.sum()
            "list.averageBy" -> list("values").mapNotNull { stdlibNumber(stdlibRead(it, text("field"))) }.takeIf { it.isNotEmpty() }?.average()
            "json.get" -> stdlibRead(args["value"], text("path")) ?: args["fallback"]
            "json.set" -> stdlibJsonSet(args["value"], text("path"), args["next"])
            "json.pick" -> stdlibJsonPick(args["value"], list("fields").map(::stdlibText))
            "json.omit" -> stdlibJsonOmit(args["value"], list("fields").map(::stdlibText))
            "json.merge" -> (args["left"] as? Map<*, *>).orEmpty().entries.associate { it.key.toString() to it.value }.toMutableMap().apply { putAll((args["right"] as? Map<*, *>).orEmpty().entries.associate { it.key.toString() to it.value }) }
            "json.stringify" -> stdlibJsonStringify(args["value"], args["pretty"] as? Boolean ?: false)
            "date.now" -> java.time.Instant.now().toString()
            "date.formatIso" -> runCatching { java.time.Instant.parse(text("value")).toString() }.getOrDefault(text("value"))
            "date.addDays" -> runCatching { java.time.Instant.parse(text("value")).plus(java.time.Duration.ofDays(number("days")?.toLong() ?: 0)).toString() }.getOrNull()
            "date.diffDays" -> runCatching { java.time.Duration.between(java.time.Instant.parse(text("start")), java.time.Instant.parse(text("end"))).toDays() }.getOrDefault(0L)
            else -> null
        }
    }

    private fun stdlibUrlParse(value: String): Map<String, Any?> = runCatching {
        val uri = android.net.Uri.parse(value)
        val query = uri.queryParameterNames.associateWith { uri.getQueryParameter(it).orEmpty() }
        mapOf(
            "ok" to true,
            "scheme" to uri.scheme,
            "host" to uri.host,
            "path" to (uri.path ?: ""),
            "query" to query,
            "fragment" to uri.fragment,
            "origin" to if (uri.scheme != null && uri.host != null) "${uri.scheme}://${uri.host}" else null,
            "isRelative" to (uri.scheme == null),
            "error" to null
        )
    }.getOrElse {
        mapOf("ok" to false, "scheme" to null, "host" to null, "path" to null, "query" to emptyMap<String, String>(), "fragment" to null, "origin" to null, "isRelative" to false, "error" to "invalid_url")
    }

    private fun stdlibUrlQuerySet(value: String, name: String, param: Any?): String = runCatching {
        val uri = android.net.Uri.parse(value)
        val builder = uri.buildUpon().clearQuery()
        uri.queryParameterNames.filter { it != name }.forEach { key ->
            uri.getQueryParameters(key).forEach { item -> builder.appendQueryParameter(key, item) }
        }
"#
