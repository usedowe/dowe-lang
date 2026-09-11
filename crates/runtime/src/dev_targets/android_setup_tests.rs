use super::*;
use tempfile::TempDir;

fn package(root: &Path, name: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "").unwrap();
}

#[test]
fn android_setup_installs_only_missing_build_packages() {
    let root = TempDir::new().unwrap();
    package(root.path(), "platform-tools/adb");
    package(root.path(), "platform-tools/adb.exe");
    package(root.path(), "platforms/android-36/android.jar");
    let missing = missing_android_packages(root.path());
    assert_eq!(missing, ["emulator", "build-tools;36.0.0"]);
}

#[test]
fn android_setup_selects_installed_image_for_host_architecture() {
    let root = TempDir::new().unwrap();
    package(
        root.path(),
        "system-images/android-33/google_apis/arm64-v8a/package.xml",
    );
    package(
        root.path(),
        "system-images/android-33/google_apis/arm64-v8a/system.img",
    );
    package(
        root.path(),
        "system-images/android-36/google_apis/x86_64/package.xml",
    );
    package(
        root.path(),
        "system-images/android-36/google_apis/x86_64/system.img",
    );
    package(
        root.path(),
        "system-images/android-99/google_apis/arm64-v8a/package.xml",
    );
    let selected = installed_android_image(root.path(), "arm64-v8a").unwrap();
    assert_eq!(selected, "system-images;android-33;google_apis;arm64-v8a");
}

#[test]
fn android_setup_creates_avd_without_overwriting_or_downloading_existing_image() {
    let root = TempDir::new().unwrap();
    let image = "system-images;android-33;google_apis;arm64-v8a";
    let mut calls = Vec::new();
    create_android_avd_with(
        root.path(),
        Path::new("sdkmanager"),
        Path::new("avdmanager"),
        Some(image.to_string()),
        "arm64-v8a",
        |config| {
            calls.push(config);
            Ok(Vec::new())
        },
    )
    .unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].command, "avdmanager");
    assert!(
        calls[0]
            .args
            .windows(2)
            .any(|pair| pair == ["--package", image])
    );
    assert!(
        !calls[0]
            .args
            .iter()
            .any(|arg| arg == "--force" || arg == "-f")
    );
    assert_eq!(calls[0].options.stdin, StreamMode::Ignore);
}

#[test]
fn android_setup_downloads_matching_image_before_creating_avd() {
    let root = TempDir::new().unwrap();
    let mut calls = Vec::new();
    create_android_avd_with(
        root.path(),
        Path::new("sdkmanager"),
        Path::new("avdmanager"),
        None,
        "x86_64",
        |config| {
            calls.push(config);
            Ok(Vec::new())
        },
    )
    .unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].command, "sdkmanager");
    assert!(
        calls[0]
            .args
            .contains(&"system-images;android-36;google_apis;x86_64".to_string())
    );
    assert_eq!(calls[1].command, "avdmanager");
}

#[test]
fn android_setup_does_not_create_avd_after_install_failure() {
    let root = TempDir::new().unwrap();
    let mut count = 0;
    let error = create_android_avd_with(
        root.path(),
        Path::new("sdkmanager"),
        Path::new("avdmanager"),
        None,
        "x86_64",
        |_| {
            count += 1;
            Err(RuntimeError::new("Android SDK download failed"))
        },
    )
    .unwrap_err();
    assert_eq!(count, 1);
    assert!(error.to_string().contains("download failed"));
}

#[test]
fn android_setup_reuses_complete_tools_when_newer_installation_is_partial() {
    let root = TempDir::new().unwrap();
    for name in ["aapt2", "zipalign"] {
        let path = executable_path(root.path().join("build-tools/35.0.0").join(name));
        package(
            root.path(),
            path.strip_prefix(root.path()).unwrap().to_str().unwrap(),
        );
    }
    for name in ["d8", "apksigner"] {
        let path = android_script_path(root.path().join("build-tools/35.0.0").join(name));
        package(
            root.path(),
            path.strip_prefix(root.path()).unwrap().to_str().unwrap(),
        );
    }
    fs::create_dir_all(root.path().join("build-tools/99.0.0")).unwrap();
    package(root.path(), "platforms/android-35/android.jar");
    fs::create_dir_all(root.path().join("platforms/android-99")).unwrap();
    let missing = missing_android_packages(root.path());
    assert!(!missing.iter().any(|package| package.starts_with("build-tools;") || package.starts_with("platforms;")));
}
