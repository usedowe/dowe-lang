use crate::artifact::IconArtifact;
use crate::render::{IconRenderer, RenderStyle};
use crate::{IconColor, IconResult, IconRounded};
use serde_json::json;
use std::collections::BTreeMap;

pub(crate) fn ios_artifacts(
    renderer: &IconRenderer,
    background: IconColor,
) -> IconResult<Vec<IconArtifact>> {
    let style = RenderStyle {
        background: Some(background),
        radius: 0.0,
        logo_scale: 0.7,
        circular_safe_zone: None,
    };
    let entries = ios_entries();
    let sizes = entries.iter().map(|entry| entry.pixels).collect::<Vec<_>>();
    let rendered = render_sizes(renderer, &sizes, style)?;
    let mut images = Vec::new();
    let mut artifacts = Vec::new();
    for entry in entries {
        artifacts.push(IconArtifact::new(
            format!("AppIcon.appiconset/{}", entry.file_name),
            rendered[&entry.pixels].clone(),
        ));
        let image = json!({
            "filename": entry.file_name,
            "idiom": entry.idiom,
            "scale": entry.scale,
            "size": entry.size,
        });
        images.push(image);
    }
    let mut contents = serde_json::to_vec_pretty(&json!({
        "images": images,
        "info": {"author": "dowe", "version": 1}
    }))?;
    contents.push(b'\n');
    artifacts.push(IconArtifact::new(
        "AppIcon.appiconset/Contents.json",
        contents,
    ));
    artifacts.push(IconArtifact::new("AppIcon.png", rendered[&1024].clone()));
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

pub(crate) fn android_artifacts(
    renderer: &IconRenderer,
    background: IconColor,
    rounded: IconRounded,
) -> IconResult<Vec<IconArtifact>> {
    let densities = [
        ("mdpi", 48, 108),
        ("hdpi", 72, 162),
        ("xhdpi", 96, 216),
        ("xxhdpi", 144, 324),
        ("xxxhdpi", 192, 432),
    ];
    let legacy_style = RenderStyle {
        background: Some(background),
        radius: rounded.ratio(),
        logo_scale: 0.7,
        circular_safe_zone: None,
    };
    let round_style = RenderStyle {
        background: Some(background),
        radius: 0.5,
        logo_scale: 0.7,
        circular_safe_zone: None,
    };
    let foreground_style = RenderStyle {
        background: None,
        radius: 0.0,
        logo_scale: 1.0,
        circular_safe_zone: Some(66.0 / 108.0),
    };
    let mut artifacts = Vec::new();
    for (density, legacy_size, adaptive_size) in densities {
        artifacts.push(IconArtifact::new(
            format!("mipmap-{density}/ic_launcher.png"),
            renderer.png(legacy_size, legacy_style)?,
        ));
        artifacts.push(IconArtifact::new(
            format!("mipmap-{density}/ic_launcher_round.png"),
            renderer.png(legacy_size, round_style)?,
        ));
        artifacts.push(IconArtifact::new(
            format!("drawable-{density}/ic_launcher_foreground.png"),
            renderer.png(adaptive_size, foreground_style)?,
        ));
    }
    let background_xml = format!(
        "<shape xmlns:android=\"http://schemas.android.com/apk/res/android\" android:shape=\"rectangle\">\n    <solid android:color=\"{}\" />\n</shape>\n",
        background.hex()
    );
    let adaptive_xml = "<adaptive-icon xmlns:android=\"http://schemas.android.com/apk/res/android\">\n    <background android:drawable=\"@drawable/ic_launcher_background\" />\n    <foreground android:drawable=\"@drawable/ic_launcher_foreground\" />\n</adaptive-icon>\n";
    artifacts.extend([
        IconArtifact::new("drawable/ic_launcher_background.xml", background_xml),
        IconArtifact::new("mipmap-anydpi-v26/ic_launcher.xml", adaptive_xml),
        IconArtifact::new("mipmap-anydpi-v26/ic_launcher_round.xml", adaptive_xml),
    ]);
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

fn render_sizes(
    renderer: &IconRenderer,
    sizes: &[u32],
    style: RenderStyle,
) -> IconResult<BTreeMap<u32, Vec<u8>>> {
    sizes
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|size| renderer.png(size, style).map(|png| (size, png)))
        .collect()
}

struct IosEntry {
    idiom: &'static str,
    size: &'static str,
    scale: &'static str,
    pixels: u32,
    file_name: String,
}

fn ios_entries() -> Vec<IosEntry> {
    let values = [
        ("iphone", "20x20", "2x", 40),
        ("iphone", "20x20", "3x", 60),
        ("iphone", "29x29", "2x", 58),
        ("iphone", "29x29", "3x", 87),
        ("iphone", "40x40", "2x", 80),
        ("iphone", "40x40", "3x", 120),
        ("iphone", "60x60", "2x", 120),
        ("iphone", "60x60", "3x", 180),
        ("ipad", "20x20", "1x", 20),
        ("ipad", "20x20", "2x", 40),
        ("ipad", "29x29", "1x", 29),
        ("ipad", "29x29", "2x", 58),
        ("ipad", "40x40", "1x", 40),
        ("ipad", "40x40", "2x", 80),
        ("ipad", "76x76", "1x", 76),
        ("ipad", "76x76", "2x", 152),
        ("ipad", "83.5x83.5", "2x", 167),
        ("ios-marketing", "1024x1024", "1x", 1024),
    ];
    values
        .into_iter()
        .map(|(idiom, size, scale, pixels)| IosEntry {
            idiom,
            size,
            scale,
            pixels,
            file_name: format!("AppIcon-{}-{}-{}.png", idiom, size.replace('.', "_"), scale),
        })
        .collect()
}
