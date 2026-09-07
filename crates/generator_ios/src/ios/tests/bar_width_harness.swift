import SwiftUI
import AppKit

final class Measurements {
    var sizes: [String: CGSize] = [:]
}

struct Measure: Layout {
    let name: String
    let measurements: Measurements

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        subviews[0].sizeThatFits(proposal)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        measurements.sizes[name] = bounds.size
        subviews[0].place(at: bounds.origin, anchor: .topLeading, proposal: ProposedViewSize(bounds.size))
    }
}

let app = NSApplication.shared
for width in [CGFloat(320), CGFloat(402), CGFloat(768)] {
    for boxed in [false, true] {
        let measurements = Measurements()
        let footer = VStack(spacing: 0) {
            HStack(spacing: 0) {
                Color.red.frame(width: 400, height: 64)
                    .padding(8)
                    .fixedSize(horizontal: true, vertical: false)
                Spacer(minLength: 0)
                Color.blue.frame(width: 400, height: 64)
                    .padding(8)
                    .fixedSize(horizontal: true, vertical: false)
            }
            .frame(maxWidth: boxed ? 1536 : .infinity, alignment: .center)
        }
        __BAR_FRAME__
        .padding(.horizontal, 16)
        let content = VStack(spacing: 0) {
            Measure(name: "header", measurements: measurements) {
                Color.green.frame(height: 48)
            }
            ScrollView {
                VStack(spacing: 0) {
                    Measure(name: "section", measurements: measurements) {
                        Measure(name: "card", measurements: measurements) {
                            Color.yellow.frame(height: 200).frame(maxWidth: .infinity)
                        }
                        .padding(.horizontal, 16)
                    }
                    Measure(name: "footer", measurements: measurements) { footer }
                }
                .frame(maxWidth: .infinity)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(maxWidth: .infinity)
        let host = NSHostingView(rootView: content.frame(width: width, height: 800, alignment: .topLeading))
        host.frame = NSRect(x: 0, y: 0, width: width, height: 800)
        let window = NSWindow(contentRect: host.frame, styleMask: .borderless, backing: .buffered, defer: false)
        window.contentView = host
        host.layoutSubtreeIfNeeded()
        RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))
        for name in ["header", "section", "footer"] {
            precondition(measurements.sizes[name]?.width == width, "\(name) escaped viewport \(width): \(measurements.sizes)")
        }
        precondition(measurements.sizes["card"] == CGSize(width: width - 32, height: 200))
        precondition(measurements.sizes["footer"]?.height == 80)
    }
}
