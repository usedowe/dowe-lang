fn swift_runtime_navigation_controls() -> &'static str {
    r#"struct DoweTabItem: Identifiable {
    let id: String
    let label: String
}

struct DoweTabs<Content: View>: View {
    let items: [DoweTabItem]
    let position: String
    let variant: String
    let backgroundColor: Color
    let contentColor: Color
    let activeBackgroundColor: Color
    let activeContentColor: Color
    let accentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let font: Font
    let content: (String) -> Content
    @State private var activeId: String

    init(items: [DoweTabItem], initialId: String, position: String, variant: String, backgroundColor: Color, contentColor: Color, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, borderColor: Color?, radius: CGFloat, font: Font, @ViewBuilder content: @escaping (String) -> Content) {
        self.items = items
        self.position = position
        self.variant = variant
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.activeBackgroundColor = activeBackgroundColor
        self.activeContentColor = activeContentColor
        self.accentColor = accentColor
        self.borderColor = borderColor
        self.radius = radius
        self.font = font
        self.content = content
        _activeId = State(initialValue: initialId)
    }

    var body: some View {
        switch position {
        case "bottom":
            VStack(alignment: .leading, spacing: CGFloat(8)) {
                panel
                tabList
            }
        case "start":
            HStack(alignment: .top, spacing: CGFloat(8)) {
                tabList
                panel
            }
        case "end":
            HStack(alignment: .top, spacing: CGFloat(8)) {
                panel
                tabList
            }
        default:
            VStack(alignment: .leading, spacing: CGFloat(8)) {
                tabList
                panel
            }
        }
    }

    private var vertical: Bool {
        position == "start" || position == "end"
    }

    private var listRadius: CGFloat {
        variant == "pills" ? CGFloat(999) : radius
    }

    private var tabRadius: CGFloat {
        variant == "pills" ? CGFloat(999) : radius
    }

    private var listPadding: CGFloat {
        variant == "line" || variant == "ghost" ? CGFloat(0) : CGFloat(4)
    }

    @ViewBuilder
    private var tabList: some View {
        if vertical {
            VStack(alignment: .leading, spacing: variant == "line" ? CGFloat(16) : CGFloat(8)) {
                ForEach(items) { item in
                    tabButton(item)
                }
            }
            .padding(listPadding)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: listRadius))
            .overlay(RoundedRectangle(cornerRadius: listRadius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil || variant == "line" ? CGFloat(0) : CGFloat(1)))
        } else {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: variant == "line" ? CGFloat(16) : CGFloat(8)) {
                    ForEach(items) { item in
                        tabButton(item)
                    }
                }
            }
            .padding(listPadding)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: listRadius))
            .overlay(RoundedRectangle(cornerRadius: listRadius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil || variant == "line" ? CGFloat(0) : CGFloat(1)))
        }
    }

    private var panel: some View {
        content(activeId)
            .frame(maxWidth: vertical ? nil : .infinity, alignment: .leading)
    }

    private func tabButton(_ item: DoweTabItem) -> some View {
        let active = activeId == item.id
        let selectedFill = variant == "solid" || variant == "outlined" || variant == "pills"
        let selectedLine = variant == "line"
        let fill = active && selectedFill ? activeBackgroundColor : Color.clear
        let foreground = active ? (selectedFill ? activeContentColor : accentColor) : contentColor
        return Button(action: {
            activeId = item.id
        }) {
            Text(item.label)
                .font(font)
                .lineLimit(1)
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(6))
                .background(fill)
                .foregroundStyle(foreground)
                .clipShape(RoundedRectangle(cornerRadius: tabRadius))
                .overlay(RoundedRectangle(cornerRadius: tabRadius).stroke(active && selectedLine ? accentColor : Color.clear, lineWidth: active && selectedLine ? CGFloat(2) : CGFloat(0)))
        }
        .buttonStyle(.plain)
    }
}

struct DoweSideNavRow<Content: View>: View {
    let active: Bool
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
        .background(active ? backgroundColor : Color.clear)
        .foregroundStyle(active ? contentColor : DoweDesign.onBackground)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusUi))
        .overlay(
            RoundedRectangle(cornerRadius: DoweDesign.radiusUi)
                .stroke(active ? borderColor ?? Color.clear : Color.clear, lineWidth: active && borderColor != nil ? CGFloat(1) : CGFloat(0))
        )
    }

    var body: some View {
        if let action {
            Button(action: action) {
                row
            }
            .buttonStyle(.plain)
        } else {
            row
        }
    }
}

struct DoweSideNavEntry: Identifiable {
    let id: String
    let kind: String
    let label: String
    let description: String?
    let status: String?
    let operation: String?
    let path: String?
    let fragment: String?
    let open: Bool
    let bordered: Bool
    let children: [DoweSideNavEntry]
}

