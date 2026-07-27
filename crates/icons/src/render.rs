use crate::{IconColor, IconError, IconResult};
use resvg::tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Pixmap, Transform};
use resvg::usvg::{Options, Tree};

pub(crate) struct IconRenderer {
    tree: Tree,
}

#[derive(Clone, Copy)]
pub(crate) struct RenderStyle {
    pub background: Option<IconColor>,
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
            fill_background(&mut pixmap, background, style.radius);
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

fn fill_background(pixmap: &mut Pixmap, color: IconColor, radius: f32) {
    let color = Color::from_rgba8(color.red, color.green, color.blue, 255);
    if radius <= 0.0 {
        pixmap.fill(color);
        return;
    }
    let side = pixmap.width() as f32;
    let radius = (side * radius).min(side * 0.5);
    let mut paint = Paint::default();
    paint.set_color(color);
    if let Some(path) = rounded_square(side, radius) {
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn rounded_square(side: f32, radius: f32) -> Option<Path> {
    let control = radius * 0.552_284_8;
    let mut path = PathBuilder::new();
    path.move_to(radius, 0.0);
    path.line_to(side - radius, 0.0);
    path.cubic_to(
        side - radius + control,
        0.0,
        side,
        radius - control,
        side,
        radius,
    );
    path.line_to(side, side - radius);
    path.cubic_to(
        side,
        side - radius + control,
        side - radius + control,
        side,
        side - radius,
        side,
    );
    path.line_to(radius, side);
    path.cubic_to(
        radius - control,
        side,
        0.0,
        side - radius + control,
        0.0,
        side - radius,
    );
    path.line_to(0.0, radius);
    path.cubic_to(0.0, radius - control, radius - control, 0.0, radius, 0.0);
    path.close();
    path.finish()
}
