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
    let accentColor: Color
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
struct DowePaginationControlContract {
    let controlSize: CGFloat
    let indicatorGap: CGFloat
    let indicatorHeight: CGFloat
    let indicatorInactiveWidth: CGFloat
    let indicatorActiveWidth: CGFloat
    let indicatorDotSize: CGFloat
    let indicatorDotActiveScale: CGFloat
}

struct DowePagination<PreviousIcon: View, NextIcon: View>: View {
    @Binding var value: String
    let pageCount: Int
    let size: String
    let control: DowePaginationControlContract
    let paginationVariant: String
    let disabled: Bool
    let ariaLabel: String?
    let backgroundColor: Color
    let contentColor: Color
    let accentColor: Color
    let borderColor: Color?
    let onChange: (() -> Void)?
    @ViewBuilder let previousIcon: () -> PreviousIcon
    @ViewBuilder let nextIcon: () -> NextIcon

    private var current: Int {
        min(pageCount, max(1, Int(value) ?? 1))
    }

    private var dimension: CGFloat { control.controlSize }

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
        HStack(spacing: control.indicatorGap) {
            if paginationVariant == "pages" || paginationVariant == "controls" {
                DoweIconButton(enabled: !disabled && current > 1, dimension: dimension, backgroundColor: backgroundColor, contentColor: contentColor, borderColor: borderColor, label: "Previous page", action: { select(current - 1) }) { previousIcon() }
            }
            if paginationVariant == "pages" {
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
            } else {
                ForEach(1...max(1, pageCount), id: \.self) { page in
                    DowePaginationIndicator(active: page == current, dot: paginationVariant == "dots", enabled: !disabled, size: size, color: accentColor, label: "Go to page \(page)", action: { select(page) }, inactiveWidth: control.indicatorInactiveWidth, activeWidth: control.indicatorActiveWidth, height: control.indicatorHeight, dotSize: control.indicatorDotSize, dotActiveScale: control.indicatorDotActiveScale)
                }
                if paginationVariant == "controls" {
                    Text("\(current) / \(pageCount)").font(.system(size: 13, weight: .semibold)).foregroundStyle(accentColor)
                }
            }
            if paginationVariant == "pages" || paginationVariant == "controls" {
                DoweIconButton(enabled: !disabled && current < pageCount, dimension: dimension, backgroundColor: backgroundColor, contentColor: contentColor, borderColor: borderColor, label: "Next page", action: { select(current + 1) }) { nextIcon() }
            }
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
        .disabled(!enabled).opacity(enabled ? 1 : 0.42)
        .accessibilityLabel(label)
    }
}

struct DoweIconButton<Content: View>: View {
    let enabled: Bool
    let dimension: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let label: String
    let action: () -> Void
    @ViewBuilder let content: () -> Content

    var body: some View {
        Button(action: action) {
            content()
                .frame(width: dimension, height: dimension)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(Circle())
                .overlay(Circle().stroke(borderColor ?? .clear, lineWidth: borderColor == nil ? 0 : 1))
        }
        .buttonStyle(.plain)
        .disabled(!enabled).opacity(enabled ? 1 : 0.42)
        .accessibilityLabel(label)
    }
}

struct DowePaginationIndicator: View {
    let active: Bool
    let dot: Bool
    let enabled: Bool
    let size: String
    let color: Color
    let label: String
    let action: () -> Void
    var inactiveWidth: CGFloat? = nil
    var activeWidth: CGFloat? = nil
    var height: CGFloat? = nil
    var dotSize: CGFloat? = nil
    var dotActiveScale: CGFloat? = nil

    private var resolvedInactiveWidth: CGFloat { inactiveWidth ?? (size == "lg" ? 40 : size == "xs" || size == "sm" ? 24 : 32) }
    private var resolvedActiveWidth: CGFloat { activeWidth ?? (size == "lg" ? 64 : size == "xs" || size == "sm" ? 32 : 48) }
    private var resolvedHeight: CGFloat { height ?? (size == "lg" ? 10 : size == "xs" || size == "sm" ? 6 : 8) }
    private var resolvedDotSize: CGFloat { dotSize ?? 10 }
    private var resolvedDotActiveScale: CGFloat { dotActiveScale ?? 1.25 }

    var body: some View {
        Button(action: action) {
            Capsule()
                .fill(color.opacity(active ? 1 : 0.28))
                .frame(width: dot ? resolvedDotSize : active ? resolvedActiveWidth : resolvedInactiveWidth, height: dot ? resolvedDotSize : resolvedHeight)
                .scaleEffect(dot && active ? resolvedDotActiveScale : 1)
        }
        .buttonStyle(.plain)
        .disabled(!enabled).opacity(enabled ? 1 : 0.42)
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

"#
}