struct DoweSideNav: View {
    let items: [DoweSideNavEntry]
    let activePath: String
    let wide: Bool
    let paddingHorizontal: CGFloat
    let paddingVertical: CGFloat
    let gap: CGFloat
    let labelFont: Font
    let descriptionFont: Font
    let backgroundColor: Color
    let contentColor: Color
    let activeContentColor: Color
    let borderColor: Color?
    let navigate: (String, String, String?) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(2)) {
            ForEach(items) { item in
                entryView(item)
            }
        }
    }

    @ViewBuilder
    private func entryView(_ item: DoweSideNavEntry) -> some View {
        switch item.kind {
        case "divider":
            Divider()
                .padding(.vertical, CGFloat(8))
        case "submenu":
            DoweSideNavSubmenu(open: item.open, bordered: item.bordered) {
                ForEach(item.children) { child in
                    row(child, header: false, action: action(for: child))
                }
            } label: { expanded in
                row(item, header: true, action: nil, expanded: expanded)
            }
        case "header":
            row(item, header: true, action: action(for: item))
        default:
            row(item, header: false, action: action(for: item))
        }
    }

    private func row(_ item: DoweSideNavEntry, header: Bool, action: (() -> Void)?, expanded: Bool? = nil) -> some View {
        DoweSideNavRow(active: item.path == activePath, wide: wide, paddingHorizontal: paddingHorizontal, paddingVertical: paddingVertical, gap: gap, backgroundColor: backgroundColor, contentColor: contentColor, borderColor: borderColor, action: action) {
            VStack(alignment: .leading, spacing: CGFloat(0)) {
                Text(item.label)
                    .font(labelFont)
                    .fontWeight(header ? .semibold : .regular)
                if let description = item.description {
                    Text(description)
                        .font(descriptionFont)
                        .opacity(0.72)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            if let status = item.status {
                Text(status)
                    .font(descriptionFont)
                    .fontWeight(.semibold)
            }
            if let expanded {
                DoweSideNavArrow(expanded: expanded)
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
    @State private var openIndex: Int? = nil
    let gap: CGFloat
    let popoverBackgroundColor: Color
    let popoverContentColor: Color
    let content: (Int?, @escaping (Int) -> Void) -> Content
    let popover: (Int?) -> Popover

    init(gap: CGFloat, popoverBackgroundColor: Color, popoverContentColor: Color, @ViewBuilder content: @escaping (Int?, @escaping (Int) -> Void) -> Content, @ViewBuilder popover: @escaping (Int?) -> Popover) {
        self.gap = gap
        self.popoverBackgroundColor = popoverBackgroundColor
        self.popoverContentColor = popoverContentColor
        self.content = content
        self.popover = popover
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            HStack(spacing: gap) {
                content(openIndex) { index in
                    openIndex = openIndex == index ? nil : index
                }
            }
            if openIndex != nil {
                VStack(alignment: .leading, spacing: CGFloat(4)) {
                    popover(openIndex)
                }
                .padding(CGFloat(8))
                .frame(minWidth: CGFloat(192), maxWidth: CGFloat(720), alignment: .leading)
                .background(popoverBackgroundColor)
                .foregroundStyle(popoverContentColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
                .shadow(color: Color.black.opacity(0.16), radius: CGFloat(16), x: CGFloat(0), y: CGFloat(8))
            }
        }
    }
}

struct DoweNavMenuItem<Content: View>: View {
    let active: Bool
    let paddingHorizontal: CGFloat
    let paddingVertical: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let content: Content

    init(active: Bool, paddingHorizontal: CGFloat, paddingVertical: CGFloat, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, @ViewBuilder content: () -> Content) {
        self.active = active
        self.paddingHorizontal = paddingHorizontal
        self.paddingVertical = paddingVertical
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.action = action
        self.content = content()
    }

    private var row: some View {
        HStack(spacing: CGFloat(8)) {
            content
        }
        .padding(.horizontal, paddingHorizontal)
        .padding(.vertical, paddingVertical)
        .background(active ? backgroundColor : Color.clear)
        .foregroundStyle(active ? contentColor : DoweDesign.onBackground)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
        .overlay(
            RoundedRectangle(cornerRadius: DoweDesign.radiusBox)
                .stroke(active ? borderColor ?? Color.clear : Color.clear, lineWidth: active && borderColor != nil ? CGFloat(1) : CGFloat(0))
        )
    }

    var body: some View {
        if let action {
            Button(action: action) {
                row
            }
            .buttonStyle(.plain)
        } else {
            row
        }
    }
}

struct DoweSideNavArrow: View {
    let expanded: Bool

    var body: some View {
        DoweSvgView(
            viewBox: DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24)),
            color: DoweDesign.onBackground,
            paths: [
                DoweSvgPathData(data: "M0 0h24v24H0z", fill: .none),
                DoweSvgPathData(data: "m19.704 12l-8.491-8.727a.75.75 0 1 1 1.075-1.046l9 9.25a.75.75 0 0 1 0 1.046l-9 9.25a.75.75 0 1 1-1.075-1.046z", fill: .currentColor)
            ]
        )
        .frame(width: CGFloat(16), height: CGFloat(16))
        .rotationEffect(.degrees(expanded ? 90 : 0))
        .animation(.easeInOut(duration: 0.16), value: expanded)
    }
}

struct DoweSideNavSubmenu<Label: View, Content: View>: View {
    @State private var expanded: Bool
    let bordered: Bool
    let label: (Bool) -> Label
    let content: Content

    init(open: Bool, bordered: Bool, @ViewBuilder content: () -> Content, @ViewBuilder label: @escaping (Bool) -> Label) {
        _expanded = State(initialValue: open)
        self.bordered = bordered
        self.content = content()
        self.label = label
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(2)) {
            Button(action: {
                withAnimation(.easeInOut(duration: 0.18)) {
                    expanded.toggle()
                }
            }) {
                label(expanded)
            }
            .buttonStyle(.plain)
            if expanded {
                content
                    .padding(.leading, bordered ? CGFloat(8) : CGFloat(0))
                    .overlay(alignment: .leading) {
                        if bordered {
                            Rectangle()
                                .fill(DoweDesign.muted)
                                .frame(width: CGFloat(1))
                        }
                    }
                    .padding(.leading, CGFloat(16))
                    .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
        .animation(.easeInOut(duration: 0.18), value: expanded)
    }
}

"#
}
