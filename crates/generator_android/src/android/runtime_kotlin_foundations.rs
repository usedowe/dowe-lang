r#"
private fun doweDynamicTextSize(value: String): TextUnit = when (value) {
    "xs" -> 12.sp
    "sm" -> 14.sp
    "lg" -> 20.sp
    "xl" -> 24.sp
    else -> 16.sp
}
private fun doweDynamicTextWeight(value: String): FontWeight = when (value) {
    "thin" -> FontWeight.Thin
    "light" -> FontWeight.Light
    "medium" -> FontWeight.Medium
    "semibold" -> FontWeight.SemiBold
    "bold" -> FontWeight.Bold
    "black" -> FontWeight.Black
    else -> FontWeight.Normal
}
private fun doweDynamicTextSpacing(value: String): TextUnit = value.toFloatOrNull()?.sp ?: 0.sp
private fun doweValidEnum(value: String, kind: String): String = when (kind) {
    "variant" -> if (value in DowePropCatalog.variants) value else "solid"
    "scheme" -> if (value in DowePropCatalog.schemes) value else "primary"
    "size" -> if (value in DowePropCatalog.sizes) value else "md"
    "rounded" -> if (value in DowePropCatalog.rounded) value else "md"
    "color" -> if (value in DowePropCatalog.colors) value else "primary"
    "icon" -> if (value in DowePropCatalog.icons) value else ""
    else -> value
}
private object DowePropCatalog {
    val icons = setOf(__DOWE_ICON_NAMES__)
    val colors = setOf(__DOWE_COLORS__)
    val variants = setOf(__DOWE_VARIANTS__)
    val schemes = setOf(__DOWE_SCHEMES__)
    val sizes = setOf(__DOWE_SIZES__)
    val rounded = setOf(__DOWE_ROUNDED__)
}

private data class DoweRow(val id: String, val value: Map<String, Any?>)

private data class DoweTreeNode(val id: String, val label: String, val path: String, val branch: Boolean, val children: List<DoweTreeNode>, val value: Map<String, Any?>)

private data class DoweInvokeAction(
    val function: String,
    val args: List<DoweStdlibArg>,
    val update: String?,
    val reset: String?,
    val successAlert: String?,
    val successMessage: String?,
    val errorAlert: String?,
    val errorMessage: String?
)

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
    data class Invoke(val action: DoweInvokeAction, val metadata: DoweActionMetadata) : DoweAction()
    data class Assign(val target: String, val source: String, val call: DoweStdlibCall?, val metadata: DoweActionMetadata) : DoweAction()
    data class Reset(val target: String, val metadata: DoweActionMetadata) : DoweAction()
    data class Sequence(val steps: List<DoweStep>, val metadata: DoweActionMetadata) : DoweAction()
}

private sealed class DoweStep {
    data class Validate(val target: String) : DoweStep()
    data class Request(val result: String, val action: DoweRequestAction) : DoweStep()
    data class Invoke(val result: String, val action: DoweInvokeAction) : DoweStep()
    data class Branch(val result: String, val success: List<DoweStep>, val error: List<DoweStep>) : DoweStep()
    data class Assign(val target: String, val source: String, val literal: Any?, val hasLiteral: Boolean, val call: DoweStdlibCall?) : DoweStep()
    data class Reset(val target: String) : DoweStep()
    data class Toast(val kind: String, val title: String, val message: String, val duration: Int?, val scheme: String?, val variant: String?, val position: String?) : DoweStep()
    data class Redirect(val path: String) : DoweStep()
}

private data class DoweToastState(val id: Long, val kind: String, val title: String, val message: String, val duration: Int, val scheme: String, val variant: String, val position: String)

internal object DoweNativeBridge {
    var handler: (suspend (String, Map<String, Any?>) -> Pair<Boolean, Any?>)? = null
    fun install(handler: suspend (String, Map<String, Any?>) -> Pair<Boolean, Any?>) {
        this.handler = handler
    }
    suspend fun invoke(function: String, args: Map<String, Any?>): Pair<Boolean, Any?> = handler?.invoke(function, args) ?: Pair(false, null)
}

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

"#
