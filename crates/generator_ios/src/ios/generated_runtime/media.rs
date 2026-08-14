fn swift_runtime_media() -> &'static str {
    r#"struct DoweCoverImage: View {
    let source: String

    var body: some View {
        if source.hasPrefix("https://"), let url = URL(string: source) {
            AsyncImage(url: url) { image in
                image.resizable().scaledToFill()
            } placeholder: {
                Color.clear
            }
        } else {
            Image(source).resizable().scaledToFill()
        }
    }
}

struct DoweCodeToken {
    let text: String
    let color: Color
}

struct DoweVideoIcon {
    let viewBox: DoweSvgViewBox
    let paths: [DoweSvgPathData]
}

struct DoweVideoIcons {
    let play: DoweVideoIcon
    let pause: DoweVideoIcon
    let volume: DoweVideoIcon
    let muted: DoweVideoIcon
    let pictureInPicture: DoweVideoIcon
    let fullscreen: DoweVideoIcon
}

struct DoweVideoView: View {
    let poster: String?
    let autoplay: Bool
    let aspect: String
    let backgroundColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let icons: DoweVideoIcons
    @State private var player: AVPlayer
    @State private var started = false
    @State private var playing = false
    @State private var muted = false
    @State private var currentTime = Double(0)
    @State private var duration = Double(0)
    @State private var pictureInPictureController: AVPictureInPictureController?
    @State private var fullscreen = false
    private let timer = Timer.publish(every: 0.25, on: .main, in: .common).autoconnect()

    init(source: String, poster: String?, autoplay: Bool, aspect: String, backgroundColor: Color, borderColor: Color?, radius: CGFloat, icons: DoweVideoIcons) {
        self.poster = poster
        self.autoplay = autoplay
        self.aspect = aspect
        self.backgroundColor = backgroundColor
        self.borderColor = borderColor
        self.radius = radius
        self.icons = icons
        _player = State(initialValue: AVPlayer(url: URL(string: source)!))
    }

    var body: some View {
        ZStack(alignment: .bottom) {
            Color.black
            DoweVideoSurface(player: player, onPlayerLayer: configurePictureInPicture)
            if let poster, !started {
                DoweCoverImage(source: poster)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .clipped()
                    .contentShape(Rectangle())
                    .onTapGesture(perform: play)
            }
            Color.clear
                .contentShape(Rectangle())
                .onTapGesture(perform: togglePlayback)
            DoweVideoControls(
                playing: playing,
                muted: muted,
                currentTime: currentTime,
                duration: duration,
                icons: icons,
                onPlayPause: togglePlayback,
                onMute: toggleMute,
                onSeek: seek,
                onPictureInPicture: togglePictureInPicture,
                onFullscreen: toggleFullscreen
            )
        }
        .frame(maxWidth: .infinity)
        .aspectRatio(doweVideoAspect(aspect), contentMode: .fit)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
        .onAppear {
            try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .moviePlayback)
            try? AVAudioSession.sharedInstance().setActive(true)
            if autoplay {
                play()
            }
        }
        .fullScreenCover(isPresented: $fullscreen) {
            ZStack(alignment: .bottom) {
                Color.black.ignoresSafeArea()
                DoweVideoSurface(player: player, onPlayerLayer: configurePictureInPicture)
                    .ignoresSafeArea()
                DoweVideoControls(
                    playing: playing,
                    muted: muted,
                    currentTime: currentTime,
                    duration: duration,
                    icons: icons,
                    onPlayPause: togglePlayback,
                    onMute: toggleMute,
                    onSeek: seek,
                    onPictureInPicture: togglePictureInPicture,
                    onFullscreen: toggleFullscreen
                )
            }
            .background(Color.black)
        }
        .onReceive(timer) { _ in
            let seconds = player.currentTime().seconds
            if seconds.isFinite {
                currentTime = max(0, seconds)
            }
            let total = player.currentItem?.duration.seconds ?? 0
            if total.isFinite {
                duration = max(0, total)
            }
            playing = player.timeControlStatus == .playing
        }
        .onDisappear {
            player.pause()
            playing = false
        }
    }

    private func play() {
        started = true
        player.play()
        playing = true
    }

    private func togglePlayback() {
        if playing {
            player.pause()
            playing = false
        } else {
            play()
        }
    }

    private func toggleMute() {
        muted.toggle()
        player.isMuted = muted
    }

    private func seek(_ value: Double) {
        currentTime = value
        if value > 0 {
            started = true
        }
        player.seek(to: CMTime(seconds: value, preferredTimescale: 600))
    }

    private func configurePictureInPicture(_ playerLayer: AVPlayerLayer) {
        guard AVPictureInPictureController.isPictureInPictureSupported() else { return }
        if pictureInPictureController?.playerLayer !== playerLayer {
            guard let controller = AVPictureInPictureController(playerLayer: playerLayer) else { return }
            controller.canStartPictureInPictureAutomaticallyFromInline = true
            pictureInPictureController = controller
        }
    }

    private func togglePictureInPicture() {
        if let controller = pictureInPictureController, controller.isPictureInPictureActive {
            controller.stopPictureInPicture()
            return
        }
        if !playing { play() }
        startPictureInPicture(attempts: 20)
    }

    private func startPictureInPicture(attempts: Int) {
        guard attempts > 0 else { return }
        if let controller = pictureInPictureController, controller.isPictureInPicturePossible {
            controller.startPictureInPicture()
            return
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            startPictureInPicture(attempts: attempts - 1)
        }
    }

    private func toggleFullscreen() {
        fullscreen.toggle()
    }
}

