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
            .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
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
                    .font(.system(size: size == "lg" ? 18 : size == "xs" ? 12 : size == "sm" ? 13 : 14, weight: .semibold))
                    .lineLimit(1)
                    .frame(maxWidth: wide ? .infinity : nil)
                    .frame(height: size == "lg" ? 48 : size == "xs" ? 24 : size == "sm" ? 32 : 40)
                    .padding(.horizontal, size == "lg" ? 16 : size == "xs" ? 8 : size == "sm" ? 10 : 12)
                    .background(value == item.id ? contentColor : Color.clear)
                    .foregroundStyle(value == item.id ? backgroundColor : contentColor.opacity(0.72))
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .contentShape(RoundedRectangle(cornerRadius: 8))
            }
            .buttonStyle(.plain)
            .accessibilityAddTraits(value == item.id ? .isSelected : [])
        }
    }
}

struct DowePagination<PreviousIcon: View, NextIcon: View>: View {
    @Binding var value: String
    let pageCount: Int
    let size: String
    let disabled: Bool
    let ariaLabel: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let onChange: (() -> Void)?
    @ViewBuilder let previousIcon: () -> PreviousIcon
    @ViewBuilder let nextIcon: () -> NextIcon

    private var current: Int {
        min(pageCount, max(1, Int(value) ?? 1))
    }

    private var dimension: CGFloat {
        size == "xs" ? 24 : size == "sm" ? 32 : size == "lg" ? 48 : 40
    }

    private var pages: [Int?] {
        if pageCount <= 7 { return Array(1...pageCount).map(Optional.some) }
        var result: [Int?] = [1]
        if current > 3 { result.append(nil) }
        result.append(contentsOf: Array(max(2, current - 1)...min(pageCount - 1, current + 1)).map(Optional.some))
        if current < pageCount - 2 { result.append(nil) }
        result.append(pageCount)
        return result
    }

    var body: some View {
        HStack(spacing: 4) {
            control(selected: true, enabled: !disabled && current > 1, label: "Previous page", action: { select(current - 1) }) { previousIcon() }
            ForEach(Array(pages.enumerated()), id: \.offset) { _, page in
                if let page {
                    control(selected: page == current, enabled: !disabled, label: "Page \(page)", action: { select(page) }) {
                        Text(String(page)).font(.system(size: size == "lg" ? 17 : size == "xs" ? 12 : size == "sm" ? 13 : 14, weight: .medium))
                    }
                    .accessibilityAddTraits(page == current ? .isSelected : [])
                } else {
                    Text("…").foregroundStyle(DoweDesign.backgroundText.opacity(0.6)).frame(width: dimension, height: dimension)
                }
            }
            control(selected: true, enabled: !disabled && current < pageCount, label: "Next page", action: { select(current + 1) }) { nextIcon() }
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel(ariaLabel ?? "Pagination")
    }

    private func select(_ page: Int) {
        let next = min(pageCount, max(1, page))
        guard next != current else { return }
        value = String(next)
        onChange?()
    }

    private func control<Content: View>(selected: Bool, enabled: Bool, label: String, action: @escaping () -> Void, @ViewBuilder content: () -> Content) -> some View {
        Button(action: action) {
            content()
                .frame(width: dimension, height: dimension)
                .background(selected ? backgroundColor : Color.clear)
                .foregroundStyle(selected ? contentColor : DoweDesign.backgroundText)
                .clipShape(RoundedRectangle(cornerRadius: 10))
                .overlay(RoundedRectangle(cornerRadius: 10).stroke(selected ? borderColor ?? .clear : .clear, lineWidth: 1))
        }
        .buttonStyle(.plain)
        .disabled(!enabled)
        .opacity(enabled ? 1 : 0.42)
        .accessibilityLabel(label)
    }
}

struct DoweCollapsible<Arrow: View, Content: View>: View {
    let label: String
    let defaultOpen: Bool
    let disabled: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @ViewBuilder let arrowIcon: () -> Arrow
    @ViewBuilder let content: () -> Content
    @State private var open: Bool

