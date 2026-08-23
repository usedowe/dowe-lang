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

struct DoweTitleColorKey: EnvironmentKey {
    static let defaultValue: Color? = nil
}

extension EnvironmentValues {
    var doweTitleColor: Color? {
        get { self[DoweTitleColorKey.self] }
        set { self[DoweTitleColorKey.self] = newValue }
    }
}

struct DoweTitleColorModifier: ViewModifier {
    @Environment(\.doweTitleColor) private var inheritedColor
    let explicitColor: Color?

    func body(content: Content) -> some View {
        content.foregroundStyle(explicitColor ?? inheritedColor ?? DoweDesign.backgroundTitle)
    }
}

enum DoweTextAlignment {
    case start
    case center
    case end
    case justify
}

private func doweJustifiedAttributedText(_ value: String) -> AttributedString {
    var attributed = AttributedString(value)
    let paragraphStyle = NSMutableParagraphStyle()
    paragraphStyle.alignment = .justified
    attributed.paragraphStyle = paragraphStyle
    return attributed
}

@ViewBuilder
func doweText(_ value: String, alignment: DoweTextAlignment) -> some View {
    switch alignment {
    case .start:
        Text(verbatim: value).multilineTextAlignment(.leading)
    case .center:
        Text(verbatim: value).multilineTextAlignment(.center)
    case .end:
        Text(verbatim: value).multilineTextAlignment(.trailing)
    case .justify:
        Text(doweJustifiedAttributedText(value))
    }
}

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
    case "accent": return DoweDesign.accent
    case "muted": return DoweDesign.muted
    case "success": return DoweDesign.success
    case "info": return DoweDesign.info
    case "warning": return DoweDesign.warning
    case "danger": return DoweDesign.danger
    default: return DoweDesign.primary
    }
}

@MainActor
func doweButtonTextFamily(_ scheme: String) -> Color {
    switch scheme {
    case "background": return DoweDesign.backgroundText
    case "surface": return DoweDesign.surfaceText
    case "secondary": return DoweDesign.secondaryText
    case "accent": return DoweDesign.accentText
    case "muted": return DoweDesign.mutedText
    case "success": return DoweDesign.successText
    case "info": return DoweDesign.infoText
    case "warning": return DoweDesign.warningText
    case "danger": return DoweDesign.dangerText
    default: return DoweDesign.primaryText
    }
}

@MainActor
func doweButtonTitleFamily(_ scheme: String) -> Color {
    switch scheme {
    case "background": return DoweDesign.backgroundTitle
    case "surface": return DoweDesign.surfaceTitle
    case "secondary": return DoweDesign.secondaryTitle
    case "accent": return DoweDesign.accentTitle
    case "muted": return DoweDesign.mutedTitle
    case "success": return DoweDesign.successTitle
    case "info": return DoweDesign.infoTitle
    case "warning": return DoweDesign.warningTitle
    case "danger": return DoweDesign.dangerTitle
    default: return DoweDesign.primaryTitle
    }
}

@MainActor
func doweButtonSoftTitleFamily(_ scheme: String) -> Color {
    switch scheme {
    case "background": return DoweDesign.backgroundTitle
    case "surface": return DoweDesign.surfaceTitle
    case "secondary": return DoweDesign.secondaryTitle
    case "accent": return DoweDesign.accentTitle
    case "muted": return DoweDesign.mutedTitle
    case "success": return DoweDesign.successTitle
    case "info": return DoweDesign.infoTitle
    case "warning": return DoweDesign.warningTitle
    case "danger": return DoweDesign.dangerTitle
    default: return DoweDesign.primaryTitle
    }
}

@MainActor
func doweSideNavHeaderColor(_ scheme: String) -> Color {
    doweButtonContent("ghost", scheme)
}

@MainActor
func doweButtonContainer(_ variant: String, _ scheme: String) -> Color {
    if variant == "soft" { return doweButtonFamily(scheme).opacity(0.14) }
    if variant == "outlined" || variant == "ghost" { return Color.clear }
    return doweButtonFamily(scheme)
}

@MainActor
func doweButtonContent(_ variant: String, _ scheme: String) -> Color {
    variant == "solid" ? doweButtonTextFamily(scheme) : doweButtonFamily(scheme)
}

@MainActor
func doweCardSoftFamily(_ scheme: String) -> Color {
    switch scheme {
    case "background": return DoweDesign.background
    case "surface": return DoweDesign.surface
    case "secondary": return DoweDesign.secondary
    case "accent": return DoweDesign.accent
    case "muted": return DoweDesign.muted
    case "success": return DoweDesign.success
    case "info": return DoweDesign.info
    case "warning": return DoweDesign.warning
    case "danger": return DoweDesign.danger
    default: return DoweDesign.primary
    }
}

