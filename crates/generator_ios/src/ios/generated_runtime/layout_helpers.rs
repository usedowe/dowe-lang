fn swift_runtime_layout_helpers() -> &'static str {
    r#"func doweResponsive<T>(_ viewportWidth: CGFloat, xs: T? = nil, sm: T? = nil, md: T? = nil, lg: T? = nil, xl: T? = nil) -> T? {
    var value: T?
    if viewportWidth >= 0, let current = xs {
        value = current
    }
    if viewportWidth >= 640, let current = sm {
        value = current
    }
    if viewportWidth >= 768, let current = md {
        value = current
    }
    if viewportWidth >= 1024, let current = lg {
        value = current
    }
    if viewportWidth >= 1280, let current = xl {
        value = current
    }
    return value
}

func doweFixedSize(_ value: DoweSize?, viewportHeight: CGFloat? = nil) -> CGFloat? {
    guard let value else {
        return nil
    }
    switch value {
    case .fixed(let size):
        return size
    case .full:
        return nil
    case .viewportMinus(let inset):
        guard let viewportHeight else {
            return nil
        }
        return max(CGFloat(0), viewportHeight - inset)
    }
}

func doweMaxSize(_ value: DoweSize?) -> CGFloat? {
    guard let value else {
        return nil
    }
    switch value {
    case .fixed, .viewportMinus:
        return nil
    case .full:
        return .infinity
    }
}

func doweHorizontalAlignment(_ value: DoweAlign?) -> HorizontalAlignment {
    switch value {
    case .center:
        return .center
    case .end:
        return .trailing
    default:
        return .leading
    }
}

func doweVerticalAlignment(_ value: DoweAlign?) -> VerticalAlignment {
    switch value {
    case .center, .stretch:
        return .center
    case .end:
        return .bottom
    default:
        return .top
    }
}

func doweFrameAlignment(_ value: DoweJustify?) -> Alignment {
    switch value {
    case .center, .around, .evenly:
        return .center
    case .end:
        return .trailing
    default:
        return .leading
    }
}

func doweColumnFrameAlignment(_ value: DoweAlign?) -> Alignment {
    switch value {
    case .center:
        return .center
    case .end:
        return .trailing
    default:
        return .leading
    }
}

func doweFlexStackSpacing(_ justify: DoweJustify?, gap: CGFloat?) -> CGFloat {
    justify == .between ? CGFloat(0) : gap ?? CGFloat(0)
}

func doweFlexBetweenSpacer(_ justify: DoweJustify?, gap: CGFloat?) -> CGFloat? {
    justify == .between ? gap ?? CGFloat(0) : nil
}

struct DoweFlowLayout: Layout {
    let justify: DoweJustify?
    let align: DoweAlign?
    let gap: CGFloat?

    private func lines(_ proposal: ProposedViewSize, _ subviews: Subviews) -> [[(LayoutSubview, CGSize)]] {
        let width = proposal.width ?? .infinity
        let spacing = gap ?? 0
        var result: [[(LayoutSubview, CGSize)]] = []
        var line: [(LayoutSubview, CGSize)] = []
        var used: CGFloat = 0
        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            let next = line.isEmpty ? size.width : used + spacing + size.width
            if !line.isEmpty && next > width {
                result.append(line)
                line = []
                used = 0
            }
            line.append((subview, size))
            used = line.count == 1 ? size.width : used + spacing + size.width
        }
        if !line.isEmpty {
            result.append(line)
        }
        return result
    }

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = lines(proposal, subviews)
        let spacing = gap ?? 0
        var contentWidth: CGFloat = 0
        var contentHeight: CGFloat = 0
        for row in rows {
            var rowWidth: CGFloat = 0
            var rowHeight: CGFloat = 0
            for (_, size) in row {
                rowWidth += size.width
                rowHeight = Swift.max(rowHeight, size.height)
            }
            let itemGapCount = Swift.max(row.count - 1, 0)
            rowWidth += CGFloat(itemGapCount) * spacing
            contentWidth = Swift.max(contentWidth, rowWidth)
            contentHeight += rowHeight
        }
        let lineGapCount = Swift.max(rows.count - 1, 0)
        contentHeight += CGFloat(lineGapCount) * spacing
        let resolvedWidth = proposal.width ?? contentWidth
        return CGSize(width: resolvedWidth, height: contentHeight)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let rows = lines(ProposedViewSize(width: bounds.width, height: proposal.height), subviews)
        let spacing = gap ?? 0
        var y = bounds.minY
        for row in rows {
            var contentWidth: CGFloat = 0
            var lineHeight: CGFloat = 0
            for (_, size) in row {
                contentWidth += size.width
                lineHeight = Swift.max(lineHeight, size.height)
            }
            let itemGapCount = Swift.max(row.count - 1, 0)
            contentWidth += CGFloat(itemGapCount) * spacing
            let free = Swift.max(bounds.width - contentWidth, 0)
            var lineGap = spacing
            var start: CGFloat = 0
            if justify == .between && row.count > 1 {
                lineGap += free / CGFloat(row.count - 1)
            } else if justify == .around && !row.isEmpty {
                let distributed = free / CGFloat(row.count)
                lineGap += distributed
                start = distributed / 2
            } else if justify == .evenly {
                let distributed = free / CGFloat(row.count + 1)
                lineGap += distributed
                start = distributed
            } else if justify == .center {
                start = free / 2
            } else if justify == .end {
                start = free
            }
            var x = bounds.minX + start
            for (subview, size) in row {
                var offset: CGFloat = 0
                if align == .center {
                    offset = (lineHeight - size.height) / 2
                } else if align == .end {
                    offset = lineHeight - size.height
                }
                subview.place(at: CGPoint(x: x, y: y + offset), proposal: ProposedViewSize(size))
                x += size.width + lineGap
            }
            y += lineHeight + spacing
        }
    }
}

