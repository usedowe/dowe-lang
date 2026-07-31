#[test]
fn synchronizes_project_icons_into_generated_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Text\n    \"Home\"",
    );
    write_icon_fixture(temp.path());

    let project = compile_dev(temp.path()).expect("project");

    assert!(
        project.web.pages[0]
            .html_document
            .contains(r#"href="/icons/web/favicon-32x32.png""#)
    );
    assert!(
        project.web.pages[0]
            .html_document
            .contains(r#"rel="apple-touch-icon" href="/icons/web/apple-touch-icon.png""#)
    );
    for path in [
        ".dowe/web/icons/web/favicon-32x32.png",
        ".dowe/apps/desktop/web/icons/web/favicon-32x32.png",
        ".dowe/apps/desktop/macos/icon.icns",
        ".dowe/apps/desktop/windows/icon.ico",
        ".dowe/apps/desktop/linux/icon.png",
        ".dowe/apps/ios/AppIcon.png",
        ".dowe/apps/ios/Assets.xcassets/AppIcon.appiconset/Contents.json",
        ".dowe/apps/android/app/src/main/res/mipmap-mdpi/ic_launcher.png",
        ".dowe/apps/android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml",
    ] {
        assert!(temp.path().join(path).is_file(), "missing {path}");
    }
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/AndroidManifest.xml"),
    )
    .expect("android manifest");
    let android_dev = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/dev/AndroidManifest.xml"),
    )
    .expect("android dev manifest");
    let ios = fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("ios plist");
    assert!(android.contains(r#"android:icon="@mipmap/ic_launcher""#));
    assert!(android.contains(r#"android:roundIcon="@mipmap/ic_launcher_round""#));
    assert!(android_dev.contains(r#"android:icon="@mipmap/ic_launcher""#));
    assert!(ios.contains("<key>CFBundleIconName</key>"));
    assert!(ios.contains("<string>AppIcon</string>"));
    assert!(ios.contains("<key>CFBundleIcons</key>"));
    assert!(ios.contains("<key>CFBundleIcons~ipad</key>"));
    assert!(ios.contains("<string>AppIcon60x60</string>"));
    assert!(ios.contains("<string>AppIcon76x76</string>"));
}

#[test]
fn removes_stale_generated_icon_copies_when_source_set_disappears() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Text\n    \"Home\"",
    );
    write_icon_fixture(temp.path());
    compile_dev(temp.path()).expect("first compile");

    fs::remove_dir_all(temp.path().join("icons")).expect("remove icons");
    let project = compile_dev(temp.path()).expect("second compile");

    assert!(!temp.path().join(".dowe/web/icons").exists());
    assert!(
        !temp
            .path()
            .join(".dowe/apps/desktop/macos/icon.icns")
            .exists()
    );
    assert!(!temp.path().join(".dowe/apps/ios/Assets.xcassets").exists());
    assert!(
        !temp
            .path()
            .join(".dowe/apps/android/app/src/main/res/mipmap-mdpi/ic_launcher.png")
            .exists()
    );
    assert!(
        project.web.pages[0]
            .html_document
            .contains("data:image/svg+xml")
    );
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_project_icon_outputs() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Text\n    \"Home\"",
    );
    let outside = temp.path().join("outside.png");
    fs::write(&outside, "icon").expect("outside");
    let favicon = temp.path().join("icons/web/favicon-32x32.png");
    fs::create_dir_all(favicon.parent().expect("parent")).expect("icon directory");
    std::os::unix::fs::symlink(outside, favicon).expect("symlink");

    let error = compile_dev(temp.path()).expect_err("symlink error");

    assert!(
        error
            .to_string()
            .contains("icon output cannot contain symlinks")
    );
}

fn write_icon_fixture(root: &Path) {
    let paths = [
        "web/favicon-32x32.png",
        "web/apple-touch-icon.png",
        "desktop/icon.icns",
        "desktop/icon.ico",
        "desktop/icon.png",
        "ios/AppIcon.png",
        "ios/AppIcon.appiconset/Contents.json",
        "android/mipmap-mdpi/ic_launcher.png",
        "android/mipmap-mdpi/ic_launcher_round.png",
        "android/mipmap-anydpi-v26/ic_launcher.xml",
        "android/mipmap-anydpi-v26/ic_launcher_round.xml",
        "android/drawable/ic_launcher_background.xml",
    ];
    for path in paths {
        let output = root.join("icons").join(path);
        fs::create_dir_all(output.parent().expect("parent")).expect("icon directory");
        fs::write(output, path.as_bytes()).expect("icon");
    }
}
