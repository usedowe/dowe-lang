fn swift_runtime_game_raycast() -> &'static str {
    r#"private struct DoweRaycastCamera {
    var x: CGFloat
    var y: CGFloat
    var angle: CGFloat
    var fov: CGFloat
    var pitch: CGFloat
}

private func doweRaycastNumber(_ value: Any?, fallback: CGFloat = 0) -> CGFloat {
    if let number = value as? NSNumber { return CGFloat(number.doubleValue) }
    if let text = value as? String, let number = Double(text) { return CGFloat(number) }
    return fallback
}

private func doweRaycastCamera(_ value: Any?) -> DoweRaycastCamera {
    let source = value as? [String: Any] ?? [:]
    return DoweRaycastCamera(
        x: doweRaycastNumber(source["x"], fallback: 1.5),
        y: doweRaycastNumber(source["y"], fallback: 1.5),
        angle: doweRaycastNumber(source["angle"]),
        fov: min(1.8, max(0.35, doweRaycastNumber(source["fov"], fallback: .pi / 3))),
        pitch: doweRaycastNumber(source["pitch"])
    )
}

private func doweRaycastRows(_ value: Any?) -> [String] {
    ((value as? [String: Any])?["map"] as? [Any] ?? []).map { String(describing: $0) }
}

private func doweRaycastCell(_ rows: [String], _ x: Int, _ y: Int) -> Character {
    guard x >= 0, y >= 0, y < rows.count else { return "1" }
    let row = Array(rows[y])
    return x < row.count ? row[x] : "1"
}

private func doweRaycastSolid(_ cell: Character) -> Bool {
    !["0", " ", ".", "_"].contains(cell)
}

private func doweRaycastCanWalk(_ rows: [String], _ x: CGFloat, _ y: CGFloat) -> Bool {
    let radius: CGFloat = 0.18
    return [
        (x - radius, y - radius), (x + radius, y - radius),
        (x - radius, y + radius), (x + radius, y + radius)
    ].allSatisfy { !doweRaycastSolid(doweRaycastCell(rows, Int(floor($0.0)), Int(floor($0.1)))) }
}

@MainActor private func doweRaycastColor(_ value: Any?, fallback: Color) -> Color {
    guard let name = value as? String else { return fallback }
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
    case "backgroundText", "foreground", "currentColor": return DoweDesign.backgroundText
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
    case "transparent": return .clear
    default: return doweColorFromHex(name, fallback: fallback)
    }
}

struct DoweRaycastGameView: View {
    @ObservedObject var state: DoweReactiveState
    let worldPath: String
    let cameraPath: String
    let controls: String
    let moveSpeed: CGFloat
    let turnSpeed: CGFloat
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
    let onFire: String?
    @State private var camera: DoweRaycastCamera
    @State private var dragTranslation: CGSize = .zero
    @State private var fireAt = Date.distantPast
    @State private var defeated = Set<Int>()

    init(state: DoweReactiveState, worldPath: String, cameraPath: String, controls: String, moveSpeed: CGFloat, turnSpeed: CGFloat, viewWidth: CGFloat, viewHeight: CGFloat, fit: String, fps: Int, autoplay: Bool, pixelated: Bool, backgroundColor: Color, label: String, onPointer: String?, onKey: String?, onFire: String?) {
        self.state = state
        self.worldPath = worldPath
        self.cameraPath = cameraPath
        self.controls = controls
        self.moveSpeed = moveSpeed
        self.turnSpeed = turnSpeed
        self.viewWidth = viewWidth
        self.viewHeight = viewHeight
        self.fit = fit
        self.fps = fps
        self.autoplay = autoplay
        self.pixelated = pixelated
        self.backgroundColor = backgroundColor
        self.label = label
        self.onPointer = onPointer
        self.onKey = onKey
        self.onFire = onFire
        _camera = State(initialValue: doweRaycastCamera(state.canvasValue(cameraPath)))
    }

