import Foundation
import SwiftUI

final class State {
    var values: [String: Any] = [:]
    var events: [[String: Any]] = []
    var onRun: ((String) -> Void)?
    func canvasValue(_ path: String) -> Any? { values[path] }
    func write(_ path: String, value: Any) { values[path] = value }
    func run(_ action: String, item: [String: Any]) {
        events.append(item.merging(["rows": values["layers"] ?? [], "selected": values["selected"] ?? ""]) { first, _ in first })
        onRun?(action)
    }
}

final class Selection { var id = "" }

final class Model {
    var state = State()
    var selection = Selection()
    var draw = true
    var drawMode = "pen"
    var drawModePath: String?
    var layersPath: String? = "layers"
    var selectedPath: String?
    var onLayerAdd: String? = "add"
    var onLayerChange: String? = "change"
    var onLayerRemove: String? = "remove"
    var onLayerSelect: String? = "select"
    var drawingLayerId: String?
    var drawingStart: CGPoint?
    var drawingPointer: ObjectIdentifier?
    var drawingMode: String?
    var nextLayerSequence = 1
    __METHODS__
}

func check(_ value: @autoclosure () -> Bool, _ message: String) {
    precondition(value(), message)
}

let model = Model()
let first = NSObject()
let second = NSObject()
let pointer = ObjectIdentifier(first)
let other = ObjectIdentifier(second)
func input(_ kind: String, _ x: CGFloat, _ y: CGFloat, _ id: ObjectIdentifier = pointer) {
    model.updateLayer(at: CGPoint(x: x, y: y), kind: kind, pointer: id)
}
func hit(_ layer: [String: Any], _ x: CGFloat, _ y: CGFloat) -> Bool {
    model.layerHit(layer, point: CGPoint(x: x, y: y))
}

