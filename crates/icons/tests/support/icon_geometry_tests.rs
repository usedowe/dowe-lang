#[test]
fn generates_every_rounded_value() {
    let project = fixture();
    for rounded in IconRounded::ALL {
        generate_project_icons(
            GenerateIconOptions::new(project.path(), "assets/icon.svg", "#abcdef", rounded)
                .with_targets([IconTarget::Web]),
        )
        .expect("rounded icons");
        assert!(project.path().join("icons/web/favicon-32x32.png").is_file());
    }
}

#[test]
fn draws_the_macos_surface_on_the_apple_icon_grid() {
    let project = fixture();
    fs::write(
        project.path().join("assets/icon.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><path fill="#ffffff" d="M0 0h100v100H0z"/></svg>"##,
    )
    .expect("square svg");
    generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#336699",
            IconRounded::Full,
        )
        .with_targets([IconTarget::Desktop]),
    )
    .expect("desktop icons");

    let first_icns = fs::read(project.path().join("icons/desktop/icon.icns")).expect("first icns");
    let icns = IconFamily::read(first_icns.as_slice()).expect("valid icns");
    let macos = icns
        .get_icon_with_type(IconType::RGBA32_512x512)
        .expect("macos icon")
        .convert_to(PixelFormat::RGBA);
    let pixels = macos.data();
    assert_eq!(pixel(pixels, 512, 0, 0), [0, 0, 0, 0]);
    assert_eq!(pixel(pixels, 512, 256, 0), [0, 0, 0, 0]);
    assert_eq!(pixel(pixels, 512, 256, 256), [255, 255, 255, 255]);
    let surface = opaque_bounds(pixels, 512, |value| value[3] > 8);
    assert!((0.80..=0.81).contains(&(surface.width() as f32 / 512.0)));
    assert!((0.80..=0.81).contains(&(surface.height() as f32 / 512.0)));
    assert!((surface.center_x() - 255.5).abs() <= 1.0);
    assert!((surface.center_y() - 255.5).abs() <= 1.0);
    assert_eq!(pixel(pixels, 512, 60, 60), [0, 0, 0, 0]);

    generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#ff0000",
            IconRounded::None,
        )
        .with_targets([IconTarget::Desktop]),
    )
    .expect("regenerated desktop icons");
    let second_icns =
        fs::read(project.path().join("icons/desktop/icon.icns")).expect("second icns");
    assert_ne!(second_icns, first_icns);
    let icns = IconFamily::read(second_icns.as_slice()).expect("valid icns");
    let macos = icns
        .get_icon_with_type(IconType::RGBA32_512x512)
        .expect("macos icon")
        .convert_to(PixelFormat::RGBA);
    let pixels = macos.data();
    assert_eq!(pixel(pixels, 512, 0, 0), [0, 0, 0, 0]);
    assert_eq!(pixel(pixels, 512, 60, 60), [255, 0, 0, 255]);
    let surface = opaque_bounds(pixels, 512, |value| value[3] > 8);
    assert!((0.80..=0.81).contains(&(surface.width() as f32 / 512.0)));

    let desktop_png = png_rgba(&project.path().join("icons/desktop/icon.png"));
    assert_eq!(pixel(&desktop_png, 512, 0, 0), [255, 0, 0, 255]);
}

#[test]
fn rejects_unsafe_sources_and_invalid_options_before_writing() {
    let project = fixture();
    fs::write(
        project.path().join("assets/external.svg"),
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M0 0h10v10z"/><image href=" /tmp/logo.png "/></svg>"#,
    )
    .expect("external svg");
    let cases = [
        ("../icon.svg", "#ffffff", "project-relative"),
        ("icons/source.svg", "#ffffff", "icons"),
        ("assets/icon.svg", "white", "#RRGGBB"),
        ("assets/external.svg", "#ffffff", "self-contained"),
    ];

    for (source, background, message) in cases {
        let result = generate_project_icons(
            GenerateIconOptions::new(project.path(), source, background, IconRounded::None)
                .with_targets([IconTarget::Web]),
        );
        assert!(result.expect_err("error").to_string().contains(message));
    }
    assert!(!project.path().join("icons/web").exists());
}

#[test]
fn accepts_svg_doctype_without_resolving_external_entities() {
    let project = fixture();
    fs::write(
        project.path().join("assets/icon.svg"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M0 0h10v10H0z"/></svg>"#,
    )
    .expect("svg with doctype");

    generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#ffffff",
            IconRounded::None,
        )
        .with_targets([IconTarget::Web]),
    )
    .expect("doctype icon");
    assert!(project.path().join("icons/web/favicon-32x32.png").is_file());

    fs::write(
        project.path().join("assets/icon.svg"),
        r#"<?xml version="1.0"?>
<!DOCTYPE svg [<!ENTITY external SYSTEM "file:///etc/passwd">]>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><text>&external;</text></svg>"#,
    )
    .expect("svg with external entity");
    let error = generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#ffffff",
            IconRounded::None,
        )
        .with_targets([IconTarget::Web]),
    )
    .expect_err("external entity");
    assert!(error.to_string().contains("invalid icon SVG"));
}

#[cfg(unix)]
#[test]
fn rejects_source_and_output_symlink_escapes() {
    let project = fixture();
    let outside = TempDir::new().expect("outside");
    fs::write(
        outside.path().join("outside.svg"),
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"><path d="M0 0h1v1z"/></svg>"#,
    )
    .expect("outside svg");
    std::os::unix::fs::symlink(
        outside.path().join("outside.svg"),
        project.path().join("assets/outside.svg"),
    )
    .expect("source symlink");
    let source_error = generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/outside.svg",
            "#ffffff",
            IconRounded::None,
        )
        .with_targets([IconTarget::Web]),
    )
    .expect_err("source escape");
    assert!(source_error.to_string().contains("inside the project"));

    std::os::unix::fs::symlink(outside.path(), project.path().join("icons"))
        .expect("output symlink");
    let output_error = generate_project_icons(
        GenerateIconOptions::new(
            project.path(),
            "assets/icon.svg",
            "#ffffff",
            IconRounded::None,
        )
        .with_targets([IconTarget::Web]),
    )
    .expect_err("output escape");
    assert!(output_error.to_string().contains("cannot use symlinks"));
    assert!(!outside.path().join("web").exists());
}

