fn swift_runtime_input_select_runtime() -> &'static str {
    r#"struct DoweInputField: View {
    let value: Binding<String>?
    let label: String?
    let placeholder: String
    let floating: Bool
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let minHeight: CGFloat
    let horizontalPadding: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var localText = ""
    @FocusState private var focused: Bool

    private var currentText: String {
        value?.wrappedValue ?? localText
    }

    private var textBinding: Binding<String> {
        Binding(
            get: { value?.wrappedValue ?? localText },
            set: { next in
                if let value {
                    value.wrappedValue = next
                } else {
                    localText = next
                }
            }
        )
    }

    private var active: Bool {
        focused || !currentText.isEmpty
    }

    private var visiblePlaceholder: String {
        if placeholder.isEmpty || (floating && !active) {
            return ""
        }
        return placeholder
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label, !floating {
                Text(label)
                    .font(.footnote)
                    .fontWeight(.semibold)
            }
            ZStack(alignment: .leading) {
                if let label, floating {
                    Text(label)
                        .font(.caption)
                        .offset(y: active ? CGFloat(-12) : CGFloat(0))
                        .scaleEffect(active ? CGFloat(0.9) : CGFloat(1), anchor: .leading)
                }
                TextField(visiblePlaceholder, text: textBinding)
                    .focused($focused)
                    .textFieldStyle(.plain)
                    .tint(contentColor)
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
            }
            .font(font)
            .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
            .foregroundStyle(contentColor)
            .padding(.horizontal, horizontalPadding)
            .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(
                RoundedRectangle(cornerRadius: radius)
                    .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
            )
        }
    }
}

struct DoweSelectOption: Identifiable {
    let value: String
    let label: String
    let description: String?

    var id: String {
        value
    }
}

struct DoweSelectAnchorPresenter: UIViewRepresentable {
    let isPresented: Bool
    let options: [DoweSelectOption]
    let selectedValue: String
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let accentColor: Color
    let radius: CGFloat
    let onSelect: (DoweSelectOption) -> Void
    let onDismiss: () -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView()
        view.backgroundColor = .clear
        view.isUserInteractionEnabled = false
        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.parent = self
        DispatchQueue.main.async {
            if isPresented {
                context.coordinator.show(from: uiView)
            } else {
                context.coordinator.dismiss()
            }
        }
    }

    static func dismantleUIView(_ uiView: UIView, coordinator: Coordinator) {
        coordinator.dismiss()
    }

    final class Coordinator: NSObject {
        var parent: DoweSelectAnchorPresenter
        private var hosting: UIHostingController<DoweSelectPopover>?
        private var backdrop: UIControl?

        init(parent: DoweSelectAnchorPresenter) {
            self.parent = parent
        }

        func show(from anchor: UIView) {
            guard let window = anchor.window else {
                return
            }
            let root = DoweSelectPopover(
                options: parent.options,
                selectedValue: parent.selectedValue,
                font: parent.font,
                fontSize: parent.fontSize,
                lineHeight: parent.lineHeight,
                accentColor: parent.accentColor,
                radius: parent.radius,
                onSelect: { [weak self] option in
                    self?.parent.onSelect(option)
                    self?.dismiss()
                }
            )
            let controller = hosting ?? UIHostingController(rootView: root)
            controller.rootView = root
            controller.view.backgroundColor = .clear
            hosting = controller
            let shield = backdrop ?? UIControl(frame: window.bounds)
            shield.frame = window.bounds
            shield.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            shield.backgroundColor = .clear
            if backdrop == nil {
                shield.addTarget(self, action: #selector(handleBackdrop), for: .touchUpInside)
                backdrop = shield
            }
            if shield.superview == nil {
                window.addSubview(shield)
            }
            if controller.view.superview == nil {
                window.addSubview(controller.view)
                controller.view.alpha = 0
                controller.view.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            }
            controller.view.frame = frame(for: anchor, in: window)
            UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                controller.view.alpha = 1
                controller.view.transform = .identity
            }
        }

        func dismiss() {
            guard let view = hosting?.view else {
                backdrop?.removeFromSuperview()
                return
            }
            UIView.animate(withDuration: 0.14, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                view.alpha = 0
                view.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            } completion: { _ in
                view.removeFromSuperview()
                self.backdrop?.removeFromSuperview()
            }
        }

        @objc private func handleBackdrop() {
            parent.onDismiss()
            dismiss()
        }

        private func frame(for anchor: UIView, in window: UIWindow) -> CGRect {
            let anchorFrame = anchor.convert(anchor.bounds, to: window)
            let safeFrame = window.safeAreaLayoutGuide.layoutFrame
            let width = min(max(anchorFrame.width, CGFloat(220)), max(CGFloat(220), safeFrame.width - CGFloat(32)))
            let maxHeight = min(CGFloat(260), max(CGFloat(44), safeFrame.height - CGFloat(32)))
            let contentHeight = CGFloat(8) + parent.options.reduce(CGFloat(0)) { total, option in
                total + (option.description == nil ? CGFloat(40) : CGFloat(58))
            }
            let height = min(maxHeight, max(CGFloat(44), contentHeight))
            let x = min(max(anchorFrame.minX, safeFrame.minX + CGFloat(16)), max(safeFrame.minX + CGFloat(16), safeFrame.maxX - width - CGFloat(16)))
            let below = anchorFrame.maxY + CGFloat(4)
            let y = below + height <= safeFrame.maxY - CGFloat(16)
                ? below
                : max(safeFrame.minY + CGFloat(16), anchorFrame.minY - height - CGFloat(4))
            return CGRect(x: x, y: y, width: width, height: height)
        }
    }
}

