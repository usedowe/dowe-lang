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
"#
