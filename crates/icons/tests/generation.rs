use dowe_icons::{GenerateIconOptions, IconRounded, IconTarget, generate_project_icons};
use icns::{IconFamily, IconType, PixelFormat};
use ico::IconDir;
use std::fs;
use std::io::BufReader;
use tempfile::TempDir;

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
    assert!(
        project
            .path()
            .join("icons/web/favicon.ico")
            .is_file()
    );
    assert!(
        project
            .path()
            .join("icons/desktop/icon.icns")
            .is_file()
    );
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
        project
            .path()
            .join("icons/ios/keep-until-ios-regenerates"),
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

    let current_web = fs::read(project.path().join("icons/web/favicon-32x32.png"))
        .expect("current web icon");
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
        fs::read(project.path().join("icons/web/favicon-32x32.png"))
            .expect("preserved web icon"),
        current_web
    );
    assert_eq!(
        fs::read(project.path().join("icons/manifest.json")).expect("preserved manifest"),
        current_manifest
    );
}

#[test]
fn generates_every_rounded_value() {
    let project = fixture();
    for rounded in IconRounded::ALL {
        generate_project_icons(
            GenerateIconOptions::new(project.path(), "assets/icon.svg", "#abcdef", rounded)
                .with_targets([IconTarget::Web]),
        )
        .expect("rounded icons");
        assert!(
            project
                .path()
                .join("icons/web/favicon-32x32.png")
                .is_file()
        );
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

    let first_icns =
        fs::read(project.path().join("icons/desktop/icon.icns")).expect("first icns");
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
    assert!(
        project
            .path()
            .join("icons/web/favicon-32x32.png")
            .is_file()
    );

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

fn fixture() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("assets")).expect("assets");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(
        temp.path().join("assets/icon.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100"><path fill="#ffffff" d="M0 0h200v100H0z"/></svg>"##,
    )
    .expect("svg");
    temp
}

fn png_dimensions(path: &std::path::Path) -> (u32, u32) {
    let decoder = png::Decoder::new(BufReader::new(fs::File::open(path).expect("png")));
    let reader = decoder.read_info().expect("valid png");
    (reader.info().width, reader.info().height)
}

fn png_rgba(path: &std::path::Path) -> Vec<u8> {
    let decoder = png::Decoder::new(BufReader::new(fs::File::open(path).expect("png")));
    let mut reader = decoder.read_info().expect("valid png");
    let mut output = vec![0; reader.output_buffer_size().expect("buffer size")];
    let info = reader.next_frame(&mut output).expect("frame");
    assert_eq!(info.color_type, png::ColorType::Rgba);
    output.truncate(info.buffer_size());
    output
}

fn pixel(data: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    data[offset..offset + 4].try_into().expect("pixel")
}

struct PixelBounds {
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
}

impl PixelBounds {
    fn width(&self) -> usize {
        self.max_x - self.min_x + 1
    }

    fn height(&self) -> usize {
        self.max_y - self.min_y + 1
    }

    fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) as f32 * 0.5
    }

    fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) as f32 * 0.5
    }
}

fn opaque_bounds(data: &[u8], width: usize, predicate: impl Fn(&[u8]) -> bool) -> PixelBounds {
    let mut bounds = PixelBounds {
        min_x: width,
        min_y: width,
        max_x: 0,
        max_y: 0,
    };
    for (index, value) in data.chunks_exact(4).enumerate() {
        if predicate(value) && value[3] > 8 {
            let x = index % width;
            let y = index / width;
            bounds.min_x = bounds.min_x.min(x);
            bounds.min_y = bounds.min_y.min(y);
            bounds.max_x = bounds.max_x.max(x);
            bounds.max_y = bounds.max_y.max(y);
        }
    }
    assert!(bounds.min_x <= bounds.max_x);
    bounds
}
