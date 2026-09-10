r#"    let id: String
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
                ForEach(Array(items.enumerated()), id: \.element.id) { index, item in
                    tabButton(item, index: index)
                    if variant == "stepper" && index < items.count - 1 {
                        Rectangle().fill(Color.secondary.opacity(0.35)).frame(width: CGFloat(2), height: CGFloat(20)).padding(.leading, CGFloat(15))
                    }
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
        HStack(spacing: variant == "stepper" ? CGFloat(0) : variant == "line" ? CGFloat(16) : CGFloat(8)) {
            ForEach(Array(items.enumerated()), id: \.element.id) { index, item in
                tabButton(item, index: index)
                if variant == "stepper" && index < items.count - 1 {
                    Rectangle().fill(Color.secondary.opacity(0.35)).frame(width: CGFloat(48), height: CGFloat(2)).padding(.horizontal, CGFloat(8))
                }
            }
        }
    }

    private var panel: some View {
        content(activeId)
            .frame(maxWidth: vertical ? nil : .infinity, alignment: .leading)
    }

    private func tabButton(_ item: DoweTabItem, index: Int) -> some View {
        let active = activeId == item.id
        let selectedFill = variant == "solid" || variant == "outlined" || variant == "pills"
        let selectedLine = variant == "line"
        let fill = active && selectedFill ? activeBackgroundColor : Color.clear
        let foreground = active ? (selectedFill ? activeContentColor : accentColor) : contentColor
        return Button(action: {
            activeId = item.id
        }) {
            Group {
                if variant == "stepper" {
                    HStack(spacing: CGFloat(10)) {
                        Text(String(index + 1))
                            .font(font.weight(.bold))
                            .frame(width: CGFloat(32), height: CGFloat(32))
                            .background(active ? activeBackgroundColor : Color.clear)
                            .foregroundStyle(active ? activeContentColor : contentColor)
                            .clipShape(Circle())
                            .overlay(Circle().stroke(active ? accentColor : Color.secondary.opacity(0.45), lineWidth: CGFloat(2)))
                        Text(item.label).font(font).lineLimit(1)
                    }
                } else {
                    Text(item.label).font(font).lineLimit(1)
                }
            }
            .padding(.horizontal, variant == "stepper" ? CGFloat(0) : CGFloat(16))
            .padding(.vertical, CGFloat(6))
            .background(variant == "stepper" ? Color.clear : fill)
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
"#
