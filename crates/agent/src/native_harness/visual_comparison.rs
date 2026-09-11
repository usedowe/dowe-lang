use crate::{AgentError, AgentResult};
use png::{BitDepth, ColorType, Decoder, Encoder, Transformations};
use serde::Serialize;
use serde_json::json;
use std::io::Cursor;

const MAX_DECODE_BYTES: usize = 64 * 1024 * 1024;
const CHANNEL_THRESHOLD: u8 = 16;
const MAX_MISMATCH_RATIO: f64 = 0.08;

#[derive(Debug, Clone)]
pub(crate) struct DecodedPng {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ComparisonSummary {
    pub status: &'static str,
    pub width: u32,
    pub height: u32,
    pub pixel_count: usize,
    pub mismatch_count: usize,
    pub mismatch_ratio: f64,
    pub average_channel_delta: f64,
    pub max_channel_delta: u8,
    pub threshold: u8,
    pub max_mismatch_ratio: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct Comparison {
    pub summary: ComparisonSummary,
    pub diff_rgba: Vec<u8>,
}

pub(crate) fn decode_png(bytes: &[u8]) -> AgentResult<DecodedPng> {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::normalize_to_color8());
    decoder.set_limits(png::Limits {
        bytes: MAX_DECODE_BYTES,
    });
    let mut reader = decoder
        .read_info()
        .map_err(|error| AgentError::new(format!("PNG decode failed: {error}")))?;
    let mut buffer = vec![
        0;
        reader.output_buffer_size().ok_or_else(|| {
            AgentError::new("PNG output buffer size is unavailable")
        })?
    ];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| AgentError::new(format!("PNG frame decode failed: {error}")))?;
    if info.width == 0
        || info.height == 0
        || info.width > 4096
        || info.height > 4096
        || usize::try_from(info.width)
            .ok()
            .and_then(|width| {
                usize::try_from(info.height)
                    .ok()
                    .map(|height| width * height)
            })
            .is_none_or(|pixels| pixels > 4096 * 4096)
    {
        return Err(AgentError::new(
            "PNG dimensions exceed the bounded visual comparison limit",
        ));
    }
    let data = &buffer[..info.buffer_size()];
    let mut rgba = Vec::with_capacity(info.width as usize * info.height as usize * 4);
    match (info.color_type, info.bit_depth) {
        (ColorType::Rgb, BitDepth::Eight) => {
            for pixel in data.chunks_exact(3) {
                rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        (ColorType::Rgba, BitDepth::Eight) => rgba.extend_from_slice(data),
        (ColorType::Grayscale, BitDepth::Eight) => {
            for &value in data {
                rgba.extend_from_slice(&[value, value, value, 255]);
            }
        }
        (ColorType::GrayscaleAlpha, BitDepth::Eight) => {
            for pixel in data.chunks_exact(2) {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
            }
        }
        (color, depth) => {
            return Err(AgentError::new(format!(
                "PNG comparison requires normalized 8-bit color, got {color:?}/{depth:?}"
            )));
        }
    }
    Ok(DecodedPng {
        width: info.width,
        height: info.height,
        rgba,
    })
}

pub(crate) fn compare_pngs(reference: &[u8], rendered: &[u8]) -> AgentResult<Comparison> {
    let reference = decode_png(reference)?;
    let rendered = decode_png(rendered)?;
    if reference.width != rendered.width || reference.height != rendered.height {
        return Ok(Comparison {
            summary: ComparisonSummary {
                status: "failed",
                width: rendered.width,
                height: rendered.height,
                pixel_count: rendered.width as usize * rendered.height as usize,
                mismatch_count: rendered.width as usize * rendered.height as usize,
                mismatch_ratio: 1.0,
                average_channel_delta: 255.0,
                max_channel_delta: 255,
                threshold: CHANNEL_THRESHOLD,
                max_mismatch_ratio: MAX_MISMATCH_RATIO,
            },
            diff_rgba: Vec::new(),
        });
    }

    let pixel_count = reference.width as usize * reference.height as usize;
    let mut mismatch_count = 0;
    let mut total_delta = 0_u64;
    let mut max_channel_delta = 0_u8;
    let mut diff_rgba = Vec::with_capacity(reference.rgba.len());
    for (reference_pixel, rendered_pixel) in reference
        .rgba
        .chunks_exact(4)
        .zip(rendered.rgba.chunks_exact(4))
    {
        let deltas = [
            reference_pixel[0].abs_diff(rendered_pixel[0]),
            reference_pixel[1].abs_diff(rendered_pixel[1]),
            reference_pixel[2].abs_diff(rendered_pixel[2]),
            reference_pixel[3].abs_diff(rendered_pixel[3]),
        ];
        let max_delta = *deltas.iter().max().unwrap_or(&0);
        let delta = deltas.iter().map(|value| u64::from(*value)).sum::<u64>();
        total_delta += delta;
        max_channel_delta = max_channel_delta.max(max_delta);
        if max_delta > CHANNEL_THRESHOLD {
            mismatch_count += 1;
            // Red highlights show where the implementation diverges while
            // preserving a dimmed version of the rendered pixel as context.
            diff_rgba.extend_from_slice(&[220, rendered_pixel[1] / 3, rendered_pixel[2] / 3, 255]);
        } else {
            let luminance = ((u16::from(max_delta) * 4).min(255)) as u8;
            diff_rgba.extend_from_slice(&[luminance, luminance, luminance, 255]);
        }
    }
    let mismatch_ratio = if pixel_count == 0 {
        1.0
    } else {
        mismatch_count as f64 / pixel_count as f64
    };
    let average_channel_delta = if pixel_count == 0 {
        255.0
    } else {
        total_delta as f64 / (pixel_count as f64 * 4.0)
    };
    Ok(Comparison {
        summary: ComparisonSummary {
            status: if mismatch_ratio <= MAX_MISMATCH_RATIO {
                "passed"
            } else {
                "failed"
            },
            width: reference.width,
            height: reference.height,
            pixel_count,
            mismatch_count,
            mismatch_ratio,
            average_channel_delta,
            max_channel_delta,
            threshold: CHANNEL_THRESHOLD,
            max_mismatch_ratio: MAX_MISMATCH_RATIO,
        },
        diff_rgba,
    })
}

pub(crate) fn encode_diff(width: u32, height: u32, rgba: &[u8]) -> AgentResult<Vec<u8>> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() != expected {
        return Err(AgentError::new(
            "visual diff buffer has unexpected dimensions",
        ));
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = Encoder::new(&mut bytes, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| AgentError::new(format!("PNG diff header failed: {error}")))?;
        writer
            .write_image_data(rgba)
            .map_err(|error| AgentError::new(format!("PNG diff write failed: {error}")))?;
    }
    Ok(bytes)
}

pub(crate) fn summary_value(summary: &ComparisonSummary) -> serde_json::Value {
    serde_json::to_value(summary).unwrap_or_else(|_| json!({"status": "not_run"}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = Encoder::new(&mut bytes, width, height);
            encoder.set_color(ColorType::Rgba);
            encoder.set_depth(BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(pixels).unwrap();
        }
        bytes
    }

    #[test]
    fn identical_images_pass_and_encode_a_diff() {
        let source = png(2, 1, &[10, 20, 30, 255, 40, 50, 60, 255]);
        let comparison = compare_pngs(&source, &source).unwrap();
        assert_eq!(comparison.summary.status, "passed");
        assert_eq!(comparison.summary.mismatch_count, 0);
        assert!(encode_diff(2, 1, &comparison.diff_rgba).is_ok());
    }

    #[test]
    fn large_pixel_difference_fails_with_a_diff() {
        let reference = png(2, 1, &[0, 0, 0, 255, 0, 0, 0, 255]);
        let rendered = png(2, 1, &[255, 255, 255, 255, 0, 0, 0, 255]);
        let comparison = compare_pngs(&reference, &rendered).unwrap();
        assert_eq!(comparison.summary.status, "failed");
        assert_eq!(comparison.summary.mismatch_count, 1);
        assert_eq!(comparison.diff_rgba.len(), 8 * 2 / 2);
    }
}
