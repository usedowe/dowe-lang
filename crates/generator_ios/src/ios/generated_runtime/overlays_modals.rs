r#"                .frame(maxWidth: modalWidth, alignment: .leading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(RoundedRectangle(cornerRadius: radius))
                .overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
                .overlay(alignment: .topTrailing) {
                    if !hideCloseButton {
                        Button(action: close) {
                            DoweOverlayCloseIcon(color: DoweDesign.mutedText)
                                .frame(width: CGFloat(28), height: CGFloat(28))
                                .background(DoweDesign.muted)
                                .foregroundStyle(DoweDesign.mutedText)
                                .clipShape(Circle())
                                .frame(width: CGFloat(44), height: CGFloat(44))
                                .contentShape(Rectangle())
                        }
                        .buttonStyle(.plain)
                        .accessibilityLabel("Close modal")
                    }
                }
            }
            .frame(width: geometry.size.width, height: geometry.size.height, alignment: .center)
            .transition(.opacity)
        }
    }
}

struct DoweAlertDialog: View {
    let open: Bool
    let close: () -> Void
    let title: String
    let description: String
    let confirmText: String
    let cancelText: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let confirmBackgroundColor: Color
    let confirmContentColor: Color
    let radius: CGFloat
    let loading: Bool
    let confirm: (() -> Void)?

    var body: some View {
        DoweModal(open: open, close: close, backgroundColor: backgroundColor, contentColor: contentColor, borderColor: borderColor, radius: radius, disableOverlayClose: true, hideCloseButton: true, hasHeader: true, hasFooter: true) {
            Text(title).font(.headline)
        } content: {
            Text(description).opacity(0.72)
        } footer: {
            HStack(spacing: CGFloat(12)) {
                Spacer()
                Button(cancelText) {
                    close()
                }
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(10))
                .frame(minHeight: CGFloat(40))
                .background(Color.clear)
                .foregroundStyle(DoweDesign.muted)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
                .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))
                .disabled(loading)
                .buttonStyle(.plain)
                Button(confirmText) {
                    confirm?()
                }
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(10))
                .frame(minHeight: CGFloat(40))
                .background(confirmBackgroundColor)
                .foregroundStyle(confirmContentColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
                .disabled(loading)
                .buttonStyle(.plain)
            }
        }
    }
}

struct DoweOverlayCloseIcon: View {
    let color: Color

    var body: some View {
        DoweSvgView(
            viewBox: DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24)),
            color: color,
            paths: [
                DoweSvgPathData(data: "M0 0h24v24H0z", fill: .none),
                DoweSvgPathData(data: "m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z", fill: .currentColor)
            ]
        )
        .frame(width: CGFloat(18), height: CGFloat(18))
    }
}

struct DoweTooltip<Content: View>: View {
    let content: Content

    init(label: String, position: String, backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        content
    }
}

struct DoweToast: View {
    let visible: Bool
    let title: String
    let description: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let showIcon: Bool
    let kind: String
    let close: (() -> Void)?
    @State private var dismissed = false

    var body: some View {
        DoweToastOverlayPresenter(isPresented: visible && !dismissed, position: position) {
            VStack {
                toast
            }
            .padding(CGFloat(16))
        }
        .onChange(of: visible) { _, next in
            if next { dismissed = false }
        }
        .frame(width: CGFloat(0), height: CGFloat(0))
        .allowsHitTesting(false)
    }

    private var toast: some View {
        HStack(spacing: CGFloat(12)) {
            HStack(spacing: CGFloat(12)) {
                if showIcon {
                    Text(icon).fontWeight(.bold)
                }
                VStack(alignment: .leading, spacing: CGFloat(4)) {
                    if !title.isEmpty {
                        Text(title).fontWeight(.semibold)
                    }
                    Text(description).opacity(0.9)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Button {
                dismissed = true
                close?()
            } label: {
                DoweOverlayCloseIcon(color: DoweDesign.mutedText)
                    .frame(width: CGFloat(28), height: CGFloat(28))
                    .background(DoweDesign.muted)
                    .foregroundStyle(DoweDesign.mutedText)
                    .clipShape(Circle())
                    .frame(width: CGFloat(44), height: CGFloat(44))
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Close toast")
        }
        .padding(CGFloat(16))
        .frame(maxWidth: CGFloat(420), alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
    }

    private var icon: String {
        switch kind {
        case "success": return "✓"
        case "warning": return "!"
        case "danger", "error": return "x"
        default: return "i"
        }
    }
}

struct DoweGlobalToast: View {
    let toast: DoweToastState?
    let close: () -> Void

    var body: some View {
        if let toast {
            DoweToast(
                visible: true,
                title: toast.title,
                description: toast.message,
                position: toast.position,
                backgroundColor: doweCardContainer(toast.variant, toast.scheme),
                contentColor: doweCardContent(toast.variant, toast.scheme),
                borderColor: doweCardBorder(toast.variant, toast.scheme),
                showIcon: false,
                kind: toast.kind,
                close: close
            )
            .id(toast)
            .task {
                try? await Task.sleep(nanoseconds: UInt64(toast.duration) * 1_000_000)
                if !Task.isCancelled {
                    close()
                }
            }
        }
    }
}

struct DoweDropdown<Trigger: View, Content: View>: View {
    let backgroundColor: Color
    let contentColor: Color
    let trigger: Trigger
    let content: (@escaping () -> Void) -> Content
    @State private var open = false

    init(backgroundColor: Color, contentColor: Color, @ViewBuilder trigger: () -> Trigger, @ViewBuilder content: @escaping (@escaping () -> Void) -> Content) {
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.trigger = trigger()
        self.content = content
    }

    var body: some View {
        ZStack {
            trigger
                .allowsHitTesting(false)
        }
        .background(
            DoweAnchoredPopoverPresenter(
                isPresented: open,
                maxHeight: CGFloat(260),
                onDismiss: { open = false }
            ) {
                DoweDropdownPopover(backgroundColor: backgroundColor, contentColor: contentColor) {
                    content { open = false }
                }
            }
        )
        .overlay {
            Button(action: { open.toggle() }) {
                Color.clear
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
        }
        .zIndex(open ? 1000 : 0)
        .onDisappear {
            if open {
"#
