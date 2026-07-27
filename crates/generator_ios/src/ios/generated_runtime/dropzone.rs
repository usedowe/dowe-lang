fn swift_runtime_dropzone() -> &'static str {
    r#"private struct DowePickedFile: Identifiable {
    let id: String
    let name: String
    let size: Int64?
}

private func doweDropzoneFileTypes(_ accept: String?) -> [UTType] {
    let values = accept?
        .split(separator: ",")
        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        .filter { !$0.isEmpty } ?? []
    let types = values.compactMap { value -> UTType? in
        switch value {
        case "image/*":
            return .image
        case "video/*":
            return .movie
        case "audio/*":
            return .audio
        default:
            return UTType(mimeType: value) ?? UTType(filenameExtension: value)
        }
    }
    return types.isEmpty ? [.item] : types
}

private func doweDropzoneSizeLabel(_ size: Int64?) -> String {
    guard let size, size >= 0 else {
        return ""
    }
    if size >= 1024 * 1024 * 1024 {
        return "\(size / (1024 * 1024 * 1024)) GB"
    }
    if size >= 1024 * 1024 {
        return "\(size / (1024 * 1024)) MB"
    }
    if size >= 1024 {
        return "\(size / 1024) KB"
    }
    return "\(size) Bytes"
}

private func doweDropzoneHeight(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(128)
    case "lg":
        return CGFloat(256)
    default:
        return CGFloat(192)
    }
}

struct DoweDropzone: View {
    let label: String?
    let placeholder: String
    let accept: String?
    let multiple: Bool
    let maxSize: Int64?
    let disabled: Bool
    let helpText: String?
    let errorText: String?
    let size: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color
    let radius: CGFloat
    @State private var importerPresented = false
    @State private var selectedFiles: [DowePickedFile] = []

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label)
                    .font(.system(size: CGFloat(14), weight: .semibold))
            }
            Button(action: { importerPresented = true }) {
                VStack(spacing: CGFloat(8)) {
                    Image(systemName: selectedFiles.isEmpty ? "paperclip" : "doc.on.doc")
                        .font(.system(size: CGFloat(28), weight: .regular))
                        .opacity(0.5)
                    if selectedFiles.isEmpty {
                        Text(placeholder)
                            .font(.system(size: CGFloat(14)))
                            .multilineTextAlignment(.center)
                            .opacity(0.7)
                    } else {
                        VStack(spacing: CGFloat(3)) {
                            ForEach(selectedFiles.prefix(3)) { file in
                                VStack(spacing: CGFloat(1)) {
                                    Text(file.name)
                                        .font(.system(size: CGFloat(14)))
                                        .lineLimit(1)
                                    let size = doweDropzoneSizeLabel(file.size)
                                    if !size.isEmpty {
                                        Text(size)
                                            .font(.system(size: CGFloat(12)))
                                            .opacity(0.7)
                                    }
                                }
                            }
                            if selectedFiles.count > 3 {
                                Text("+\(selectedFiles.count - 3) more")
                                    .font(.system(size: CGFloat(12)))
                                    .opacity(0.7)
                            }
                        }
                    }
                }
                .frame(maxWidth: .infinity, minHeight: doweDropzoneHeight(size))
            }
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor, style: StrokeStyle(lineWidth: CGFloat(2), dash: [CGFloat(6)])))
            .buttonStyle(.plain)
            .disabled(disabled)
            .opacity(disabled ? 0.5 : 1)
            .fileImporter(
                isPresented: $importerPresented,
                allowedContentTypes: doweDropzoneFileTypes(accept),
                allowsMultipleSelection: multiple
            ) { result in
                guard case .success(let urls) = result else {
                    return
                }
                let files = urls.compactMap { url -> DowePickedFile? in
                    let secured = url.startAccessingSecurityScopedResource()
                    defer {
                        if secured {
                            url.stopAccessingSecurityScopedResource()
                        }
                    }
                    let values = try? url.resourceValues(forKeys: [.nameKey, .fileSizeKey])
                    let size = values?.fileSize.map(Int64.init)
                    if let maxSize, let size, size > maxSize {
                        return nil
                    }
                    return DowePickedFile(
                        id: url.absoluteString,
                        name: values?.name ?? url.lastPathComponent,
                        size: size
                    )
                }
                if multiple {
                    let known = Set(selectedFiles.map(\.id))
                    selectedFiles.append(contentsOf: files.filter { !known.contains($0.id) })
                } else {
                    selectedFiles = Array(files.prefix(1))
                }
            }
            if let text = errorText ?? helpText {
                Text(text)
                    .font(.system(size: CGFloat(13)))
                    .foregroundStyle(errorText == nil ? DoweDesign.muted : DoweDesign.danger)
            }
        }
    }
}

"#
}
