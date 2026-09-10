#[test]
fn generates_platform_icon_sets_from_one_svg() {
    let project = fixture();
    let report = generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#336699",
            IconRounded::Md,
        )
        .with_targets(IconTarget::ALL),
    )
    .expect("icons");

    assert_eq!(report.targets, IconTarget::ALL);
    assert!(project.path().join("icons/web/favicon.ico").is_file());
    assert!(project.path().join("icons/desktop/icon.icns").is_file());
    assert!(
        project
            .path()
            .join("icons/ios/AppIcon.appiconset/Contents.json")
            .is_file()
    );
    assert!(
        project
            .path()
            .join("icons/android/mipmap-anydpi-v26/ic_launcher.xml")
            .is_file()
    );
    assert_eq!(
        png_dimensions(&project.path().join("icons/ios/AppIcon.png")),
        (1024, 1024)
    );
    let ios_pixels = png_rgba(&project.path().join("icons/ios/AppIcon.png"));
    assert!(ios_pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    let ios_contents = fs::read_to_string(
        project
            .path()
            .join("icons/ios/AppIcon.appiconset/Contents.json"),
    )
    .expect("contents");
    let ios_contents: serde_json::Value = serde_json::from_str(&ios_contents).expect("json");
    let marketing = ios_contents["images"]
        .as_array()
        .expect("images")
        .iter()
        .find(|image| image["idiom"] == "ios-marketing")
        .expect("marketing icon");
    assert_eq!(marketing["scale"], "1x");

    let icon = IconDir::read(BufReader::new(
        fs::File::open(project.path().join("icons/web/favicon.ico")).expect("ico"),
    ))
    .expect("valid ico");
    assert_eq!(
        icon.entries()
            .iter()
            .map(|entry| entry.width())
            .collect::<Vec<_>>(),
        [16, 32, 48]
    );
    let icns = IconFamily::read(
        fs::File::open(project.path().join("icons/desktop/icon.icns")).expect("icns"),
    )
    .expect("valid icns");
    let available_icons = icns.available_icons();
    assert_eq!(available_icons.len(), 10);
    for icon_type in [
        IconType::RGBA32_16x16,
        IconType::RGBA32_16x16_2x,
        IconType::RGBA32_32x32,
        IconType::RGBA32_32x32_2x,
        IconType::RGBA32_128x128,
        IconType::RGBA32_128x128_2x,
        IconType::RGBA32_256x256,
        IconType::RGBA32_256x256_2x,
        IconType::RGBA32_512x512,
        IconType::RGBA32_512x512_2x,
    ] {
        assert!(available_icons.contains(&icon_type));
    }
    let macos = icns
        .get_icon_with_type(IconType::RGBA32_512x512)
        .expect("macos icon")
        .convert_to(PixelFormat::RGBA);
    let macos_pixels = macos.data();
    assert_eq!(&macos_pixels[0..4], &[0, 0, 0, 0]);
    assert_eq!(pixel(macos_pixels, 512, 256, 0), [0, 0, 0, 0]);
    assert_eq!(pixel(macos_pixels, 512, 256, 60), [51, 102, 153, 255]);
    assert_eq!(pixel(macos_pixels, 512, 256, 256), [255, 255, 255, 255]);
    let macos_surface = opaque_bounds(macos_pixels, 512, |value| value[3] > 8);
    assert!((0.80..=0.81).contains(&(macos_surface.width() as f32 / 512.0)));
    assert!((0.80..=0.81).contains(&(macos_surface.height() as f32 / 512.0)));
    assert!((macos_surface.center_x() - 255.5).abs() <= 1.0);
    assert!((macos_surface.center_y() - 255.5).abs() <= 1.0);
    let macos_logo = opaque_bounds(macos_pixels, 512, |value| {
        value[0] > 240 && value[1] > 240 && value[2] > 240
    });
    let logo_share = macos_logo.width() as f32 / macos_surface.width() as f32;
    assert!((0.69..=0.71).contains(&logo_share));
    assert!((macos_logo.center_x() - 255.5).abs() <= 1.0);
    assert!((macos_logo.center_y() - 255.5).abs() <= 1.0);

    let desktop_png = png_rgba(&project.path().join("icons/desktop/icon.png"));
    assert_eq!(&desktop_png[0..4], &[0, 0, 0, 0]);
    assert_eq!(pixel(&desktop_png, 512, 256, 0), [51, 102, 153, 255]);
}

