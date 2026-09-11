r#"    let active: Bool
    let wide: Bool
    let paddingHorizontal: CGFloat
    let paddingVertical: CGFloat
    let gap: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let content: Content

    init(active: Bool, wide: Bool, paddingHorizontal: CGFloat, paddingVertical: CGFloat, gap: CGFloat, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, @ViewBuilder content: () -> Content) {
        self.active = active
        self.wide = wide
        self.paddingHorizontal = paddingHorizontal
        self.paddingVertical = paddingVertical
        self.gap = gap
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.action = action
        self.content = content()
    }

    private var row: some View {
        HStack(spacing: gap) {
            content
        }
        .padding(.horizontal, paddingHorizontal)
        .padding(.vertical, paddingVertical)
        .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
        .contentShape(Rectangle())
        .background(active ? backgroundColor : Color.clear)
        .foregroundStyle(active ? contentColor : DoweDesign.backgroundText)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        .overlay(
            RoundedRectangle(cornerRadius: DoweDesign.radius)
                .stroke(active ? borderColor ?? Color.clear : Color.clear, lineWidth: active && borderColor != nil ? CGFloat(1) : CGFloat(0))
        )
    }

    var body: some View {
        if let action {
            Button(action: action) {
                row
            }
            .buttonStyle(.plain)
            .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
        } else {
            row
        }
    }
}

struct DoweSideNavStatus: View {
    let text: String
    let font: Font

    var body: some View {
        Text(text)
            .font(font)
            .fontWeight(.semibold)
            .padding(.horizontal, CGFloat(8))
            .padding(.vertical, CGFloat(2))
            .background(DoweDesign.muted)
            .foregroundStyle(DoweDesign.mutedText)
            .clipShape(Capsule())
    }
}

struct DoweRailNavIcon {
    let viewBox: DoweSvgViewBox
    let color: Color
    let paths: [DoweSvgPathData]
    let animated: Bool

    init(viewBox: DoweSvgViewBox, color: Color, paths: [DoweSvgPathData], animated: Bool = false) {
        self.viewBox = viewBox
        self.color = color
        self.paths = paths
        self.animated = animated
    }
}

struct DoweRailNavItem: View {
    let label: String
    let showLabel: Bool
    let active: Bool
    let itemSize: CGFloat
    let iconSize: CGFloat
    let labelSize: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    var featured: Bool = false
    let icon: DoweRailNavIcon
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(alignment: .center, spacing: CGFloat(4)) {
                DoweSvgView(viewBox: icon.viewBox, color: icon.color, paths: icon.paths, animated: icon.animated)
                    .frame(width: iconSize, height: iconSize)
                if showLabel {
                    Text(label)
                        .font(.system(size: labelSize, weight: .semibold))
                        .lineLimit(1)
                        .frame(maxWidth: .infinity, alignment: .center)
                }
            }
            .frame(width: itemSize)
            .frame(minHeight: itemSize)
            .background(active || featured ? backgroundColor : Color.clear)
            .foregroundStyle(active || featured ? contentColor : DoweDesign.backgroundText)
            .clipShape(RoundedRectangle(cornerRadius: featured ? itemSize / 2 : DoweDesign.radius))
            .overlay(
                RoundedRectangle(cornerRadius: featured ? itemSize / 2 : DoweDesign.radius)
                    .stroke(active || featured ? borderColor ?? Color.clear : Color.clear, lineWidth: (active || featured) && borderColor != nil ? CGFloat(1) : CGFloat(0))
            )
        }
        .buttonStyle(.plain)
        .accessibilityLabel(label)
    }
}

struct DoweSideNavIcon {
    let viewBox: DoweSvgViewBox
    let color: Color?
    let paths: [DoweSvgPathData]
    let width: CGFloat?
    let maxWidth: CGFloat?
    let height: CGFloat?
    let maxHeight: CGFloat?
    let minWidth: CGFloat?
    let minHeight: CGFloat?
}

struct DoweSideNavEntry: Identifiable {
    let id: String
    let kind: String
    let label: String
    let description: String?
    let status: String?
    let icon: DoweSideNavIcon?
    let operation: String?
    let path: String?
    let fragment: String?
    let open: Bool
    let bordered: Bool
    let children: [DoweSideNavEntry]
}

struct DoweSideNav: View {
    let items: [DoweSideNavEntry]
    let stateKey: String
    let activePath: String
    let wide: Bool
    let paddingHorizontal: CGFloat
    let paddingVertical: CGFloat
    let gap: CGFloat
    let labelFont: Font
    let descriptionFont: Font
    let backgroundColor: Color
    let contentColor: Color
    let titleColor: Color
    let activeContentColor: Color
    let borderColor: Color?
    let navigate: (String, String, String?) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(2)) {
            ForEach(items) { item in
                entryView(item)
            }
        }
        .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
    }

    @ViewBuilder
    private func entryView(_ item: DoweSideNavEntry) -> some View {
        switch item.kind {
        case "divider":
            Divider()
                .padding(.vertical, CGFloat(8))
        case "submenu":
            DoweSideNavSubmenu(stateKey: stateKey + ":" + item.id, open: item.open, bordered: item.bordered, wide: wide) {
                ForEach(item.children) { child in
                    row(child, header: false, action: action(for: child))
                }
            } label: { expanded in
                row(item, header: false, action: nil, expanded: expanded)
            }
        case "header":
            row(item, header: true, action: action(for: item))
        default:
            row(item, header: false, action: action(for: item))
        }
    }

    private func row(_ item: DoweSideNavEntry, header: Bool, action: (() -> Void)?, expanded: Bool? = nil) -> some View {
        DoweSideNavRow(active: item.path == activePath, wide: wide, paddingHorizontal: paddingHorizontal, paddingVertical: paddingVertical, gap: gap, backgroundColor: backgroundColor, contentColor: contentColor, borderColor: borderColor, action: action) {
            if let icon = item.icon {
                DoweSvgView(viewBox: icon.viewBox, color: icon.color ?? (header ? titleColor : (item.path == activePath ? activeContentColor : DoweDesign.backgroundText)), paths: icon.paths)
                    .frame(width: icon.width)
                    .frame(maxWidth: icon.maxWidth)
                    .frame(height: icon.height)
                    .frame(maxHeight: icon.maxHeight)
                    .frame(minWidth: icon.minWidth)
                    .frame(minHeight: icon.minHeight)
            }
            VStack(alignment: .leading, spacing: CGFloat(0)) {
                Text(item.label)
                    .font(labelFont)
                    .fontWeight(header ? .semibold : .regular)
                    .foregroundStyle(header ? titleColor : (item.path == activePath ? contentColor : DoweDesign.backgroundText))
                if let description = item.description {
                    Text(description)
                        .font(descriptionFont)
                        .opacity(0.72)
                }
            }
            .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
            if item.status != nil || expanded != nil {
                HStack(spacing: gap) {
                    if let status = item.status {
                        DoweSideNavStatus(text: status, font: descriptionFont)
                    }
                    if let expanded {
                        DoweSideNavArrow(expanded: expanded)
                    }
                }
            }
        }
    }

    private func action(for item: DoweSideNavEntry) -> (() -> Void)? {
        guard let path = item.path else {
            return nil
        }
        return {
            navigate(item.operation ?? "push", path, item.fragment)
        }
    }
}

struct DoweNavMenu<Content: View, Popover: View>: View {
"#
