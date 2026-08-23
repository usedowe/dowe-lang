fn swift_runtime_accordion() -> &'static str {
    r##"struct DoweAccordionView<Content: View>: View {
    let multiple: Bool
    let variant: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let itemBackgroundColor: Color
    let itemBorderColor: Color
    let itemBorderOpacity: Double
    let radius: CGFloat
    @ViewBuilder let content: (Set<String>, @escaping (String) -> Void) -> Content
    @State private var openIds: Set<String>

    init(multiple: Bool, variant: String, defaultOpenIds: Set<String>, backgroundColor: Color, contentColor: Color, borderColor: Color?, itemBackgroundColor: Color, itemBorderColor: Color, itemBorderOpacity: Double, radius: CGFloat, @ViewBuilder content: @escaping (Set<String>, @escaping (String) -> Void) -> Content) {
        self.multiple = multiple
        self.variant = variant
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.itemBackgroundColor = itemBackgroundColor
        self.itemBorderColor = itemBorderColor
        self.itemBorderOpacity = itemBorderOpacity
        self.radius = radius
        self.content = content
        _openIds = State(initialValue: defaultOpenIds)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: variant == "ghost" || variant == "line" ? CGFloat(0) : CGFloat(8)) {
            content(openIds) { id in
                if openIds.contains(id) {
                    openIds.remove(id)
                } else if multiple {
                    openIds.insert(id)
                } else {
                    openIds = [id]
                }
            }
        }
        .padding(variant == "ghost" || variant == "line" ? CGFloat(0) : CGFloat(4))
        .frame(maxWidth: .infinity, alignment: .leading)
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }
}

struct DoweAccordionItemView<Arrow: View, Content: View>: View {
    let label: String
    let disabled: Bool
    let open: Bool
    let backgroundColor: Color
    let borderColor: Color
    let borderOpacity: Double
    let borderStyle: String
    let contentColor: Color
    let radius: CGFloat
    let action: () -> Void
    @ViewBuilder let arrowIcon: () -> Arrow
    @ViewBuilder let content: () -> Content

    init(label: String, disabled: Bool, open: Bool, backgroundColor: Color, borderColor: Color, borderOpacity: Double, borderStyle: String, contentColor: Color, radius: CGFloat, action: @escaping () -> Void, @ViewBuilder arrowIcon: @escaping () -> Arrow, @ViewBuilder content: @escaping () -> Content) {
        self.label = label
        self.disabled = disabled
        self.open = open
        self.backgroundColor = backgroundColor
        self.borderColor = borderColor
        self.borderOpacity = borderOpacity
        self.borderStyle = borderStyle
        self.contentColor = contentColor
        self.radius = radius
        self.action = action
        self.arrowIcon = arrowIcon
        self.content = content
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            Button(action: action) {
                HStack {
                    Text(label)
                        .font(.system(size: CGFloat(15), weight: .bold))
                        .foregroundStyle(contentColor)
                    Spacer()
                    arrowIcon()
                        .frame(width: CGFloat(20), height: CGFloat(20))
                        .rotationEffect(open ? .degrees(90) : .degrees(0))
                }
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(12))
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .buttonStyle(.plain)
            .disabled(disabled)
            if open {
                VStack(alignment: .leading, spacing: CGFloat(8)) {
                    content()
                }
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(12))
                .frame(maxWidth: .infinity, alignment: .leading)
                .transition(.opacity.combined(with: .scale(scale: CGFloat(0.98), anchor: .top)))
            }
        }
        .animation(.easeInOut(duration: 0.16), value: open)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay {
            if borderStyle == "separator" {
                if borderOpacity > 0 {
                    Rectangle()
                        .fill(borderColor.opacity(borderOpacity))
                        .frame(height: CGFloat(1))
                        .frame(maxHeight: .infinity, alignment: .bottom)
                }
            } else if borderStyle == "full" && borderOpacity > 0 {
                RoundedRectangle(cornerRadius: radius)
                    .stroke(borderColor.opacity(borderOpacity), lineWidth: CGFloat(1))
            }
        }
        .opacity(disabled ? 0.5 : 1)
    }
}

"##
}
