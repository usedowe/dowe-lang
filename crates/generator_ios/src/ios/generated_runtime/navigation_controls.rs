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
            ViewThatFits(in: .horizontal) {
                horizontalTabButtons
                ScrollView(.horizontal, showsIndicators: false) {
                    horizontalTabButtons
                }
            }
            .padding(listPadding)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: listRadius))
            .overlay(RoundedRectangle(cornerRadius: listRadius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil || variant == "line" ? CGFloat(0) : CGFloat(1)))
        }
    }

    private var horizontalTabButtons: some View {
        HStack(spacing: variant == "line" ? CGFloat(16) : CGFloat(8)) {
            ForEach(items) { item in
                tabButton(item)
            }
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
                .overlay {
                    if active && selectedLine {
                        tabLineIndicator
                    }
                }
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private var tabLineIndicator: some View {
        if position == "start" {
            HStack(spacing: CGFloat(0)) {
                Rectangle().fill(accentColor).frame(width: CGFloat(2))
                Spacer(minLength: CGFloat(0))
            }
        } else if position == "end" {
            HStack(spacing: CGFloat(0)) {
                Spacer(minLength: CGFloat(0))
                Rectangle().fill(accentColor).frame(width: CGFloat(2))
            }
        } else {
            VStack(spacing: CGFloat(0)) {
                Spacer(minLength: CGFloat(0))
                Rectangle().fill(accentColor).frame(height: CGFloat(2))
            }
        }
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
        .contentShape(Rectangle())
        .background(active ? backgroundColor : Color.clear)
        .foregroundStyle(active ? contentColor : DoweDesign.onBackground)
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
            .background(DoweDesign.softMuted)
            .foregroundStyle(DoweDesign.onSoftMuted)
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
            .foregroundStyle(active || featured ? contentColor : DoweDesign.onBackground)
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
        .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
    }

    @ViewBuilder
    private func entryView(_ item: DoweSideNavEntry) -> some View {
        switch item.kind {
        case "divider":
            Divider()
                .padding(.vertical, CGFloat(8))
        case "submenu":
            DoweSideNavSubmenu(open: item.open, bordered: item.bordered, wide: wide) {
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
            if let icon = item.icon {
                DoweSvgView(viewBox: icon.viewBox, color: icon.color ?? (item.path == activePath ? activeContentColor : DoweDesign.onBackground), paths: icon.paths)
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
    @State private var openIndex: Int? = nil
    let gap: CGFloat
    let wideIndices: Set<Int>
    let popoverBackgroundColor: Color
    let popoverContentColor: Color
    let content: (Int?, @escaping (Int) -> Void) -> Content
    let popover: (Int?) -> Popover

    init(gap: CGFloat, wideIndices: Set<Int>, popoverBackgroundColor: Color, popoverContentColor: Color, @ViewBuilder content: @escaping (Int?, @escaping (Int) -> Void) -> Content, @ViewBuilder popover: @escaping (Int?) -> Popover) {
        self.gap = gap
        self.wideIndices = wideIndices
        self.popoverBackgroundColor = popoverBackgroundColor
        self.popoverContentColor = popoverContentColor
        self.content = content
        self.popover = popover
    }

    var body: some View {
        HStack(spacing: gap) {
            content(openIndex) { index in
                openIndex = openIndex == index ? nil : index
            }
        }
        .background(
            DoweAnchoredPopoverPresenter(
                isPresented: openIndex != nil,
                minWidth: wideIndices.contains(openIndex ?? -1) ? CGFloat(600) : CGFloat(192),
                maxWidth: wideIndices.contains(openIndex ?? -1) ? CGFloat(720) : CGFloat(360),
                maxHeight: wideIndices.contains(openIndex ?? -1) ? CGFloat(640) : CGFloat(360),
                onDismiss: { openIndex = nil }
            ) {
                DoweNavMenuPopover(backgroundColor: popoverBackgroundColor, contentColor: popoverContentColor) {
                    popover(openIndex)
                }
                .simultaneousGesture(TapGesture().onEnded {
                    openIndex = nil
                })
            }
        )
        .zIndex(openIndex == nil ? 0 : 1000)
        .onDisappear {
            openIndex = nil
        }
    }
}

struct DoweNavMenuPopover<Content: View>: View {
    let backgroundColor: Color
    let contentColor: Color
    let content: Content

    init(backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(4)) {
            content
        }
        .padding(CGFloat(8))
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(contentColor.opacity(0.08), lineWidth: CGFloat(1)))
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
    let wide: Bool
    let label: (Bool) -> Label
    let content: Content

    init(open: Bool, bordered: Bool, wide: Bool, @ViewBuilder content: () -> Content, @ViewBuilder label: @escaping (Bool) -> Label) {
        _expanded = State(initialValue: open)
        self.bordered = bordered
        self.wide = wide
        self.content = content()
        self.label = label
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            Button(action: {
                withAnimation(.easeInOut(duration: 0.18)) {
                    expanded.toggle()
                }
            }) {
                label(expanded)
                    .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
            .contentShape(Rectangle())
            VStack(alignment: .leading, spacing: CGFloat(0)) {
                if expanded {
                    VStack(alignment: .leading, spacing: CGFloat(2)) {
                        content
                    }
                    .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
                    .padding(.leading, bordered ? CGFloat(8) : CGFloat(0))
                    .overlay(alignment: .leading) {
                        if bordered {
                            Rectangle()
                                .fill(DoweDesign.muted)
                                .frame(width: CGFloat(1))
                        }
                    }
                    .padding(.leading, CGFloat(16))
                    .padding(.top, CGFloat(2))
                    .transition(.opacity)
                }
            }
            .clipped()
        }
        .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
        .clipped()
        .animation(.easeInOut(duration: 0.18), value: expanded)
    }
}

"#
}
