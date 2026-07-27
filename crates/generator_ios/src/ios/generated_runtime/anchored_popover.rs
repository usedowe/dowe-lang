fn swift_runtime_anchored_popover() -> &'static str {
    r#"struct DoweAnchoredPopoverPresenter<Content: View>: UIViewRepresentable {
    let isPresented: Bool
    let minWidth: CGFloat
    let maxWidth: CGFloat
    let maxHeight: CGFloat
    let preferredHeight: CGFloat?
    let content: Content
    let onDismiss: () -> Void

    init(isPresented: Bool, minWidth: CGFloat = CGFloat(220), maxWidth: CGFloat = CGFloat(360), maxHeight: CGFloat = CGFloat(260), preferredHeight: CGFloat? = nil, onDismiss: @escaping () -> Void, @ViewBuilder content: () -> Content) {
        self.isPresented = isPresented
        self.minWidth = minWidth
        self.maxWidth = maxWidth
        self.maxHeight = maxHeight
        self.preferredHeight = preferredHeight
        self.onDismiss = onDismiss
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
        coordinator.dismiss(immediately: true)
    }

    @MainActor final class Coordinator: NSObject {
        var parent: DoweAnchoredPopoverPresenter
        private var hosting: UIHostingController<Content>?
        private var containerView: UIView?
        private var scrollView: UIScrollView?
        private var backdrop: UIControl?
        private var presentationRevision = 0
        private weak var trackedAnchor: UIView?
        private var anchorDisplayLink: CADisplayLink?
        private var trackedWidth = CGFloat(0)
        private var trackedContentHeight = CGFloat(0)
        private var lastLayoutFrame = CGRect.null

        init(parent: DoweAnchoredPopoverPresenter) {
            self.parent = parent
        }

        func scheduleShow(from anchor: UIView) {
            presentationRevision += 1
            let revision = presentationRevision
            DispatchQueue.main.async {
                guard revision == self.presentationRevision, self.parent.isPresented else {
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
            anchor.superview?.layoutIfNeeded()
            anchor.layoutIfNeeded()
            let controller = hosting ?? UIHostingController(rootView: parent.content)
            controller.rootView = parent.content
            controller.view.backgroundColor = .clear
            hosting = controller
            let container = containerView ?? UIView()
            if containerView == nil {
                container.backgroundColor = .clear
                container.layer.shadowColor = UIColor.black.cgColor
                container.layer.shadowOpacity = Float(0.12)
                container.layer.shadowRadius = CGFloat(16)
                container.layer.shadowOffset = CGSize(width: CGFloat(0), height: CGFloat(8))
                containerView = container
            }
            let scroller = scrollView ?? UIScrollView()
            if scrollView == nil {
                scroller.backgroundColor = .clear
                scroller.contentInsetAdjustmentBehavior = .never
                scroller.alwaysBounceHorizontal = false
                scroller.clipsToBounds = true
                scrollView = scroller
            }
            let shield = backdrop ?? UIControl(frame: window.bounds)
            shield.frame = window.bounds
            shield.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            shield.backgroundColor = .clear
            if backdrop == nil {
                shield.addTarget(self, action: #selector(handleBackdrop), for: .touchUpInside)
                backdrop = shield
            }
            let needsMount = container.superview == nil
            if scroller.superview !== container {
                scroller.removeFromSuperview()
                container.addSubview(scroller)
            }
            if controller.view.superview !== scroller {
                controller.view.removeFromSuperview()
                scroller.addSubview(controller.view)
            }
            let measured = layout(for: anchor, controller: controller, in: window)
            trackedWidth = measured.frame.width
            trackedContentHeight = measured.contentHeight
            trackedAnchor = anchor
            lastLayoutFrame = .null
            applyTrackedLayout(for: anchor, in: window, resetScroll: true)
            if shield.superview == nil {
                window.addSubview(shield)
            }
            if needsMount {
                container.alpha = CGFloat(0)
                container.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
                window.addSubview(container)
            }
            startTracking(anchor: anchor)
            container.layer.removeAllAnimations()
            UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                container.alpha = CGFloat(1)
                container.transform = .identity
            }
        }

        func dismiss(immediately: Bool = false) {
            presentationRevision += 1
            stopTracking()
            guard let container = containerView else {
                backdrop?.removeFromSuperview()
                return
            }
            container.layer.removeAllAnimations()
            if immediately {
                container.removeFromSuperview()
                backdrop?.removeFromSuperview()
                return
            }
            UIView.animate(withDuration: 0.14, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                container.alpha = CGFloat(0)
                container.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            } completion: { _ in
                if !self.parent.isPresented {
                    container.removeFromSuperview()
                    self.backdrop?.removeFromSuperview()
                }
            }
        }

        @objc private func handleBackdrop() {
            parent.onDismiss()
            dismiss()
        }

        private func startTracking(anchor: UIView) {
            trackedAnchor = anchor
            guard anchorDisplayLink == nil else {
                return
            }
            let displayLink = CADisplayLink(target: self, selector: #selector(refreshAnchorPosition))
            displayLink.add(to: .main, forMode: .common)
            anchorDisplayLink = displayLink
        }

        private func stopTracking() {
            anchorDisplayLink?.invalidate()
            anchorDisplayLink = nil
            trackedAnchor = nil
            lastLayoutFrame = .null
        }

        @objc private func refreshAnchorPosition() {
            guard parent.isPresented, let anchor = trackedAnchor, let window = anchor.window else {
                stopTracking()
                return
            }
            applyTrackedLayout(for: anchor, in: window, resetScroll: false)
        }

        private func applyTrackedLayout(for anchor: UIView, in window: UIWindow, resetScroll: Bool) {
            guard let controller = hosting, let container = containerView, let scroller = scrollView else {
                return
            }
            let frame = trackedFrame(for: anchor, in: window)
            if !frame.equalTo(lastLayoutFrame) {
                controller.view.frame = CGRect(x: CGFloat(0), y: CGFloat(0), width: trackedWidth, height: trackedContentHeight)
                container.bounds = CGRect(origin: .zero, size: frame.size)
                container.center = CGPoint(x: frame.midX, y: frame.midY)
                container.layer.shadowPath = UIBezierPath(roundedRect: container.bounds, cornerRadius: DoweDesign.radius).cgPath
                scroller.frame = container.bounds
                scroller.contentSize = CGSize(width: trackedWidth, height: trackedContentHeight)
                scroller.isScrollEnabled = trackedContentHeight > frame.height
                scroller.alwaysBounceVertical = scroller.isScrollEnabled
                scroller.showsVerticalScrollIndicator = scroller.isScrollEnabled
                lastLayoutFrame = frame
            }
            if resetScroll {
                scroller.setContentOffset(.zero, animated: false)
            }
        }

        private func trackedFrame(for anchor: UIView, in window: UIWindow) -> CGRect {
            popoverFrame(
                for: anchor.convert(anchor.bounds, to: window),
                width: trackedWidth,
                contentHeight: trackedContentHeight,
                safeFrame: window.safeAreaLayoutGuide.layoutFrame
            )
        }

        private func layout(for anchor: UIView, controller: UIHostingController<Content>, in window: UIWindow) -> (frame: CGRect, contentHeight: CGFloat) {
            let anchorFrame = anchor.convert(anchor.bounds, to: window)
            let safeFrame = window.safeAreaLayoutGuide.layoutFrame
            let availableWidth = max(parent.minWidth, safeFrame.width - CGFloat(32))
            let width = min(max(anchorFrame.width, parent.minWidth), min(parent.maxWidth, availableWidth))
            let measuredSize = controller.sizeThatFits(in: CGSize(width: width, height: UIView.layoutFittingExpandedSize.height))
            let measuredHeight = parent.preferredHeight ?? measuredSize.height
            let contentHeight = max(CGFloat(44), measuredHeight)
            return (popoverFrame(for: anchorFrame, width: width, contentHeight: contentHeight, safeFrame: safeFrame), contentHeight)
        }

        private func popoverFrame(for anchorFrame: CGRect, width: CGFloat, contentHeight: CGFloat, safeFrame: CGRect) -> CGRect {
            let availableHeight = max(CGFloat(44), safeFrame.height - CGFloat(32))
            let heightLimit = min(parent.maxHeight, availableHeight)
            let height = min(heightLimit, contentHeight)
            let minimumX = safeFrame.minX + CGFloat(16)
            let maximumX = max(minimumX, safeFrame.maxX - width - CGFloat(16))
            let x = min(max(anchorFrame.minX, minimumX), maximumX)
            let below = anchorFrame.maxY + CGFloat(4)
            let y = below + height <= safeFrame.maxY - CGFloat(16)
                ? below
                : max(safeFrame.minY + CGFloat(16), anchorFrame.minY - height - CGFloat(4))
            return CGRect(x: x, y: y, width: width, height: height)
        }
    }
}

"#
}
