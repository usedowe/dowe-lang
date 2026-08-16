fn swift_runtime_capture() -> &'static str {
    r#"struct DoweCameraView: View {
    @ObservedObject var state: DoweReactiveState
    let facing: String
    let label: String
    let disabled: Bool
    let onStart: String?
    let onCapture: String?
    let onError: String?
    let backgroundColor: Color
    let contentColor: Color
    let radius: CGFloat
    @State private var showingCamera = false

    var body: some View {
        Button {
            guard !disabled else { return }
            if let onStart { state.run(onStart, item: ["source": "camera", "kind": "start", "facing": facing]) }
            showingCamera = true
        } label: {
            Label(label, systemImage: "camera")
                .frame(maxWidth: .infinity)
                .padding(.vertical, CGFloat(12))
        }
        .buttonStyle(.borderedProminent)
        .tint(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .disabled(disabled)
        .sheet(isPresented: $showingCamera) {
            DoweCameraPicker(facing: facing, onImage: { image in
                showingCamera = false
                guard let data = image.jpegData(compressionQuality: 0.92) else {
                    if let onError { state.run(onError, item: ["source": "camera", "kind": "error", "error": "capture_failed"]) }
                    return
                }
                let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".jpg")
                do {
                    try data.write(to: url)
                    if let onCapture { state.run(onCapture, item: ["source": "camera", "kind": "capture", "facing": facing, "mimeType": "image/jpeg", "url": url.absoluteString, "width": Double(image.size.width), "height": Double(image.size.height)]) }
                } catch {
                    if let onError { state.run(onError, item: ["source": "camera", "kind": "error", "error": "write_failed"]) }
                }
            }, onError: { error in
                showingCamera = false
                if let onError { state.run(onError, item: ["source": "camera", "kind": "error", "error": error]) }
            })
        }
    }
}

private struct DoweCameraPicker: UIViewControllerRepresentable {
    let facing: String
    let onImage: (UIImage) -> Void
    let onError: (String) -> Void

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    func makeUIViewController(context: Context) -> UIImagePickerController {
        let picker = UIImagePickerController()
        let sourceType: UIImagePickerController.SourceType = UIImagePickerController.isSourceTypeAvailable(.camera) ? .camera : .photoLibrary
        picker.sourceType = sourceType
        if sourceType == .camera {
            picker.cameraDevice = facing == "user" ? .front : .rear
        }
        picker.delegate = context.coordinator
        return picker
    }

    func updateUIViewController(_ controller: UIImagePickerController, context: Context) {}

    final class Coordinator: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
        let parent: DoweCameraPicker
        init(_ parent: DoweCameraPicker) { self.parent = parent }
        func imagePickerController(_ picker: UIImagePickerController, didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
            if let image = info[.originalImage] as? UIImage { parent.onImage(image) } else { parent.onError("capture_failed") }
        }
        func imagePickerControllerDidCancel(_ picker: UIImagePickerController) { parent.onError("cancelled") }
    }
}

struct DoweMicrophoneView: View {
    @ObservedObject var state: DoweReactiveState
    let label: String
    let maxDuration: Double?
    let disabled: Bool
    let onStart: String?
    let onStop: String?
    let onError: String?
    let backgroundColor: Color
    let contentColor: Color
    let radius: CGFloat
    @StateObject private var recorder: DoweMicrophoneController

    init(state: DoweReactiveState, label: String, maxDuration: Double?, disabled: Bool, onStart: String?, onStop: String?, onError: String?, backgroundColor: Color, contentColor: Color, radius: CGFloat) {
        self.state = state
        self.label = label
        self.maxDuration = maxDuration
        self.disabled = disabled
        self.onStart = onStart
        self.onStop = onStop
        self.onError = onError
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.radius = radius
        _recorder = StateObject(wrappedValue: DoweMicrophoneController(state: state, maxDuration: maxDuration, onStart: onStart, onStop: onStop, onError: onError))
    }