final class DoweVideoPlayerView: UIView {
    override class var layerClass: AnyClass { AVPlayerLayer.self }

    var playerLayer: AVPlayerLayer { layer as! AVPlayerLayer }

    override init(frame: CGRect) {
        super.init(frame: frame)
        playerLayer.videoGravity = .resizeAspect
        backgroundColor = .black
    }

    required init?(coder: NSCoder) {
        nil
    }
}

struct DoweVideoSurface: UIViewRepresentable {
    let player: AVPlayer
    let onPlayerLayer: (AVPlayerLayer) -> Void

    func makeUIView(context: Context) -> DoweVideoPlayerView {
        let view = DoweVideoPlayerView()
        view.playerLayer.player = player
        DispatchQueue.main.async { onPlayerLayer(view.playerLayer) }
        return view
    }

    func updateUIView(_ view: DoweVideoPlayerView, context: Context) {
        view.playerLayer.player = player
        DispatchQueue.main.async { onPlayerLayer(view.playerLayer) }
    }
}

struct DoweVideoControls: View {
    let playing: Bool
    let muted: Bool
    let currentTime: Double
    let duration: Double
    let icons: DoweVideoIcons
    let onPlayPause: () -> Void
    let onMute: () -> Void
    let onSeek: (Double) -> Void
    let onPictureInPicture: () -> Void
    let onFullscreen: () -> Void

    var body: some View {
        VStack(spacing: CGFloat(2)) {
            HStack(spacing: CGFloat(8)) {
                DoweVideoControlButton(icon: playing ? icons.pause : icons.play, label: playing ? "Pause video" : "Play video", action: onPlayPause)
                Text("\(doweVideoTime(currentTime)) / \(doweVideoTime(duration))")
                    .font(.system(size: CGFloat(12), weight: .medium))
                    .foregroundStyle(Color.white)
                    .monospacedDigit()
                Spacer(minLength: CGFloat(0))
                DoweVideoControlButton(icon: muted ? icons.muted : icons.volume, label: muted ? "Unmute video" : "Mute video", action: onMute)
                DoweVideoControlButton(icon: icons.pictureInPicture, label: "Picture in picture", action: onPictureInPicture)
                DoweVideoControlButton(icon: icons.fullscreen, label: "Toggle fullscreen", action: onFullscreen)
            }
            Slider(
                value: Binding(
                    get: { min(max(currentTime, 0), max(duration, 0.01)) },
                    set: onSeek
                ),
                in: 0...max(duration, 0.01)
            )
            .tint(Color.white)
        }
        .padding(.horizontal, CGFloat(10))
        .padding(.vertical, CGFloat(8))
        .background(
            LinearGradient(colors: [Color.clear, Color.black.opacity(0.78)], startPoint: .top, endPoint: .bottom)
        )
    }
}

struct DoweVideoControlButton: View {
    let icon: DoweVideoIcon
    let label: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            DoweSvgView(viewBox: icon.viewBox, color: Color.white, paths: icon.paths)
                .frame(width: CGFloat(20), height: CGFloat(20))
                .frame(width: CGFloat(32), height: CGFloat(32))
                .background(Color.black.opacity(0.48))
                .clipShape(Circle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(label)
    }
}

