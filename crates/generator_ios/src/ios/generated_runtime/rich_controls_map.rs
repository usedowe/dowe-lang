fn swift_runtime_rich_controls_map() -> &'static str {
    r#"struct DoweToggleGroupItem: Identifiable {
    let id: String
    let label: String
    let icon: String?
}

struct DoweToggleGroup: View {
    @Binding var value: String
    let items: [DoweToggleGroupItem]
    let size: String
    let wide: Bool
    let vertical: Bool
    let disabled: Bool
    let ariaLabel: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let onChange: (() -> Void)?

    var body: some View {
        let stack = Group {
            if vertical {
                VStack(spacing: 4) { buttons }
            } else {
                HStack(spacing: 4) { buttons }
            }
        }
        stack
            .padding(4)
            .frame(maxWidth: wide ? .infinity : nil)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .overlay(RoundedRectangle(cornerRadius: 10).stroke(borderColor ?? .clear, lineWidth: 1))
            .opacity(disabled ? 0.5 : 1)
            .disabled(disabled)
            .accessibilityLabel(ariaLabel ?? "Toggle group")
    }

    private var buttons: some View {
        ForEach(items) { item in
            Button {
                value = item.id
                onChange?()
            } label: {
                Text(item.label)
                    .font(.system(size: size == "lg" ? 17 : size == "xs" ? 12 : size == "sm" ? 13 : 14, weight: .semibold))
                    .frame(maxWidth: wide ? .infinity : nil)
                    .frame(height: size == "lg" ? 44 : size == "xs" ? 24 : size == "sm" ? 32 : 40)
                    .padding(.horizontal, size == "lg" ? 16 : 10)
                    .background(value == item.id ? contentColor : Color.clear)
                    .foregroundStyle(value == item.id ? backgroundColor : contentColor.opacity(0.72))
                    .clipShape(RoundedRectangle(cornerRadius: 8))
            }
            .buttonStyle(.plain)
            .accessibilityAddTraits(value == item.id ? .isSelected : [])
        }
    }
}

struct DoweCollapsible<Content: View>: View {
    let label: String
    let defaultOpen: Bool
    let disabled: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    @ViewBuilder let content: () -> Content
    @State private var open: Bool

    init(label: String, defaultOpen: Bool, disabled: Bool, backgroundColor: Color, contentColor: Color, borderColor: Color?, @ViewBuilder content: @escaping () -> Content) {
        self.label = label
        self.defaultOpen = defaultOpen
        self.disabled = disabled
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.content = content
        _open = State(initialValue: defaultOpen)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                if !disabled { withAnimation(.easeInOut(duration: 0.16)) { open.toggle() } }
            } label: {
                HStack {
                    Text(label).font(.subheadline.weight(.semibold))
                    Spacer()
                    Image(systemName: "chevron.down").rotationEffect(open ? .degrees(180) : .degrees(0))
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 12)
            }
            .buttonStyle(.plain)
            if open {
                VStack(alignment: .leading, spacing: 8) { content() }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
            }
        }
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .overlay(RoundedRectangle(cornerRadius: 16).stroke(borderColor ?? .clear, lineWidth: 1))
        .opacity(disabled ? 0.5 : 1)
    }
}

struct DoweCountdown: View {
    let target: String
    let showDays: Bool
    let showHours: Bool
    let showMinutes: Bool
    let showSeconds: Bool
    let size: String
    let daysLabel: String
    let hoursLabel: String
    let minutesLabel: String
    let secondsLabel: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let onComplete: (() -> Void)?
    @State private var now = Date()
    @State private var completed = false

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            if showDays {
                countdownUnit(value: values.days, label: daysLabel)
                if showHours || showMinutes || showSeconds { countdownSeparator }
            }
            if showHours {
                countdownUnit(value: values.hours, label: hoursLabel)
                if showMinutes || showSeconds { countdownSeparator }
            }
            if showMinutes {
                countdownUnit(value: values.minutes, label: minutesLabel)
                if showSeconds { countdownSeparator }
            }
            if showSeconds { countdownUnit(value: values.seconds, label: secondsLabel) }
        }
        .onReceive(Timer.publish(every: 1, on: .main, in: .common).autoconnect()) { value in
            now = value
            if remaining <= 0 && !completed {
                completed = true
                onComplete?()
            }
        }
    }

    private func countdownUnit(value: Int, label: String) -> some View {
        VStack(spacing: 4) {
            ZStack {
                Text(String(format: "%02d", value))
                    .font(.system(size: metrics.0, weight: .bold, design: .rounded))
                    .monospacedDigit()
            }
            .frame(width: metrics.1, height: metrics.2)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: 16))
            .overlay(RoundedRectangle(cornerRadius: 16).stroke(borderColor ?? .clear, lineWidth: 1))
            Text(label.uppercased())
                .font(.system(size: labelSize, weight: .medium))
                .tracking(1.2)
                .opacity(0.72)
        }
        .foregroundStyle(contentColor)
    }

    private var countdownSeparator: some View {
        Text(":")
            .font(.system(size: metrics.0, weight: .bold, design: .rounded))
            .foregroundStyle(contentColor.opacity(0.5))
            .padding(.top, separatorOffset)
    }

    private var targetDate: Date {
        ISO8601DateFormatter().date(from: target) ?? Date()
    }

    private var remaining: Int {
        max(0, Int(targetDate.timeIntervalSince(now)))
    }

    private var values: (days: Int, hours: Int, minutes: Int, seconds: Int) {
        (remaining / 86400, remaining % 86400 / 3600, remaining % 3600 / 60, remaining % 60)
    }

    private var metrics: (CGFloat, CGFloat, CGFloat) {
        size == "xl" ? (72, 112, 128) : size == "lg" ? (48, 80, 96) : size == "sm" ? (20, 40, 48) : (30, 56, 64)
    }

    private var labelSize: CGFloat {
        size == "xl" ? 16 : size == "lg" ? 14 : size == "sm" ? 10 : 12
    }

    private var separatorOffset: CGFloat {
        size == "xl" ? 28 : size == "lg" ? 20 : size == "sm" ? 8 : 12
    }
}

