import kotlin.math.*

__DOWE_TYPES__

private class DiagramState {
    val values = mutableMapOf<String, Any?>()
    var writes = 0
    fun canvasValue(path: String): Any? = values[path]
    fun candles(path: String): List<Map<String, Any?>> =
        (values[path] as? List<*>)?.mapNotNull { it as? Map<String, Any?> } ?: emptyList()
    fun write(path: String, value: Any?) { writes++; values[path] = value }
}

private class DiagramModelHarness {
    val state = DiagramState()
    val nodesPath = "nodes"
    val edgesPath = "edges"

    __DOWE_METHODS__

    fun validate() {
        val a = mapOf("id" to "a", "x" to 0f, "y" to 0f, "metadata" to "keep")
        val b = mapOf("id" to "b", "x" to 300f, "y" to 0f)
        state.values["nodes"] = listOf(a, b)
        state.values["edges"] = listOf(mapOf("id" to "edge-1", "source" to "b", "target" to "a"))
        check(persistConnection("a", "b"))
        check(state.writes == 1)
        check(state.candles("edges")[1]["id"] == "edge-2")
        check(!persistConnection("a", "b"))
        check(!persistConnection("a", "a"))
        check(!persistConnection("missing", "b"))
        check(!persistConnection("a", "missing"))
        check(state.writes == 1)
        state.values["edges"] = listOf(mapOf("id" to "edge-2", "source" to "b", "target" to "a"))
        check(persistConnection("a", "b"))
        check(state.candles("edges")[1]["id"] == "edge-1")
        val node = DoweDiagramNode("a", 0f, 0f, 160f, 56f, "A")
        val malformed = mapOf("other" to "preserved")
        state.values["nodes"] = listOf(a, b, null, malformed)
        val item = updateNode(node, 40f, 80f)!!
        check(item["x"] == 40f && item["y"] == 80f && item["metadata"] == "keep")
        val rows = state.values["nodes"] as List<*>
        check(rows.size == 4 && rows[2] == null && rows[3] === malformed)
        check(state.writes == 3)
        state.values["nodes"] = listOf(b)
        check(updateNode(node, 0f, 0f) == null)
        check(!persistConnection("a", "b"))
        check(state.writes == 3)
    }
}

fun main() { DiagramModelHarness().validate() }
