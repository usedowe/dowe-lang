fn ios_toolchain_signature() -> RuntimeResult<Vec<u8>> {
    static SIGNATURE: OnceLock<Vec<u8>> = OnceLock::new();
    if let Some(signature) = SIGNATURE.get() {
        return Ok(signature.clone());
    }
    let mut signature = b"dowe-ios-cache-v1".to_vec();
    for args in ios_toolchain_signature_commands() {
        let output = run_ios_required(
            SpawnConfig::new("xcrun", args.clone())
                .with_options(quiet_command_options(None, StreamMode::Pipe)),
        )?;
        let command = args.join("\0");
        signature.extend(command.len().to_le_bytes());
        signature.extend(command.as_bytes());
        signature.extend(output.stdout_bytes.len().to_le_bytes());
        signature.extend(output.stdout_bytes);
    }
    let _ = SIGNATURE.set(signature.clone());
    Ok(signature)
}

fn ios_toolchain_signature_commands() -> Vec<Vec<&'static str>> {
    vec![
        vec!["swiftc", "--version"],
        vec!["xcodebuild", "-version"],
        vec!["--sdk", "iphonesimulator", "--show-sdk-path"],
        vec!["--sdk", "iphonesimulator", "--show-sdk-version"],
    ]
}

fn ios_swift_job_count() -> usize {
    std::thread::available_parallelism()
        .map(|value| bounded_ios_swift_job_count(value.get()))
        .unwrap_or(2)
}

fn bounded_ios_swift_job_count(parallelism: usize) -> usize {
    parallelism.clamp(1, 2)
}

fn ios_swift_object_files(swift_files: &[String], objects_root: &Path) -> Vec<PathBuf> {
    swift_files
        .iter()
        .map(|file| {
            let stem = Path::new(file)
                .file_stem()
                .and_then(|value| value.to_str())
                .expect("Swift source has a file stem");
            objects_root.join(format!("{stem}.o"))
        })
        .collect()
}

fn ios_swift_output_map(swift_files: &[String], object_files: &[PathBuf]) -> Value {
    let entries = swift_files
        .iter()
        .zip(object_files)
        .map(|(source, object)| {
            let mut outputs = serde_json::Map::new();
            outputs.insert(
                "object".to_string(),
                Value::String(object.to_string_lossy().to_string()),
            );
            (source.clone(), Value::Object(outputs))
        })
        .collect();
    Value::Object(entries)
}

fn ios_swift_compile_args(
    swift_files: &[String],
    output_map: &Path,
    target: String,
    jobs: usize,
) -> Vec<String> {
    let jobs = jobs.to_string();
    let mut args = vec![
        "--sdk".to_string(),
        "iphonesimulator".to_string(),
        "swiftc".to_string(),
        "-parse-as-library".to_string(),
        "-enable-batch-mode".to_string(),
        "-driver-batch-size-limit".to_string(),
        "1".to_string(),
        "-target".to_string(),
        target,
        "-j".to_string(),
        jobs,
        "-c".to_string(),
    ];
    args.extend(swift_files.iter().cloned());
    args.extend([
        "-output-file-map".to_string(),
        output_map.to_string_lossy().to_string(),
    ]);
    args
}

fn ios_swift_link_args(object_files: &[PathBuf], bundle: &Path, target: String) -> Vec<String> {
    let mut args = vec![
        "--sdk".to_string(),
        "iphonesimulator".to_string(),
        "swiftc".to_string(),
        "-target".to_string(),
        target,
    ];
    args.extend(
        object_files
            .iter()
            .map(|path| path.to_string_lossy().to_string()),
    );
    args.extend([
        "-o".to_string(),
        bundle.join("DoweIosApp").to_string_lossy().to_string(),
    ]);
    args
}
