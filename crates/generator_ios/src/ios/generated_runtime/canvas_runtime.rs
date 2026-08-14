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
    @State private var started = Date()
    @StateObject private var imageStore = DoweCanvasImageStore()

    private var commands: [[String: Any]] {
        state.candles(scenePath).map(boundCommand)
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
        ZStack {
            TimelineView(.animation(minimumInterval: 1.0 / Double(max(1, fps)), paused: !autoplay || UIAccessibility.isReduceMotionEnabled)) { timeline in
                Canvas { context, size in
                    drawScene(context: &context, size: size, date: timeline.date)
                }
            }
            if onPointer != nil || onKey != nil || onMotion != nil {
                DoweCanvasInputBridge(state: state, viewWidth: viewWidth, viewHeight: viewHeight, fit: fit, onPointer: onPointer, onKey: onKey, onMotion: onMotion, motionRate: motionRate)
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

    private func drawScene(context: inout GraphicsContext, size: CGSize, date: Date) {
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
            draw(command, elapsed: elapsed, context: &context)
        }
    }

    private func draw(_ command: [String: Any], elapsed: TimeInterval, context: inout GraphicsContext) {
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
            let text = Text(String(describing: command["text"] ?? "")).font(.system(size: max(1, number(command["size"], fallback: 16)))).foregroundColor(fill ?? DoweDesign.backgroundText)
            drawing.draw(text, at: CGPoint(x: x, y: y), anchor: alignment)
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
        case "tertiary": return DoweDesign.tertiary
        case "tertiaryText": return DoweDesign.tertiaryText
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
        case "softPrimary": return DoweDesign.softPrimary
        case "softPrimaryText": return DoweDesign.softPrimaryText
        case "softSecondary": return DoweDesign.softSecondary
        case "softSecondaryText": return DoweDesign.softSecondaryText
        case "softTertiary": return DoweDesign.softTertiary
        case "softTertiaryText": return DoweDesign.softTertiaryText
        case "softMuted": return DoweDesign.softMuted
        case "softMutedText": return DoweDesign.softMutedText
        case "softSuccess": return DoweDesign.softSuccess
        case "softSuccessText": return DoweDesign.softSuccessText
        case "softInfo": return DoweDesign.softInfo
        case "softInfoText": return DoweDesign.softInfoText
        case "softWarning": return DoweDesign.softWarning
        case "softWarningText": return DoweDesign.softWarningText
        case "softDanger": return DoweDesign.softDanger
        case "softDangerText": return DoweDesign.softDangerText
        case "transparent": return Color.clear
        default: return doweColorFromHex(name, fallback: DoweDesign.backgroundText)
        }
    }
}

struct DoweCanvasInputBridge: UIViewRepresentable {
    @ObservedObject var state: DoweReactiveState
    let viewWidth: CGFloat
    let viewHeight: CGFloat
    let fit: String
    let onPointer: String?
    let onKey: String?
    let onMotion: String?
    let motionRate: Int

    func makeUIView(context: Context) -> DoweCanvasInputUIView {
        DoweCanvasInputUIView(state: state, viewWidth: viewWidth, viewHeight: viewHeight, fit: fit, onPointer: onPointer, onKey: onKey, onMotion: onMotion, motionRate: motionRate)
    }

    func updateUIView(_ view: DoweCanvasInputUIView, context: Context) {
        view.update(state: state, viewWidth: viewWidth, viewHeight: viewHeight, fit: fit, onPointer: onPointer, onKey: onKey, onMotion: onMotion, motionRate: motionRate)
    }
}

@MainActor
final class DoweCanvasInputUIView: UIView {
    private var state: DoweReactiveState
    private var viewWidth: CGFloat
    private var viewHeight: CGFloat
    private var fit: String
    private var onPointer: String?
    private var onKey: String?
    private var onMotion: String?
    private var motionRate: Int
    private let started = ProcessInfo.processInfo.systemUptime * 1000
    private var points: [ObjectIdentifier: CGPoint] = [:]
    private let motion = CMMotionManager()

    override var canBecomeFirstResponder: Bool { onKey != nil }

    init(state: DoweReactiveState, viewWidth: CGFloat, viewHeight: CGFloat, fit: String, onPointer: String?, onKey: String?, onMotion: String?, motionRate: Int) {
        self.state = state
        self.viewWidth = viewWidth
        self.viewHeight = viewHeight
        self.fit = fit
        self.onPointer = onPointer
        self.onKey = onKey
        self.onMotion = onMotion
        self.motionRate = motionRate
        super.init(frame: .zero)
        isMultipleTouchEnabled = true
        backgroundColor = .clear
    }

    required init?(coder: NSCoder) { nil }

    func update(state: DoweReactiveState, viewWidth: CGFloat, viewHeight: CGFloat, fit: String, onPointer: String?, onKey: String?, onMotion: String?, motionRate: Int) {
        self.state = state
        self.viewWidth = viewWidth
        self.viewHeight = viewHeight
        self.fit = fit
        self.onPointer = onPointer
        self.onKey = onKey
        let restart = self.onMotion != onMotion || self.motionRate != motionRate
        self.onMotion = onMotion
        self.motionRate = motionRate
        if restart { stopMotion(); startMotion() }
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()
        if window == nil { stopMotion(); points.removeAll() } else { startMotion() }
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) { if onKey != nil { becomeFirstResponder() }; emitTouches(touches, kind: "down", event: event) }
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) { emitTouches(touches, kind: "move", event: event) }
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) { emitTouches(touches, kind: "up", event: event) }
    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) { emitTouches(touches, kind: "cancel", event: event) }

    private func emitTouches(_ touches: Set<UITouch>, kind: String, event: UIEvent?) {
        guard let action = onPointer else { return }
        let primary = event?.allTouches?.first
        for touch in touches {
            let id = ObjectIdentifier(touch)
            let raw = touch.location(in: self)
            let logical = logicalPoint(raw)
            let previous = points[id] ?? logical.point
            let pointerType: String
            switch touch.type { case .pencil: pointerType = "pen"; case .indirectPointer: pointerType = "mouse"; default: pointerType = "touch" }
            state.run(action, item: [
                "source": "pointer", "kind": kind, "pointerType": pointerType, "id": id.hashValue,
                "x": logical.point.x, "y": logical.point.y, "dx": logical.point.x - previous.x, "dy": logical.point.y - previous.y,
                "inside": logical.inside, "buttons": kind == "up" || kind == "cancel" ? 0 : 1, "pressure": touch.maximumPossibleForce > 0 ? min(1, max(0, touch.force / touch.maximumPossibleForce)) : (kind == "up" || kind == "cancel" ? 0 : 1),
                "primary": primary.map { touch === $0 } ?? false, "timestamp": timestamp()
            ])
            if kind == "up" || kind == "cancel" { points.removeValue(forKey: id) } else { points[id] = logical.point }
        }
    }

    override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) { emitPresses(presses, kind: "down"); super.pressesBegan(presses, with: event) }
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

    private func timestamp() -> Double { max(0, ProcessInfo.processInfo.systemUptime * 1000 - started) }
}
"#
}
