fn swift_runtime_badge_chip_skeleton() -> &'static str {
    r#"struct DoweBadge<Content: View>: View {
    let text: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let content: Content

    init(text: String, position: String, backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.text = text
        self.position = position
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.content = content()
    }

    var body: some View {
        ZStack(alignment: alignment) {
            content
            Text(text)
                .font(.caption2.weight(.semibold))
                .padding(.horizontal, CGFloat(6))
                .padding(.vertical, CGFloat(2))
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(Capsule())
        }
    }

    private var alignment: Alignment {
        switch position {
        case "top-left": return .topLeading
        case "bottom-left": return .bottomLeading
        case "bottom-right": return .bottomTrailing
        default: return .topTrailing
        }
    }
}

struct DoweChip<Start: View, End: View>: View {
    let text: String
    let size: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let hasStart: Bool
    let hasEnd: Bool
    let start: Start
    let end: End

    init(text: String, size: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, hasStart: Bool, hasEnd: Bool, @ViewBuilder start: () -> Start, @ViewBuilder end: () -> End) {
        self.text = text
        self.size = size
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.action = action
        self.hasStart = hasStart
        self.hasEnd = hasEnd
        self.start = start()
        self.end = end()
    }

    var body: some View {
        HStack(spacing: CGFloat(8)) {
            if hasStart { start }
            Text(text)
                .lineLimit(1)
            if hasEnd { end }
            if let action {
                Button(action: action) {
                    Text("x")
                        .fontWeight(.bold)
                        .opacity(0.72)
                }
                .buttonStyle(.plain)
            }
        }
        .font(.system(size: textSize, weight: .medium))
        .foregroundStyle(contentColor)
        .padding(.horizontal, horizontalPadding)
        .frame(height: height)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusUi))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radiusUi).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
    }

    private var height: CGFloat {
        switch size {
        case "xs": return CGFloat(20)
        case "sm": return CGFloat(24)
        case "lg": return CGFloat(40)
        case "xl": return CGFloat(48)
        default: return CGFloat(32)
        }
    }

    private var horizontalPadding: CGFloat {
        switch size {
        case "xs", "sm": return CGFloat(12)
        case "lg": return CGFloat(20)
        case "xl": return CGFloat(24)
        default: return CGFloat(16)
        }
    }

    private var textSize: CGFloat {
        switch size {
        case "xs", "sm": return CGFloat(12)
        case "lg": return CGFloat(18)
        case "xl": return CGFloat(24)
        default: return CGFloat(14)
        }
    }
}

struct DoweSkeleton: View {
    let variant: String
    let animation: String
    @State private var active = false

    var body: some View {
        Rectangle()
            .fill(DoweDesign.muted)
            .opacity(animation == "pulse" && active ? 0.45 : 1)
            .frame(height: variant == "text" ? CGFloat(16) : nil)
            .clipShape(shape)
            .onAppear {
                guard animation != "none" else { return }
                withAnimation(.easeInOut(duration: 0.9).repeatForever(autoreverses: true)) {
                    active = true
                }
            }
    }

    private var shape: AnyShape {
        switch variant {
        case "circular": return AnyShape(Circle())
        case "rectangular": return AnyShape(Rectangle())
        case "rounded": return AnyShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
        default: return AnyShape(RoundedRectangle(cornerRadius: CGFloat(6)))
        }
    }
}

struct AnyShape: Shape {
    private let pathBuilder: @Sendable (CGRect) -> Path

    init<S: Shape & Sendable>(_ shape: S) {
        pathBuilder = { rect in shape.path(in: rect) }
    }

    func path(in rect: CGRect) -> Path {
        pathBuilder(rect)
    }
}

"#
}
