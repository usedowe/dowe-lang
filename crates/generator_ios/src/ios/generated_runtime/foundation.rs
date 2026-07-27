fn swift_runtime_foundation() -> &'static str {
    r#"import SwiftUI
import UIKit
import SafariServices
import Foundation
import UniformTypeIdentifiers
import AVFoundation
import AVKit
import Combine
import CoreMotion
import WebKit

__DOWE_DESIGN__

struct DoweShadowSpec {
    let color: Color
    let blurRadius: CGFloat
    let offsetY: CGFloat
}

struct DoweShadowSurface: View {
    let shadow: DoweShadowSpec
    let cornerRadius: CGFloat

    var body: some View {
        GeometryReader { geometry in
            let extent = max(shadow.blurRadius * CGFloat(2) + abs(shadow.offsetY), CGFloat(1))
            let radius = min(cornerRadius, min(geometry.size.width, geometry.size.height) / CGFloat(2))
            Canvas { context, _ in
                let path = Path(
                    roundedRect: CGRect(
                        x: extent,
                        y: extent,
                        width: geometry.size.width,
                        height: geometry.size.height
                    ),
                    cornerRadius: radius
                )
                context.drawLayer { layer in
                    layer.addFilter(.shadow(
                        color: shadow.color,
                        radius: shadow.blurRadius,
                        x: CGFloat(0),
                        y: shadow.offsetY,
                        options: .shadowOnly
                    ))
                    layer.fill(path, with: .color(.black))
                }
                context.blendMode = .destinationOut
                context.fill(path, with: .color(.black))
            }
            .frame(
                width: geometry.size.width + extent * CGFloat(2),
                height: geometry.size.height + extent * CGFloat(2)
            )
            .offset(x: -extent, y: -extent)
        }
        .allowsHitTesting(false)
        .accessibilityHidden(true)
    }
}

@MainActor
func doweButtonFamily(_ scheme: String) -> Color {
    switch scheme {
    case "background": return DoweDesign.background
    case "surface": return DoweDesign.surface
    case "secondary": return DoweDesign.secondary
    case "tertiary": return DoweDesign.tertiary
    case "muted": return DoweDesign.muted
    case "success": return DoweDesign.success
    case "info": return DoweDesign.info
    case "warning": return DoweDesign.warning
    case "danger": return DoweDesign.danger
    default: return DoweDesign.primary
    }
}

@MainActor
func doweButtonOnFamily(_ scheme: String) -> Color {
    switch scheme {
    case "background": return DoweDesign.onBackground
    case "surface": return DoweDesign.onSurface
    case "secondary": return DoweDesign.onSecondary
    case "tertiary": return DoweDesign.onTertiary
    case "muted": return DoweDesign.onMuted
    case "success": return DoweDesign.onSuccess
    case "info": return DoweDesign.onInfo
    case "warning": return DoweDesign.onWarning
    case "danger": return DoweDesign.onDanger
    default: return DoweDesign.onPrimary
    }
}

@MainActor
func doweButtonContainer(_ variant: String, _ scheme: String) -> Color {
    if variant == "soft" { return doweButtonFamily(scheme).opacity(0.14) }
    if variant == "outlined" || variant == "ghost" { return Color.clear }
    return doweButtonFamily(scheme)
}

@MainActor
func doweButtonContent(_ variant: String, _ scheme: String) -> Color {
    variant == "solid" ? doweButtonOnFamily(scheme) : doweButtonFamily(scheme)
}

func doweSideNavMetric(_ size: String, small: Int, medium: Int, large: Int) -> Int {
    if size == "sm" { return small }
    if size == "lg" { return large }
    return medium
}

@MainActor
func doweButtonRadius(_ value: String) -> CGFloat {
    switch value {
    case "xs": return CGFloat(2)
    case "sm": return CGFloat(4)
    case "lg": return CGFloat(12)
    case "xl": return CGFloat(16)
    case "full": return CGFloat(9999)
    default: return DoweDesign.radius
    }
}

func doweButtonHorizontalPadding(_ value: String) -> CGFloat {
    switch value {
    case "xs": return CGFloat(10)
    case "sm": return CGFloat(12)
    case "lg": return CGFloat(20)
    case "xl": return CGFloat(24)
    default: return CGFloat(16)
    }
}

func doweButtonVerticalPadding(_ value: String) -> CGFloat {
    switch value {
    case "xs": return CGFloat(6)
    case "sm": return CGFloat(8)
    case "lg": return CGFloat(12)
    case "xl": return CGFloat(14)
    default: return CGFloat(10)
    }
}

func doweButtonMinHeight(_ value: String) -> CGFloat {
    switch value {
    case "xs": return CGFloat(28)
    case "sm": return CGFloat(32)
    case "lg": return CGFloat(44)
    case "xl": return CGFloat(48)
    default: return CGFloat(40)
    }
}

enum DoweSize {
    case fixed(CGFloat)
    case full
    case viewportMinus(CGFloat)
}

enum DoweJustify: Equatable {
    case start
    case center
    case end
    case between
    case around
    case evenly
}

enum DoweFlexDirection: Equatable {
    case row
    case column
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