check(hit(["type": "line", "x1": 0, "y1": 0, "x2": 100, "y2": 0], 50, 4), "line midpoint tolerance")
check(!hit(["type": "line", "x1": 0, "y1": 0, "x2": 100, "y2": 0], 50, 5), "line outside tolerance")
check(hit(["type": "line", "x1": 10, "y1": 10, "x2": 10, "y2": 10, "strokeWidth": 8], 17, 10), "degenerate segment")
check(hit(["type": "polyline", "points": [["x": 0, "y": 0], ["x": 100, "y": 0]]], 50, 6), "polyline midpoint")
check(!hit(["type": "polyline", "points": [["x": 0, "y": 0], ["x": 100, "y": 0]]], 50, 7), "polyline outside")
let triangle: [[String: Any]] = [["x": 0, "y": 0], ["x": 100, "y": 0], ["x": 100, "y": 100]]
check(hit(["type": "polyline", "points": triangle, "closed": true], 50, 50), "closed polyline closing segment")
check(!hit(["type": "polyline", "points": triangle, "closed": false], 50, 50), "open polyline excludes closing segment")
check(hit(["type": "circle", "x": 20, "y": 20, "radius": 10, "strokeWidth": 8], 38, 20), "circle tolerance")
for type in ["rect", "image"] {
    check(hit(["type": type, "x": 10, "y": 10, "width": 20, "height": 30], 30, 40), "bounds inclusive")
    check(!hit(["type": type, "x": 10, "y": 10, "width": 20, "height": 30], 31, 40), "bounds outside")
}
for (align, left) in [("start", 50.0), ("center", 38.0), ("end", 26.0)] {
    let text: [String: Any] = ["type": "text", "x": 50, "y": 40, "size": 10, "text": "abcd", "align": align]
    check(hit(text, left, 30) && hit(text, left + 24, 40), "text edges")
    check(!hit(text, left - 1, 35) && !hit(text, left, 41), "text outside")
}
check(hit(["type": "text", "x": 0, "y": 10, "size": 10, "text": ""], 6, 0), "empty text minimum width")
for type in ["rect", "image", "circle", "line", "polyline", "text"] {
    let command: [String: Any] = ["type": type, "x": 10, "y": 20, "width": 30, "height": 40, "radius": 10, "x1": 10, "y1": 20, "x2": 40, "y2": 60, "points": [["x": 10, "y": 20], ["x": 40, "y": 60]], "text": "abc", "size": 10]
    let bounds = model.selectionPath(command)?.boundingRect
    check(bounds != nil && bounds!.width > 0 && bounds!.height > 0, "visible selection for \(type)")
}
check(model.selectionPath(["type": "line", "x1": 0, "y1": 0, "x2": 100, "y2": 0])?.boundingRect == CGRect(x: -4, y: -4, width: 108, height: 8), "line highlight bounds")
check(model.selectionPath(["type": "text", "x": 50, "y": 40, "size": 10, "text": "abcd", "align": "end"])?.boundingRect == CGRect(x: 22, y: 26, width: 32, height: 18), "text highlight uses baseline and alignment")
input("down", 0, 0)
input("down", 50, 50, other)
input("move", 80, 80, other)
input("up", 80, 80, other)
check(model.layerRows().count == 1 && model.drawingPointer == pointer, "single drawing pointer")
input("move", 10, 10)
model.drawMode = "circle"
input("up", 20, 20)
let committed = model.layerRows()[0]
check(committed["type"] as? String == "polyline", "frozen pen mode")
check((committed["points"] as? [[String: Any]])?.count == 3, "final up point retained")
check(model.selection.id == "layer-1", "internal selection")
check(model.state.events.compactMap { $0["event"] as? String } == ["add", "change"], "ordered commit events")
check((model.state.events[0]["rows"] as? [[String: Any]])?.count == 1, "commit writes before events")
model.removeLayer("layer-1")
check(model.selection.id.isEmpty, "remove clears internal selection")
input("down", 0, 0)
check(model.layerRows()[0]["id"] as? String == "layer-2", "deleted id never reused")
let count = model.state.events.count
input("cancel", 20, 20)
check(model.layerRows().isEmpty && model.state.events.count == count, "cancel rollback without events")
model.drawMode = "rect"
input("down", 20, 20)
model.drawMode = "erase"
input("up", 0, 0)
check(model.number(model.layerRows()[0]["width"]) == 20, "frozen rectangle and final up")
model.selectedPath = "selected"
model.selection.id = "stale"
model.state.write("selected", value: "")
check(model.selectedLayerId().isEmpty, "binding authoritative when empty")
model.state.write("selected", value: "other")
input("down", 10, 10)
check(model.layerRows().isEmpty && model.selectedLayerId() == "other", "erase preserves unrelated selection")
input("move", 50, 50)
input("up", 100, 100)
check(model.layerRows().isEmpty, "erase never draws")
model.state.write("layers", value: [["id": "bottom", "type": "rect", "x": 0, "y": 0, "width": 100, "height": 100], ["id": "top", "type": "line", "x1": 0, "y1": 0, "x2": 100, "y2": 0], ["type": "rect", "x": 0, "y": 0, "width": 100, "height": 100]])
model.drawMode = "select"
input("down", 50, 0)
check(model.selectedLayerId() == "top", "topmost eligible selection")
model.drawMode = "erase"
input("down", 50, 0)
check(model.selectedLayerId().isEmpty && model.layerRows().count == 2, "erase selected clears binding")
check((model.state.events.last?["layer"] as? [String: Any])?["id"] as? String == "top", "remove snapshot")
model.drawMode = "select"
input("down", 200, 200)
check((model.state.events.last?["layer"] as? [String: Any])?.isEmpty == true, "empty selection object")
model.drawMode = "pen"
model.state.onRun = { action in if action == "add" { model.state.write("layers", value: []) } }
input("down", 1, 1)
input("up", 2, 2)
check((model.state.events.last?["layer"] as? [String: Any])?["type"] as? String == "polyline", "change retains snapshot after add handler mutation")
model.state.onRun = nil
model.onLayerAdd = nil
model.onLayerChange = nil
model.drawModePath = "mode"
model.state.write("mode", value: "pen")
input("down", 0, 0)
for index in 1...300 { input("move", CGFloat(index), 0) }
model.state.write("mode", value: "erase")
input("up", 301, 0)
check((model.layerRows()[0]["points"] as? [[String: Any]])?.count == 302, "unbounded samples and reactive mode frozen")
check(!model.selectedLayerId().isEmpty, "commit without event handlers updates selection")
input("down", 150, 0)
check(model.layerRows().isEmpty, "next gesture reads reactive erase mode")
print("Draw iOS interaction assertions passed")