private func doweVideoTime(_ value: Double) -> String {
    let seconds = max(0, Int(value))
    return String(format: "%d:%02d", seconds / 60, seconds % 60)
}

struct DoweDeviceIcon {
    let profile: String
    let viewBox: DoweSvgViewBox
    let paths: [DoweSvgPathData]
}

struct DoweDevicePreview: View {
    let initialProfile: String
    let source: String
    let title: String
    let sandbox: [String]?
    let autoplay: Bool
    let icons: [DoweDeviceIcon]
    @State private var profile: String

    init(initialProfile: String, source: String, title: String, sandbox: [String]?, autoplay: Bool, icons: [DoweDeviceIcon]) {
        self.initialProfile = initialProfile
        self.source = source
        self.title = title
        self.sandbox = sandbox
        self.autoplay = autoplay
        self.icons = icons
        _profile = State(initialValue: initialProfile)
    }

    private var dimensions: CGSize {
        switch profile {
        case "tablet": CGSize(width: 768, height: 1024)
        case "laptop": CGSize(width: 1440, height: 900)
        case "monitor": CGSize(width: 1920, height: 1080)
        default: CGSize(width: 390, height: 844)
        }
    }

    var body: some View {
        VStack(spacing: 12) {
            HStack(spacing: 4) {
                ForEach(icons, id: \.profile) { option in
                    Button { profile = option.profile } label: {
                        DoweSvgView(viewBox: option.viewBox, color: profile == option.profile ? DoweDesign.primary : DoweDesign.backgroundText, paths: option.paths)
                            .frame(width: CGFloat(24), height: CGFloat(24))
                    }
                    .frame(width: CGFloat(40), height: CGFloat(40))
                    .foregroundStyle(profile == option.profile ? DoweDesign.primary : DoweDesign.backgroundText)
                    .background(profile == option.profile ? DoweDesign.softPrimary : Color.clear)
                    .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
                    .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(profile == option.profile ? DoweDesign.primary : DoweDesign.backgroundText, lineWidth: CGFloat(1)))
                    .buttonStyle(.plain)
                    .accessibilityLabel(option.profile)
                    .accessibilityAddTraits(profile == option.profile ? .isSelected : [])
                }
            }
            .padding(CGFloat(4))
            GeometryReader { geometry in
                let zoom = min(CGFloat(1), geometry.size.width / dimensions.width)
                DoweIframeView(source: source, title: title, sandbox: sandbox, autoplay: autoplay)
                    .frame(width: dimensions.width, height: dimensions.height)
                    .scaleEffect(zoom, anchor: .top)
                    .frame(width: geometry.size.width, height: dimensions.height * zoom, alignment: .top)
            }
            .frame(maxWidth: dimensions.width)
            .aspectRatio(dimensions.width / dimensions.height, contentMode: .fit)
        }
    }
}

struct DoweIframeView: UIViewRepresentable {
    let source: String
    let title: String
    let sandbox: [String]?
    let autoplay: Bool

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = sandbox == nil || sandbox?.contains("scripts") == true
        configuration.allowsInlineMediaPlayback = true
        configuration.mediaTypesRequiringUserActionForPlayback = autoplay ? [] : .all
        let view = WKWebView(frame: .zero, configuration: configuration)
        view.navigationDelegate = context.coordinator
        view.uiDelegate = context.coordinator
        view.isOpaque = false
        view.backgroundColor = .clear
        view.scrollView.isScrollEnabled = true
        view.accessibilityLabel = title
        if let url = doweIframeURL(source) {
            context.coordinator.source = source
            view.load(URLRequest(url: url))
        }
        return view
    }

    func updateUIView(_ view: WKWebView, context: Context) {
        guard context.coordinator.source != source, let url = doweIframeURL(source) else { return }
        context.coordinator.source = source
        view.load(URLRequest(url: url))
    }

    final class Coordinator: NSObject, WKNavigationDelegate, WKUIDelegate {
        var source: String?

        func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            guard let url = navigationAction.request.url else {
                decisionHandler(.cancel)
                return
            }
            decisionHandler(doweIframeURLAllowed(url) || url.absoluteString == "about:blank" ? .allow : .cancel)
        }

        func webView(_ webView: WKWebView, createWebViewWith configuration: WKWebViewConfiguration, for navigationAction: WKNavigationAction, windowFeatures: WKWindowFeatures) -> WKWebView? {
            nil
        }
    }
}

