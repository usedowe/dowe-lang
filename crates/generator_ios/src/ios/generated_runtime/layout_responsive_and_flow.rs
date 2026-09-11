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
    if justify == .around || justify == .evenly {
        return CGFloat(0)
    }
    return justify == .between ? CGFloat(0) : gap ?? CGFloat(0)
}

func doweFlexLeadingSpacer(_ justify: DoweJustify?, gap: CGFloat?) -> CGFloat? {
    if justify == .end || justify == .endSafe || justify == .center || justify == .centerSafe {
        return CGFloat(0)
    }
    if justify == .around {
        return (gap ?? CGFloat(0)) / CGFloat(2)
    }
    if justify == .evenly {
        return gap ?? CGFloat(0)
    }
    return nil
}

func doweFlexTrailingSpacer(_ justify: DoweJustify?, gap: CGFloat?) -> CGFloat? {
    if justify == .center || justify == .centerSafe {
        return CGFloat(0)
    }
    if justify == .around {
        return (gap ?? CGFloat(0)) / CGFloat(2)
    }
    if justify == .evenly {
        return gap ?? CGFloat(0)
    }
    return nil
}

func doweFlexBetweenSpacer(_ justify: DoweJustify?, gap: CGFloat?) -> CGFloat? {
    if justify == .between || justify == .around || justify == .evenly {
        return gap ?? CGFloat(0)
    }
    return nil
}

struct DoweFlowLayout: Layout {
    let justify: DoweJustify?
"#
