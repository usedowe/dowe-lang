fn swift_runtime_foundation() -> &'static str {
    r#"import SwiftUI
import UIKit
import SafariServices
import Foundation
import AVKit

__DOWE_DESIGN__

enum DoweSize {
    case fixed(CGFloat)
    case full
}

enum DoweJustify {
    case start
    case center
    case end
    case between
    case around
    case evenly
}

enum DoweAlign {
    case start
    case center
    case end
    case stretch
    case baseline
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
    @State private var active = false

    func body(content: Content) -> some View {
        content
            .opacity(opacity)
            .offset(offset)
            .scaleEffect(scale)
            .animation(.easeOut(duration: 0.22), value: active)
            .onAppear {
                active = true
            }
    }

    private var opacity: Double {
        switch preset {
        case .none:
            return 1
        default:
            return active ? 1 : 0
        }
    }

    private var offset: CGSize {
        if active {
            return .zero
        }
        switch preset {
        case .slideUp:
            return CGSize(width: CGFloat(0), height: CGFloat(16))
        case .slideDown:
            return CGSize(width: CGFloat(0), height: CGFloat(-16))
        case .slideLeft:
            return CGSize(width: CGFloat(16), height: CGFloat(0))
        case .slideRight:
            return CGSize(width: CGFloat(-16), height: CGFloat(0))
        default:
            return .zero
        }
    }

    private var scale: CGFloat {
        if preset == .scaleIn && !active {
            return CGFloat(0.96)
        }
        return CGFloat(1)
    }
}

struct DoweOverlayView: View {
    let overlay: DoweOverlay

    var body: some View {
        switch overlay {
        case .color(let color):
            color
        case .gradient(let start, let end):
            LinearGradient(colors: [start, end], startPoint: .top, endPoint: .bottom)
        }
    }
}

struct DoweSectionBackgroundView: View {
    let background: DoweSectionBackground

    var body: some View {
        switch background {
        case .soft:
            LinearGradient(colors: [DoweDesign.surface, DoweDesign.background], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .aurora:
            LinearGradient(colors: [DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .sunrise:
            LinearGradient(colors: [DoweDesign.softWarning, DoweDesign.softDanger, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .ocean:
            LinearGradient(colors: [DoweDesign.softInfo, DoweDesign.softPrimary, DoweDesign.softTertiary], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .meadow:
            LinearGradient(colors: [DoweDesign.softSuccess, DoweDesign.softTertiary, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .slate:
            LinearGradient(colors: [DoweDesign.softMuted, DoweDesign.surface, DoweDesign.background], startPoint: .topLeading, endPoint: .bottomTrailing)
        }
    }
}

"#
}