struct DoweGridLayout: Layout {
    let columns: Int
    let rowGap: CGFloat?
    let columnGap: CGFloat?
    let justify: DoweAlign?
    let align: DoweAlign?

    private func resolvedWidth(_ proposal: ProposedViewSize, _ subviews: Subviews) -> CGFloat {
        if let width = proposal.width {
            return width
        }
        let widest = subviews.reduce(CGFloat(0)) { result, subview in
            Swift.max(result, subview.sizeThatFits(.unspecified).width)
        }
        let count = Swift.max(columns, 1)
        return widest * CGFloat(count) + CGFloat(Swift.max(count - 1, 0)) * (columnGap ?? 0)
    }

    private func trackWidth(_ width: CGFloat) -> CGFloat {
        let count = Swift.max(columns, 1)
        let gaps = CGFloat(Swift.max(count - 1, 0)) * (columnGap ?? 0)
        return Swift.max((width - gaps) / CGFloat(count), 0)
    }

    private func itemSizes(_ width: CGFloat, _ subviews: Subviews) -> [CGSize] {
        let itemWidth = trackWidth(width)
        return subviews.map { subview in
            subview.sizeThatFits(ProposedViewSize(width: itemWidth, height: nil))
        }
    }

    private func rowHeights(_ sizes: [CGSize]) -> [CGFloat] {
        let count = Swift.max(columns, 1)
        return stride(from: 0, to: sizes.count, by: count).map { start in
            sizes[start..<Swift.min(start + count, sizes.count)].reduce(CGFloat(0)) { result, size in
                Swift.max(result, size.height)
            }
        }
    }

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = resolvedWidth(proposal, subviews)
        let heights = rowHeights(itemSizes(width, subviews))
        let gaps = CGFloat(Swift.max(heights.count - 1, 0)) * (rowGap ?? 0)
        return CGSize(width: width, height: heights.reduce(CGFloat(0), +) + gaps)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let count = Swift.max(columns, 1)
        let itemWidth = trackWidth(bounds.width)
        let sizes = itemSizes(bounds.width, subviews)
        let heights = rowHeights(sizes)
        var y = bounds.minY
        for row in heights.indices {
            let start = row * count
            let end = Swift.min(start + count, subviews.count)
            for index in start..<end {
                let column = index - start
                let size = sizes[index]
                var xOffset: CGFloat = 0
                if justify == .center {
                    xOffset = (itemWidth - size.width) / 2
                } else if justify == .end {
                    xOffset = itemWidth - size.width
                }
                var yOffset: CGFloat = 0
                if align == .center {
                    yOffset = (heights[row] - size.height) / 2
                } else if align == .end {
                    yOffset = heights[row] - size.height
                }
                let x = bounds.minX + CGFloat(column) * (itemWidth + (columnGap ?? 0)) + Swift.max(xOffset, 0)
                let itemHeight = align == .stretch ? heights[row] : nil
                subviews[index].place(
                    at: CGPoint(x: x, y: y + Swift.max(yOffset, 0)),
                    anchor: .topLeading,
                    proposal: ProposedViewSize(width: itemWidth, height: itemHeight)
                )
            }
            y += heights[row] + (rowGap ?? 0)
        }
    }
}

