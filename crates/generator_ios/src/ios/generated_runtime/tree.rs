fn swift_runtime_tree() -> String {
    let icon = |name: &str, color: &str, modifier: &str| {
        let icon = solar_control_icon(name).expect("bundled Tree icon");
        format!(
            "DoweSvgView(viewBox: {}, color: {}, paths: {}, animated: false){}",
            swift_svg_view_box(&icon.props.view_box),
            color,
            swift_svg_paths(&icon.paths),
            modifier
        )
    };
    let arrow = icon("alt-arrow-down", "contentColor.opacity(0.72)", ".frame(width: CGFloat(16), height: CGFloat(16)).rotationEffect(.degrees(open ? 0 : -90))");
    let folder = icon("folder-with-files", "contentColor.opacity(0.88)", ".frame(width: CGFloat(16), height: CGFloat(16))");
    let file = icon("file-text", "contentColor.opacity(0.72)", ".frame(width: CGFloat(16), height: CGFloat(16))");
    let mut output = r##"struct DoweTreeView: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @ObservedObject var state: DoweReactiveState
    let dataPath: String
    let bindPath: String?
    let defaultOpen: Bool
    let emptyLabel: String
    let ariaLabel: String
    let onSelect: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var openIds: [String: Bool] = [:]
    @State private var selectedPath: String = ""

    private var nodes: [DoweTreeNode] {
        state.treeNodes(dataPath)
    }

    private var selected: String {
        guard let bindPath else { return selectedPath }
        return state.text(bindPath)
    }

    var body: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(alignment: .leading, spacing: CGFloat(1)) {
                if nodes.isEmpty {
                    Text(emptyLabel)
                        .font(.system(size: CGFloat(14)))
                        .foregroundStyle(contentColor.opacity(0.64))
                        .padding(.horizontal, CGFloat(12))
                        .padding(.vertical, CGFloat(10))
                } else {
                    ForEach(nodes) { node in
                        treeNodeView(node, depth: 0)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.vertical, CGFloat(4))
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
        .accessibilityElement(children: .contain)
        .accessibilityLabel(ariaLabel)
    }

    private func toggleNode(_ id: String, open: Bool) {
        if reduceMotion {
            openIds[id] = !open
        } else {
            withAnimation(.easeInOut(duration: 0.16)) {
                openIds[id] = !open
            }
        }
    }

    private func treeNodeView(_ node: DoweTreeNode, depth: Int) -> AnyView {
        let open = openIds[node.id] ?? defaultOpen
        let isSelected = !node.branch && selected == node.path
        return AnyView(VStack(alignment: .leading, spacing: CGFloat(0)) {
            HStack(spacing: CGFloat(4)) {
                ForEach(0..<depth, id: \.self) { _ in
                    Rectangle()
                        .fill(contentColor.opacity(0.16))
                        .frame(width: CGFloat(1), height: CGFloat(28))
                        .frame(width: CGFloat(15), height: CGFloat(28), alignment: .trailing)
                }
                Button(action: { if node.branch { toggleNode(node.id, open: open) } }) {
                    if node.branch {
                        __TREE_ARROW_ICON__
                    } else {
                        Color.clear.frame(width: CGFloat(16), height: CGFloat(16))
                    }
                }
                .buttonStyle(.plain)
                .frame(width: CGFloat(20), height: CGFloat(28))
                Button(action: {
                    if node.branch {
                        toggleNode(node.id, open: open)
                    } else {
                        if let bindPath {
                            state.write(bindPath, value: node.path)
                        } else {
                            selectedPath = node.path
                        }
                        if let onSelect { state.run(onSelect, item: node.value) }
                    }
                }) {
                    HStack(spacing: CGFloat(6)) {
                        if node.branch {
                            __TREE_FOLDER_ICON__
                        } else {
                            __TREE_FILE_ICON__
                        }
                        Text(node.label)
                            .font(.system(size: CGFloat(14), weight: node.branch ? .semibold : .regular))
                            .lineLimit(1)
                            .truncationMode(.tail)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .frame(maxWidth: .infinity, minHeight: CGFloat(28), alignment: .leading)
                }
                .buttonStyle(.plain)
                .foregroundStyle(contentColor)
            }
            .padding(.horizontal, CGFloat(4))
            .background(isSelected ? contentColor.opacity(0.14) : Color.clear)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(5)))
            .contentShape(Rectangle())
            .accessibilityLabel(node.label)
            .accessibilityAddTraits(.isButton)
            .accessibilityValue(isSelected ? "Selected" : "")
            if node.branch && open {
                ForEach(node.children) { child in
                    treeNodeView(child, depth: depth + 1)
                }
            }
        })
    }
}
"##.to_string();
    output = output.replace("__TREE_ARROW_ICON__", &arrow);
    output = output.replace("__TREE_FOLDER_ICON__", &folder);
    output = output.replace("__TREE_FILE_ICON__", &file);
    output
}