    private var world: [String: Any] { state.canvasValue(worldPath) as? [String: Any] ?? [:] }

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / Double(max(1, fps)), paused: !autoplay || UIAccessibility.isReduceMotionEnabled)) { _ in
            Canvas { context, size in
                context.fill(Path(CGRect(origin: .zero, size: size)), with: .color(backgroundColor))
                let scaleX = size.width / max(1, viewWidth)
                let scaleY = size.height / max(1, viewHeight)
                let scale = fit == "stretch" ? 1 : (fit == "cover" ? max(scaleX, scaleY) : min(scaleX, scaleY))
                let sx = fit == "stretch" ? scaleX : scale
                let sy = fit == "stretch" ? scaleY : scale
                context.translateBy(x: (size.width - viewWidth * sx) / 2, y: (size.height - viewHeight * sy) / 2)
                context.scaleBy(x: sx, y: sy)
                doweRaycastDraw(&context, world: world, camera: camera, width: viewWidth, height: viewHeight, fireAt: fireAt, defeated: defeated)
            }
            .contentShape(Rectangle())
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in
                        let dx = value.translation.width - dragTranslation.width
                        let dy = value.translation.height - dragTranslation.height
                        dragTranslation = value.translation
                        if controls == "doom" {
                            camera.angle = doweRaycastWrapped(camera.angle + dx * 0.006)
                            move(dy: dy)
                        }
                        emitPointer("move", value: value, dx: dx, dy: dy)
                    }
                    .onEnded { value in
                        let dx = value.translation.width
                        let dy = value.translation.height
                        if controls == "doom" && abs(dx) < 8 && abs(dy) < 8 { fire() }
                        emitPointer("up", value: value, dx: dx - dragTranslation.width, dy: dy - dragTranslation.height)
                        dragTranslation = .zero
                    }
            )
        }
        .clipped()
        .accessibilityElement(children: .ignore)
        .accessibilityAddTraits(.isImage)
        .accessibilityLabel(Text(label))
        .background(backgroundColor)
    }

    private func move(dy: CGFloat) {
        let amount = -dy * 0.012 * max(0.1, moveSpeed / 2)
        guard abs(amount) > 0.001 else { return }
        let rows = doweRaycastRows(world)
        let nextX = camera.x + cos(camera.angle) * amount
        let nextY = camera.y + sin(camera.angle) * amount
        if doweRaycastCanWalk(rows, nextX, camera.y) { camera.x = nextX }
        if doweRaycastCanWalk(rows, camera.x, nextY) { camera.y = nextY }
    }

    private func fire() {
        let now = Date()
        guard now.timeIntervalSince(fireAt) > 0.13 else { return }
        fireAt = now
        let rows = doweRaycastRows(world)
        let wall = doweRaycastWallDistance(rows, camera: camera, angle: camera.angle)
        var target: Any?
        var nearest = CGFloat.greatestFiniteMagnitude
        if let sprites = world["sprites"] as? [[String: Any]] {
            for (index, sprite) in sprites.enumerated() where !defeated.contains(index) {
                let dx = doweRaycastNumber(sprite["x"]) - camera.x
                let dy = doweRaycastNumber(sprite["y"]) - camera.y
                let distance = hypot(dx, dy)
                let relative = abs(doweRaycastWrapped(atan2(dy, dx) - camera.angle))
                if distance < wall && distance < nearest && relative < 0.1 {
                    target = sprite["id"] ?? index
                    nearest = distance
                    defeated.insert(index)
                }
            }
        }
        if let onFire { state.run(onFire, item: ["source": "game", "kind": "fire", "data": ["x": camera.x, "y": camera.y, "angle": camera.angle, "target": target ?? NSNull(), "hit": target != nil]]) }
    }

    private func emitPointer(_ kind: String, value: DragGesture.Value, dx: CGFloat, dy: CGFloat) {
        guard let onPointer else { return }
        let x = max(0, min(viewWidth, value.location.x))
        let y = max(0, min(viewHeight, value.location.y))
        state.run(onPointer, item: ["source": "pointer", "kind": kind, "pointerType": "touch", "id": 0, "x": x, "y": y, "dx": dx, "dy": dy, "inside": true, "buttons": kind == "up" ? 0 : 1, "pressure": kind == "up" ? 0 : 1, "primary": true, "timestamp": Date().timeIntervalSince1970 * 1000])
    }
}