private func doweIframeURL(_ source: String) -> URL? {
    if let url = URL(string: source), url.scheme == "https" {
        return url
    }
    guard source.hasPrefix("/"), !source.hasPrefix("//") else {
        return nil
    }
    let configured = DoweEnvironment.BACKEND_URL.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    let development = (UserDefaults.standard.string(forKey: "dowe.hmr.endpoint") ?? "").trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    guard let baseURL = [development, configured].compactMap({ URL(string: $0) }).first(where: doweIframeURLAllowed) else { return nil }
    return URL(string: source, relativeTo: baseURL)?.absoluteURL
}

private func doweIframeURLAllowed(_ url: URL) -> Bool {
    if url.scheme == "https" { return true }
    return url.scheme == "http" && (url.host == "localhost" || url.host == "127.0.0.1" || url.host == "::1")
}

private func doweVideoAspect(_ value: String) -> CGFloat {
    switch value {
    case "vertical":
        return CGFloat(9) / CGFloat(16)
    case "square":
        return CGFloat(1)
    default:
        return CGFloat(16) / CGFloat(9)
    }
}

struct DoweAudioView: View {
    let source: String
    let subtitle: String?
    let avatarSource: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var playing = false

    var body: some View {
        HStack(spacing: CGFloat(12)) {
            Button(playing ? "Pause" : "Play") {
                playing.toggle()
            }
            .buttonStyle(.bordered)
            VStack(alignment: .leading, spacing: CGFloat(6)) {
                Text(subtitle ?? source)
                    .lineLimit(1)
                HStack(spacing: CGFloat(3)) {
                    ForEach(0..<24, id: \.self) { index in
                        RoundedRectangle(cornerRadius: CGFloat(2))
                            .fill(contentColor.opacity(playing ? 0.9 : 0.35))
                            .frame(width: CGFloat(3), height: CGFloat((index % 7) + 4))
                    }
                }
            }
            if let avatarSource {
                DoweCoverImage(source: avatarSource)
                    .frame(width: CGFloat(36), height: CGFloat(36))
                    .clipShape(Circle())
            }
        }
        .padding(.horizontal, CGFloat(12))
        .padding(.vertical, CGFloat(8))
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }
}

struct DoweImageView: View {
    let source: String
    let alt: String
    let aspect: String
    let objectFit: String
    let loading: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat

    var body: some View {
        DoweImageAspectLayout(ratio: doweImageAspect(aspect)) {
            ZStack {
                AsyncImage(url: URL(string: source)) { image in
                    if objectFit == "contain" {
                        image.resizable().scaledToFit()
                    } else {
                        image.resizable().scaledToFill()
                    }
                } placeholder: {
                    Rectangle().fill(contentColor.opacity(0.12))
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(maxWidth: .infinity)
        .background(backgroundColor)
        .clipped()
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
        .accessibilityElement(children: .ignore)
        .accessibilityAddTraits(.isImage)
        .accessibilityLabel(Text(alt))
        .accessibilityHidden(alt.isEmpty)
    }
}

struct DoweImageAspectLayout: Layout {
    let ratio: CGFloat

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let resolvedRatio = ratio > 0 ? ratio : CGFloat(1)
        if let width = proposal.width, width.isFinite {
            let resolvedWidth = Swift.max(width, 0)
            return CGSize(width: resolvedWidth, height: resolvedWidth / resolvedRatio)
        }
        if let height = proposal.height, height.isFinite {
            let resolvedHeight = Swift.max(height, 0)
            return CGSize(width: resolvedHeight * resolvedRatio, height: resolvedHeight)
        }
        let idealWidth = subviews.first?.sizeThatFits(.unspecified).width ?? CGFloat(0)
        let resolvedWidth = Swift.max(idealWidth, 0)
        return CGSize(width: resolvedWidth, height: resolvedWidth / resolvedRatio)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        for subview in subviews {
            subview.place(at: bounds.origin, anchor: .topLeading, proposal: ProposedViewSize(bounds.size))
        }
    }
}

private func doweImageAspect(_ value: String) -> CGFloat {
    switch value {
    case "vertical":
        return CGFloat(9) / CGFloat(16)
    case "square":
        return CGFloat(1)
    default:
        return CGFloat(16) / CGFloat(9)
    }
}

"#
}