    init(label: String, defaultOpen: Bool, disabled: Bool, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: CGFloat, @ViewBuilder arrowIcon: @escaping () -> Arrow, @ViewBuilder content: @escaping () -> Content) {
        self.label = label
        self.defaultOpen = defaultOpen
        self.disabled = disabled
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.radius = radius
        self.arrowIcon = arrowIcon
        self.content = content
        _open = State(initialValue: defaultOpen)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                if !disabled { withAnimation(.easeInOut(duration: 0.16)) { open.toggle() } }
            } label: {
                HStack {
                    Text(label).font(.system(size: CGFloat(14), weight: .semibold))
                    Spacer()
                    arrowIcon()
                        .frame(width: CGFloat(20), height: CGFloat(20))
                        .rotationEffect(open ? .degrees(180) : .degrees(0))
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 12)
            }
            .buttonStyle(.plain)
            .disabled(disabled)
            if open {
                VStack(alignment: .leading, spacing: 8) { content() }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
                    .transition(.opacity.combined(with: .scale(scale: CGFloat(0.98), anchor: .top)))
            }
        }
        .animation(.easeInOut(duration: 0.16), value: open)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? .clear, lineWidth: borderColor == nil ? 0 : 1))
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
        ViewThatFits(in: .horizontal) {
            countdownContent(displaySize: size)
            countdownContent(displaySize: "sm")
            ScrollView(.horizontal) {
                countdownContent(displaySize: "sm")
            }
        }
        .frame(maxWidth: .infinity, alignment: .center)
        .onAppear {
            let value = Date()
            now = value
            if targetDate <= value && !completed {
                completed = true
                onComplete?()
            }
        }
        .onReceive(Timer.publish(every: 1, on: .main, in: .common).autoconnect()) { value in
            now = value
            if targetDate <= value && !completed {
                completed = true
                onComplete?()
            }
        }
    }

    private func countdownContent(displaySize: String) -> some View {
        HStack(alignment: .top, spacing: 8) {
            if showDays {
                countdownUnit(value: values.days, label: daysLabel, displaySize: displaySize)
                if showHours || showMinutes || showSeconds { countdownSeparator(displaySize: displaySize) }
            }
            if showHours {
                countdownUnit(value: values.hours, label: hoursLabel, displaySize: displaySize)
                if showMinutes || showSeconds { countdownSeparator(displaySize: displaySize) }
            }
            if showMinutes {
                countdownUnit(value: values.minutes, label: minutesLabel, displaySize: displaySize)
                if showSeconds { countdownSeparator(displaySize: displaySize) }
            }
            if showSeconds { countdownUnit(value: values.seconds, label: secondsLabel, displaySize: displaySize) }
        }
        .fixedSize(horizontal: true, vertical: false)
    }

    private func countdownUnit(value: Int, label: String, displaySize: String) -> some View {
        VStack(spacing: 4) {
            ZStack {
                Text(String(format: "%02d", value))
                    .font(.system(size: metrics(for: displaySize).0, weight: .bold, design: .rounded))
                    .monospacedDigit()
            }
            .frame(minWidth: metrics(for: displaySize).1, minHeight: metrics(for: displaySize).2)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: 16))
            .overlay(RoundedRectangle(cornerRadius: 16).stroke(borderColor ?? .clear, lineWidth: 1))
            Text(label.uppercased())
                .font(.system(size: labelSize(for: displaySize), weight: .medium))
                .tracking(1.2)
                .opacity(0.72)
        }
        .foregroundStyle(contentColor)
    }

    private func countdownSeparator(displaySize: String) -> some View {
        Text(":")
            .font(.system(size: metrics(for: displaySize).0, weight: .bold, design: .rounded))
            .foregroundStyle(contentColor.opacity(0.5))
            .padding(.top, separatorOffset(for: displaySize))
    }

    private var targetDate: Date {
        ISO8601DateFormatter().date(from: target) ?? .distantPast
    }

    private var remaining: Int {
        max(0, Int(targetDate.timeIntervalSince(now)))
    }

    private var values: (days: Int, hours: Int, minutes: Int, seconds: Int) {
        (remaining / 86400, remaining % 86400 / 3600, remaining % 3600 / 60, remaining % 60)
    }

    private func metrics(for displaySize: String) -> (CGFloat, CGFloat, CGFloat) {
        displaySize == "xl" ? (72, 112, 128) : displaySize == "lg" ? (48, 80, 96) : displaySize == "sm" ? (20, 40, 48) : (30, 56, 64)
    }

    private func labelSize(for displaySize: String) -> CGFloat {
        displaySize == "xl" ? 16 : displaySize == "lg" ? 14 : displaySize == "sm" ? 10 : 12
    }

    private func separatorOffset(for displaySize: String) -> CGFloat {
        displaySize == "xl" ? 28 : displaySize == "lg" ? 20 : displaySize == "sm" ? 8 : 12
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
