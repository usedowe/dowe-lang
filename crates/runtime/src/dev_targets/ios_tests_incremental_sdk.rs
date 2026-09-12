#[test]
#[cfg(target_os = "macos")]
fn ios_batches_reuse_unchanged_objects_and_validate_sdk_availability() {
    use super::{IosHotModuleSnapshot, IosIncrementalWorkspace};
    use dowe_compiler::GeneratedFile;
    use std::process::Command;

    let sdk = Command::new("xcrun")
        .args(["--sdk", "iphonesimulator", "--show-sdk-path"])
        .output()
        .expect("Xcode SDK query");
    if !sdk.status.success() {
        eprintln!("skipping iOS incremental SDK test: simulator SDK unavailable");
        return;
    }

    let root = tempfile::tempdir().unwrap();
    let target = ios_simulator_target();
    let source = |path: &str, content: String| GeneratedFile {
        relative_path: Path::new(path).to_path_buf(),
        content,
        kind: "view".to_string(),
        target: "ios".to_string(),
    };
    let mut files = (0..4)
        .map(|index| {
            source(
                &format!("apps/ios/Page{index}.swift"),
                format!("func page{index}() -> Int {{ {index} }}"),
            )
        })
        .collect::<Vec<_>>();
    files.extend([
        source("apps/ios/dev/DoweIosDevHost.swift", "host".to_string()),
        source(
            "apps/ios/dev/DoweIosViewModule.swift",
            "let sourceRevision = \"__DOWE_IOS_SOURCE_REVISION__\"".to_string(),
        ),
    ]);
    let prepare = |files: &[GeneratedFile]| {
        let snapshot =
            IosHotModuleSnapshot::from_generated_files(files, &target, b"sdk-test").unwrap();
        IosIncrementalWorkspace::prepare(root.path(), &snapshot).unwrap()
    };
    let compile = |workspace: &IosIncrementalWorkspace| {
        std::process::Command::new("xcrun")
            .args(ios_hot_module_compile_args(
                &workspace.source_files(),
                &workspace.output_map,
                target.clone(),
                2,
            ))
            .output()
            .expect("Swift compiler")
    };
    let first = prepare(&files);
    let compiled = compile(&first);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(first.is_complete());
    let times = first
        .object_files()
        .iter()
        .map(|path| fs::metadata(path).unwrap().modified().unwrap())
        .collect::<Vec<_>>();

    files[0].content = "func page0() -> Int { 42 }".to_string();
    let second = prepare(&files);
    let compiled = compile(&second);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(second.is_complete());
    let reused = second
        .object_files()
        .iter()
        .zip(times)
        .filter(|(path, previous)| fs::metadata(path).unwrap().modified().unwrap() == *previous)
        .count();
    assert_eq!(
        reused, 3,
        "only the edited page and module revision may rebuild"
    );
    let linked = std::process::Command::new("xcrun")
        .args(ios_hot_module_link_args(
            &second.object_files(),
            &second.linked_module,
            target.clone(),
        ))
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );

    files[0].content = "@available(iOS 9999, *)\nfunc futureValue() -> Int { 42 }\nfunc page0() -> Int { futureValue() }".to_string();
    let invalid = compile(&prepare(&files));
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("9999"));
}
