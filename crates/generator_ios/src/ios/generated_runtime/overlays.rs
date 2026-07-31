fn swift_runtime_overlays() -> &'static str {
    r#"struct DoweWindowOverlayPresenter: UIViewRepresentable {
    let isPresented: Bool
    let content: AnyView

    init<Content: View>(isPresented: Bool, @ViewBuilder content: () -> Content) {
        self.isPresented = isPresented
        self.content = AnyView(content())
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView(frame: .zero)
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
        coordinator.dismiss(immediate: true)
    }

    final class Coordinator {
        var parent: DoweWindowOverlayPresenter
        private var hosting: UIHostingController<AnyView>?

        init(parent: DoweWindowOverlayPresenter) {
            self.parent = parent
        }

        func show(from anchor: UIView) {
            guard let window = anchor.window else {
                return
            }
            let controller = hosting ?? UIHostingController(rootView: parent.content)
            controller.rootView = parent.content
            controller.view.backgroundColor = .clear
            controller.view.frame = window.bounds
            controller.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            hosting = controller
            if controller.view.superview == nil {
                controller.view.alpha = 0
                window.addSubview(controller.view)
                UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                    controller.view.alpha = 1
                }
            }
        }

        func dismiss(immediate: Bool = false) {
            guard let view = hosting?.view, view.superview != nil else {
                return
            }
            let remove = {
                view.removeFromSuperview()
            }
            if immediate {
                remove()
            } else {
                UIView.animate(withDuration: 0.12, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                    view.alpha = 0
                } completion: { _ in
                    remove()
                }
            }
        }
    }
}

struct DoweToastOverlayPresenter<Content: View>: UIViewRepresentable {
    let isPresented: Bool
    let position: String
    let content: Content

    init(isPresented: Bool, position: String, @ViewBuilder content: () -> Content) {
        self.isPresented = isPresented
        self.position = position
        self.content = content()
    }

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
        if isPresented {
            context.coordinator.scheduleShow(from: uiView)
        } else {
            context.coordinator.dismiss()
        }
    }

    static func dismantleUIView(_ uiView: UIView, coordinator: Coordinator) {
        coordinator.dismiss(immediate: true)
    }

    @MainActor final class Coordinator: NSObject {
        var parent: DoweToastOverlayPresenter
        private var hosting: UIHostingController<Content>?
        private var containerView: UIView?
        private var presentationRevision = 0
        private var showScheduled = false
        private var isDismissing = false

        init(parent: DoweToastOverlayPresenter) {
            self.parent = parent
        }

        func scheduleShow(from anchor: UIView) {
            if containerView?.superview != nil {
                showScheduled = false
                isDismissing = false
                show(from: anchor)
                return
            }
            guard !showScheduled else {
                return
            }
            showScheduled = true
            presentationRevision += 1
            let revision = presentationRevision
            DispatchQueue.main.async {
                guard revision == self.presentationRevision else {
                    return
                }
                self.showScheduled = false
                guard self.parent.isPresented else {
                    return
                }
                self.show(from: anchor)
            }
        }

        func show(from anchor: UIView) {
            guard let window = anchor.window else {
                return
            }
            window.layoutIfNeeded()
            let controller = hosting ?? UIHostingController(rootView: parent.content)
            controller.rootView = parent.content
            controller.view.backgroundColor = .clear
            let safeArea = window.safeAreaLayoutGuide.layoutFrame
            let availableWidth = max(CGFloat(0), safeArea.width - CGFloat(32))
            let targetWidth = min(CGFloat(420), max(CGFloat(1), availableWidth))
            let measured = controller.sizeThatFits(
                in: CGSize(width: targetWidth, height: UIView.layoutFittingExpandedSize.height)
            )
            let width = max(CGFloat(1), min(targetWidth, measured.width))
            let height = max(CGFloat(1), measured.height)
            let x = parent.position.hasSuffix("right")
                ? safeArea.maxX - width - CGFloat(16)
                : safeArea.minX + CGFloat(16)
            let y = parent.position.hasPrefix("top")
                ? safeArea.minY + CGFloat(16)
                : safeArea.maxY - height - CGFloat(16)
            let frame = CGRect(x: x, y: y, width: width, height: height)
            hosting = controller
            let container = containerView ?? UIView()
            if containerView == nil {
                container.backgroundColor = .clear
                containerView = container
            }
            if controller.view.superview !== container {
                controller.view.removeFromSuperview()
                container.addSubview(controller.view)
            }
            controller.view.frame = CGRect(origin: .zero, size: frame.size)
            container.bounds = CGRect(origin: .zero, size: frame.size)
            container.center = CGPoint(x: frame.midX, y: frame.midY)
            let animateIn = container.superview == nil
            if animateIn {
                container.alpha = CGFloat(0)
                container.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
                window.addSubview(container)
            }
            isDismissing = false
            container.layer.removeAllAnimations()
            if animateIn {
                UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                    container.alpha = CGFloat(1)
                    container.transform = .identity
                }
            } else {
                container.alpha = CGFloat(1)
                container.transform = .identity
            }
        }

        func dismiss(immediate: Bool = false) {
            guard immediate || !isDismissing else {
                return
            }
            guard immediate || showScheduled || containerView?.superview != nil else {
                return
            }
            presentationRevision += 1
            let revision = presentationRevision
            showScheduled = false
            if immediate {
                isDismissing = false
                containerView?.removeFromSuperview()
                return
            }
            guard let container = containerView, container.superview != nil else {
                return
            }
            isDismissing = true
            container.layer.removeAllAnimations()
            UIView.animate(withDuration: 0.12, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                container.alpha = CGFloat(0)
                container.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            } completion: { _ in
                guard revision == self.presentationRevision, !self.parent.isPresented, self.isDismissing else {
                    return
                }
                self.isDismissing = false
                container.removeFromSuperview()
            }
        }
    }
}

