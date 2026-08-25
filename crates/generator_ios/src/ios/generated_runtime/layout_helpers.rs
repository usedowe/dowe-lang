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
    case .percent:
        return nil
    case .full, .auto:
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
    case .fixed, .percent, .viewportMinus, .auto:
        return nil
    case .full:
        return .infinity
    }
}

private struct DoweParentHeightCapLayout: Layout {
    let enabled: Bool

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        guard let subview = subviews.first else {
            return .zero
        }
        let content = subview.sizeThatFits(proposal)
        guard enabled, let maximumHeight = proposal.height else {
            return content
        }
        return CGSize(width: content.width, height: Swift.min(content.height, maximumHeight))
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        guard let subview = subviews.first else {
            return
        }
        let content = subview.sizeThatFits(ProposedViewSize(width: bounds.width, height: enabled ? bounds.height : nil))
        let height = enabled ? Swift.min(content.height, bounds.height) : content.height
        subview.place(at: bounds.origin, proposal: ProposedViewSize(width: content.width, height: height))
    }
}

extension View {
    func doweMaxHeight(_ value: DoweSize?) -> some View {
        let enabled: Bool
        if case .full? = value {
            enabled = true
        } else {
            enabled = false
        }
        return DoweParentHeightCapLayout(enabled: enabled) {
            self
        }
    }
}

private func dowePercentage(_ value: DoweSize?) -> CGFloat? {
    guard let value else {
        return nil
    }
    if case .percent(let fraction) = value {
        return fraction
    }
    return nil
}

private struct DowePercentageWidthLayout: Layout {
    let widthFraction: CGFloat?
    let minimumWidthFraction: CGFloat?

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        guard let subview = subviews.first, let availableWidth = proposal.width else {
            return subviews.first?.sizeThatFits(proposal) ?? .zero
        }
        let exactWidth = widthFraction.map { max(CGFloat(0), availableWidth * $0) }
        let minimumWidth = minimumWidthFraction.map { max(CGFloat(0), availableWidth * $0) }
        guard exactWidth != nil || minimumWidth != nil else {
            return subview.sizeThatFits(proposal)
        }
        let intrinsicWidth = exactWidth == nil ? subview.sizeThatFits(.unspecified).width : CGFloat(0)
        let resolvedWidth = max(exactWidth ?? intrinsicWidth, minimumWidth ?? CGFloat(0))
        let measured = subview.sizeThatFits(ProposedViewSize(width: resolvedWidth, height: proposal.height))
        return CGSize(width: resolvedWidth, height: measured.height)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        guard let subview = subviews.first else {
            return
        }
        subview.place(at: bounds.origin, proposal: ProposedViewSize(width: bounds.width, height: bounds.height))
    }
}

extension View {
    func dowePercentageWidth(width: DoweSize?, minWidth: DoweSize?) -> some View {
        DowePercentageWidthLayout(
            widthFraction: dowePercentage(width),
            minimumWidthFraction: dowePercentage(minWidth)
        ) {
            self
        }
    }
}

func doweHorizontalAlignment(_ value: DoweAlign?) -> HorizontalAlignment {
    switch value {
    case .center, .centerSafe:
        return .center
    case .end, .endSafe:
        return .trailing
    default:
        return .leading
    }
}

func doweVerticalAlignment(_ value: DoweAlign?) -> VerticalAlignment {
    switch value {
    case .center, .stretch, .centerSafe:
        return .center
    case .baseline:
        return .firstTextBaseline
    case .baselineLast:
        return .lastTextBaseline
    case .end, .endSafe:
        return .bottom
    default:
        return .top
    }
}

func doweFrameAlignment(_ value: DoweJustify?) -> Alignment {
    switch value {
    case .center, .around, .evenly, .centerSafe:
        return .center
    case .end, .endSafe:
        return .trailing
    default:
        return .leading
    }
}

