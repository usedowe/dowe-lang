fn ios_build_root(project_root: &Path) -> PathBuf {
    project_root
        .join(".dowe/dev/ios/build")
        .join(std::process::id().to_string())
}

fn copy_ios_resources(ios_root: &Path, bundle: &Path) -> RuntimeResult<()> {
    let fonts = ios_root.join("Fonts");
    if fonts.is_dir() {
        copy_dir(&fonts, &bundle.join("Fonts"))?;
    }
    let assets = ios_root.join("assets");
    if assets.is_dir() {
        copy_dir(&assets, &bundle.join("assets"))?;
    }
    for entry in fs::read_dir(ios_root)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("lproj")
            && let Some(name) = path.file_name()
        {
            copy_dir(&path, &bundle.join(name))?;
        }
    }
    Ok(())
}

fn compile_ios_asset_catalog(
    ios_root: &Path,
    bundle: &Path,
    build_root: &Path,
) -> RuntimeResult<()> {
    let catalog = ios_root.join("Assets.xcassets");
    if !catalog.is_dir() {
        return Ok(());
    }
    run_ios_required(
        SpawnConfig::new(
            "xcrun",
            ios_asset_catalog_args(
                &catalog,
                bundle,
                &build_root.join("asset-catalog-info.plist"),
            ),
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(())
}

fn ios_asset_catalog_args(catalog: &Path, bundle: &Path, partial_plist: &Path) -> Vec<String> {
    vec![
        "actool".to_string(),
        catalog.to_string_lossy().to_string(),
        "--compile".to_string(),
        bundle.to_string_lossy().to_string(),
        "--platform".to_string(),
        "iphonesimulator".to_string(),
        "--minimum-deployment-target".to_string(),
        "17.0".to_string(),
        "--target-device".to_string(),
        "iphone".to_string(),
        "--target-device".to_string(),
        "ipad".to_string(),
        "--app-icon".to_string(),
        "AppIcon".to_string(),
        "--output-partial-info-plist".to_string(),
        partial_plist.to_string_lossy().to_string(),
    ]
}

fn copy_dir(source: &Path, destination: &Path) -> RuntimeResult<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        let target = destination.join(path.file_name().expect("directory entry has a file name"));
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(path, target)?;
        }
    }
    Ok(())
}
