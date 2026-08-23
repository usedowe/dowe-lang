fn swift_runtime_drag_drop_controls() -> &'static str {
    r#"struct DoweCsvField: View {
    let label: String?
    let buttonText: String
    let modalTitle: String
    let instructions: String
    let columns: [DoweCsvColumn]
    let backgroundColor: Color
    let contentColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
            }
            Button(action: {}) {
                Text(buttonText)
                    .fontWeight(.semibold)
                    .padding(.horizontal, CGFloat(14))
                    .padding(.vertical, CGFloat(10))
                    .frame(maxWidth: .infinity, alignment: .center)
            }
            .buttonStyle(.plain)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.18), lineWidth: CGFloat(1)))

            VStack(alignment: .leading, spacing: CGFloat(8)) {
                Text(modalTitle)
                    .fontWeight(.bold)
                Text(instructions)
                    .font(.footnote)
                    .foregroundStyle(contentColor.opacity(0.7))
                ForEach(columns) { column in
                    Text(column.label ?? column.name)
                        .font(.footnote)
                        .fontWeight(.semibold)
                        .padding(.horizontal, CGFloat(10))
                        .padding(.vertical, CGFloat(7))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(contentColor.opacity(0.07))
                        .clipShape(RoundedRectangle(cornerRadius: CGFloat(9)))
                }
            }
            .padding(CGFloat(12))
            .background(backgroundColor.opacity(0.72))
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(14)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(14)).stroke(contentColor.opacity(0.16), lineWidth: CGFloat(1)))
            .foregroundStyle(contentColor)
        }
    }
}

struct DoweDragDrop: View {
    let label: String?
    let emptyText: String
    let direction: String
    let items: [DoweDragItem]
    let groups: [DoweDragGroup]
    let backgroundColor: Color
    let contentColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
            }
            if groups.isEmpty {
                dragItems(items)
                    .padding(CGFloat(8))
                    .background(backgroundColor)
                    .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(alignment: .top, spacing: CGFloat(12)) {
                        ForEach(groups) { group in
                            DoweDragGroupView(title: group.title ?? group.id, items: group.items, emptyText: emptyText, contentColor: contentColor)
                        }
                    }
                    .padding(CGFloat(8))
                }
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
            }
        }
        .foregroundStyle(contentColor)
    }

    @ViewBuilder
    private func dragItems(_ source: [DoweDragItem]) -> some View {
        if direction == "horizontal" {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: CGFloat(8)) {
                    if source.isEmpty {
                        Text(emptyText)
                            .foregroundStyle(contentColor.opacity(0.65))
                    }
                    ForEach(source) { item in
                        DoweDragItemView(item: item, contentColor: contentColor)
                    }
                }
            }
        } else {
            VStack(alignment: .leading, spacing: CGFloat(8)) {
                if source.isEmpty {
                    Text(emptyText)
                        .foregroundStyle(contentColor.opacity(0.65))
                }
                ForEach(source) { item in
                    DoweDragItemView(item: item, contentColor: contentColor)
                }
            }
        }
    }
}

struct DoweDragGroupView: View {
    let title: String
    let items: [DoweDragItem]
    let emptyText: String
    let contentColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            Text(title)
                .fontWeight(.bold)
            if items.isEmpty {
                Text(emptyText)
                    .foregroundStyle(contentColor.opacity(0.65))
            }
            ForEach(items) { item in
                DoweDragItemView(item: item, contentColor: contentColor)
            }
        }
        .frame(minWidth: CGFloat(220), alignment: .topLeading)
        .padding(CGFloat(8))
        .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.18), lineWidth: CGFloat(1)))
    }
}

struct DoweDragItemView: View {
    let item: DoweDragItem
    let contentColor: Color

    var body: some View {
        HStack(alignment: .center, spacing: CGFloat(8)) {
            Text("::")
                .fontWeight(.bold)
                .foregroundStyle(contentColor.opacity(item.disabled ? 0.3 : 0.55))
            VStack(alignment: .leading, spacing: CGFloat(2)) {
                Text(item.label ?? item.id)
                    .fontWeight(.semibold)
                if let description = item.description {
                    Text(description)
                        .font(.caption)
                        .foregroundStyle(contentColor.opacity(0.68))
                }
            }
        }
        .padding(CGFloat(10))
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(contentColor.opacity(item.disabled ? 0.04 : 0.08))
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
        .opacity(item.disabled ? 0.58 : 1)
    }
}

"#
}
