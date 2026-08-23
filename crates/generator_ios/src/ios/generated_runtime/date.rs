fn swift_runtime_date() -> &'static str {
    r##"private func doweDateParser() -> DateFormatter {
    let formatter = DateFormatter()
    formatter.calendar = Calendar(identifier: .gregorian)
    formatter.locale = Locale(identifier: "en_US_POSIX")
    formatter.timeZone = TimeZone(secondsFromGMT: 0)
    formatter.dateFormat = "yyyy-MM-dd"
    return formatter
}

private func doweParseDate(_ value: String) -> Date? {
    doweDateParser().date(from: value)
}

private func doweDateValue(_ date: Date) -> String {
    doweDateParser().string(from: date)
}

private func doweDateTodayValue() -> String {
    doweDateValue(Date())
}

private func doweDateMonth(_ date: Date) -> Date {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
    return calendar.date(from: calendar.dateComponents([.year, .month], from: date)) ?? date
}

private func doweDateStep(_ date: Date, amount: Int) -> Date {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
    return calendar.date(byAdding: .month, value: amount, to: date) ?? date
}

private func doweDateAllowed(_ value: String, min: String?, max: String?) -> Bool {
    guard let date = doweParseDate(value) else { return false }
    if let min, let minimum = doweParseDate(min), date < minimum { return false }
    if let max, let maximum = doweParseDate(max), date > maximum { return false }
    return true
}

private func doweDateGrid(_ month: Date) -> [Date?] {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
    let first = doweDateMonth(month)
    let weekday = calendar.component(.weekday, from: first)
    let leading = (weekday + 5) % 7
    let count = calendar.range(of: .day, in: .month, for: first)?.count ?? 0
    let days = (0..<count).compactMap { calendar.date(byAdding: .day, value: $0, to: first) }
    return Array(repeating: nil, count: leading) + days.map { Optional($0) }
}

private func doweDateLabel(_ value: String) -> String {
    guard let date = doweParseDate(value) else { return "" }
    return date.formatted(.dateTime.year().month(.abbreviated).day())
}

private func doweDateMonthLabel(_ month: Date) -> String {
    month.formatted(.dateTime.month(.wide).year())
}

private struct DoweDateCalendar: View {
    let month: Date
    let selected: String
    let start: String
    let end: String
    let min: String?
    let max: String?
    let contentColor: Color
    let accentColor: Color
    let showPrevious: Bool
    let showNext: Bool
    let onPrevious: () -> Void
    let onNext: () -> Void
    let onSelect: (String) -> Void

    private let weekdays = ["M", "T", "W", "T", "F", "S", "S"]

    var body: some View {
        VStack(spacing: CGFloat(8)) {
            HStack {
                Button("‹", action: onPrevious)
                    .disabled(!showPrevious)
                Spacer()
                Text(doweDateMonthLabel(month)).fontWeight(.semibold)
                Spacer()
                Button("›", action: onNext)
                    .disabled(!showNext)
            }
            .font(.title3)
            .foregroundStyle(contentColor)
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: CGFloat(4)), count: 7), spacing: CGFloat(4)) {
                ForEach(Array(weekdays.enumerated()), id: \.offset) { _, weekday in
                    Text(weekday)
                        .font(.caption2)
                        .fontWeight(.semibold)
                        .foregroundStyle(contentColor.opacity(0.68))
                }
                ForEach(Array(doweDateGrid(month).enumerated()), id: \.offset) { _, date in
                    if let date {
                        let value = doweDateValue(date)
                        let isStart = value == start
                        let isEnd = value == end
                        let isSelected = value == selected || isStart || isEnd
                        let isToday = value == doweDateTodayValue()
                        let inRange = !start.isEmpty && !end.isEmpty && value > start && value < end
                        let enabled = doweDateAllowed(value, min: min, max: max)
                        Button(action: { onSelect(value) }) {
                            Text(String(calendarDay(date)))
                                .font(.caption)
                                .fontWeight(isSelected ? .bold : .regular)
                                .frame(maxWidth: .infinity, minHeight: CGFloat(32))
                                .background(isSelected ? accentColor : inRange ? accentColor.opacity(0.16) : Color.clear)
                                .foregroundStyle(isSelected ? Color.white : contentColor)
                                .clipShape(RoundedRectangle(cornerRadius: CGFloat(8)))
                                .overlay(RoundedRectangle(cornerRadius: CGFloat(8)).stroke(isToday && !isSelected ? accentColor : Color.clear, lineWidth: CGFloat(1)))
                        }
                        .buttonStyle(.plain)
                        .disabled(!enabled)
                        .opacity(enabled ? 1 : 0.35)
                    } else {
                        Color.clear.frame(minHeight: CGFloat(32))
                    }
                }
            }
        }
        .padding(CGFloat(12))
        .background(Color(uiColor: .systemBackground))
        .foregroundStyle(contentColor)
    }

    private func calendarDay(_ date: Date) -> Int {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
        return calendar.component(.day, from: date)
    }
}

