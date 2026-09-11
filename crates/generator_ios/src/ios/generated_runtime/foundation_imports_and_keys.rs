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

struct DowePageEntranceSuppressedKey: EnvironmentKey {
    static let defaultValue = false
}

extension EnvironmentValues {
    var dowePageEntranceSuppressed: Bool {
        get { self[DowePageEntranceSuppressedKey.self] }
        set { self[DowePageEntranceSuppressedKey.self] = newValue }
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
func doweSideNavHeaderColor(_ scheme: String) -> Color {
    doweButtonContent("ghost", scheme)
}

@MainActor
func doweButtonContainer(_ variant: String, _ scheme: String) -> Color {
    if variant == "solid" { return doweButtonFamily(scheme) }
    if variant == "outlined" || variant == "ghost" { return Color.clear }
    return doweButtonFamily(scheme)
}

@MainActor
func doweButtonContent(_ variant: String, _ scheme: String) -> Color {
    variant == "solid" ? doweButtonTextFamily(scheme) : doweButtonFamily(scheme)
}

@MainActor
func doweCardFamily(_ scheme: String) -> Color {
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
func doweCardContent(_ scheme: String) -> Color {
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
    if variant == "solid" { return doweCardFamily(scheme) }
    return Color.clear
}

@MainActor
func doweCardContent(_ variant: String, _ scheme: String) -> Color {
    if variant == "solid" { return doweButtonTextFamily(scheme) }
    return doweButtonFamily(scheme)
}

@MainActor
func doweCardTitle(_ variant: String, _ scheme: String) -> Color {
    if variant == "solid" { return doweButtonTitleFamily(scheme) }
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
"#