    var body: some View {
        HStack(spacing: CGFloat(12)) {
            Text(label).fontWeight(.semibold)
            Spacer()
            Text(formatDuration(recorder.elapsed)).monospacedDigit().opacity(0.72)
            Button(recorder.recording ? "Stop" : label) { recorder.recording ? recorder.stop() : recorder.start() }
                .buttonStyle(.borderedProminent)
                .tint(backgroundColor)
                .foregroundStyle(contentColor)
                .disabled(disabled)
        }
        .padding(CGFloat(12))
        .background(backgroundColor.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: radius))
    }

    private func formatDuration(_ value: Double) -> String {
        let seconds = max(0, Int(value))
        return String(format: "%d:%02d", seconds / 60, seconds % 60)
    }
}

@MainActor
private final class DoweMicrophoneController: NSObject, ObservableObject, AVAudioRecorderDelegate {
    @Published var recording = false
    @Published var elapsed = 0.0
    private let state: DoweReactiveState
    private let maxDuration: Double?
    private let onStart: String?
    private let onStop: String?
    private let onError: String?
    private var recorder: AVAudioRecorder?
    private var startedAt = Date()
    private var timer: Timer?

    init(state: DoweReactiveState, maxDuration: Double?, onStart: String?, onStop: String?, onError: String?) {
        self.state = state
        self.maxDuration = maxDuration
        self.onStart = onStart
        self.onStop = onStop
        self.onError = onError
    }

    deinit {
        timer?.invalidate()
        recorder?.stop()
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }

    func start() {
        let handlePermission: @Sendable (Bool) -> Void = { [weak self] granted in
            Task { @MainActor in
                guard let self else { return }
                guard granted else { self.emitError("permission_denied"); return }
                do {
                    let session = AVAudioSession.sharedInstance()
                    try session.setCategory(.record, mode: .default)
                    try session.setActive(true)
                    let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".m4a")
                    self.recorder = try AVAudioRecorder(url: url, settings: [AVFormatIDKey: Int(kAudioFormatMPEG4AAC), AVSampleRateKey: 44100, AVNumberOfChannelsKey: 1, AVEncoderAudioQualityKey: AVAudioQuality.high.rawValue])
                    self.recorder?.delegate = self
                    self.recorder?.record()
                    self.recording = true
                    self.startedAt = Date()
                    if let onStart = self.onStart { self.state.run(onStart, item: ["source": "microphone", "kind": "start"]) }
                    self.timer = Timer.scheduledTimer(withTimeInterval: 0.25, repeats: true) { [weak self] _ in
                        Task { @MainActor [weak self] in self?.tick() }
                    }
                } catch { self.emitError("unavailable") }
            }
        }
        if #available(iOS 17.0, *) {
            AVAudioApplication.requestRecordPermission(completionHandler: handlePermission)
        } else {
            AVAudioSession.sharedInstance().requestRecordPermission(handlePermission)
        }
    }

    func stop() { recorder?.stop() }

    private func tick() {
        elapsed = Date().timeIntervalSince(startedAt)
        if let maxDuration, elapsed >= maxDuration { recorder?.stop() }
    }

    nonisolated func audioRecorderDidFinishRecording(_ recorder: AVAudioRecorder, successfully flag: Bool) {
        let url = recorder.url.absoluteString
        Task { @MainActor [weak self] in
            self?.finishRecording(successfully: flag, url: url)
        }
    }

    private func finishRecording(successfully flag: Bool, url: String) {
        timer?.invalidate(); timer = nil
        elapsed = Date().timeIntervalSince(startedAt); recording = false
        guard flag else { emitError("recording_failed"); return }
        if let onStop { state.run(onStop, item: ["source": "microphone", "kind": "stop", "mimeType": "audio/m4a", "url": url, "durationMs": elapsed * 1000]) }
    }

    private func emitError(_ value: String) {
        recording = false
        if let onError { state.run(onError, item: ["source": "microphone", "kind": "error", "error": value]) }
    }
}
"#
}