@MainActor
func doweCardSoftContent(_ scheme: String) -> Color {
    switch scheme {
    case "background": return DoweDesign.backgroundText
    case "surface": return DoweDesign.surfaceText
    case "secondary": return DoweDesign.secondaryText
    case "accent": return DoweDesign.accentText
    case "muted": return DoweDesign.mutedText
    case "success": return DoweDesign.successText
    case "info": return DoweDesign.infoText
    case "warning": return DoweDesign.warningText
    case "danger": return DoweDesign.dangerText
    default: return DoweDesign.primaryText
    }
}

@MainActor
func doweCardContainer(_ variant: String, _ scheme: String) -> Color {
    if variant == "soft" { return doweCardSoftFamily(scheme) }
    if variant == "outlined" { return scheme == "background" ? DoweDesign.background : DoweDesign.surface }
    if variant == "ghost" { return Color.clear }
    return doweButtonFamily(scheme)
}

@MainActor
func doweCardContent(_ variant: String, _ scheme: String) -> Color {
    if variant == "solid" { return doweButtonTextFamily(scheme) }
    if variant == "soft" { return doweCardSoftContent(scheme) }
    if variant == "outlined" { return scheme == "background" ? DoweDesign.backgroundText : DoweDesign.surfaceText }
    if variant == "ghost" && (scheme == "background" || scheme == "surface") { return doweButtonTextFamily(scheme) }
    return doweButtonFamily(scheme)
}

@MainActor
func doweCardBorder(_ variant: String, _ scheme: String) -> Color? {
    variant == "outlined" ? doweButtonFamily(scheme) : nil
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

enum DoweGesturePreset {
    case none
    case lift
    case press
    case grow
    case tilt
}

enum DoweTransitionPreset {
    case none
    case quick
    case smooth
    case spring
}

struct DoweGestureModifier: ViewModifier {
    let preset: DoweGesturePreset
    let transition: DoweTransitionPreset
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var hovered = false
    @GestureState private var pressed = false

    func body(content: Content) -> some View {
        content
            .offset(y: verticalOffset)
            .scaleEffect(scale)
            .rotationEffect(.degrees(rotation))
            .shadow(color: shadowColor, radius: shadowRadius, x: CGFloat(0), y: shadowOffset)
            .animation(animation, value: hovered)
            .animation(animation, value: pressed)
            .onHover { value in
                hovered = value
            }
            .simultaneousGesture(pressGesture)
    }

    private var pressGesture: some Gesture {
        DragGesture(minimumDistance: 0)
            .updating($pressed) { _, state, _ in
                state = true
            }
    }

    private var activeHover: Bool {
        hovered && !reduceMotion
    }

    private var activePress: Bool {
        pressed && !reduceMotion
    }

    private var lifted: Bool {
        (activeHover && !activePress) || (activePress && !hovered)
    }

    private var verticalOffset: CGFloat {
        if preset == .lift && lifted {
            return CGFloat(-4)
        }
        return CGFloat(0)
    }

    private var scale: CGFloat {
        if activePress && preset == .press {
            return CGFloat(0.94)
        }
        if activePress && preset == .lift {
            return CGFloat(0.98)
        }
        if preset == .grow && (activeHover || activePress) {
            return activeHover && activePress ? CGFloat(1.01) : CGFloat(1.04)
        }
        return CGFloat(1)
    }

    private var rotation: Double {
        if preset == .tilt && (activeHover || activePress) {
            return activeHover && activePress ? 1 : 3
        }
        return 0
    }

    private var shadowColor: Color {
        preset == .lift && lifted ? Color.black.opacity(0.16) : Color.clear
    }

    private var shadowRadius: CGFloat {
        preset == .lift && lifted ? CGFloat(12) : CGFloat(0)
    }

    private var shadowOffset: CGFloat {
        preset == .lift && lifted ? CGFloat(6) : CGFloat(0)
    }

    private var animation: Animation? {
        if reduceMotion || transition == .none {
            return nil
        }
        switch transition {
        case .quick:
            return .easeOut(duration: 0.12)
        case .smooth:
            return .easeInOut(duration: 0.22)
        case .spring:
            return .spring(response: 0.32, dampingFraction: 0.72)
        case .none:
            return nil
        }
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
            LinearGradient(colors: [DoweDesign.primary, DoweDesign.secondary, DoweDesign.accent], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .sunrise:
            LinearGradient(colors: [DoweDesign.warning, DoweDesign.danger, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .ocean:
            LinearGradient(colors: [DoweDesign.info, DoweDesign.primary, DoweDesign.accent], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .meadow:
            LinearGradient(colors: [DoweDesign.success, DoweDesign.accent, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .slate:
            LinearGradient(colors: [DoweDesign.muted, DoweDesign.surface, DoweDesign.background], startPoint: .topLeading, endPoint: .bottomTrailing)
        }
    }
}

"#
}