struct DoweMapMarker: Identifiable {
    let id: String
    let lat: String
    let lng: String
    let label: String?
    let popup: String?
    let icon: String
    let action: (() -> Void)?
}

struct DoweMapWaypoint {
    let lat: String
    let lng: String
}

struct DoweMap: View {
    let centerLat: String
    let centerLng: String
    let zoom: UInt16
    let height: String
    let width: String
    let showControls: Bool
    let showScale: Bool
    let showLocationControl: Bool
    let interactive: Bool
    let markers: [DoweMapMarker]
    let waypoints: [DoweMapWaypoint]
    let backgroundColor: Color
    let contentColor: Color
    let onLocation: (() -> Void)?
    let onLocationError: (() -> Void)?
    let onRoute: (() -> Void)?

    var body: some View {
        GeometryReader { proxy in
            ZStack {
                backgroundColor.opacity(0.18)
                GridPattern().stroke(contentColor.opacity(0.16), lineWidth: 1)
                if !waypoints.isEmpty {
                    Capsule().fill(contentColor.opacity(0.6)).frame(width: proxy.size.width * 0.7, height: 4).rotationEffect(.degrees(-10))
                }
                ForEach(Array(markers.enumerated()), id: \.element.id) { index, marker in
                    Button(action: { marker.action?() }) {
                        VStack(spacing: 4) {
                            Image(systemName: "mappin.circle.fill").font(.title2)
                            if let label = marker.label ?? marker.popup {
                                Text(label).font(.caption.weight(.semibold)).padding(.horizontal, 8).padding(.vertical, 2).background(.ultraThinMaterial).clipShape(Capsule())
                            }
                        }
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(marker.icon == "start" ? DoweDesign.success : marker.icon == "end" ? DoweDesign.danger : contentColor)
                    .position(mapPoint(index: index, total: max(markers.count, 1), size: proxy.size))
                }
                if showControls {
                    VStack(spacing: 0) { Text("+"); Divider(); Text("-") }
                        .font(.headline.weight(.bold))
                        .frame(width: 34)
                        .background(.ultraThinMaterial)
                        .clipShape(RoundedRectangle(cornerRadius: 10))
                        .position(x: proxy.size.width - 28, y: 46)
                }
                if showScale {
                    Text("1 km").font(.caption.weight(.bold)).padding(6).background(.ultraThinMaterial).clipShape(Capsule()).position(x: 42, y: proxy.size.height - 24)
                }
                if showLocationControl {
                    Button(action: { onLocation?() }) { Image(systemName: "location.fill") }
                        .buttonStyle(.borderedProminent)
                        .position(x: proxy.size.width - 28, y: proxy.size.height - 28)
                }
            }
        }
        .frame(height: doweMapHeight(height))
        .clipShape(RoundedRectangle(cornerRadius: 16))
    }

    private func mapPoint(index: Int, total: Int, size: CGSize) -> CGPoint {
        let step = size.width / CGFloat(total + 1)
        let x = min(max(step * CGFloat(index + 1), 36), size.width - 36)
        let y = min(max(size.height * (0.3 + CGFloat((index * 23) % 46) / 100), 36), size.height - 36)
        return CGPoint(x: x, y: y)
    }
}

struct GridPattern: Shape {
    func path(in rect: CGRect) -> Path {
        var path = Path()
        let step: CGFloat = 32
        stride(from: CGFloat(0), through: rect.width, by: step).forEach { x in
            path.move(to: CGPoint(x: x, y: 0))
            path.addLine(to: CGPoint(x: x, y: rect.height))
        }
        stride(from: CGFloat(0), through: rect.height, by: step).forEach { y in
            path.move(to: CGPoint(x: 0, y: y))
            path.addLine(to: CGPoint(x: rect.width, y: y))
        }
        return path
    }
}

func doweMapHeight(_ value: String) -> CGFloat {
    if value.hasSuffix("px") {
        return CGFloat(Double(value.dropLast(2)) ?? 400)
    }
    return CGFloat(Double(value) ?? 400)
}

"#
}
