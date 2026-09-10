r#"    @State private var openIndex: Int? = nil
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
            color: DoweDesign.backgroundText,
            paths: [
                DoweSvgPathData(data: "M0 0h24v24H0z", fill: .none),
                DoweSvgPathData(data: "__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__", fill: .currentColor)
            ]
        )
        .frame(width: CGFloat(16), height: CGFloat(16))
        .rotationEffect(.degrees(expanded ? 90 : 0))
        .animation(.easeInOut(duration: 0.16), value: expanded)
    }
}

final class DoweSideNavMemory {
    static let shared = DoweSideNavMemory()
    private var expanded: [String: Bool] = [:]

    func value(for key: String, initial: Bool) -> Bool {
        if let value = expanded[key] {
            return value
        }
        expanded[key] = initial
        return initial
    }

    func set(_ value: Bool, for key: String) {
        expanded[key] = value
    }
}

struct DoweSideNavSubmenu<Label: View, Content: View>: View {
    @State private var expanded: Bool
    let stateKey: String
    let bordered: Bool
    let wide: Bool
    let label: (Bool) -> Label
    let content: Content

    init(stateKey: String, open: Bool, bordered: Bool, wide: Bool, @ViewBuilder content: () -> Content, @ViewBuilder label: @escaping (Bool) -> Label) {
        _expanded = State(initialValue: DoweSideNavMemory.shared.value(for: stateKey, initial: open))
        self.stateKey = stateKey
        self.bordered = bordered
        self.wide = wide
        self.content = content()
        self.label = label
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            Button(action: {
                withAnimation(.easeInOut(duration: 0.18)) {
                    let next = !expanded
                    expanded = next
                    DoweSideNavMemory.shared.set(next, for: stateKey)
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