struct DoweSelectField: View {
    let value: Binding<String>?
    let label: String?
    let placeholder: String
    let floating: Bool
    let options: [DoweSelectOption]
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let minHeight: CGFloat
    let horizontalPadding: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var localValue = ""
    @State private var expanded = false

    private var selectedValue: String {
        value?.wrappedValue ?? localValue
    }

    private var selectedOption: DoweSelectOption? {
        options.first { $0.value == selectedValue }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label, !floating {
                Text(label)
                    .font(.footnote)
                    .fontWeight(.semibold)
            }
            Button(action: togglePopover) {
                ZStack(alignment: .leading) {
                    if let label, floating {
                        Text(label)
                            .font(.caption)
                            .offset(y: selectedOption == nil && !expanded ? CGFloat(0) : CGFloat(-12))
                            .scaleEffect(selectedOption == nil && !expanded ? CGFloat(1) : CGFloat(0.9), anchor: .leading)
                    }
                    HStack {
                        if selectedOption != nil || !floating || expanded {
                            Text(selectedOption?.label ?? placeholder)
                                .lineLimit(1)
                        }
                        Spacer()
                        DoweSelectArrow(color: contentColor)
                    }
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
                .font(font)
                .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                .foregroundStyle(contentColor)
                .padding(.horizontal, horizontalPadding)
                .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: radius))
                .overlay(
                    RoundedRectangle(cornerRadius: radius)
                        .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
                )
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .background(
                DoweSelectAnchorPresenter(
                    isPresented: expanded,
                    options: options,
                    selectedValue: selectedValue,
                    font: font,
                    fontSize: fontSize,
                    lineHeight: lineHeight,
                    accentColor: contentColor,
                    radius: radius,
                    onSelect: { option in
                        setValue(option.value)
                        expanded = false
                    },
                    onDismiss: {
                        expanded = false
                    }
                )
            )
        }
        .zIndex(expanded ? 1000 : 0)
        .onDisappear {
            if expanded {
                expanded = false
            }
        }
    }

    private func setValue(_ next: String) {
        if let value {
            value.wrappedValue = next
        } else {
            localValue = next
        }
    }

    private func togglePopover() {
        if expanded {
            expanded = false
        } else {
            expanded = true
        }
    }
}

struct DoweSelectPopover: View {
    let options: [DoweSelectOption]
    let selectedValue: String
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let accentColor: Color
    let radius: CGFloat
    let onSelect: (DoweSelectOption) -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: CGFloat(0)) {
                ForEach(options) { option in
                    Button(action: { onSelect(option) }) {
                        VStack(alignment: .leading, spacing: CGFloat(3)) {
                            Text(option.label)
                                .fontWeight(.semibold)
                            if let description = option.description {
                                Text(description).font(.caption)
                                    .foregroundStyle(DoweDesign.onSurface.opacity(0.68))
                            }
                        }
                        .font(font)
                        .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                        .foregroundStyle(DoweDesign.onSurface)
                        .padding(.horizontal, CGFloat(16))
                        .padding(.vertical, CGFloat(10))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(option.value == selectedValue ? accentColor.opacity(0.08) : Color.clear)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.vertical, CGFloat(4))
        }
        .frame(minWidth: CGFloat(220), maxWidth: .infinity, alignment: .leading)
        .background(DoweDesign.surface)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(RoundedRectangle(cornerRadius: radius).stroke(DoweDesign.onSurface.opacity(0.08), lineWidth: CGFloat(1)))
        .shadow(color: Color.black.opacity(0.12), radius: CGFloat(16), x: CGFloat(0), y: CGFloat(8))
    }
}

struct DoweSelectArrow: View {
    let color: Color

    var body: some View {
        DoweSvgView(
            viewBox: DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24)),
            color: color,
            paths: [
                DoweSvgPathData(data: "M0 0h24v24H0z", fill: .none),
                DoweSvgPathData(data: "M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z", fill: .currentColor)
            ]
        )
        .frame(width: CGFloat(16), height: CGFloat(16))
    }
}

"#
}
