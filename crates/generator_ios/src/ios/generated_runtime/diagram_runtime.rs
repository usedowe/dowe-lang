fn swift_runtime_diagram() -> &'static str {
    r#"struct DoweDiagramView: View {
    @ObservedObject var state: DoweReactiveState
    let nodesPath: String
    let edgesPath: String
    let fitView: Bool
    let panOnDrag: Bool
    let zoomOnScroll: Bool
    let controls: Bool
    let minimap: Bool
    let showGrid: Bool
    let emptyLabel: String
    let onNodeClick: String?
    let onNodeDrag: String?
    let onConnect: String?
    let backgroundColor: Color
    let contentColor: Color
    @State private var scale: CGFloat = 1
    @State private var offset: CGSize = .zero
    @State private var fitted = false
    @State private var selectedKey: String?
    @State private var dragNode: String?
    @State private var dragOrigin: CGPoint = .zero
    @State private var dragTranslation: CGSize = .zero
    @State private var connectFrom: String?
    @State private var connectPoint: CGPoint?
    @State private var panStart: CGSize?
    @State private var lastMagnification: CGFloat = 1

    private var nodes: [[String: Any]] { state.candles(nodesPath) }
    private var edges: [[String: Any]] { state.candles(edgesPath) }
    private func number(_ value: Any?, _ fallback: CGFloat = 0) -> CGFloat { if let value = value as? NSNumber { return CGFloat(truncating: value) }; return CGFloat(Double(String(describing: value ?? "")) ?? Double(fallback)) }
    private func nodeWidth(_ node: [String: Any]) -> CGFloat { max(1, number(node["width"], 160)) }
    private func nodeHeight(_ node: [String: Any]) -> CGFloat { max(1, number(node["height"], 56)) }
    private func nodeId(_ node: [String: Any]) -> String { String(describing: node["id"] ?? "") }
    private func nodeCenter(_ node: [String: Any]) -> CGPoint { CGPoint(x: number(node["x"]) + nodeWidth(node) / 2, y: number(node["y"]) + nodeHeight(node) / 2) }
    private func effectivePosition(_ node: [String: Any]) -> CGPoint {
        let id = nodeId(node)
        if dragNode == id { return CGPoint(x: dragOrigin.x + dragTranslation.width / scale, y: dragOrigin.y + dragTranslation.height / scale) }
        return CGPoint(x: number(node["x"]), y: number(node["y"]))
    }
    private func screenPoint(_ point: CGPoint) -> CGPoint { CGPoint(x: offset.width + point.x * scale, y: offset.height + point.y * scale) }
    private func graphPoint(from location: CGPoint) -> CGPoint { CGPoint(x: (location.x - offset.width) / scale, y: (location.y - offset.height) / scale) }
    private func borderPoint(_ node: [String: Any], toward: CGPoint) -> CGPoint {
        let center = nodeCenter(node)
        let dx = toward.x - center.x, dy = toward.y - center.y
        if dx == 0 && dy == 0 { return CGPoint(x: center.x, y: number(node["y"])) }
        let sx = dx == 0 ? CGFloat.infinity : nodeWidth(node) / 2 / abs(dx)
        let sy = dy == 0 ? CGFloat.infinity : nodeHeight(node) / 2 / abs(dy)
        let factor = min(sx, sy)
        return CGPoint(x: center.x + dx * factor, y: center.y + dy * factor)
    }
    private func nodeById(_ id: String) -> [String: Any]? { nodes.first { nodeId($0) == id } }
    private func hitNode(at point: CGPoint) -> [String: Any]? {
        for node in nodes.reversed() {
            let x = number(node["x"]), y = number(node["y"])
            if point.x >= x && point.x <= x + nodeWidth(node) && point.y >= y && point.y <= y + nodeHeight(node) { return node }
        }
        return nil
    }
    private func isConnectionTarget(_ node: [String: Any]) -> Bool {
        guard let source = connectFrom, source != nodeId(node), let point = connectPoint else { return false }
        guard let target = hitNode(at: point) else { return false }
        return nodeId(target) == nodeId(node)
    }
    private func edgeGeometry(_ source: [String: Any], _ target: [String: Any], type: String?) -> (path: Path, label: CGPoint) {
        let sc = nodeCenter(source), tc = nodeCenter(target)
        let from = screenPoint(borderPoint(source, toward: tc))
        let to = screenPoint(borderPoint(target, toward: sc))
        var path = Path()
        if type == "straight" {
            path.move(to: from)
            path.addLine(to: to)
            return (path, CGPoint(x: (from.x + to.x) / 2, y: (from.y + to.y) / 2))
        }
        if type == "step" {
            let midX = (from.x + to.x) / 2
            path.move(to: from)
            path.addLine(to: CGPoint(x: midX, y: from.y))
            path.addLine(to: CGPoint(x: midX, y: to.y))
            path.addLine(to: to)
            return (path, CGPoint(x: midX, y: (from.y + to.y) / 2))
        }
        let dx = max(40 * scale, abs(to.x - from.x) / 2)
        let c1 = CGPoint(x: from.x + dx, y: from.y)
        let c2 = CGPoint(x: to.x - dx, y: to.y)
        path.move(to: from)
        path.addCurve(to: to, control1: c1, control2: c2)
        return (path, CGPoint(x: (from.x + 3 * c1.x + 3 * c2.x + to.x) / 8, y: (from.y + 3 * c1.y + 3 * c2.y + to.y) / 8))
    }
    private func fitViewport(_ size: CGSize) {
        guard !nodes.isEmpty else { scale = 1; offset = .zero; return }
        var minX = CGFloat.infinity, minY = CGFloat.infinity, maxX = -CGFloat.infinity, maxY = -CGFloat.infinity
        for node in nodes {
            minX = min(minX, number(node["x"]))
            minY = min(minY, number(node["y"]))
            maxX = max(maxX, number(node["x"]) + nodeWidth(node))
            maxY = max(maxY, number(node["y"]) + nodeHeight(node))
        }
        let graphWidth = max(1, maxX - minX), graphHeight = max(1, maxY - minY)
        let padding: CGFloat = 40
        let next = min(2.5, max(0.1, min((size.width - padding * 2) / graphWidth, (size.height - padding * 2) / graphHeight)))
        scale = next
        offset = CGSize(width: (size.width - graphWidth * next) / 2 - minX * next, height: (size.height - graphHeight * next) / 2 - minY * next)
    }
    private func fitIfNeeded(_ size: CGSize) {
        if fitted || !fitView || nodes.isEmpty || size.width < 1 { return }
        fitted = true
        fitViewport(size)
    }
    private func zoomAtCenter(_ factor: CGFloat, _ size: CGSize) {
        applyZoom(factor, size)
    }
    private func applyZoom(_ factor: CGFloat, _ size: CGSize) {
        let center = CGPoint(x: size.width / 2, y: size.height / 2)
        let anchor = graphPoint(from: center)
        let next = min(2.5, max(0.1, scale * factor))
        scale = next
        offset = CGSize(width: center.x - anchor.x * next, height: center.y - anchor.y * next)
    }
    private func moveNode(_ node: [String: Any], to point: CGPoint) {
        var updated = nodes
        guard let index = updated.firstIndex(where: { nodeId($0) == nodeId(node) }) else { return }
        updated[index]["x"] = point.x
        updated[index]["y"] = point.y
        state.write(nodesPath, value: updated)
    }
    private func persistConnection(source: String, target: String) {
        var updated = edges
        if updated.contains(where: { String(describing: $0["source"] ?? "") == source && String(describing: $0["target"] ?? "") == target }) { return }
        updated.append(["id": "edge-\(UUID().uuidString)", "source": source, "target": target, "type": "default", "label": ""])
        state.write(edgesPath, value: updated)
    }
    private func distanceToSegment(_ point: CGPoint, _ a: CGPoint, _ b: CGPoint) -> CGFloat {
        let abx = b.x - a.x, aby = b.y - a.y
        let lengthSquared = abx * abx + aby * aby
        if lengthSquared == 0 { return hypot(point.x - a.x, point.y - a.y) }
        var t = ((point.x - a.x) * abx + (point.y - a.y) * aby) / lengthSquared
        t = min(1, max(0, t))
        return hypot(point.x - (a.x + t * abx), point.y - (a.y + t * aby))
    }
    private func hitEdge(at location: CGPoint) -> [String: Any]? {
        for edge in edges {
            guard let source = edge["source"].map({ String(describing: $0) }).flatMap(nodeById),
                  let target = edge["target"].map({ String(describing: $0) }).flatMap(nodeById) else { continue }
            let from = screenPoint(borderPoint(source, toward: nodeCenter(target)))
            let to = screenPoint(borderPoint(target, toward: nodeCenter(source)))
            if distanceToSegment(location, from, to) <= 8 { return edge }
        }
        return nil
    }

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                backgroundColor
                Canvas { context, size in
                    if showGrid {
                        let step = 28 * scale
                        if step > 6 {
                            var grid = Path()
                            var x = offset.width.truncatingRemainder(dividingBy: step)
                            while x < size.width { grid.move(to: CGPoint(x: x, y: 0)); grid.addLine(to: CGPoint(x: x, y: size.height)); x += step }
                            var y = offset.height.truncatingRemainder(dividingBy: step)
                            while y < size.height { grid.move(to: CGPoint(x: 0, y: y)); grid.addLine(to: CGPoint(x: size.width, y: y)); y += step }
                            context.stroke(grid, with: .color(contentColor.opacity(0.08)), lineWidth: 1)
                        }
                    }
                    let byId = nodes.reduce(into: [String: [String: Any]]()) { $0[nodeId($1)] = $1 }
                    for edge in edges {
                        guard let source = edge["source"].flatMap({ byId[String(describing: $0)] }),
                              let target = edge["target"].flatMap({ byId[String(describing: $0)] }) else { continue }
                        let geometry = edgeGeometry(source, target, type: edge["type"].map { String(describing: $0) })
                        let isSelected = selectedKey == "edge:" + String(describing: edge["id"] ?? "")
                        context.stroke(geometry.path, with: .color(contentColor.opacity(isSelected ? 1 : 0.45)), lineWidth: isSelected ? 2.5 : 2)
                        if let label = edge["label"].map({ String(describing: $0) }), !label.isEmpty {
                            context.draw(Text(label).font(.system(size: 11)).foregroundColor(contentColor), at: CGPoint(x: geometry.label.x, y: geometry.label.y - 6))
                        }
                    }
                    if let from = connectFrom, let point = connectPoint, let source = nodeById(from) {
                        let start = screenPoint(borderPoint(source, toward: point))
                        let end = screenPoint(point)
                        let dx = max(40 * scale, abs(end.x - start.x) / 2)
                        var preview = Path()
                        preview.move(to: start)
                        preview.addCurve(to: end, control1: CGPoint(x: start.x + dx, y: start.y), control2: CGPoint(x: end.x - dx, y: end.y))
                        context.stroke(preview, with: .color(contentColor.opacity(0.9)), style: StrokeStyle(lineWidth: 2, dash: [6, 4]))
                    }
                }
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(coordinateSpace: .named("doweDiagramCanvas"))
                        .onChanged { value in
                            if panOnDrag {
                                if panStart == nil { panStart = offset }
                                offset = CGSize(width: panStart!.width + value.translation.width, height: panStart!.height + value.translation.height)
                            }
                        }
                        .onEnded { _ in panStart = nil }
                )
                .gesture(
                    SpatialTapGesture(coordinateSpace: .named("doweDiagramCanvas"))
                        .onEnded { value in
                            if let edge = hitEdge(at: value.location) {
                                selectedKey = "edge:" + String(describing: edge["id"] ?? "")
                            } else {
                                selectedKey = nil
                            }
                        }
                )
                ForEach(Array(nodes.enumerated()), id: \.offset) { entry in
                    let node = entry.element
                    let id = nodeId(node)
                    let position = effectivePosition(node)
                    let width = nodeWidth(node) * scale
                    let height = nodeHeight(node) * scale
                    let isSelected = selectedKey == "node:" + id
                    let isTarget = isConnectionTarget(node)
                    ZStack {
                        RoundedRectangle(cornerRadius: 10)
                            .fill(contentColor.opacity(isSelected ? 0.16 : 0.08))
                        RoundedRectangle(cornerRadius: 10)
                            .stroke(contentColor.opacity(isTarget ? 1 : 0.35), style: StrokeStyle(lineWidth: isTarget ? 2 : 1, dash: isTarget ? [5, 3] : []))
                        Text(String(describing: node["label"] ?? id))
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundStyle(contentColor)
                            .lineLimit(1)
                            .padding(.horizontal, 8)
                        Circle()
                            .fill(contentColor)
                            .frame(width: 8, height: 8)
                            .overlay(Circle().stroke(backgroundColor, lineWidth: 2))
                            .offset(x: width / 2)
                            .frame(width: 24, height: height)
                            .highPriorityGesture(
                                DragGesture(coordinateSpace: .named("doweDiagramCanvas"))
                                    .onChanged { value in
                                        connectFrom = id
                                        connectPoint = graphPoint(from: value.location)
                                    }
                                    .onEnded { value in
                                        let point = graphPoint(from: value.location)
                                        if let target = hitNode(at: point), nodeId(target) != id {
                                            let targetId = nodeId(target)
                                            persistConnection(source: id, target: targetId)
                                            if let onConnect { state.run(onConnect, item: ["source": id, "target": targetId]) }
                                        }
                                        connectFrom = nil
                                        connectPoint = nil
                                    }
                            )
                    }
                    .frame(width: width, height: height)
                    .position(x: offset.width + (position.x + nodeWidth(node) / 2) * scale, y: offset.height + (position.y + nodeHeight(node) / 2) * scale)
                    .gesture(
                        DragGesture()
                            .onChanged { value in
                                if dragNode == nil {
                                    dragNode = id
                                    dragOrigin = CGPoint(x: number(node["x"]), y: number(node["y"]))
                                }
                                if dragNode == id { dragTranslation = value.translation }
                            }
                            .onEnded { value in
                                defer {
                                    dragNode = nil
                                    dragTranslation = .zero
                                }
                                guard dragNode == id else { return }
                                if hypot(value.translation.width, value.translation.height) > 4 {
                                    selectedKey = "node:" + id
                                    let point = CGPoint(x: dragOrigin.x + value.translation.width / scale, y: dragOrigin.y + value.translation.height / scale)
                                    moveNode(node, to: point)
                                    if let onNodeDrag {
                                        var item = node
                                        item["x"] = point.x
                                        item["y"] = point.y
                                        state.run(onNodeDrag, item: item)
                                    }
                                } else {
                                    selectedKey = "node:" + id
                                    if let onNodeClick { state.run(onNodeClick, item: node) }
                                }
                            }
                    )
                }
                if nodes.isEmpty { Text(emptyLabel).foregroundStyle(contentColor.opacity(0.64)) }
                if minimap && !nodes.isEmpty { minimapView(geometry.size) }
                if controls { controlsView(geometry.size) }
            }
            .clipShape(RoundedRectangle(cornerRadius: 12))
            .coordinateSpace(name: "doweDiagramCanvas")
            .simultaneousGesture(
                MagnificationGesture()
                    .onChanged { value in
                        if zoomOnScroll {
                            applyZoom(value / lastMagnification, geometry.size)
                        }
                        lastMagnification = value
                    }
                    .onEnded { _ in lastMagnification = 1 }
            )
            .onAppear { fitIfNeeded(geometry.size) }
            .onChange(of: nodes.count) { _, _ in fitIfNeeded(geometry.size) }
        }
        .frame(minHeight: 300)
        .accessibilityLabel(Text("Diagram"))
    }

    private var minimapProjection: (fit: CGFloat, minX: CGFloat, minY: CGFloat)? {
        guard !nodes.isEmpty else { return nil }
        var minX = CGFloat.infinity, minY = CGFloat.infinity, maxX = -CGFloat.infinity, maxY = -CGFloat.infinity
        for node in nodes {
            minX = min(minX, number(node["x"]))
            minY = min(minY, number(node["y"]))
            maxX = max(maxX, number(node["x"]) + nodeWidth(node))
            maxY = max(maxY, number(node["y"]) + nodeHeight(node))
        }
        let fit = min(104 / max(1, maxX - minX), 64 / max(1, maxY - minY))
        return (fit, minX, minY)
    }

    private func minimapView(_ canvasSize: CGSize) -> some View {
        Canvas { context, _ in
            guard let projection = minimapProjection else { return }
            for node in nodes {
                let x = (number(node["x"]) - projection.minX) * projection.fit + 8
                let y = (number(node["y"]) - projection.minY) * projection.fit + 8
                let rect = CGRect(x: x, y: y, width: max(3, nodeWidth(node) * projection.fit), height: max(2, nodeHeight(node) * projection.fit))
                context.fill(Path(roundedRect: rect, cornerRadius: 2), with: .color(contentColor.opacity(0.45)))
            }
            let viewX = (-offset.width / scale - projection.minX) * projection.fit + 8
            let viewY = (-offset.height / scale - projection.minY) * projection.fit + 8
            let viewRect = CGRect(x: viewX, y: viewY, width: canvasSize.width / scale * projection.fit, height: canvasSize.height / scale * projection.fit)
            context.fill(Path(viewRect), with: .color(contentColor.opacity(0.12)))
            context.stroke(Path(viewRect), with: .color(contentColor.opacity(0.8)), lineWidth: 1)
        }
        .frame(width: 120, height: 80)
        .gesture(
            DragGesture()
                .onChanged { value in
                    moveViewport(minimapPoint: value.location, canvasSize: canvasSize)
                }
        )
        .background(backgroundColor.opacity(0.9))
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(RoundedRectangle(cornerRadius: 8).stroke(contentColor.opacity(0.2)))
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
        .padding(10)
    }

    private func moveViewport(minimapPoint: CGPoint, canvasSize: CGSize) {
        guard let projection = minimapProjection else { return }
        let graphX = (minimapPoint.x - 8) / projection.fit + projection.minX
        let graphY = (minimapPoint.y - 8) / projection.fit + projection.minY
        offset = CGSize(width: canvasSize.width / 2 - graphX * scale, height: canvasSize.height / 2 - graphY * scale)
    }

    private func controlsView(_ size: CGSize) -> some View {
        VStack(spacing: 4) {
            diagramControlButton("+") { zoomAtCenter(1.2, size) }
            diagramControlButton("−") { zoomAtCenter(1 / 1.2, size) }
            diagramControlButton("⤢") { fitViewport(size) }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
        .padding(10)
    }

    private func diagramControlButton(_ label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(label)
                .font(.system(size: 14, weight: .bold))
                .foregroundStyle(contentColor)
                .frame(width: 26, height: 26)
                .background(backgroundColor.opacity(0.95))
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .overlay(RoundedRectangle(cornerRadius: 8).stroke(contentColor.opacity(0.15)))
        }
        .buttonStyle(.plain)
    }
}
"#
}
