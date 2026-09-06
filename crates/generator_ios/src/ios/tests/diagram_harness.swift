import Foundation
import SwiftUI

final class DoweReactiveState: ObservableObject {
    @Published var values: [String: Any] = [:]
    var writes = 0
    func canvasValue(_ path: String) -> Any? { values[path] }
    func candles(_ path: String) -> [[String: Any]] { values[path] as? [[String: Any]] ?? [] }
    func write(_ path: String, value: Any) { writes += 1; values[path] = value }
    func run(_ name: String, item: [String: Any]) {}
}

__DOWE_RUNTIME__

final class DiagramModelHarness {
    let state = DoweReactiveState()
    let nodesPath = "nodes", edgesPath = "edges"
    var scale: CGFloat = 1
    var dragNode: String?
    var dragOrigin: CGPoint = .zero
    var dragTranslation: CGSize = .zero

    __DOWE_MODEL_METHODS__

    func validate() {
        let a: [String: Any] = ["id": "a", "x": 0, "y": 0, "metadata": "keep"]
        let b: [String: Any] = ["id": "b", "x": 300, "y": 0]
        state.values["nodes"] = [a, b] as [Any]
        state.values["edges"] = [["id": "edge-1", "source": "b", "target": "a"]]
        precondition(persistConnection(source: "a", target: "b"))
        precondition(state.writes == 1)
        precondition(state.candles("edges")[1]["id"] as? String == "edge-2")
        precondition(!persistConnection(source: "a", target: "b"))
        precondition(!persistConnection(source: "a", target: "a"))
        precondition(!persistConnection(source: "missing", target: "b"))
        precondition(!persistConnection(source: "a", target: "missing"))
        precondition(state.writes == 1)
        state.values["edges"] = [["id": "edge-2", "source": "b", "target": "a"]]
        precondition(persistConnection(source: "a", target: "b"))
        precondition(state.candles("edges")[1]["id"] as? String == "edge-1")
        scale = 2
        dragNode = "a"
        dragTranslation = CGSize(width: 40, height: 80)
        precondition(effectivePosition(a) == CGPoint(x: 20, y: 40))
        precondition(nodeCenter(a) == CGPoint(x: 100, y: 68))
        precondition(number(state.candles("nodes")[0]["x"]) == 0)
        precondition(state.writes == 2)
        dragNode = nil
        precondition(nodeCenter(a) == CGPoint(x: 80, y: 28))
        state.values["nodes"] = [a, b, NSNull(), ["other": "preserved"]] as [Any]
        let item = moveNode(a, to: CGPoint(x: 40, y: 80))!
        precondition(number(item["x"]) == 40 && number(item["y"]) == 80)
        precondition(item["metadata"] as? String == "keep")
        let rows = state.values["nodes"] as! [Any]
        precondition(rows.count == 4 && rows[2] is NSNull)
        precondition((rows[3] as? [String: String])?["other"] == "preserved")
        precondition(moveNode(["id": "missing"], to: .zero) == nil)
        precondition(state.writes == 3)
        state.values["nodes"] = [b]
        precondition(moveNode(a, to: .zero) == nil)
        precondition(!persistConnection(source: "a", target: "b"))
        precondition(state.writes == 3)
    }
}

DiagramModelHarness().validate()
