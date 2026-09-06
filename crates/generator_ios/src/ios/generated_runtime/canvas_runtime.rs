fn swift_runtime_canvas() -> &'static str {
    r#"@MainActor
final class DoweCanvasImageStore: ObservableObject {
    @Published var images: [String: UIImage] = [:]

    func load(_ sources: [String]) {
        for source in sources where images[source] == nil {
            if let local = UIImage(named: source.trimmingCharacters(in: CharacterSet(charactersIn: "/"))) {
                images[source] = local
            } else if let url = URL(string: source), url.scheme == "https" {
                Task {
                    if let (data, _) = try? await URLSession.shared.data(from: url), let image = UIImage(data: data) {
                        images[source] = image
                    }
                }
            }
        }
    }
}

@MainActor
final class DoweCanvasSelection: ObservableObject {
    @Published var id = ""
}

struct DoweCanvasView: View {
    @ObservedObject var state: DoweReactiveState
    let scenePath: String
    let viewWidth: CGFloat
    let viewHeight: CGFloat
    let fit: String
    let fps: Int
    let autoplay: Bool
    let pixelated: Bool
    let backgroundColor: Color
    let label: String
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
    @State private var started = Date()
    @StateObject private var imageStore = DoweCanvasImageStore()
    @StateObject private var selection = DoweCanvasSelection()

    private var commands: [[String: Any]] {
        state.candles(layersPath ?? scenePath).map(boundCommand)
    }

    private func boundCommand(_ command: [String: Any]) -> [String: Any] {
        guard let bindings = command["bind"] as? [String: Any] else { return command }
        var output = command
        for (field, path) in bindings {
            if let path = path as? String, let value = state.canvasValue(path) { output[field] = value }
        }
        return output
    }

    private var imageSources: [String] {
        commands.compactMap { command in command["type"] as? String == "image" ? command["src"] as? String : nil }
    }

    var body: some View {
        let renderedCommands = commands
        let selectedId = selectedPath.map { state.canvasValue($0) as? String ?? "" } ?? selection.id
        return ZStack {
            TimelineView(.animation(minimumInterval: 1.0 / Double(max(1, fps)), paused: !autoplay || UIAccessibility.isReduceMotionEnabled)) { timeline in
                Canvas { context, size in
                    drawScene(context: &context, size: size, date: timeline.date, commands: renderedCommands, selectedId: selectedId)
                }
            }
            if onPointer != nil || onKey != nil || onMotion != nil || draw || layersPath != nil {
                DoweCanvasInputBridge(state: state, selection: selection, viewWidth: viewWidth, viewHeight: viewHeight, fit: fit, onPointer: onPointer, onKey: onKey, onMotion: onMotion, motionRate: motionRate, draw: draw, drawMode: drawMode, drawModePath: drawModePath, layersPath: layersPath, selectedPath: selectedPath, onLayerAdd: onLayerAdd, onLayerChange: onLayerChange, onLayerRemove: onLayerRemove, onLayerSelect: onLayerSelect)
                    .accessibilityHidden(true)
            }
        }
        .background(backgroundColor)
        .clipped()
        .accessibilityElement(children: .ignore)
        .accessibilityAddTraits(.isImage)
        .accessibilityLabel(Text(label))
        .onAppear {
            started = Date()
            imageStore.load(imageSources)
        }
        .onChange(of: imageSources) { _, sources in
            imageStore.load(sources)
        }
    }

    private func drawScene(context: inout GraphicsContext, size: CGSize, date: Date, commands: [[String: Any]], selectedId: String) {
        let scaleX = size.width / max(CGFloat(1), viewWidth)
        let scaleY = size.height / max(CGFloat(1), viewHeight)
        let scale = fit == "cover" ? max(scaleX, scaleY) : min(scaleX, scaleY)
        let sx = fit == "stretch" ? scaleX : scale
        let sy = fit == "stretch" ? scaleY : scale
        context.clip(to: Path(CGRect(origin: .zero, size: size)))
        context.translateBy(x: (size.width - viewWidth * sx) / 2, y: (size.height - viewHeight * sy) / 2)
        context.scaleBy(x: sx, y: sy)
        let elapsed = autoplay && !UIAccessibility.isReduceMotionEnabled ? max(0, date.timeIntervalSince(started)) : 0
        for command in commands {
            draw(command, elapsed: elapsed, selected: !selectedId.isEmpty && command["id"] as? String == selectedId, context: &context)
        }
    }

