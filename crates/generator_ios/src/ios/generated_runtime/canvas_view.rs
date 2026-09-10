r#"    @ObservedObject var state: DoweReactiveState
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
"#
