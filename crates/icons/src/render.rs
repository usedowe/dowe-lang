use crate::{IconColor, IconError, IconResult};
use resvg::tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Pixmap, Transform};
use resvg::usvg::{Options, Tree};

pub(crate) struct IconRenderer {
    tree: Tree,
}

#[derive(Clone, Copy)]
pub(crate) struct RenderStyle {
    pub background: Option<IconColor>,
    pub background_scale: f32,
    pub radius: f32,
    pub logo_scale: f32,
    pub circular_safe_zone: Option<f32>,
}

impl IconRenderer {
    pub fn new(data: &[u8]) -> IconResult<Self> {
        let mut options = Options::default();
        options.image_href_resolver.resolve_string = Box::new(|_, _| None);
        let tree = Tree::from_data(data, &options)
            .map_err(|error| IconError::new(format!("invalid icon SVG: {error}")))?;
        if !tree.root().has_children() {
            return Err(IconError::new("icon SVG has no renderable content"));
        }
        Ok(Self { tree })
    }

    pub fn png(&self, size: u32, style: RenderStyle) -> IconResult<Vec<u8>> {
        let pixmap = self.pixmap(size, style)?;
        pixmap
            .encode_png()
            .map_err(|error| IconError::new(format!("failed to encode PNG: {error}")))
    }

    fn pixmap(&self, size: u32, style: RenderStyle) -> IconResult<Pixmap> {
        let mut pixmap = Pixmap::new(size, size)
            .ok_or_else(|| IconError::new(format!("invalid icon canvas size {size}")))?;
        if let Some(background) = style.background {
            fill_background(
                &mut pixmap,
                background,
                style.radius,
                style.background_scale,
            );
        }
        let source_size = self.tree.size();
        let source_width = source_size.width();
        let source_height = source_size.height();
        if source_width <= 0.0 || source_height <= 0.0 {
            return Err(IconError::new("icon SVG has an invalid viewport"));
        }
        let scale = match style.circular_safe_zone {
            Some(diameter) => {
                size as f32 * diameter
                    / (source_width * source_width + source_height * source_height).sqrt()
            }
            None => size as f32 * style.logo_scale / source_width.max(source_height),
        };
        let x = (size as f32 - source_width * scale) * 0.5;
        let y = (size as f32 - source_height * scale) * 0.5;
        let transform = Transform::from_row(scale, 0.0, 0.0, scale, x, y);
        resvg::render(&self.tree, transform, &mut pixmap.as_mut());
        Ok(pixmap)
    }
}

fn fill_background(pixmap: &mut Pixmap, color: IconColor, radius: f32, scale: f32) {
    let color = Color::from_rgba8(color.red, color.green, color.blue, 255);
    if radius <= 0.0 && scale >= 1.0 {
        pixmap.fill(color);
        return;
    }
    let canvas_side = pixmap.width() as f32;
    let side = canvas_side * scale;
    let origin = (canvas_side - side) * 0.5;
    let radius = (side * radius).min(side * 0.5);
    let mut paint = Paint::default();
    paint.set_color(color);
    if let Some(path) = rounded_square(origin, side, radius) {
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn rounded_square(origin: f32, side: f32, radius: f32) -> Option<Path> {
    let end = origin + side;
    let control = radius * 0.552_284_8;
    let mut path = PathBuilder::new();
    path.move_to(origin + radius, origin);
    path.line_to(end - radius, origin);
    path.cubic_to(
        end - radius + control,
        origin,
        end,
        origin + radius - control,
        end,
        origin + radius,
    );
    path.line_to(end, end - radius);
    path.cubic_to(
        end,
        end - radius + control,
        end - radius + control,
        end,
        end - radius,
        end,
    );
    path.line_to(origin + radius, end);
    path.cubic_to(
        origin + radius - control,
        end,
        origin,
        end - radius + control,
        origin,
        end - radius,
    );
    path.line_to(origin, origin + radius);
    path.cubic_to(
        origin,
        origin + radius - control,
        origin + radius - control,
        origin,
        origin + radius,
        origin,
    );
    path.close();
    path.finish()
}
