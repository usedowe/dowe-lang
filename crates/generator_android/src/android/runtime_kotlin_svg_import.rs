r#"private object DoweSvgImporter {
    private val identity = DoweSvgImportMatrix(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    private val tokens = listOf("primary", "secondary", "accent", "muted", "success", "info", "warning", "danger")
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

"#