func doweColumnFrameAlignment(_ value: DoweAlign?) -> Alignment {
    switch value {
    case .center, .centerSafe:
        return .center
    case .end, .endSafe:
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
    let tracks: [CGFloat]
    let rowGap: CGFloat?
    let columnGap: CGFloat?
    let justify: DoweAlign?
    let align: DoweAlign?
    let fillHeight: Bool

    private var normalizedTracks: [CGFloat] {
        tracks.isEmpty ? [CGFloat(1)] : tracks.map { Swift.max($0, 0) }
    }

    private func resolvedWidth(_ proposal: ProposedViewSize, _ subviews: Subviews) -> CGFloat {
        if let width = proposal.width {
            return width
        }
        let widest = subviews.reduce(CGFloat(0)) { result, subview in
            Swift.max(result, subview.sizeThatFits(.unspecified).width)
        }
        let count = normalizedTracks.count
        return widest * CGFloat(count) + CGFloat(Swift.max(count - 1, 0)) * (columnGap ?? 0)
    }

    private func trackWidths(_ width: CGFloat) -> [CGFloat] {
        let tracks = normalizedTracks
        let gaps = CGFloat(Swift.max(tracks.count - 1, 0)) * (columnGap ?? 0)
        let available = Swift.max(width - gaps, 0)
        let total = Swift.max(tracks.reduce(CGFloat(0), +), 1)
        return tracks.map { available * $0 / total }
    }

    private func itemSizes(_ width: CGFloat, _ subviews: Subviews) -> [CGSize] {
        let widths = trackWidths(width)
        return subviews.enumerated().map { index, subview in
            subview.sizeThatFits(ProposedViewSize(width: widths[index % widths.count], height: nil))
        }
    }

    private func rowHeights(_ sizes: [CGSize]) -> [CGFloat] {
        let count = normalizedTracks.count
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
        let intrinsicHeight = heights.reduce(CGFloat(0), +) + gaps
        let proposedHeight = proposal.height ?? intrinsicHeight
        return CGSize(width: width, height: fillHeight ? Swift.max(intrinsicHeight, proposedHeight) : intrinsicHeight)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let tracks = normalizedTracks
        let widths = trackWidths(bounds.width)
        let sizes = itemSizes(bounds.width, subviews)
        let intrinsicHeights = rowHeights(sizes)
        let contentHeight = intrinsicHeights.reduce(CGFloat(0), +) + CGFloat(Swift.max(intrinsicHeights.count - 1, 0)) * (rowGap ?? 0)
        let extraHeight = Swift.max(bounds.height - contentHeight, 0)
        let rowExtra = intrinsicHeights.isEmpty ? CGFloat(0) : extraHeight / CGFloat(intrinsicHeights.count)
        let heights = intrinsicHeights.map { $0 + rowExtra }
        var y = bounds.minY
        for row in heights.indices {
            let start = row * tracks.count
            let end = Swift.min(start + tracks.count, subviews.count)
            var x = bounds.minX
            for index in start..<end {
                let column = index - start
                let size = sizes[index]
                let itemWidth = widths[column]
                var xOffset: CGFloat = 0
                if justify == .center || justify == .centerSafe {
                    xOffset = (itemWidth - size.width) / 2
                } else if justify == .end || justify == .endSafe {
                    xOffset = itemWidth - size.width
                }
                var yOffset: CGFloat = 0
                if align == .center || align == .centerSafe {
                    yOffset = (heights[row] - size.height) / 2
                } else if align == .end || align == .endSafe {
                    yOffset = heights[row] - size.height
                }
                let stretchesByDefault = align == nil || align == .stretch
                let stretches = stretchesByDefault && subviews[index][DoweGridItemStretchKey.self]
                let itemHeight = stretches ? heights[row] : nil
                subviews[index].place(
                    at: CGPoint(x: x + Swift.max(xOffset, 0), y: y + Swift.max(yOffset, 0)),
                    anchor: .topLeading,
                    proposal: ProposedViewSize(width: itemWidth, height: itemHeight)
                )
                x += itemWidth + (columnGap ?? 0)
            }
            y += heights[row] + (rowGap ?? 0)
        }
    }
}

private struct DoweGridItemStretchKey: LayoutValueKey {
    static let defaultValue = true
}

extension View {
    func doweGridItemStretches(_ value: Bool) -> some View {
        layoutValue(key: DoweGridItemStretchKey.self, value: value)
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