    private func draw(_ command: [String: Any], elapsed: TimeInterval, selected: Bool, context: inout GraphicsContext) {
        guard let type = command["type"] as? String else { return }
        let motion = command["motion"] as? [String: Any] ?? [:]
        let x = number(command["x"])
        let y = number(command["y"])
        var dx = number(motion["vx"]) * elapsed
        var dy = number(motion["vy"]) * elapsed
        if motion["wrap"] as? Bool == true {
            dx = ((x + dx).truncatingRemainder(dividingBy: viewWidth) + viewWidth).truncatingRemainder(dividingBy: viewWidth) - x
            dy = ((y + dy).truncatingRemainder(dividingBy: viewHeight) + viewHeight).truncatingRemainder(dividingBy: viewHeight) - y
        }
        let rotation = number(command["rotation"]) + number(motion["rotation"]) * elapsed
        let pulse = number(motion["pulse"])
        let alpha = min(1, max(0, number(command["opacity"], fallback: 1) * (pulse == 0 ? 1 : 0.55 + 0.45 * sin(elapsed * pulse * .pi * 2))))
        var drawing = context
        drawing.opacity = alpha
        drawing.translateBy(x: dx, y: dy)
        if rotation != 0 {
            drawing.translateBy(x: x, y: y)
            drawing.rotate(by: .degrees(rotation))
            drawing.translateBy(x: -x, y: -y)
        }
        let fill = color(command["fill"])
        let stroke = color(command["stroke"])
        let strokeWidth = max(0, number(command["strokeWidth"], fallback: 1))
        switch type {
        case "rect":
            let width = max(0, number(command["width"]))
            let height = max(0, number(command["height"]))
            let radius = max(0, min(number(command["radius"]), min(width, height) / 2))
            let path = Path(roundedRect: CGRect(x: x, y: y, width: width, height: height), cornerRadius: radius)
            if let fill { drawing.fill(path, with: .color(fill)) }
            if let stroke { drawing.stroke(path, with: .color(stroke), lineWidth: strokeWidth) }
        case "circle":
            let radius = max(0, number(command["radius"]))
            let path = Path(ellipseIn: CGRect(x: x - radius, y: y - radius, width: radius * 2, height: radius * 2))
            if let fill { drawing.fill(path, with: .color(fill)) }
            if let stroke { drawing.stroke(path, with: .color(stroke), lineWidth: strokeWidth) }
        case "line":
            var path = Path()
            path.move(to: CGPoint(x: number(command["x1"]), y: number(command["y1"])))
            path.addLine(to: CGPoint(x: number(command["x2"]), y: number(command["y2"])))
            drawing.stroke(path, with: .color(stroke ?? DoweDesign.backgroundText), lineWidth: strokeWidth)
        case "polyline":
            let points = command["points"] as? [[String: Any]] ?? []
            guard let first = points.first else { return }
            var path = Path()
            path.move(to: CGPoint(x: number(first["x"]), y: number(first["y"])))
            for point in points.dropFirst() { path.addLine(to: CGPoint(x: number(point["x"]), y: number(point["y"]))) }
            if command["closed"] as? Bool == true { path.closeSubpath() }
            if let fill { drawing.fill(path, with: .color(fill)) }
            if let stroke { drawing.stroke(path, with: .color(stroke), lineWidth: strokeWidth) }
        case "text":
            let alignment: UnitPoint = command["align"] as? String == "center" ? .center : command["align"] as? String == "end" ? .trailing : .leading
            let size = max(1, number(command["size"], fallback: 16))
            let text = Text(String(describing: command["text"] ?? "")).font(.system(size: size)).foregroundColor(fill ?? DoweDesign.backgroundText)
            drawing.draw(text, at: CGPoint(x: x, y: y - size / 2), anchor: alignment)
        case "image":
            if let source = command["src"] as? String, let image = imageStore.images[source] {
                let width = max(0, number(command["width"]))
                let height = max(0, number(command["height"]))
                let rect = imageRect(image.size, destination: CGRect(x: x, y: y, width: width, height: height), fit: command["fit"] as? String ?? "contain")
                drawing.draw(Image(uiImage: image).interpolation(pixelated ? .none : .medium), in: rect)
            }
        default:
            break
        }
        if selected, let path = selectionPath(command) {
            drawing.opacity = 1
            drawing.stroke(path, with: .color(DoweDesign.primary), style: StrokeStyle(lineWidth: 2, dash: [5, 3]))
        }
    }

