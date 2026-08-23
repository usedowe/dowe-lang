fn swift_runtime_editor_image_controls() -> &'static str {
    r#"struct DoweEditorField: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let minHeight: CGFloat
    let hideToolbar: Bool
    let readOnly: Bool
    let backgroundColor: Color
    let contentColor: Color
    @State private var localValue: String?

    private var currentText: String {
        value?.wrappedValue ?? localValue ?? initialValue
    }

    private var textBinding: Binding<String> {
        Binding(
            get: { value?.wrappedValue ?? localValue ?? initialValue },
            set: { next in
                if !readOnly {
                    if let value {
                        value.wrappedValue = next
                    } else {
                        localValue = next
                    }
                }
            }
        )
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
                    .padding(.horizontal, CGFloat(12))
                    .padding(.top, CGFloat(10))
            }
            if !hideToolbar {
                HStack(spacing: CGFloat(4)) {
                    ForEach(["B", "I", "U", "List"], id: \.self) { item in
                        Text(item)
                            .font(.footnote)
                            .fontWeight(.bold)
                            .padding(.horizontal, CGFloat(8))
                            .padding(.vertical, CGFloat(5))
                            .background(contentColor.opacity(0.08))
                            .clipShape(RoundedRectangle(cornerRadius: CGFloat(8)))
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(CGFloat(6))
                .background(contentColor.opacity(0.08))
            }
            ZStack(alignment: .topLeading) {
                if currentText.isEmpty && !placeholder.isEmpty {
                    Text(placeholder)
                        .foregroundStyle(contentColor.opacity(0.52))
                        .padding(CGFloat(8))
                }
                TextEditor(text: textBinding)
                    .foregroundStyle(contentColor)
                    .frame(minHeight: minHeight)
                    .disabled(readOnly)
                    .scrollContentBackground(.hidden)
            }
            .padding(CGFloat(8))
        }
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
        .overlay(RoundedRectangle(cornerRadius: CGFloat(16)).stroke(contentColor.opacity(0.18), lineWidth: CGFloat(1)))
    }
}

private func doweImageCropperSize(_ size: String) -> CGFloat {
    switch size {
    case "xs": return CGFloat(96)
    case "sm": return CGFloat(112)
    case "lg": return CGFloat(160)
    case "xl": return CGFloat(192)
    default: return CGFloat(128)
    }
}

private func doweImageFromDataURL(_ source: String) -> UIImage? {
    guard source.hasPrefix("data:image/"), let comma = source.firstIndex(of: ",") else { return nil }
    let encoded = String(source[source.index(after: comma)...])
    guard let data = Data(base64Encoded: encoded, options: .ignoreUnknownCharacters) else { return nil }
    return UIImage(data: data)
}

private func doweImageDataURL(_ image: UIImage, mime: String) -> String? {
    let jpeg = mime.contains("jpeg") || mime.contains("jpg")
    let data = jpeg ? image.jpegData(compressionQuality: CGFloat(0.92)) : image.pngData()
    guard let data else { return nil }
    return "data:\(jpeg ? "image/jpeg" : "image/png");base64,\(data.base64EncodedString())"
}

private func doweCropImage(_ image: UIImage, aspect: CGFloat, zoom: CGFloat, offset: CGSize, frame: CGSize, minWidth: Int, minHeight: Int, maxWidth: Int?, maxHeight: Int?) -> UIImage? {
    guard let cgImage = image.cgImage else { return nil }
    let width = CGFloat(1000)
    let height = width / max(aspect, CGFloat(0.01))
    let scale = max(width / CGFloat(cgImage.width), height / CGFloat(cgImage.height)) * zoom
    let imageWidth = CGFloat(cgImage.width) * scale
    let imageHeight = CGFloat(cgImage.height) * scale
    let normalizedOffset = CGSize(width: offset.width * width / max(frame.width, CGFloat(1)), height: offset.height * width / max(frame.width, CGFloat(1)))
    let left = (width - imageWidth) / CGFloat(2) + normalizedOffset.width
    let top = (height - imageHeight) / CGFloat(2) + normalizedOffset.height
    let sourceX = max(CGFloat(0), min(CGFloat(cgImage.width), -left / scale))
    let sourceY = max(CGFloat(0), min(CGFloat(cgImage.height), -top / scale))
    let sourceWidth = min(CGFloat(cgImage.width) - sourceX, width / scale)
    let sourceHeight = min(CGFloat(cgImage.height) - sourceY, height / scale)
    guard sourceWidth >= CGFloat(minWidth), sourceHeight >= CGFloat(minHeight) else { return nil }
    var outputWidth = max(1, Int(sourceWidth.rounded()))
    var outputHeight = max(1, Int(sourceHeight.rounded()))
    let limit = min(maxWidth.map { CGFloat($0) / CGFloat(outputWidth) } ?? CGFloat(1), maxHeight.map { CGFloat($0) / CGFloat(outputHeight) } ?? CGFloat(1))
    if limit < 1 {
        outputWidth = max(1, Int((CGFloat(outputWidth) * limit).rounded()))
        outputHeight = max(1, Int((CGFloat(outputHeight) * limit).rounded()))
    }
    let cropRect = CGRect(x: sourceX, y: sourceY, width: sourceWidth, height: sourceHeight).integral
    guard let cropped = cgImage.cropping(to: cropRect) else { return nil }
    let renderer = UIGraphicsImageRenderer(size: CGSize(width: outputWidth, height: outputHeight))
    return renderer.image { _ in UIImage(cgImage: cropped).draw(in: CGRect(x: 0, y: 0, width: CGFloat(outputWidth), height: CGFloat(outputHeight))) }
}

