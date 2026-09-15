fn swift_runtime_carousel() -> &'static str {
    r##"struct DoweCarouselGeometry {
    let contentGap: CGFloat
    let viewportPadding: CGFloat
    let verticalViewportHeight: CGFloat
    let slideFraction: CGFloat
    let slideMaxWidth: Int?
}

struct DoweCarouselControlContract {
    let navigationSize: CGFloat
    let navigationInset: CGFloat
    let controlSize: CGFloat
    let controlGap: CGFloat
    let indicatorGap: CGFloat
    let indicatorHeight: CGFloat
    let indicatorInactiveWidth: CGFloat
    let indicatorActiveWidth: CGFloat
    let indicatorDotSize: CGFloat
    let indicatorDotActiveScale: CGFloat
}

struct DoweCarouselView<Content: View>: View {
    let variant: String
    let snap: Bool
    let geometry: DoweCarouselGeometry
    let control: DoweCarouselControlContract
    let slideIds: [String]
    let autoplay: Bool
    let autoplayInterval: Int
    let disableLoop: Bool
    let hideControls: Bool
    let hideIndicators: Bool
    let showNavigation: Bool
    let showCounter: Bool
    let orientation: String
    let size: String
    let indicatorType: String
    let title: String?
    let slideWidth: Int?
    let slideHeight: Int?
    let slidesPerView: Int
    let gap: Int
    let accentColor: Color
    @ViewBuilder var content: Content
    @State private var currentIndex = 0
    @State private var scrollId: String?
    @State private var userInteracting = false

    init(variant: String, snap: Bool, control: DoweCarouselControlContract, geometry: DoweCarouselGeometry, slideIds: [String], autoplay: Bool, autoplayInterval: Int, disableLoop: Bool, hideControls: Bool, hideIndicators: Bool, showNavigation: Bool, showCounter: Bool, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, accentColor: Color, @ViewBuilder content: () -> Content) {
        self.variant = variant
        self.snap = snap
        self.control = control
        self.geometry = geometry
        self.slideIds = slideIds
        self.autoplay = autoplay
        self.autoplayInterval = autoplayInterval
        self.disableLoop = disableLoop
        self.hideControls = hideControls
        self.hideIndicators = hideIndicators
        self.showNavigation = showNavigation
        self.showCounter = showCounter
        self.orientation = orientation
        self.size = size
        self.indicatorType = indicatorType
        self.title = title
        self.slideWidth = slideWidth
        self.slideHeight = slideHeight
        self.slidesPerView = slidesPerView
        self.gap = gap
        self.accentColor = accentColor
        self.content = content()
    }

