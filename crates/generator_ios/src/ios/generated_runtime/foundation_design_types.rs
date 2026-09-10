r#"    case fixed(CGFloat)
    case percent(CGFloat)
    case full
    case auto
    case viewportMinus(CGFloat)
}

enum DoweJustify: Equatable {
    case start
    case center
    case end
    case between
    case around
    case evenly
    case stretch
    case normal
    case endSafe
    case centerSafe
}

enum DoweFlexDirection: Equatable {
    case row
    case column
}

enum DoweFlexItem: Equatable {
    case initial
    case auto
    case none
    case fill
}

extension View {
    @ViewBuilder
    func doweFlexItem(_ value: DoweFlexItem, horizontal: Bool) -> some View {
        switch value {
        case .initial, .none:
            self
        case .auto:
            if horizontal {
                self.frame(maxWidth: .infinity, alignment: .leading).layoutPriority(1)
            } else {
                self.frame(maxHeight: .infinity, alignment: .top).layoutPriority(1)
            }
        case .fill:
            if horizontal {
                self.frame(maxWidth: .infinity, alignment: .leading).layoutPriority(1)
            } else {
                self.frame(maxHeight: .infinity, alignment: .top).layoutPriority(1)
            }
        }
    }
}

enum DoweAlign {
    case start
    case end
    case endSafe
    case center
    case centerSafe
    case between
    case around
    case evenly
    case stretch
    case baseline
    case baselineLast
    case normal
}

enum DoweFont {
__DOWE_FONT_CASES__
}

enum DoweOverlay {
    case color(Color)
    case gradient(Color, Color)
}

enum DoweSectionBackground {
    case soft
    case aurora
    case sunrise
    case ocean
    case meadow
    case slate
}


enum DoweAnimationPreset: Equatable {
    case none
    case fadeIn
    case slideUp
    case slideDown
    case slideLeft
    case slideRight
    case scaleIn
}

struct DoweAnimationModifier: ViewModifier {
    let preset: DoweAnimationPreset
"#