private func doweImageCropperTypes(_ accept: String) -> [UTType] {
    let values = accept.split(separator: ",").compactMap { value -> UTType? in
        let item = value.trimmingCharacters(in: .whitespacesAndNewlines)
        if item == "image/*" { return .image }
        return UTType(mimeType: item) ?? UTType(filenameExtension: item)
    }
    return values.isEmpty ? [.image] : values
}

private func doweLoadCropperImage(_ source: String) async -> UIImage? {
    if let image = doweImageFromDataURL(source) { return image }
    guard let url = URL(string: source) else { return nil }
    if url.isFileURL { return UIImage(contentsOfFile: url.path) }
    guard let (data, _) = try? await URLSession.shared.data(from: url) else { return nil }
    return UIImage(data: data)
}

struct DoweImageCropper: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let alt: String
    let accept: String
    let aspectRatio: String?
    let minWidth: Int
    let minHeight: Int
    let maxWidth: Int?
    let maxHeight: Int?
    let shape: String
    let size: String
    let disabled: Bool
    let helpText: String?
    let errorText: String?
    let backgroundColor: Color
    let contentColor: Color
    @State private var localValue = ""
    @State private var cleared = false
    @State private var appliedImage: UIImage?
    @State private var draftImage: UIImage?
    @State private var draftMime = "image/png"
    @State private var pickerPresented = false
    @State private var editorPresented = false
    @State private var zoom: CGFloat = 1
    @State private var offset = CGSize.zero
    @State private var cropError: String?

    private var currentValue: String {
        if let value, !value.wrappedValue.isEmpty { return value.wrappedValue }
        if cleared { return "" }
        return localValue.isEmpty ? initialValue : localValue
    }

    private var aspect: CGFloat {
        max(CGFloat(0.01), CGFloat(Double(aspectRatio ?? "1") ?? 1))
    }

    private var radius: CGFloat {
        shape == "circle" ? CGFloat(999) : CGFloat(18)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label { Text(label).fontWeight(.semibold) }
            preview
            HStack(spacing: CGFloat(8)) {
                Button(action: { pickerPresented = true }) { Text(currentValue.isEmpty ? "Upload" : "Change") }.buttonStyle(.plain).disabled(disabled)
                if !currentValue.isEmpty { Button(action: clearValue) { Text("Remove").foregroundStyle(contentColor.opacity(0.72)) }.buttonStyle(.plain).disabled(disabled) }
            }
            if let text = cropError ?? errorText ?? helpText { Text(text).font(.system(size: CGFloat(13))).foregroundStyle(cropError != nil || errorText != nil ? DoweDesign.danger : contentColor.opacity(0.7)) }
        }
        .foregroundStyle(contentColor)
        .task(id: currentValue) { appliedImage = await doweLoadCropperImage(currentValue) }
        .fileImporter(isPresented: $pickerPresented, allowedContentTypes: doweImageCropperTypes(accept), allowsMultipleSelection: false) { result in
            guard case .success(let urls) = result, let url = urls.first else { return }
            let secured = url.startAccessingSecurityScopedResource()
            defer { if secured { url.stopAccessingSecurityScopedResource() } }
            guard let data = try? Data(contentsOf: url), let image = UIImage(data: data) else { cropError = "The selected image could not be decoded."; return }
            draftImage = image
            draftMime = UTType(filenameExtension: url.pathExtension)?.preferredMIMEType ?? "image/png"
            zoom = 1
            offset = .zero
            cropError = nil
            editorPresented = true
        }
        .overlay { if editorPresented { editor } }
    }

    private var preview: some View {
        ZStack {
            if let appliedImage { Image(uiImage: appliedImage).resizable().scaledToFill().accessibilityLabel(Text(alt)) } else if let url = URL(string: currentValue), !currentValue.isEmpty { AsyncImage(url: url) { phase in switch phase { case .success(let image): image.resizable().scaledToFill(); default: placeholderView } } } else { placeholderView }
        }
        .frame(width: doweImageCropperSize(size), height: doweImageCropperSize(size))
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(RoundedRectangle(cornerRadius: radius).stroke(contentColor.opacity(0.2), lineWidth: CGFloat(1)))
        .contentShape(RoundedRectangle(cornerRadius: radius))
        .onTapGesture { if !currentValue.isEmpty && !disabled { draftImage = appliedImage; editorPresented = draftImage != nil; zoom = 1; offset = .zero } }
        .accessibilityLabel(Text(alt))
    }

    private var placeholderView: some View {
        VStack(spacing: CGFloat(6)) { Image(systemName: "photo").font(.system(size: CGFloat(24))).opacity(0.5); Text(placeholder).fontWeight(.bold) }.foregroundStyle(contentColor).frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    @ViewBuilder
    private var editor: some View {
        ZStack {
            Color.black.opacity(0.5).ignoresSafeArea()
            VStack(alignment: .leading, spacing: CGFloat(12)) {
                HStack { Text("Adjust image").fontWeight(.bold); Spacer(); Button("Cancel") { editorPresented = false }.buttonStyle(.plain) }
                GeometryReader { geometry in
                    let frame = cropFrame(geometry.size)
                    ZStack {
                        Color.black
                        if let draftImage { Image(uiImage: draftImage).resizable().scaledToFill().frame(width: frame.width, height: frame.height).scaleEffect(zoom).offset(offset).gesture(DragGesture().onChanged { offset = CGSize(width: $0.translation.width, height: $0.translation.height) }).simultaneousGesture(MagnificationGesture().onChanged { zoom = min(CGFloat(3), max(CGFloat(1), $0)) }) }
                        Canvas { context, _ in
                            for fraction in [CGFloat(1) / CGFloat(3), CGFloat(2) / CGFloat(3)] {
                                var path = Path()
                                path.move(to: CGPoint(x: frame.minX + frame.width * fraction, y: frame.minY))
                                path.addLine(to: CGPoint(x: frame.minX + frame.width * fraction, y: frame.maxY))
                                path.move(to: CGPoint(x: frame.minX, y: frame.minY + frame.height * fraction))
                                path.addLine(to: CGPoint(x: frame.maxX, y: frame.minY + frame.height * fraction))
                                context.stroke(path, with: .color(.white.opacity(0.65)), lineWidth: CGFloat(1))
                            }
                        }
                    }
                    .frame(width: frame.width, height: frame.height)
                    .clipShape(shape == "circle" ? AnyShape(Circle()) : AnyShape(Rectangle()))
                    .overlay {
                        if shape == "circle" {
                            Circle().stroke(.white, lineWidth: CGFloat(2))
                        } else {
                            Rectangle().stroke(.white, lineWidth: CGFloat(2))
                        }
                    }
                    .position(x: geometry.size.width / CGFloat(2), y: geometry.size.height / CGFloat(2))
                }
                .frame(height: CGFloat(320))
                HStack { Text("Zoom").font(.system(size: CGFloat(12))); Slider(value: $zoom, in: CGFloat(1)...CGFloat(3)); }
                HStack { Button("Reset") { zoom = 1; offset = .zero }.buttonStyle(.plain); Spacer(); Button("Cancel") { editorPresented = false }.buttonStyle(.plain); Button("Apply") { applyCrop() }.buttonStyle(.borderedProminent) }
                if let cropError { Text(cropError).font(.system(size: CGFloat(13))).foregroundStyle(DoweDesign.danger) }
            }
            .padding(CGFloat(16))
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(20)))
            .padding(CGFloat(16))
        }
    }

    private func cropFrame(_ size: CGSize) -> CGRect {
        let inset = CGFloat(24)
        let maxWidth = max(CGFloat(1), size.width - inset * CGFloat(2))
        let maxHeight = max(CGFloat(1), size.height - inset * CGFloat(2))
        var width = maxWidth
        var height = width / aspect
        if height > maxHeight { height = maxHeight; width = height * aspect }
        return CGRect(x: (size.width - width) / CGFloat(2), y: (size.height - height) / CGFloat(2), width: width, height: height)
    }

    private func applyCrop() {
        guard let draftImage else { return }
        let frame = CGSize(width: CGFloat(1000), height: CGFloat(1000) / aspect)
        guard let cropped = doweCropImage(draftImage, aspect: aspect, zoom: zoom, offset: offset, frame: frame, minWidth: minWidth, minHeight: minHeight, maxWidth: maxWidth, maxHeight: maxHeight), let next = doweImageDataURL(cropped, mime: draftMime) else { cropError = "Image must be at least \(minWidth) × \(minHeight) pixels."; return }
        localValue = next
        cleared = false
        value?.wrappedValue = next
        appliedImage = cropped
        editorPresented = false
    }

    private func clearValue() {
        cleared = true
        localValue = ""
        value?.wrappedValue = ""
        appliedImage = nil
    }
}

"#
}