    var body: some View {
        let previousIconName = orientation == "vertical" ? "chevron.up" : "chevron.left"
        let nextIconName = orientation == "vertical" ? "chevron.down" : "chevron.right"
        VStack(alignment: .leading, spacing: geometry.contentGap) {
            if let title {
                Text(title).font(.system(size: CGFloat(24), weight: .bold)).foregroundStyle(accentColor)
            }
            ZStack(alignment: .center) {
                if orientation == "vertical" {
                    Group {
                        if shouldSnap {
                            ScrollView(.vertical, showsIndicators: false) {
                                LazyVStack(spacing: CGFloat(gap)) { content }
                                    .scrollTargetLayout()
                            }
                            .scrollTargetBehavior(.viewAligned)
                        } else {
                            ScrollView(.vertical, showsIndicators: false) {
                                LazyVStack(spacing: CGFloat(gap)) { content }
                                    .scrollTargetLayout()
                            }
                        }
                    }
                    .padding(geometry.viewportPadding)
                    .scrollPosition(id: $scrollId)
                    .frame(height: geometry.verticalViewportHeight)
                    .simultaneousGesture(carouselDragGesture)
                } else {
                    Group {
                        if shouldSnap {
                            ScrollView(.horizontal, showsIndicators: false) {
                                LazyHStack(spacing: CGFloat(gap)) { content }
                                    .scrollTargetLayout()
                            }
                            .scrollTargetBehavior(.viewAligned)
                        } else {
                            ScrollView(.horizontal, showsIndicators: false) {
                                LazyHStack(spacing: CGFloat(gap)) { content }
                                    .scrollTargetLayout()
                            }
                        }
                    }
                    .padding(geometry.viewportPadding)
                    .scrollPosition(id: $scrollId)
                    .environment(\.layoutDirection, variant == "rtl" ? .rightToLeft : .leftToRight)
                    .simultaneousGesture(carouselDragGesture)
                }
                if showNavigation {
                    if orientation == "vertical" {
                        VStack {
                            DoweIconButton(enabled: !disableLoop || currentIndex > 0, dimension: control.navigationSize, backgroundColor: DoweDesign.surface, contentColor: accentColor, borderColor: accentColor.opacity(0.24), label: "Previous slide", action: { move(-1) }) { Image(systemName: previousIconName) }
                            Spacer()
                            DoweIconButton(enabled: !disableLoop || currentIndex < slideIds.count - 1, dimension: control.navigationSize, backgroundColor: DoweDesign.surface, contentColor: accentColor, borderColor: accentColor.opacity(0.24), label: "Next slide", action: { move(1) }) { Image(systemName: nextIconName) }
                        }
                        .frame(maxHeight: .infinity)
                        .padding(.vertical, control.navigationInset)
                    } else {
                        HStack {
                            DoweIconButton(enabled: !disableLoop || currentIndex > 0, dimension: control.navigationSize, backgroundColor: DoweDesign.surface, contentColor: accentColor, borderColor: accentColor.opacity(0.24), label: "Previous slide", action: { move(-1) }) { Image(systemName: previousIconName) }
                            Spacer()
                            DoweIconButton(enabled: !disableLoop || currentIndex < slideIds.count - 1, dimension: control.navigationSize, backgroundColor: DoweDesign.surface, contentColor: accentColor, borderColor: accentColor.opacity(0.24), label: "Next slide", action: { move(1) }) { Image(systemName: nextIconName) }
                        }
                        .padding(.horizontal, control.navigationInset)
                    }
                }
            }
            if !hideControls || variant == "controls" || !hideIndicators || variant == "dots" || variant == "thumbnails" || showCounter {
                HStack(spacing: control.controlGap) {
                    if !hideControls || variant == "controls" {
                        DoweIconButton(enabled: !disableLoop || currentIndex > 0, dimension: control.controlSize, backgroundColor: DoweDesign.surface, contentColor: accentColor, borderColor: accentColor.opacity(0.24), label: "Previous slide", action: { move(-1) }) { Image(systemName: previousIconName) }
                    }
                    if !hideIndicators || variant == "dots" || variant == "thumbnails" {
                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: control.indicatorGap) {
                                ForEach(Array(slideIds.enumerated()), id: \.offset) { index, id in
                                    if variant == "thumbnails" {
                                        Button {
                                            currentIndex = index
                                            withAnimation { scrollId = id }
                                        } label: {
                                            Text(id)
                                                .font(.footnote)
                                                .foregroundStyle(index == currentIndex ? accentColor : accentColor.opacity(0.45))
                                        }
                                        .buttonStyle(.plain)
                                        .accessibilityLabel(Text(id))
                                    } else {
                                        DowePaginationIndicator(active: index == currentIndex, dot: indicatorType == "dot" || variant == "dots", enabled: true, size: size, color: accentColor, label: "Go to slide \(index + 1)", action: { currentIndex = index; withAnimation { scrollId = id } }, inactiveWidth: control.indicatorInactiveWidth, activeWidth: control.indicatorActiveWidth, height: control.indicatorHeight, dotSize: control.indicatorDotSize, dotActiveScale: control.indicatorDotActiveScale)
                                    }
                                }
                            }
                        }
                        .frame(maxWidth: indicatorContentWidth)
                    }
                    if showCounter {
                        Text("\(currentIndex + 1) / \(slideIds.count)").foregroundStyle(accentColor).lineLimit(1)
                    }
                    if !hideControls || variant == "controls" {
                        DoweIconButton(enabled: !disableLoop || currentIndex < slideIds.count - 1, dimension: control.controlSize, backgroundColor: DoweDesign.surface, contentColor: accentColor, borderColor: accentColor.opacity(0.24), label: "Next slide", action: { move(1) }) { Image(systemName: nextIconName) }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .center)
            }
        }
        .onAppear {
            if scrollId == nil { scrollId = slideIds.first }
        }
        .onChange(of: scrollId) { _, value in
            guard let value, let index = slideIds.firstIndex(of: value) else { return }
            currentIndex = index
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text(title ?? "Carousel"))
        .accessibilityValue(Text("\(currentIndex + 1) of \(slideIds.count)"))
        .task(id: "\(autoplay)-\(autoplayInterval)-\(disableLoop)") {
            guard autoplay, slideIds.count > 1 else { return }
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: UInt64(max(500, autoplayInterval)) * 1_000_000)
                guard !Task.isCancelled else { return }
                if !userInteracting && !(disableLoop && currentIndex >= slideIds.count - 1) { move(1) }
            }
        }
    }

    private var indicatorContentWidth: CGFloat? {
        if variant == "thumbnails" { return nil }
        let count = CGFloat(slideIds.count)
        let gaps = CGFloat(max(0, slideIds.count - 1)) * control.indicatorGap
        let dot = indicatorType == "dot" || variant == "dots"
        return dot ? count * control.indicatorDotSize + gaps : max(0, count - 1) * control.indicatorInactiveWidth + control.indicatorActiveWidth + gaps
    }

    private var shouldSnap: Bool {
        snap
    }

    private var carouselDragGesture: some Gesture {
        DragGesture(minimumDistance: CGFloat(1))
            .onChanged { _ in userInteracting = true }
            .onEnded { _ in userInteracting = false }
    }

    private func move(_ step: Int) {
        guard !slideIds.isEmpty else { return }
        var next = currentIndex + step
        if next < 0 { next = disableLoop ? 0 : slideIds.count - 1 }
        if next >= slideIds.count { next = disableLoop ? slideIds.count - 1 : 0 }
        currentIndex = next
        withAnimation { scrollId = slideIds[next] }
    }
}

