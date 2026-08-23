fn swift_runtime_carousel() -> &'static str {
    r##"struct DoweCarouselView<Content: View>: View {
    let variant: String
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

    init(variant: String, slideIds: [String], autoplay: Bool, autoplayInterval: Int, disableLoop: Bool, hideControls: Bool, hideIndicators: Bool, showNavigation: Bool, showCounter: Bool, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, accentColor: Color, @ViewBuilder content: () -> Content) {
        self.variant = variant
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
        VStack(alignment: .leading, spacing: CGFloat(12)) {
            if let title {
                Text(title).font(.title2).fontWeight(.bold).foregroundStyle(accentColor)
            }
            ZStack(alignment: .center) {
                if orientation == "vertical" {
                    ScrollView(.vertical, showsIndicators: false) {
                        LazyVStack(spacing: CGFloat(gap)) { content }
                            .scrollTargetLayout()
                    }
                    .scrollTargetBehavior(.viewAligned)
                    .scrollPosition(id: $scrollId)
                    .frame(maxHeight: CGFloat(560))
                } else {
                    ScrollView(.horizontal, showsIndicators: false) {
                        LazyHStack(spacing: CGFloat(gap)) { content }
                            .scrollTargetLayout()
                    }
                    .scrollTargetBehavior(.viewAligned)
                    .scrollPosition(id: $scrollId)
                    .environment(\.layoutDirection, variant == "rtl" ? .rightToLeft : .leftToRight)
                }
                if showNavigation {
                    HStack {
                        Button("‹") { move(-1) }
                            .disabled(disableLoop && currentIndex == 0)
                        Spacer()
                        Button("›") { move(1) }
                            .disabled(disableLoop && currentIndex == slideIds.count - 1)
                    }
                    .padding(.horizontal, CGFloat(8))
                }
            }
            if !hideControls || variant == "controls" {
                HStack {
                    Button("Previous") { move(-1) }
                        .disabled(disableLoop && currentIndex == 0)
                    Spacer()
                    Button("Next") { move(1) }
                        .disabled(disableLoop && currentIndex == slideIds.count - 1)
                }
            }
            if !hideIndicators || variant == "dots" || variant == "thumbnails" {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: CGFloat(8)) {
                        ForEach(Array(slideIds.enumerated()), id: \.offset) { index, id in
                            Button(variant == "thumbnails" ? "Slide \(index + 1)" : indicatorType == "dot" || variant == "dots" ? "•" : "\(index + 1)") {
                                currentIndex = index
                                withAnimation { scrollId = id }
                            }
                            .foregroundStyle(index == currentIndex ? accentColor : accentColor.opacity(0.45))
                        }
                    }
                }
            }
            if showCounter {
                Text("\(currentIndex + 1) / \(slideIds.count)").foregroundStyle(accentColor)
            }
        }
        .onAppear {
            if scrollId == nil { scrollId = slideIds.first }
        }
        .onChange(of: scrollId) { _, value in
            guard let value, let index = slideIds.firstIndex(of: value) else { return }
            currentIndex = index
        }
        .task(id: currentIndex) {
            guard autoplay, slideIds.count > 1 else { return }
            try? await Task.sleep(nanoseconds: UInt64(max(500, autoplayInterval)) * 1_000_000)
            guard !Task.isCancelled else { return }
            move(1)
        }
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
    let orientation: String
    let slideWidth: Int?
    let slideHeight: Int?
    let slidesPerView: Int
    let gap: Int
    @ViewBuilder var content: Content

    init(id: String, variant: String, index: Int, orientation: String, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, @ViewBuilder content: () -> Content) {
        self.id = id
        self.variant = variant
        self.index = index
        self.orientation = orientation
        self.slideWidth = slideWidth
        self.slideHeight = slideHeight
        self.slidesPerView = slidesPerView
        self.gap = gap
        self.content = content()
    }

    var body: some View {
        sizedContent
            .scrollTransition(.interactive, axis: .horizontal) { view, phase in
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
        } else if variant == "masonry" {
            content
                .frame(minWidth: CGFloat(180))
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
        return variant == "coverFlow" || variant == "stories" || variant == "flipbook" ? 1 - distance * 0.22 : 1
    }
}

"##
}