private func doweRaycastWrapped(_ value: CGFloat) -> CGFloat {
    var next = value
    while next > .pi { next -= .pi * 2 }
    while next < -.pi { next += .pi * 2 }
    return next
}

private func doweRaycastWallDistance(_ rows: [String], camera: DoweRaycastCamera, angle: CGFloat) -> CGFloat {
    let rayX = cos(angle), rayY = sin(angle)
    var mapX = Int(floor(camera.x)), mapY = Int(floor(camera.y))
    let deltaX = abs(rayX) < 0.00001 ? CGFloat.greatestFiniteMagnitude : abs(1 / rayX)
    let deltaY = abs(rayY) < 0.00001 ? CGFloat.greatestFiniteMagnitude : abs(1 / rayY)
    let stepX = rayX < 0 ? -1 : 1, stepY = rayY < 0 ? -1 : 1
    var sideX = rayX < 0 ? (camera.x - CGFloat(mapX)) * deltaX : (CGFloat(mapX) + 1 - camera.x) * deltaX
    var sideY = rayY < 0 ? (camera.y - CGFloat(mapY)) * deltaY : (CGFloat(mapY) + 1 - camera.y) * deltaY
    var side = 0
    for _ in 0..<128 {
        if sideX < sideY { sideX += deltaX; mapX += stepX; side = 0 } else { sideY += deltaY; mapY += stepY; side = 1 }
        if doweRaycastSolid(doweRaycastCell(rows, mapX, mapY)) {
            let distance = side == 0 ? (CGFloat(mapX) - camera.x + CGFloat(1 - stepX) / 2) / rayX : (CGFloat(mapY) - camera.y + CGFloat(1 - stepY) / 2) / rayY
            return max(0.01, abs(distance))
        }
    }
    return 64
}