    private func selectionPath(_ command: [String: Any]) -> Path? {
        let x = number(command["x"])
        let y = number(command["y"])
        let type = command["type"] as? String ?? ""
        switch type {
        case "circle":
            let radius = max(0, number(command["radius"])) + 4
            return Path(ellipseIn: CGRect(x: x - radius, y: y - radius, width: radius * 2, height: radius * 2))
        case "rect", "image":
            return Path(CGRect(x: x, y: y, width: number(command["width"]), height: number(command["height"])).insetBy(dx: -4, dy: -4))
        case "line", "polyline":
            let points: [[String: Any]] = type == "line" ? [["x": number(command["x1"]), "y": number(command["y1"])], ["x": number(command["x2"]), "y": number(command["y2"])]] : command["points"] as? [[String: Any]] ?? []
            guard let first = points.first else { return nil }
            var bounds = CGRect(x: number(first["x"]), y: number(first["y"]), width: 0, height: 0)
            for point in points.dropFirst() {
                bounds = bounds.union(CGRect(x: number(point["x"]), y: number(point["y"]), width: 0, height: 0))
            }
            return Path(bounds.insetBy(dx: -4, dy: -4))
        case "text":
            let size = max(1, number(command["size"], fallback: 16))
            let width = CGFloat(max(1, String(describing: command["text"] ?? "").utf16.count)) * size * 0.6
            let align = command["align"] as? String ?? "start"
            let left = align == "center" ? x - width / 2 : align == "end" ? x - width : x
            return Path(CGRect(x: left, y: y - size, width: width, height: size).insetBy(dx: -4, dy: -4))
        default: return nil
        }
    }

    private func imageRect(_ source: CGSize, destination: CGRect, fit: String) -> CGRect {
        guard fit != "stretch", source.width > 0, source.height > 0 else { return destination }
        let scale = fit == "cover" ? max(destination.width / source.width, destination.height / source.height) : min(destination.width / source.width, destination.height / source.height)
        let size = CGSize(width: source.width * scale, height: source.height * scale)
        return CGRect(x: destination.midX - size.width / 2, y: destination.midY - size.height / 2, width: size.width, height: size.height)
    }

    private func number(_ value: Any?, fallback: CGFloat = 0) -> CGFloat {
        if let number = value as? NSNumber { return CGFloat(number.doubleValue) }
        if let text = value as? String, let number = Double(text) { return CGFloat(number) }
        return fallback
    }

    private func color(_ value: Any?) -> Color? {
        guard let name = value as? String else { return nil }
        switch name {
        case "primary": return DoweDesign.primary
        case "primaryText": return DoweDesign.primaryText
        case "secondary": return DoweDesign.secondary
        case "secondaryText": return DoweDesign.secondaryText
        case "accent": return DoweDesign.accent
        case "accentText": return DoweDesign.accentText
        case "muted": return DoweDesign.muted
        case "mutedText": return DoweDesign.mutedText
        case "background": return DoweDesign.background
        case "foreground", "currentColor", "backgroundText": return DoweDesign.backgroundText
        case "surface": return DoweDesign.surface
        case "surfaceText": return DoweDesign.surfaceText
        case "success": return DoweDesign.success
        case "successText": return DoweDesign.successText
        case "info": return DoweDesign.info
        case "infoText": return DoweDesign.infoText
        case "warning": return DoweDesign.warning
        case "warningText": return DoweDesign.warningText
        case "danger": return DoweDesign.danger
        case "dangerText": return DoweDesign.dangerText
        case "primary": return DoweDesign.primary
        case "primaryText": return DoweDesign.primaryText
        case "secondary": return DoweDesign.secondary
        case "secondaryText": return DoweDesign.secondaryText
        case "accent": return DoweDesign.accent
        case "accentText": return DoweDesign.accentText
        case "muted": return DoweDesign.muted
        case "mutedText": return DoweDesign.mutedText
        case "success": return DoweDesign.success
        case "successText": return DoweDesign.successText
        case "info": return DoweDesign.info
        case "infoText": return DoweDesign.infoText
        case "warning": return DoweDesign.warning
        case "warningText": return DoweDesign.warningText
        case "danger": return DoweDesign.danger
        case "dangerText": return DoweDesign.dangerText
        case "transparent": return Color.clear
        default: return doweColorFromHex(name, fallback: DoweDesign.backgroundText)
        }
    }
}

struct DoweCanvasInputBridge: UIViewRepresentable {
    @ObservedObject var state: DoweReactiveState
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
}
