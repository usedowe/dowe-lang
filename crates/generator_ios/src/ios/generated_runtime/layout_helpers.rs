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

func doweFixedSize(_ value: DoweSize?) -> CGFloat? {
    guard let value else {
        return nil
    }
    switch value {
    case .fixed(let size):
        return size
    case .full:
        return nil
    }
}

func doweMaxSize(_ value: DoweSize?) -> CGFloat? {
    guard let value else {
        return nil
    }
    switch value {
    case .fixed:
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

func doweGridColumns(_ count: Int?, spacing: CGFloat?) -> [GridItem] {
    Array(
        repeating: GridItem(.flexible(), spacing: spacing ?? 0, alignment: .topLeading),
        count: Swift.max(count ?? 1, 1)
    )
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