struct DoweDateField: View {
    let value: Binding<String>
    let label: String?
    let placeholder: String
    let floating: Bool
    let size: String
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let name: String?
    let helpText: String?
    let errorText: String?
    let min: String?
    let max: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let validationRules: [DoweValidationRule]
    @State private var expanded = false
    @State private var touched = false
    @State private var month: Date

    init(value: Binding<String>, label: String?, placeholder: String, floating: Bool, size: String, fontSize: CGFloat, lineHeight: CGFloat, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, backgroundColor: Color, contentColor: Color, borderColor: Color?, validationRules: [DoweValidationRule]) {
        self.value = value
        self.label = label
        self.placeholder = placeholder
        self.floating = floating
        self.size = size
        self.fontSize = fontSize
        self.lineHeight = lineHeight
        self.name = name
        self.helpText = helpText
        self.errorText = errorText
        self.min = min
        self.max = max
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.validationRules = validationRules
        _month = State(initialValue: doweDateMonth(doweParseDate(value.wrappedValue) ?? Date()))
    }

    var body: some View {
        let validationError = errorText ?? (touched ? doweValidationError(value.wrappedValue, rules: validationRules) : nil)
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating { Text(label).font(.footnote).fontWeight(.semibold) }
            Button(action: { if expanded { touched = true }; expanded.toggle() }) {
                ZStack(alignment: .leading) {
                    if let label, floating {
                        Text(label)
                            .font(.caption)
                            .offset(y: value.wrappedValue.isEmpty && !expanded ? CGFloat(0) : CGFloat(-12))
                            .scaleEffect(value.wrappedValue.isEmpty && !expanded ? CGFloat(1) : CGFloat(0.9), anchor: .leading)
                    }
                    HStack {
                        Text(doweDateLabel(value.wrappedValue).isEmpty ? placeholder : doweDateLabel(value.wrappedValue)).lineLimit(1)
                        Spacer()
                        DoweSelectArrow(color: contentColor)
                    }
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
                .font(.system(size: fontSize))
                .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                .foregroundStyle(contentColor)
                .padding(.horizontal, CGFloat(12))
                .frame(maxWidth: .infinity, minHeight: doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0)), alignment: .leading)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
                .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(validationError == nil ? (borderColor ?? Color.clear) : DoweDesign.danger, lineWidth: validationError == nil && borderColor == nil ? CGFloat(0) : CGFloat(1)))
            }
            .buttonStyle(.plain)
            .background(DoweAnchoredPopoverPresenter(isPresented: expanded, minWidth: CGFloat(286), maxWidth: CGFloat(340), maxHeight: CGFloat(420), preferredHeight: CGFloat(370), onDismiss: { expanded = false; touched = true }) {
                DoweDateCalendar(month: month, selected: value.wrappedValue, start: "", end: "", min: min, max: max, contentColor: contentColor, accentColor: contentColor, showPrevious: true, showNext: true, onPrevious: { month = doweDateStep(month, amount: -1) }, onNext: { month = doweDateStep(month, amount: 1) }, onSelect: { next in value.wrappedValue = next; month = doweDateMonth(doweParseDate(next) ?? month); expanded = false; touched = true })
            })
            DoweValidationFeedback(helpText: helpText, error: validationError, contentColor: contentColor)
        }
    }
}

