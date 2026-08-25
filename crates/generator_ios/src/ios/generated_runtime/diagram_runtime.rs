fn swift_runtime_diagram() -> &'static str {
    r#"struct DoweDiagramView: View {
    @ObservedObject var state: DoweReactiveState
    let nodesPath: String
    let edgesPath: String
    let fitView: Bool
    let panOnDrag: Bool
    let zoomOnScroll: Bool
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
    @State private var dragOffset: CGSize = .zero
    @State private var connectingNode: String?

    private var nodes: [[String: Any]] { state.candles(nodesPath) }
    private var edges: [[String: Any]] { state.candles(edgesPath) }
    private func number(_ value: Any?, _ fallback: CGFloat = 0) -> CGFloat { if let value = value as? NSNumber { return CGFloat(truncating: value) }; return CGFloat(Double(String(describing: value ?? "")) ?? Double(fallback)) }
    private func id(_ node: [String: Any]) -> String { String(describing: node["id"] ?? "") }
    private func center(_ node: [String: Any]) -> CGPoint { CGPoint(x: number(node["x"]) + number(node["width"], 160) / 2, y: number(node["y"]) + number(node["height"], 56) / 2) }
    private func node(_ id: String) -> [String: Any]? { nodes.first { self.id($0) == id } }
    private func persistConnection(source: String, target: String) {
        var updated = edges
        if updated.contains(where: { String(describing: $0["source"] ?? "") == source && String(describing: $0["target"] ?? "") == target }) { return }
        updated.append(["id": "edge-\(UUID().uuidString)", "source": source, "target": target, "type": "default", "label": ""])
        state.write(edgesPath, value: updated)
    }
    private func moveNode(_ node: [String: Any], translation: CGSize) {
        let targetId = id(node)
        var updated = nodes
        guard let index = updated.firstIndex(where: { id($0) == targetId }) else { return }
        updated[index]["x"] = number(node["x"]) + translation.width / scale
        updated[index]["y"] = number(node["y"]) + translation.height / scale
        state.write(nodesPath, value: updated)
    }

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                backgroundColor
                Canvas { context, size in
                    if showGrid {
                        let step = CGFloat(28) * scale
                        if step > 6 {
                            for x in stride(from: offset.width.truncatingRemainder(dividingBy: step), through: size.width, by: step) { context.stroke(Path(CGRect(x: x, y: 0, width: 0, height: size.height)), with: .color(contentColor.opacity(0.08))) }
                            for y in stride(from: offset.height.truncatingRemainder(dividingBy: step), through: size.height, by: step) { context.stroke(Path(CGRect(x: 0, y: y, width: size.width, height: 0)), with: .color(contentColor.opacity(0.08))) }
                        }
                    }
                    for edge in edges {
                        guard let source = edge["source"].map({ String(describing: $0) }).flatMap(node), let target = edge["target"].map({ String(describing: $0) }).flatMap(node) else { continue }
                        var path = Path()
                        let from = CGPoint(x: offset.width + center(source).x * scale, y: offset.height + center(source).y * scale)
                        let to = CGPoint(x: offset.width + center(target).x * scale, y: offset.height + center(target).y * scale)
                        path.move(to: from)
                        path.addLine(to: to)
                        context.stroke(path, with: .color(contentColor.opacity(0.45)), lineWidth: 2)
                    }
                }
                .gesture(
                    DragGesture()
                        .onChanged { value in if panOnDrag { offset = CGSize(width: value.translation.width + dragOffset.width, height: value.translation.height + dragOffset.height) } }
                        .onEnded { _ in dragOffset = offset }
                )
                ForEach(Array(nodes.enumerated()), id: \.offset) { _, node in
                    let nodeId = id(node)
                    let width = number(node["width"], 160) * scale
                    let height = number(node["height"], 56) * scale
                    Text(String(describing: node["label"] ?? nodeId))
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(contentColor)
                        .frame(width: width, height: height)
                        .background(contentColor.opacity(0.08))
                        .overlay(RoundedRectangle(cornerRadius: 10).stroke(contentColor.opacity(0.35)))
                        .position(x: offset.width + (number(node["x"]) + number(node["width"], 160) / 2) * scale, y: offset.height + (number(node["y"]) + number(node["height"], 56) / 2) * scale)
                        .highPriorityGesture(DragGesture().onChanged { value in
                            if value.startLocation.x >= width - 24 { connectingNode = nodeId }
                            else if connectingNode == nil { moveNode(node, translation: value.translation) }
                        }.onEnded { value in
                            if connectingNode == nodeId {
                                let point = CGPoint(x: number(node["x"]) + number(node["width"], 160) + value.translation.width / scale, y: number(node["y"]) + number(node["height"], 56) / 2 + value.translation.height / scale)
                                if let target = nodes.first(where: { candidate in
                                    let x = number(candidate["x"]), y = number(candidate["y"]), w = number(candidate["width"], 160), h = number(candidate["height"], 56)
                                    return id(candidate) != nodeId && point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h
                                }) {
                                    let targetId = id(target)
                                    persistConnection(source: nodeId, target: targetId)
                                    if let onConnect { state.run(onConnect, item: ["source": nodeId, "target": targetId]) }
                                }
                                connectingNode = nil
                            } else if let onNodeDrag {
                                var item = node
                                item["x"] = number(node["x"]) + value.translation.width / scale
                                item["y"] = number(node["y"]) + value.translation.height / scale
                                state.run(onNodeDrag, item: item)
                            }
                        })
                        .overlay(alignment: .trailing) { Circle().fill(contentColor).frame(width: 8, height: 8).offset(x: 4) }
                        .onTapGesture { if connectingNode == nil, let onNodeClick { state.run(onNodeClick, item: node) } }
                }
                if nodes.isEmpty { Text(emptyLabel).foregroundStyle(contentColor.opacity(0.64)) }
                if minimap && !nodes.isEmpty { Text("").frame(width: 96, height: 64).background(contentColor.opacity(0.04)).clipShape(RoundedRectangle(cornerRadius: 8)).overlay(RoundedRectangle(cornerRadius: 8).stroke(contentColor.opacity(0.2))).frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing).padding(8) }
            }
            .clipShape(RoundedRectangle(cornerRadius: 12))
            .contentShape(Rectangle())
            .gesture(MagnificationGesture().onChanged { value in if zoomOnScroll { scale = min(3, max(0.2, value)) } })
        }
        .frame(minHeight: 300)
        .accessibilityLabel(Text("Diagram"))
    }
}
"#
}