@MainActor private func doweRaycastDraw(_ context: inout GraphicsContext, world: [String: Any], camera: DoweRaycastCamera, width: CGFloat, height: CGFloat, fireAt: Date, defeated: Set<Int>) {
    let rows = doweRaycastRows(world)
    let horizon = height * (0.5 + camera.pitch * 0.2)
    context.fill(Path(CGRect(x: 0, y: 0, width: width, height: horizon)), with: .color(doweRaycastColor(world["ceiling"], fallback: Color(red: 0.06, green: 0.09, blue: 0.15))))
    context.fill(Path(CGRect(x: 0, y: horizon, width: width, height: max(0, height - horizon))), with: .color(doweRaycastColor(world["floor"], fallback: Color(red: 0.16, green: 0.2, blue: 0.27))))
    let columns = max(120, min(480, Int(width)))
    let step = width / CGFloat(columns)
    var depth = Array(repeating: CGFloat(64), count: columns)
    for column in 0..<columns {
        let cameraX = (CGFloat(column) + 0.5) / CGFloat(columns) * 2 - 1
        let angle = camera.angle + atan(cameraX * tan(camera.fov / 2))
        let rayX = cos(angle), rayY = sin(angle)
        var mapX = Int(floor(camera.x)), mapY = Int(floor(camera.y))
        let deltaX = abs(rayX) < 0.00001 ? CGFloat.greatestFiniteMagnitude : abs(1 / rayX)
        let deltaY = abs(rayY) < 0.00001 ? CGFloat.greatestFiniteMagnitude : abs(1 / rayY)
        let stepX = rayX < 0 ? -1 : 1, stepY = rayY < 0 ? -1 : 1
        var sideX = rayX < 0 ? (camera.x - CGFloat(mapX)) * deltaX : (CGFloat(mapX) + 1 - camera.x) * deltaX
        var sideY = rayY < 0 ? (camera.y - CGFloat(mapY)) * deltaY : (CGFloat(mapY) + 1 - camera.y) * deltaY
        var side = 0
        var cell: Character = "1"
        for _ in 0..<128 {
            if sideX < sideY { sideX += deltaX; mapX += stepX; side = 0 } else { sideY += deltaY; mapY += stepY; side = 1 }
            cell = doweRaycastCell(rows, mapX, mapY)
            if doweRaycastSolid(cell) { break }
        }
        let rawDistance = side == 0 ? (CGFloat(mapX) - camera.x + CGFloat(1 - stepX) / 2) / rayX : (CGFloat(mapY) - camera.y + CGFloat(1 - stepY) / 2) / rayY
        let corrected = max(0.05, abs(rawDistance) * cos(angle - camera.angle))
        depth[column] = corrected
        let wallHeight = min(height * 3, height / corrected)
        let wallMap = world["walls"] as? [String: Any]
        let color = doweRaycastColor(wallMap?[String(cell)] ?? world["wall"], fallback: DoweDesign.primary)
        context.fill(Path(CGRect(x: CGFloat(column) * step, y: horizon - wallHeight / 2, width: step + 1, height: wallHeight)), with: .color(color.opacity(side == 1 ? 0.58 : max(0.18, 1 - corrected / 20))))
    }
    if let sprites = world["sprites"] as? [[String: Any]] {
        let ordered = sprites.enumerated().filter { !defeated.contains($0.offset) }.sorted { left, right in
            let ld = hypot(doweRaycastNumber(left.element["x"]) - camera.x, doweRaycastNumber(left.element["y"]) - camera.y)
            let rd = hypot(doweRaycastNumber(right.element["x"]) - camera.x, doweRaycastNumber(right.element["y"]) - camera.y)
            return ld > rd
        }
        for (_, sprite) in ordered {
            let dx = doweRaycastNumber(sprite["x"]) - camera.x, dy = doweRaycastNumber(sprite["y"]) - camera.y
            let distance = hypot(dx, dy), relative = doweRaycastWrapped(atan2(dy, dx) - camera.angle)
            if distance < 0.2 || abs(relative) > camera.fov * 0.7 { continue }
            let projected = distance * cos(relative)
            let screenX = width / 2 + tan(relative) / tan(camera.fov / 2) * width / 2
            let size = max(5, height / max(0.1, projected) * doweRaycastNumber(sprite["size"], fallback: 0.75))
            let depthColumn = max(0, min(columns - 1, Int(screenX / width * CGFloat(columns))))
            if depth[depthColumn] < projected { continue }
            let rect = CGRect(x: screenX - size * 0.32, y: height / 2 - size * 0.33, width: size * 0.64, height: size * 0.64)
            context.fill(Path(rect), with: .color(doweRaycastColor(sprite["color"], fallback: .pink)))
            context.fill(Path(ellipseIn: CGRect(x: screenX - size * 0.2, y: height / 2 - size * 0.12, width: size * 0.12, height: size * 0.12)), with: .color(doweRaycastColor(sprite["eye"], fallback: .yellow)))
            context.fill(Path(ellipseIn: CGRect(x: screenX + size * 0.08, y: height / 2 - size * 0.12, width: size * 0.12, height: size * 0.12)), with: .color(doweRaycastColor(sprite["eye"], fallback: .yellow)))
        }
    }
    var crosshair = Path()
    crosshair.move(to: CGPoint(x: width / 2 - 8, y: height / 2)); crosshair.addLine(to: CGPoint(x: width / 2 - 2, y: height / 2)); crosshair.move(to: CGPoint(x: width / 2 + 2, y: height / 2)); crosshair.addLine(to: CGPoint(x: width / 2 + 8, y: height / 2)); crosshair.move(to: CGPoint(x: width / 2, y: height / 2 - 8)); crosshair.addLine(to: CGPoint(x: width / 2, y: height / 2 - 2)); crosshair.move(to: CGPoint(x: width / 2, y: height / 2 + 2)); crosshair.addLine(to: CGPoint(x: width / 2, y: height / 2 + 8))
    context.stroke(crosshair, with: .color(.yellow.opacity(0.9)), lineWidth: max(1, width / 480))
    context.draw(Text("DRAG TO TURN / MOVE  ·  TAP TO FIRE").font(.system(size: max(9, height / 34), design: .monospaced)).foregroundColor(.yellow), at: CGPoint(x: 18 + width / 4, y: height - 14))
    if Date().timeIntervalSince(fireAt) < 0.11 { context.fill(Path(CGRect(x: 0, y: 0, width: width, height: height)), with: .color(.yellow.opacity(0.22))) }
}
"#
}
