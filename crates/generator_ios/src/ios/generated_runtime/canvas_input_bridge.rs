r#"    @ObservedObject var state: DoweReactiveState
    let selection: DoweCanvasSelection
    let viewWidth: CGFloat
    let viewHeight: CGFloat
    let fit: String
    let onPointer: String?
    let onKey: String?
    let onMotion: String?
    let motionRate: Int
    let draw: Bool
    let drawMode: String
    let drawModePath: String?
    let layersPath: String?
    let selectedPath: String?
    let onLayerAdd: String?
    let onLayerChange: String?
    let onLayerRemove: String?
    let onLayerSelect: String?

    func makeUIView(context: Context) -> DoweCanvasInputUIView {
        let view = DoweCanvasInputUIView(state: state, viewWidth: viewWidth, viewHeight: viewHeight, fit: fit, onPointer: onPointer, onKey: onKey, onMotion: onMotion, motionRate: motionRate, draw: draw, drawMode: drawMode, drawModePath: drawModePath, layersPath: layersPath, selectedPath: selectedPath, onLayerAdd: onLayerAdd, onLayerChange: onLayerChange, onLayerRemove: onLayerRemove, onLayerSelect: onLayerSelect)
        view.selection = selection
        return view
    }

    func updateUIView(_ view: DoweCanvasInputUIView, context: Context) {
        view.update(state: state, viewWidth: viewWidth, viewHeight: viewHeight, fit: fit, onPointer: onPointer, onKey: onKey, onMotion: onMotion, motionRate: motionRate, draw: draw, drawMode: drawMode, drawModePath: drawModePath, layersPath: layersPath, selectedPath: selectedPath, onLayerAdd: onLayerAdd, onLayerChange: onLayerChange, onLayerRemove: onLayerRemove, onLayerSelect: onLayerSelect)
    }
}

@MainActor
final class DoweCanvasInputUIView: UIView {
    var selection = DoweCanvasSelection()
    private var state: DoweReactiveState
    private var viewWidth: CGFloat
    private var viewHeight: CGFloat
    private var fit: String
    private var onPointer: String?
    private var onKey: String?
    private var onMotion: String?
    private var motionRate: Int
    private var draw: Bool
    private var drawMode: String
    private var drawModePath: String?
    private var layersPath: String?
    private var selectedPath: String?
    private var onLayerAdd: String?
    private var onLayerChange: String?
    private var onLayerRemove: String?
    private var onLayerSelect: String?
    private let started = ProcessInfo.processInfo.systemUptime * 1000
    private var points: [ObjectIdentifier: CGPoint] = [:]
    private var drawingLayerId: String?
    private var drawingStart: CGPoint?
    private var drawingPointer: ObjectIdentifier?
    private var drawingMode: String?
    private var nextLayerSequence = 1
    private let motion = CMMotionManager()

    override var canBecomeFirstResponder: Bool { onKey != nil || layersPath != nil }

    init(state: DoweReactiveState, viewWidth: CGFloat, viewHeight: CGFloat, fit: String, onPointer: String?, onKey: String?, onMotion: String?, motionRate: Int, draw: Bool, drawMode: String, drawModePath: String?, layersPath: String?, selectedPath: String?, onLayerAdd: String?, onLayerChange: String?, onLayerRemove: String?, onLayerSelect: String?) {
        self.state = state
        self.viewWidth = viewWidth
        self.viewHeight = viewHeight
        self.fit = fit
        self.onPointer = onPointer
        self.onKey = onKey
        self.onMotion = onMotion
        self.motionRate = motionRate
        self.draw = draw
        self.drawMode = drawMode
        self.drawModePath = drawModePath
        self.layersPath = layersPath
        self.selectedPath = selectedPath
        self.onLayerAdd = onLayerAdd
        self.onLayerChange = onLayerChange
        self.onLayerRemove = onLayerRemove
        self.onLayerSelect = onLayerSelect
        super.init(frame: .zero)
        isMultipleTouchEnabled = true
        backgroundColor = .clear
    }

    required init?(coder: NSCoder) { nil }

