fn swift_runtime_slider() -> &'static str {
    r##"struct DoweSliderView: View {
    let value: Binding<Double>
    let label: String?
    let hideLabel: Bool
    let lowerBound: Double
    let upperBound: Double
    let step: Double?
    let size: String
    let accentColor: Color

    var body: some View {
        VStack(spacing: CGFloat(6)) {
            if !hideLabel {
                HStack {
                    Text(label ?? "")
                    Spacer()
                    Text(String(format: "%.0f", value.wrappedValue))
                }
                .font(.system(size: CGFloat(14), weight: .semibold))
                .foregroundStyle(accentColor)
            }
            GeometryReader { geometry in
                let thumb = doweSliderThumbSize(size)
                let track = doweSliderTrackHeight(size)
                let available = Swift.max(geometry.size.width - thumb, CGFloat(1))
                let progress = doweSliderProgress(value.wrappedValue, lowerBound: lowerBound, upperBound: upperBound)
                ZStack(alignment: .leading) {
                    Capsule()
                        .fill(accentColor.opacity(0.18))
                        .frame(height: track)
                    Capsule()
                        .fill(accentColor)
                        .frame(width: thumb / CGFloat(2) + available * CGFloat(progress), height: track)
                    Circle()
                        .fill(accentColor)
                        .overlay(Circle().stroke(Color.white, lineWidth: CGFloat(1)))
                        .shadow(color: Color.black.opacity(0.16), radius: CGFloat(2), x: CGFloat(0), y: CGFloat(1))
                        .frame(width: thumb, height: thumb)
                        .offset(x: available * CGFloat(progress))
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: CGFloat(0))
                        .onChanged { event in
                            let ratio = Double(Swift.min(Swift.max(event.location.x / Swift.max(geometry.size.width, CGFloat(1)), CGFloat(0)), CGFloat(1)))
                            value.wrappedValue = doweSliderSteppedValue(ratio: ratio, lowerBound: lowerBound, upperBound: upperBound, step: step)
                        }
                )
            }
            .frame(height: doweSliderThumbSize(size))
        }
        .frame(maxWidth: .infinity)
    }
}

func doweSliderTrackHeight(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(4)
    case "lg":
        return CGFloat(8)
    default:
        return CGFloat(6)
    }
}

func doweSliderThumbSize(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(16)
    case "lg":
        return CGFloat(24)
    default:
        return CGFloat(20)
    }
}

func doweSliderProgress(_ value: Double, lowerBound: Double, upperBound: Double) -> Double {
    if upperBound <= lowerBound {
        return 0
    }
    return Swift.min(Swift.max((value - lowerBound) / (upperBound - lowerBound), 0), 1)
}

func doweSliderSteppedValue(ratio: Double, lowerBound: Double, upperBound: Double, step: Double?) -> Double {
    let raw = lowerBound + (upperBound - lowerBound) * Swift.min(Swift.max(ratio, 0), 1)
    if let step, step > 0 {
        let snapped = lowerBound + ((raw - lowerBound) / step).rounded() * step
        return Swift.min(Swift.max(snapped, lowerBound), upperBound)
    }
    return Swift.min(Swift.max(raw, lowerBound), upperBound)
}

"##
}