#[test]
fn centers_svg_aspect_ratios_and_applies_rounded_background() {
    let project = fixture();
    generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#ff0000",
            IconRounded::Full,
        )
        .with_targets([IconTarget::Web, IconTarget::Android]),
    )
    .expect("icons");

    let web = png_rgba(&project.path().join("icons/web/favicon-48x48.png"));
    assert_eq!(&web[0..4], &[0, 0, 0, 0]);
    assert_eq!(pixel(&web, 48, 24, 2), [255, 0, 0, 255]);
    let white = opaque_bounds(&web, 48, |value| {
        value[0] > 240 && value[1] > 240 && value[2] > 240
    });
    assert!((white.center_x() - 23.5).abs() <= 1.0);
    assert!((white.center_y() - 23.5).abs() <= 1.0);
    assert!((1.7..=2.2).contains(&(white.width() as f32 / white.height() as f32)));
    assert!((0.86..=0.92).contains(&(white.width() as f32 / 48.0)));

    let web_icon = png_rgba(&project.path().join("icons/web/apple-touch-icon.png"));
    let web_icon_logo = opaque_bounds(&web_icon, 180, |value| {
        value[0] > 240 && value[1] > 240 && value[2] > 240
    });
    assert!((0.69..=0.71).contains(&(web_icon_logo.width() as f32 / 180.0)));

    let adaptive = png_rgba(
        &project
            .path()
            .join("icons/android/drawable-mdpi/ic_launcher_foreground.png"),
    );
    let center = 53.5_f32;
    let furthest = adaptive
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, value)| value[3] > 8)
        .map(|(index, _)| {
            let x = (index % 108) as f32;
            let y = (index / 108) as f32;
            ((x - center).powi(2) + (y - center).powi(2)).sqrt()
        })
        .fold(0.0_f32, f32::max);
    assert!(furthest <= 34.0, "adaptive foreground radius {furthest}");

    for (width, height, expected_ratio) in [(100, 200, 0.5_f32), (100, 100, 1.0_f32)] {
        fs::write(
            project.path().join("assets/icon.svg"),
            format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}"><path fill="#ffffff" d="M0 0h{width}v{height}H0z"/></svg>"##
            ),
        )
        .expect("svg");
        generate_project_icons(
            GenerateIconOptions::new(
                project.path(),
                "assets/icon.svg",
                "#ff0000",
                IconRounded::None,
            )
            .with_targets([IconTarget::Web]),
        )
        .expect("icons");
        let pixels = png_rgba(&project.path().join("icons/web/favicon-48x48.png"));
        let bounds = opaque_bounds(&pixels, 48, |value| {
            value[0] > 240 && value[1] > 240 && value[2] > 240
        });
        assert!((bounds.center_x() - 23.5).abs() <= 1.0);
        assert!((bounds.center_y() - 23.5).abs() <= 1.0);
        let ratio = bounds.width() as f32 / bounds.height() as f32;
        assert!((ratio - expected_ratio).abs() <= 0.12);
    }
}

#[test]
fn regeneration_is_deterministic_and_preserves_unselected_targets() {
    let project = fixture();
    let all = GenerateIconOptions::new(
        project.path(),
        "assets/icon.svg",
        "#ffffff",
        IconRounded::Lg,
    )
    .with_targets(IconTarget::ALL);
    generate_project_icons(all.clone()).expect("first");
    let first_png =
        fs::read(project.path().join("icons/web/favicon-32x32.png")).expect("first png");
    let first_manifest =
        fs::read(project.path().join("icons/manifest.json")).expect("first manifest");
    generate_project_icons(all).expect("second");
    assert_eq!(
        fs::read(project.path().join("icons/web/favicon-32x32.png")).expect("second png"),
        first_png
    );
    assert_eq!(
        fs::read(project.path().join("icons/manifest.json")).expect("second manifest"),
        first_manifest
    );

    fs::write(
        project.path().join("icons/ios/keep-until-ios-regenerates"),
        "preserved",
    )
    .expect("sentinel");
    generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#000000",
            IconRounded::Full,
        )
        .with_targets([IconTarget::Web]),
    )
    .expect("web only");
    assert!(
        project
            .path()
            .join("icons/ios/keep-until-ios-regenerates")
            .is_file()
    );

    let current_web =
        fs::read(project.path().join("icons/web/favicon-32x32.png")).expect("current web icon");
    let current_manifest =
        fs::read(project.path().join("icons/manifest.json")).expect("current manifest");
    fs::write(project.path().join("assets/icon.svg"), "not svg").expect("invalid svg");
    generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#ffffff",
            IconRounded::None,
        )
        .with_targets([IconTarget::Web]),
    )
    .expect_err("invalid render");
    assert_eq!(
        fs::read(project.path().join("icons/web/favicon-32x32.png")).expect("preserved web icon"),
        current_web
    );
    assert_eq!(
        fs::read(project.path().join("icons/manifest.json")).expect("preserved manifest"),
        current_manifest
    );
}