struct DoweModal<Header: View, Content: View, Footer: View>: View {
    let open: Bool
    let close: () -> Void
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let hasHeader: Bool
    let hasFooter: Bool
    let header: Header
    let content: Content
    let footer: Footer

    init(open: Bool, close: @escaping () -> Void, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: CGFloat, disableOverlayClose: Bool, hideCloseButton: Bool, hasHeader: Bool, hasFooter: Bool, @ViewBuilder header: () -> Header, @ViewBuilder content: () -> Content, @ViewBuilder footer: () -> Footer) {
        self.open = open
        self.close = close
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.radius = radius
        self.disableOverlayClose = disableOverlayClose
        self.hideCloseButton = hideCloseButton
        self.hasHeader = hasHeader
        self.hasFooter = hasFooter
        self.header = header()
        self.content = content()
        self.footer = footer()
    }

    var body: some View {
        DoweWindowOverlayPresenter(isPresented: open) {
            modalLayer
        }
        .frame(width: CGFloat(0), height: CGFloat(0))
        .allowsHitTesting(false)
    }

    private var modalLayer: some View {
        GeometryReader { geometry in
            let modalWidth = geometry.size.width * 0.95
            ZStack {
                Color.black.opacity(0.48)
                    .ignoresSafeArea()
                    .contentShape(Rectangle())
                    .onTapGesture {
                        if !disableOverlayClose {
                            close()
                        }
                    }
                VStack(alignment: .leading, spacing: CGFloat(16)) {
                    if hasHeader { header }
                    content
                    if hasFooter { footer }
                }
                .padding(CGFloat(20))
                .frame(maxWidth: modalWidth, alignment: .leading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(RoundedRectangle(cornerRadius: radius))
                .overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
                .overlay(alignment: .topTrailing) {
                    if !hideCloseButton {
                        Button(action: close) {
                            DoweOverlayCloseIcon(color: DoweDesign.onSoftMuted)
                                .frame(width: CGFloat(28), height: CGFloat(28))
                                .background(DoweDesign.softMuted)
                                .foregroundStyle(DoweDesign.onSoftMuted)
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
                DoweOverlayCloseIcon(color: DoweDesign.onSoftMuted)
                    .frame(width: CGFloat(28), height: CGFloat(28))
                    .background(DoweDesign.softMuted)
                    .foregroundStyle(DoweDesign.onSoftMuted)
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
                open = false
            }
        }
    }
}

struct DoweDropdownPopover<Content: View>: View {
    let backgroundColor: Color
    let contentColor: Color
    let content: Content

    init(backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(4)) {
            content
        }
        .padding(CGFloat(8))
        .frame(minWidth: CGFloat(220), maxWidth: CGFloat(360), alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(contentColor.opacity(0.08), lineWidth: CGFloat(1)))
    }
}

struct DoweOverlayItem<Icon: View>: View {
    let label: String
    let description: String?
    let disabled: Bool
    let backgroundColor: Color
    let contentColor: Color
    let action: (() -> Void)?
    let icon: Icon

    init(label: String, description: String?, disabled: Bool, backgroundColor: Color, contentColor: Color, action: (() -> Void)?, @ViewBuilder icon: () -> Icon) {
        self.label = label
        self.description = description
        self.disabled = disabled
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.action = action
        self.icon = icon()
    }

    var body: some View {
        Button(action: { action?() }) {
            HStack(spacing: CGFloat(10)) {
                icon
                VStack(alignment: .leading, spacing: CGFloat(2)) {
                    Text(label).fontWeight(.medium)
                    if let description {
                        Text(description).font(.caption).opacity(0.68)
                    }
                }
                Spacer()
            }
            .padding(.horizontal, CGFloat(12))
            .padding(.vertical, CGFloat(8))
            .background(backgroundColor.opacity(action == nil ? 0 : 0.08))
            .foregroundStyle(contentColor.opacity(disabled ? 0.48 : 1))
            .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        }
        .disabled(disabled || action == nil)
        .buttonStyle(.plain)
    }
}

struct DoweCommand<Content: View>: View {
    let open: Bool
    let close: () -> Void
    let placeholder: String
    let emptyText: String
    let closeText: String
    let navigateText: String
    let selectText: String
    let toggleText: String
    let shortcut: String
    let showFooter: Bool
    let backgroundColor: Color
    let contentColor: Color
    let accentColor: Color
    let content: Content

    init(open: Bool, close: @escaping () -> Void, placeholder: String, emptyText: String, closeText: String, navigateText: String, selectText: String, toggleText: String, shortcut: String, showFooter: Bool, backgroundColor: Color, contentColor: Color, accentColor: Color, @ViewBuilder content: () -> Content) {
        self.open = open
        self.close = close
        self.placeholder = placeholder
        self.emptyText = emptyText
        self.closeText = closeText
        self.navigateText = navigateText
        self.selectText = selectText
        self.toggleText = toggleText
        self.shortcut = shortcut
        self.showFooter = showFooter
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.accentColor = accentColor
        self.content = content()
    }

    var body: some View {
        DoweWindowOverlayPresenter(isPresented: open) {
            ZStack(alignment: .top) {
                Color.black.opacity(0.48)
                    .ignoresSafeArea()
                    .onTapGesture(perform: close)
                VStack(alignment: .leading, spacing: CGFloat(10)) {
                    Text(placeholder).opacity(0.56)
                    Divider()
                    content
                    if showFooter {
                        HStack {
                            Text("Esc \(closeText)")
                            Spacer()
                            Text("Ctrl+\(shortcut.uppercased()) \(toggleText)")
                                .foregroundStyle(accentColor)
                                .fontWeight(.semibold)
                        }
                        .font(.caption)
                        .opacity(0.72)
                    }
                }
                .padding(CGFloat(12))
                .frame(minWidth: CGFloat(320), maxWidth: CGFloat(560), alignment: .leading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
                .padding(.top, CGFloat(64))
            }
        }
        .frame(width: CGFloat(0), height: CGFloat(0))
        .allowsHitTesting(false)
    }
}

"#
}