struct DoweDateRangeField: View {
    let startValue: Binding<String>
    let endValue: Binding<String>
    let label: String?
    let placeholder: String
    let floating: Bool
    let size: String
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let name: String?
    let helpText: String?
    let errorText: String?
    let min: String?
    let max: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    @State private var expanded = false
    @State private var month: Date
    @State private var selectingEnd = false

    init(startValue: Binding<String>, endValue: Binding<String>, label: String?, placeholder: String, floating: Bool, size: String, fontSize: CGFloat, lineHeight: CGFloat, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
        self.startValue = startValue
        self.endValue = endValue
        self.label = label
        self.placeholder = placeholder
        self.floating = floating
        self.size = size
        self.fontSize = fontSize
        self.lineHeight = lineHeight
        self.name = name
        self.helpText = helpText
        self.errorText = errorText
        self.min = min
        self.max = max
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        _month = State(initialValue: doweDateMonth(doweParseDate(startValue.wrappedValue) ?? Date()))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating { Text(label).font(.footnote).fontWeight(.semibold) }
            Button(action: { if expanded { expanded = false } else { selectingEnd = false; expanded = true } }) {
                ZStack(alignment: .leading) {
                    if let label, floating {
                        Text(label)
                            .font(.caption)
                            .offset(y: startValue.wrappedValue.isEmpty && endValue.wrappedValue.isEmpty && !expanded ? CGFloat(0) : CGFloat(-12))
                            .scaleEffect(startValue.wrappedValue.isEmpty && endValue.wrappedValue.isEmpty && !expanded ? CGFloat(1) : CGFloat(0.9), anchor: .leading)
                    }
                    HStack {
                        Text(rangeLabel).lineLimit(1)
                        Spacer()
                        DoweSelectArrow(color: contentColor)
                    }
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
                .font(.system(size: fontSize))
                .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                .foregroundStyle(contentColor)
                .padding(.horizontal, CGFloat(12))
                .frame(maxWidth: .infinity, minHeight: doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0)), alignment: .leading)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
                .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
            }
            .buttonStyle(.plain)
            .background(DoweAnchoredPopoverPresenter(isPresented: expanded, minWidth: CGFloat(600), maxWidth: CGFloat(720), maxHeight: CGFloat(460), preferredHeight: CGFloat(390), onDismiss: { expanded = false }) {
                HStack(spacing: CGFloat(8)) {
                    DoweDateCalendar(month: month, selected: "", start: startValue.wrappedValue, end: endValue.wrappedValue, min: min, max: max, contentColor: contentColor, accentColor: contentColor, showPrevious: true, showNext: false, onPrevious: { month = doweDateStep(month, amount: -1) }, onNext: {}, onSelect: select)
                    DoweDateCalendar(month: doweDateStep(month, amount: 1), selected: "", start: startValue.wrappedValue, end: endValue.wrappedValue, min: min, max: max, contentColor: contentColor, accentColor: contentColor, showPrevious: false, showNext: true, onPrevious: {}, onNext: { month = doweDateStep(month, amount: 1) }, onSelect: select)
                }
            })
            if let message = errorText ?? helpText { Text(message).font(.caption).foregroundStyle(contentColor.opacity(0.7)) }
        }
    }

    private var rangeLabel: String {
        if !startValue.wrappedValue.isEmpty && !endValue.wrappedValue.isEmpty { return "\(doweDateLabel(startValue.wrappedValue)) – \(doweDateLabel(endValue.wrappedValue))" }
        if !startValue.wrappedValue.isEmpty { return "\(doweDateLabel(startValue.wrappedValue)) – …" }
        return placeholder
    }

    private func select(_ next: String) {
        guard doweDateAllowed(next, min: min, max: max) else { return }
        if !selectingEnd {
            startValue.wrappedValue = next
            endValue.wrappedValue = ""
            selectingEnd = true
            month = doweDateMonth(doweParseDate(next) ?? month)
        } else {
            if next < startValue.wrappedValue { endValue.wrappedValue = startValue.wrappedValue; startValue.wrappedValue = next } else { endValue.wrappedValue = next }
            selectingEnd = false
            expanded = false
        }
    }
}

"##
}