private struct DoweAppBarDockedKey: EnvironmentKey {
    static let defaultValue = false
}

private extension EnvironmentValues {
    var doweAppBarDocked: Bool {
        get { self[DoweAppBarDockedKey.self] }
        set { self[DoweAppBarDockedKey.self] = newValue }
    }
}

private final class DoweDockingState: ObservableObject {
    @Published var scrollOffset = CGFloat(0)
}

private struct DoweDockingStateKey: EnvironmentKey {
    static let defaultValue: DoweDockingState? = nil
}

private extension EnvironmentValues {
    var doweDockingState: DoweDockingState? {
        get { self[DoweDockingStateKey.self] }
        set { self[DoweDockingStateKey.self] = newValue }
    }
}

struct DoweDockingScrollObserver: UIViewRepresentable {
    @Environment(\.doweDockingState) private var state

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView(frame: .zero)
        view.isUserInteractionEnabled = false
        context.coordinator.connect(from: view, state: state)
        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.connect(from: uiView, state: state)
    }

    static func dismantleUIView(_ uiView: UIView, coordinator: Coordinator) {
        coordinator.disconnect()
    }

    final class Coordinator {
        private weak var observedScrollView: UIScrollView?
        private var observation: NSKeyValueObservation?

        fileprivate func connect(from view: UIView, state: DoweDockingState?) {
            DispatchQueue.main.async { [weak self, weak view, weak state] in
                guard let self, let view, let scrollView = self.enclosingScrollView(from: view) else {
                    return
                }
                guard self.observedScrollView !== scrollView else {
                    return
                }
                self.disconnect()
                self.observedScrollView = scrollView
                self.observation = scrollView.observe(\.contentOffset, options: [.initial, .new]) { [weak state] scrollView, _ in
                    state?.scrollOffset = max(
                        CGFloat(0),
                        scrollView.contentOffset.y + scrollView.adjustedContentInset.top
                    )
                }
            }
        }

        func disconnect() {
            observation?.invalidate()
            observation = nil
            observedScrollView = nil
        }

        private func enclosingScrollView(from view: UIView) -> UIScrollView? {
            var current: UIView? = view
            while let candidate = current {
                if let scrollView = candidate as? UIScrollView {
                    return scrollView
                }
                current = candidate.superview
            }
            return nil
        }
    }
}

struct DoweDockingScaffold<Content: View>: View {
    @StateObject private var state = DoweDockingState()
    let content: Content

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        content
            .environment(\.doweDockingState, state)
            .environment(\.doweAppBarDocked, state.scrollOffset > CGFloat(100))
    }
}

struct DoweDockingAppBarModifier: ViewModifier {
    @Environment(\.doweAppBarDocked) private var docked
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    let backgroundColor: Color
    let contentColor: Color

    func body(content: Content) -> some View {
        let radius = docked ? CGFloat(0) : DoweDesign.radius
        content
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(RoundedRectangle(cornerRadius: radius).stroke(DoweDesign.muted, lineWidth: CGFloat(1)).opacity(docked ? 0 : 1))
            .overlay(alignment: .bottom) {
                Rectangle()
                    .fill(DoweDesign.muted)
                    .frame(height: CGFloat(1))
                    .opacity(docked ? 1 : 0)
            }
            .padding(.horizontal, docked ? CGFloat(0) : CGFloat(16))
            .padding(.vertical, docked ? CGFloat(0) : CGFloat(8))
            .animation(reduceMotion ? nil : .timingCurve(0.4, 0, 0.2, 1, duration: 0.3), value: docked)
    }
}

func doweTextSize(_ viewportWidth: CGFloat, min: CGFloat, preferredBase: CGFloat, preferredViewport: CGFloat, max: CGFloat) -> CGFloat {
    Swift.max(min, Swift.min(preferredBase + viewportWidth * preferredViewport / 100, max))
}

func doweTextLineSpacing(fontSize: CGFloat, lineHeight: CGFloat) -> CGFloat {
    Swift.max(fontSize * lineHeight - fontSize, 0)
}

func doweTextTracking(fontSize: CGFloat, em: CGFloat) -> CGFloat {
    fontSize * em
}

func doweFont(_ value: DoweFont?, size: CGFloat) -> Font {
    switch value {
__DOWE_FONT_SWITCH__
    case .none:
        return __DOWE_DEFAULT_FONT__
    }
}

"#
}