struct DoweCarouselSlideView<Content: View>: View {
    let id: String
    let variant: String
    let index: Int
    let geometry: DoweCarouselGeometry
    let orientation: String
    let slideWidth: Int?
    let slideHeight: Int?
    let slidesPerView: Int
    let gap: Int
    @ViewBuilder var content: Content

    init(id: String, variant: String, index: Int, geometry: DoweCarouselGeometry, orientation: String, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, @ViewBuilder content: () -> Content) {
        self.id = id
        self.variant = variant
        self.index = index
        self.geometry = geometry
        self.orientation = orientation
        self.slideWidth = slideWidth
        self.slideHeight = slideHeight
        self.slidesPerView = slidesPerView
        self.gap = gap
        self.content = content()
    }

    var body: some View {
        sizedContent
            .scrollTransition(.interactive, axis: orientation == "vertical" ? .vertical : .horizontal) { view, phase in
                view
                    .scaleEffect(carouselScale(phase.value))
                    .rotationEffect(.degrees(carouselTilt(phase.value)))
                    .rotation3DEffect(.degrees(carouselRotation(phase.value)), axis: (x: 0, y: 1, z: 0), perspective: 0.72)
                    .offset(x: orientation == "vertical" ? CGFloat(0) : carouselHorizontalOffset(phase.value))
                    .offset(y: orientation == "vertical" && variant == "slideshow" ? CGFloat(phase.value * 24) : carouselOffset(phase.value))
                    .opacity(carouselOpacity(phase.value))
            }
            .id(id)
    }

    @ViewBuilder
    private var sizedContent: some View {
        if orientation == "vertical" {
            content
                .frame(maxWidth: .infinity)
                .frame(height: slideHeight.map { CGFloat($0) })
        } else if let slideWidth {
            content
                .frame(width: CGFloat(slideWidth))
                .frame(height: slideHeight.map { CGFloat($0) })
        } else if let maximum = geometry.slideMaxWidth {
            content
                .containerRelativeFrame(.horizontal) { length, _ in min(length * geometry.slideFraction, CGFloat(maximum)) }
                .frame(height: slideHeight.map { CGFloat($0) })
        } else {
            content
                .containerRelativeFrame(.horizontal, count: max(1, slidesPerView), span: 1, spacing: CGFloat(gap))
                .frame(height: slideHeight.map { CGFloat($0) })
        }
    }

    nonisolated private func carouselRotation(_ phase: Double) -> Double {
        if variant == "stories" { return phase * 30 }
        if variant == "flipbook" { return phase * 52 }
        if variant == "coverFlow" { return phase * 24 }
        return 0
    }

    nonisolated private func carouselScale(_ phase: Double) -> CGFloat {
        let distance = min(abs(phase), 1)
        if variant == "coverFlow" || variant == "stories" || variant == "flipbook" { return CGFloat(1 - distance * 0.1) }
        if variant == "smartStack" || variant == "cardStack" { return CGFloat(1 - distance * 0.055) }
        return CGFloat(1)
    }

    nonisolated private func carouselTilt(_ phase: Double) -> Double {
        variant == "smartStack" ? phase * 1.5 : 0
    }

    nonisolated private func carouselOffset(_ phase: Double) -> CGFloat {
        variant == "smartStack" || variant == "cardStack" ? CGFloat(abs(phase) * 8) : 0
    }

    nonisolated private func carouselHorizontalOffset(_ phase: Double) -> CGFloat {
        variant == "slideshow" ? CGFloat(phase * 24) : 0
    }

    nonisolated private func carouselOpacity(_ phase: Double) -> Double {
        let distance = min(abs(phase), 1)
        if variant == "slideshow" { return 1 - distance * 0.12 }
        return variant == "coverFlow" || variant == "stories" || variant == "flipbook" ? 1 - distance * 0.22 : 1
    }
}

"##
}
