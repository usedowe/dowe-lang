use crate::artifact::IconArtifact;
use crate::render::{IconRenderer, RenderStyle};
use crate::{IconColor, IconError, IconResult, IconRounded};
use icns::{IconFamily, IconType, Image};
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use std::collections::BTreeMap;
use std::io::Cursor;

pub(crate) fn web_artifacts(
    renderer: &IconRenderer,
    background: IconColor,
    rounded: IconRounded,
) -> IconResult<Vec<IconArtifact>> {
    let style = composite_style(background, rounded);
    let rendered = render_sizes(renderer, &[16, 32, 48, 180, 192, 512], style)?;
    let mut artifacts = vec![
        IconArtifact::new("favicon.ico", encode_ico(&rendered, &[16, 32, 48])?),
        png_artifact("favicon-16x16.png", &rendered, 16),
        png_artifact("favicon-32x32.png", &rendered, 32),
        png_artifact("favicon-48x48.png", &rendered, 48),
        png_artifact("apple-touch-icon.png", &rendered, 180),
        png_artifact("icon-192x192.png", &rendered, 192),
        png_artifact("icon-512x512.png", &rendered, 512),
    ];
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

pub(crate) fn desktop_artifacts(
    renderer: &IconRenderer,
    background: IconColor,
    rounded: IconRounded,
) -> IconResult<Vec<IconArtifact>> {
    let style = composite_style(background, rounded);
    let sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024];
    let rendered = render_sizes(renderer, &sizes, style)?;
    let mut artifacts = vec![
        IconArtifact::new(
            "icon.ico",
            encode_ico(&rendered, &[16, 24, 32, 48, 64, 128, 256])?,
        ),
        IconArtifact::new(
            "icon.icns",
            encode_icns(&rendered, &[16, 32, 64, 128, 256, 512, 1024])?,
        ),
        png_artifact("icon.png", &rendered, 512),
    ];
    for size in [16, 32, 64, 128, 256, 512, 1024] {
        artifacts.push(png_artifact(
            &format!("png/icon-{size}x{size}.png"),
            &rendered,
            size,
        ));
    }
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

fn composite_style(background: IconColor, rounded: IconRounded) -> RenderStyle {
    RenderStyle {
        background: Some(background),
        radius: rounded.ratio(),
        logo_scale: 0.7,
        circular_safe_zone: None,
    }
}

fn render_sizes(
    renderer: &IconRenderer,
    sizes: &[u32],
    style: RenderStyle,
) -> IconResult<BTreeMap<u32, Vec<u8>>> {
    sizes
        .iter()
        .copied()
        .map(|size| renderer.png(size, style).map(|png| (size, png)))
        .collect()
}

fn png_artifact(path: &str, rendered: &BTreeMap<u32, Vec<u8>>, size: u32) -> IconArtifact {
    IconArtifact::new(path, rendered[&size].clone())
}

fn encode_ico(rendered: &BTreeMap<u32, Vec<u8>>, sizes: &[u32]) -> IconResult<Vec<u8>> {
    let mut directory = IconDir::new(ResourceType::Icon);
    for size in sizes {
        let image = IconImage::read_png(Cursor::new(&rendered[size]))
            .map_err(|error| IconError::new(format!("failed to decode generated PNG: {error}")))?;
        let entry = IconDirEntry::encode(&image)
            .map_err(|error| IconError::new(format!("failed to encode ICO: {error}")))?;
        directory.add_entry(entry);
    }
    let mut output = Cursor::new(Vec::new());
    directory
        .write(&mut output)
        .map_err(|error| IconError::new(format!("failed to write ICO: {error}")))?;
    Ok(output.into_inner())
}

fn encode_icns(rendered: &BTreeMap<u32, Vec<u8>>, sizes: &[u32]) -> IconResult<Vec<u8>> {
    let mut family = IconFamily::new();
    for size in sizes {
        let image = Image::read_png(Cursor::new(&rendered[size]))
            .map_err(|error| IconError::new(format!("failed to decode generated PNG: {error}")))?;
        let icon_type = IconType::from_pixel_size(*size, *size).ok_or_else(|| {
            IconError::new(format!("ICNS does not support a {size}x{size} image"))
        })?;
        family
            .add_icon_with_type(&image, icon_type)
            .map_err(|error| IconError::new(format!("failed to encode ICNS: {error}")))?;
    }
    let mut output = Vec::new();
    family
        .write(&mut output)
        .map_err(|error| IconError::new(format!("failed to write ICNS: {error}")))?;
    Ok(output)
}