    func update(state: DoweReactiveState, viewWidth: CGFloat, viewHeight: CGFloat, fit: String, onPointer: String?, onKey: String?, onMotion: String?, motionRate: Int, draw: Bool, drawMode: String, drawModePath: String?, layersPath: String?, selectedPath: String?, onLayerAdd: String?, onLayerChange: String?, onLayerRemove: String?, onLayerSelect: String?) {
        self.state = state
        self.viewWidth = viewWidth
        self.viewHeight = viewHeight
        self.fit = fit
        self.onPointer = onPointer
        self.onKey = onKey
        let restart = self.onMotion != onMotion || self.motionRate != motionRate
        self.onMotion = onMotion
        self.motionRate = motionRate
        self.draw = draw
        self.drawMode = drawMode
        self.drawModePath = drawModePath
        self.layersPath = layersPath
        self.selectedPath = selectedPath
        self.onLayerAdd = onLayerAdd
        self.onLayerChange = onLayerChange
        self.onLayerRemove = onLayerRemove
        self.onLayerSelect = onLayerSelect
        if restart { stopMotion(); startMotion() }
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()
        if window == nil {
            if let pointer = drawingPointer { updateLayer(at: .zero, kind: "cancel", pointer: pointer) }
            stopMotion()
            points.removeAll()
        } else { startMotion() }
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) { if onKey != nil || layersPath != nil { becomeFirstResponder() }; emitTouches(touches, kind: "down", event: event) }
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) { emitTouches(touches, kind: "move", event: event) }
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) { emitTouches(touches, kind: "up", event: event) }
    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) { emitTouches(touches, kind: "cancel", event: event) }

    private func emitTouches(_ touches: Set<UITouch>, kind: String, event: UIEvent?) {
        guard onPointer != nil || layersPath != nil else { return }
        let primary = event?.allTouches?.first
        for touch in touches {
            let id = ObjectIdentifier(touch)
            let raw = touch.location(in: self)
            let logical = logicalPoint(raw)
            let previous = points[id] ?? logical.point
            if kind != "down" || logical.inside {
                updateLayer(at: logical.point, kind: kind, pointer: id)
            }
            let pointerType: String
            switch touch.type { case .pencil: pointerType = "pen"; case .indirectPointer: pointerType = "mouse"; default: pointerType = "touch" }
            if let action = onPointer {
                state.run(action, item: [
                    "source": "pointer", "kind": kind, "pointerType": pointerType, "id": id.hashValue,
                    "x": logical.point.x, "y": logical.point.y, "dx": logical.point.x - previous.x, "dy": logical.point.y - previous.y,
                    "inside": logical.inside, "buttons": kind == "up" || kind == "cancel" ? 0 : 1, "pressure": touch.maximumPossibleForce > 0 ? min(1, max(0, touch.force / touch.maximumPossibleForce)) : (kind == "up" || kind == "cancel" ? 0 : 1),
                    "primary": primary.map { touch === $0 } ?? false, "timestamp": timestamp()
                ])
            }
            if kind == "up" || kind == "cancel" { points.removeValue(forKey: id) } else { points[id] = logical.point }
        }
    }

    override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) { removeSelectedIfNeeded(presses); emitPresses(presses, kind: "down"); super.pressesBegan(presses, with: event) }
    override func pressesEnded(_ presses: Set<UIPress>, with event: UIPressesEvent?) { emitPresses(presses, kind: "up"); super.pressesEnded(presses, with: event) }

    private func emitPresses(_ presses: Set<UIPress>, kind: String) {
        guard let action = onKey else { return }
        for press in presses {
            guard let key = press.key else { continue }
            let flags = key.modifierFlags
            state.run(action, item: [
                "source": "key", "kind": kind, "key": key.charactersIgnoringModifiers, "code": String(key.keyCode.rawValue), "repeat": false,
                "alt": flags.contains(.alternate), "ctrl": flags.contains(.control), "meta": flags.contains(.command), "shift": flags.contains(.shift), "timestamp": timestamp()
            ])
        }
    }

    private func activeDrawMode() -> String {
        drawModePath.flatMap { state.canvasValue($0).map { String(describing: $0) } } ?? drawMode
    }

    private func layerRows() -> [[String: Any]] {
        guard let layersPath else { return [] }
        return state.canvasValue(layersPath) as? [[String: Any]] ?? []
    }

    private func layerId(_ layer: [String: Any]) -> String { layer["id"] as? String ?? "" }

    private func layerHit(_ layer: [String: Any], point: CGPoint) -> Bool {
        var layer = layer
        for (field, path) in layer["bind"] as? [String: String] ?? [:] {
            if let value = state.canvasValue(path) { layer[field] = value }
        }
        let x = number(layer["x"])
        let y = number(layer["y"])
        switch String(describing: layer["type"] ?? "") {
        case "circle":
            let radius = number(layer["radius"])
            return hypot(point.x - x, point.y - y) <= radius + max(4, number(layer["strokeWidth"], fallback: 1))
        case "rect", "image":
            return point.x >= x && point.x <= x + number(layer["width"]) && point.y >= y && point.y <= y + number(layer["height"])
        case "line":
            return segmentDistance(point, start: CGPoint(x: number(layer["x1"]), y: number(layer["y1"])), end: CGPoint(x: number(layer["x2"]), y: number(layer["y2"]))) <= max(4, number(layer["strokeWidth"], fallback: 1))
        case "polyline":
            let points = (layer["points"] as? [[String: Any]] ?? []).map { CGPoint(x: number($0["x"]), y: number($0["y"])) }
            let tolerance = max(6, number(layer["strokeWidth"], fallback: 1))
            if points.count == 1, let first = points.first { return hypot(point.x - first.x, point.y - first.y) <= tolerance }
            if layer["closed"] as? Bool == true, let first = points.first, let last = points.last,
               segmentDistance(point, start: last, end: first) <= tolerance { return true }
            return zip(points, points.dropFirst()).contains { segmentDistance(point, start: $0.0, end: $0.1) <= tolerance }
        case "text":
            let size = max(1, number(layer["size"], fallback: 16))
            let width = CGFloat(max(1, String(describing: layer["text"] ?? "").utf16.count)) * size * 0.6
            let align = layer["align"] as? String ?? "start"
            let left = align == "center" ? x - width / 2 : align == "end" ? x - width : x
            return point.x >= left && point.x <= left + width && point.y >= y - size && point.y <= y
        default:
            return false
        }
    }

    private func segmentDistance(_ point: CGPoint, start: CGPoint, end: CGPoint) -> CGFloat {
        let dx = end.x - start.x
        let dy = end.y - start.y
        let length = dx * dx + dy * dy
        let t = length == 0 ? 0 : min(1, max(0, ((point.x - start.x) * dx + (point.y - start.y) * dy) / length))
        return hypot(point.x - start.x - t * dx, point.y - start.y - t * dy)
    }

    private func selectedLayerId() -> String {
        selectedPath.map { state.canvasValue($0) as? String ?? "" } ?? selection.id
    }

    private func selectLayer(_ id: String) {
        if let selectedPath { state.write(selectedPath, value: id) } else { selection.id = id }
    }

    private func finishDrawing() {
        drawingLayerId = nil
        drawingStart = nil
        drawingPointer = nil
        drawingMode = nil
    }

    private func removeLayer(_ id: String) {
        guard let layersPath else { return }
        var rows = layerRows()
        guard let index = rows.firstIndex(where: { layerId($0) == id }) else { return }
        let removed = rows.remove(at: index)
        state.write(layersPath, value: rows)
        if selectedLayerId() == id { selectLayer("") }
        if drawingLayerId == id { finishDrawing() }
        runLayerEvent(onLayerRemove, event: "remove", layer: removed)
    }

    private func runLayerEvent(_ action: String?, event: String, layer: [String: Any]?) {
        guard let action else { return }
        state.run(action, item: ["event": event, "layer": layer ?? [:]])
    }

    private func updateLayer(at point: CGPoint, kind: String, pointer: ObjectIdentifier) {
        guard let layersPath else { return }
        var rows = layerRows()
        if kind == "down" {
            guard draw, drawingPointer == nil else { return }
            let mode = activeDrawMode()
            if mode == "select" || mode == "erase" {
                let selected = rows.reversed().first { layerId($0).isEmpty == false && layerHit($0, point: point) }
                let id = selected.map(layerId) ?? ""
                if mode == "erase" {
                    if !id.isEmpty { removeLayer(id) }
                } else {
                    selectLayer(id)
                    runLayerEvent(onLayerSelect, event: "select", layer: selected)
                }
                return
            }
            let used = Set(rows.map(layerId))
            while used.contains("layer-\(nextLayerSequence)") { nextLayerSequence += 1 }
            let id = "layer-\(nextLayerSequence)"
            nextLayerSequence += 1
            let layer: [String: Any]
            if mode == "rect" {
                layer = ["id": id, "type": "rect", "x": point.x, "y": point.y, "width": 0, "height": 0, "fill": "transparent", "stroke": "primary", "strokeWidth": 3, "radius": 4]
            } else if mode == "circle" {
                layer = ["id": id, "type": "circle", "x": point.x, "y": point.y, "radius": 0, "fill": "transparent", "stroke": "primary", "strokeWidth": 3]
            } else {
                layer = ["id": id, "type": "polyline", "points": [["x": point.x, "y": point.y]], "stroke": "primary", "strokeWidth": 3, "closed": false]
            }
            rows.append(layer)
            state.write(layersPath, value: rows)
            drawingLayerId = id
            drawingStart = point
            drawingPointer = pointer
            drawingMode = mode
            return
        }
        guard drawingPointer == pointer else { return }
        guard let drawingLayerId, let index = rows.firstIndex(where: { layerId($0) == drawingLayerId }), let start = drawingStart, let mode = drawingMode else { finishDrawing(); return }
        var layer = rows[index]
        if kind == "move" || kind == "up" {
            if mode == "rect" {
                layer["x"] = min(start.x, point.x); layer["y"] = min(start.y, point.y)
                layer["width"] = abs(point.x - start.x); layer["height"] = abs(point.y - start.y)
            } else if mode == "circle" {
                layer["x"] = (start.x + point.x) / 2; layer["y"] = (start.y + point.y) / 2
                layer["radius"] = hypot(point.x - start.x, point.y - start.y) / 2
            } else {
                var points = layer["points"] as? [[String: Any]] ?? []
                if points.last.map({ number($0["x"]) != point.x || number($0["y"]) != point.y }) ?? true {
                    points.append(["x": point.x, "y": point.y])
                }
                layer["points"] = points
            }
            rows[index] = layer
            state.write(layersPath, value: rows)
        }
        if kind == "up" {
            selectLayer(drawingLayerId)
            finishDrawing()
            runLayerEvent(onLayerAdd, event: "add", layer: layer)
            runLayerEvent(onLayerChange, event: "change", layer: layer)
        } else if kind == "cancel" {
            rows.remove(at: index)
            state.write(layersPath, value: rows)
            finishDrawing()
        }
    }

    private func removeSelectedIfNeeded(_ presses: Set<UIPress>) {
        guard layersPath != nil else { return }
        for press in presses {
            guard let key = press.key else { continue }
            let value = key.charactersIgnoringModifiers
            guard key.keyCode == .keyboardDeleteOrBackspace || key.keyCode == .keyboardDeleteForward || value == "\u{8}" || value == "\u{7f}" || value.lowercased() == "delete" else { continue }
            let selected = selectedLayerId()
            if !selected.isEmpty { removeLayer(selected) }
        }
    }

    private func logicalPoint(_ point: CGPoint) -> (point: CGPoint, inside: Bool) {
        var sx = bounds.width / max(1, viewWidth)
        var sy = bounds.height / max(1, viewHeight)
        var left: CGFloat = 0
        var top: CGFloat = 0
        if fit != "stretch" {
            let scale = fit == "cover" ? max(sx, sy) : min(sx, sy)
            sx = scale; sy = scale
            left = (bounds.width - viewWidth * scale) / 2
            top = (bounds.height - viewHeight * scale) / 2
        }
        let x = (point.x - left) / max(0.0001, sx)
        let y = (point.y - top) / max(0.0001, sy)
        return (CGPoint(x: min(viewWidth, max(0, x)), y: min(viewHeight, max(0, y))), x >= 0 && x <= viewWidth && y >= 0 && y <= viewHeight)
    }

    private func startMotion() {
        guard let action = onMotion, motion.isDeviceMotionAvailable, !motion.isDeviceMotionActive else { return }
        motion.deviceMotionUpdateInterval = 1.0 / Double(max(1, motionRate))
        motion.startDeviceMotionUpdates(to: .main) { [weak self] value, _ in
            guard let self, let value else { return }
            let vector = self.screenVector(x: value.userAcceleration.x * 9.80665, y: -value.userAcceleration.y * 9.80665)
            self.state.run(action, item: [
                "source": "motion", "acceleration": ["x": vector.x, "y": vector.y, "z": value.userAcceleration.z * 9.80665],
                "rotation": ["alpha": value.attitude.yaw * 180 / .pi, "beta": value.attitude.pitch * 180 / .pi, "gamma": value.attitude.roll * 180 / .pi],
                "interval": self.motion.deviceMotionUpdateInterval * 1000, "timestamp": self.timestamp()
            ])
        }
    }

    private func stopMotion() { if motion.isDeviceMotionActive { motion.stopDeviceMotionUpdates() } }

    private func screenVector(x: Double, y: Double) -> (x: Double, y: Double) {
        switch window?.windowScene?.interfaceOrientation { case .landscapeLeft: return (-y, x); case .landscapeRight: return (y, -x); case .portraitUpsideDown: return (-x, -y); default: return (x, y) }
    }

    private func number(_ value: Any?, fallback: CGFloat = 0) -> CGFloat {
        if let number = value as? NSNumber { return CGFloat(number.doubleValue) }
        if let text = value as? String, let number = Double(text) { return CGFloat(number) }
        return fallback
    }

    private func timestamp() -> Double { max(0, ProcessInfo.processInfo.systemUptime * 1000 - started) }
}
"#
