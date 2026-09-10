r#"    let justify: DoweJustify?
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
            } else if justify == .center || justify == .centerSafe {
                start = free / 2
            } else if justify == .end || justify == .endSafe {
                start = free
            }
            var x = bounds.minX + start
            for (subview, size) in row {
                var offset: CGFloat = 0
                if align == .center || align == .centerSafe {
                    offset = (lineHeight - size.height) / 2
                } else if align == .end || align == .endSafe {
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
"#
