fn validate_svg_logo_source(source: &str) -> ComponentResult<()> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let tag = element.name();
                if !matches!(
                    tag.as_ref(),
                    b"svg"
                        | b"title"
                        | b"desc"
                        | b"defs"
                        | b"g"
                        | b"path"
                        | b"circle"
                        | b"ellipse"
                        | b"rect"
                        | b"line"
                        | b"polygon"
                        | b"polyline"
                        | b"linearGradient"
                        | b"radialGradient"
                        | b"stop"
                        | b"clipPath"
                        | b"mask"
                        | b"use"
                        | b"filter"
                        | b"feGaussianBlur"
                        | b"feColorMatrix"
                        | b"feComposite"
                        | b"feFlood"
                        | b"feBlend"
                        | b"feOffset"
                        | b"feMerge"
                        | b"feMergeNode"
                        | b"feMorphology"
                        | b"pattern"
                        | b"image"
                        | b"style"
                ) {
                    return Err(ComponentError::invalid_prop(
                        "name",
                        "portable SVG Logos elements",
                    ));
                }
                for (name, value) in xml_attrs(&element) {
                    let lower = value.to_ascii_lowercase();
                    let reference = name == "href" || name.ends_with(":href");
                    if name.starts_with("on")
                        || lower.contains("javascript:")
                        || (reference
                            && !value.starts_with('#')
                            && !lower.starts_with("data:image/"))
                    {
                        return Err(ComponentError::invalid_prop(
                            "name",
                            "safe bundled SVG Logos source",
                        ));
                    }
                }
            }
            Ok(Event::Text(text)) => {
                let value = String::from_utf8_lossy(text.as_ref()).to_ascii_lowercase();
                if value.contains("@import")
                    || value.contains("url(http")
                    || value.contains("javascript:")
                    || value.contains("expression(")
                {
                    return Err(ComponentError::invalid_prop(
                        "name",
                        "portable bundled SVG Logos CSS",
                    ));
                }
            }
            Ok(Event::DocType(_)) => {
                return Err(ComponentError::invalid_prop(
                    "name",
                    "SVG Logos source without a document type",
                ));
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                return Err(ComponentError::invalid_prop(
                    "name",
                    "valid SVG Logos source",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_svg_logo(source: &str) -> ComponentResult<(SvgViewBox, Vec<SvgPath>)> {
    let view_box = svg_logo_view_box(source)?;
    let tree = usvg::Tree::from_str(source, &usvg::Options::default())
        .map_err(|_| ComponentError::invalid_prop("name", "valid SVG Logos source"))?;
    let mut paths = Vec::new();
    collect_svg_logo_paths(tree.root(), 255, &mut paths);
    Ok((view_box, paths))
}

fn svg_logo_view_box(source: &str) -> ComponentResult<SvgViewBox> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element))
                if element.name().as_ref() == b"svg" =>
            {
                let attrs = xml_attrs(&element);
                let value = attr(&attrs, "viewBox")
                    .ok_or_else(|| ComponentError::invalid_prop("name", "SVG Logos viewBox"))?;
                return parse_svg_view_box("viewBox", &PropValue::String(value.to_string()));
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    Err(ComponentError::invalid_prop("name", "SVG Logos viewBox"))
}

fn collect_svg_logo_paths(group: &usvg::Group, inherited_opacity: u8, paths: &mut Vec<SvgPath>) {
    let opacity = multiply_opacity(inherited_opacity, group.opacity().get());
    for node in group.children() {
        match node {
            usvg::Node::Group(child) => collect_svg_logo_paths(child, opacity, paths),
            usvg::Node::Path(path) if path.is_visible() => {
                let data = tiny_skia_path_data(path.data());
                let transform = svg_logo_transform(path.abs_transform());
                let fill = svg_logo_fill_path(path, &data, transform.as_ref(), opacity);
                let stroke = svg_logo_stroke_path(path, &data, transform.as_ref(), opacity);
                match path.paint_order() {
                    usvg::PaintOrder::FillAndStroke => {
                        paths.extend(fill);
                        paths.extend(stroke);
                    }
                    usvg::PaintOrder::StrokeAndFill => {
                        paths.extend(stroke);
                        paths.extend(fill);
                    }
                }
            }
            _ => {}
        }
    }
}

fn svg_logo_fill_path(
    path: &usvg::Path,
    data: &str,
    transform: Option<&SvgTransform>,
    opacity: u8,
) -> Option<SvgPath> {
    let fill = path.fill()?;
    let (red, green, blue, paint_opacity) = svg_logo_paint_color(fill.paint());
    Some(SvgPath {
        data: data.to_string(),
        fill: SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity: multiply_opacity(
                multiply_opacity(opacity, fill.opacity().get()),
                paint_opacity,
            ),
            even_odd: fill.rule() == usvg::FillRule::EvenOdd,
        },
        transform: transform.cloned(),
    })
}

fn svg_logo_stroke_path(
    path: &usvg::Path,
    data: &str,
    transform: Option<&SvgTransform>,
    opacity: u8,
) -> Option<SvgPath> {
    let stroke = path.stroke()?;
    let (red, green, blue, paint_opacity) = svg_logo_paint_color(stroke.paint());
    Some(SvgPath {
        data: data.to_string(),
        fill: SvgPathFill::LiteralStroke {
            red,
            green,
            blue,
            opacity: multiply_opacity(
                multiply_opacity(opacity, stroke.opacity().get()),
                paint_opacity,
            ),
            width: (stroke.width().get() * 100.0)
                .round()
                .clamp(1.0, u16::MAX as f32) as u16,
            line_cap: match stroke.linecap() {
                usvg::LineCap::Butt => SvgLineCap::Butt,
                usvg::LineCap::Round => SvgLineCap::Round,
                usvg::LineCap::Square => SvgLineCap::Square,
            },
            line_join: match stroke.linejoin() {
                usvg::LineJoin::Round => SvgLineJoin::Round,
                usvg::LineJoin::Bevel => SvgLineJoin::Bevel,
                _ => SvgLineJoin::Miter,
            },
        },
        transform: transform.cloned(),
    })
}

fn multiply_opacity(base: u8, value: f32) -> u8 {
    (base as f32 * value.clamp(0.0, 1.0)).round() as u8
}

fn svg_logo_paint_color(paint: &usvg::Paint) -> (u8, u8, u8, f32) {
    match paint {
        usvg::Paint::Color(color) => (color.red, color.green, color.blue, 1.0),
        usvg::Paint::LinearGradient(gradient) => svg_logo_gradient_color(gradient.stops()),
        usvg::Paint::RadialGradient(gradient) => svg_logo_gradient_color(gradient.stops()),
        usvg::Paint::Pattern(_) => (0, 0, 0, 1.0),
    }
}

fn svg_logo_gradient_color(stops: &[usvg::Stop]) -> (u8, u8, u8, f32) {
    let Some(first) = stops.first() else {
        return (0, 0, 0, 1.0);
    };
    let target = 0.5;
    let mut left = first;
    let mut right = first;
    for stop in stops {
        if stop.offset().get() <= target {
            left = stop;
        }
        if stop.offset().get() >= target {
            right = stop;
            break;
        }
        right = stop;
    }
    let span = right.offset().get() - left.offset().get();
    let amount = if span.abs() < f32::EPSILON {
        0.0
    } else {
        ((target - left.offset().get()) / span).clamp(0.0, 1.0)
    };
    let left_color = left.color();
    let right_color = right.color();
    (
        interpolate_channel(left_color.red, right_color.red, amount),
        interpolate_channel(left_color.green, right_color.green, amount),
        interpolate_channel(left_color.blue, right_color.blue, amount),
        left.opacity().get() + (right.opacity().get() - left.opacity().get()) * amount,
    )
}

fn interpolate_channel(left: u8, right: u8, amount: f32) -> u8 {
    (left as f32 + (right as f32 - left as f32) * amount).round() as u8
}

fn tiny_skia_path_data(path: &usvg::tiny_skia_path::Path) -> String {
    use usvg::tiny_skia_path::PathSegment;

    let mut data = String::new();
    for segment in path.segments() {
        if !data.is_empty() {
            data.push(' ');
        }
        match segment {
            PathSegment::MoveTo(point) => {
                data.push_str(&format!(
                    "M{} {}",
                    svg_logo_number(point.x),
                    svg_logo_number(point.y)
                ));
            }
            PathSegment::LineTo(point) => {
                data.push_str(&format!(
                    "L{} {}",
                    svg_logo_number(point.x),
                    svg_logo_number(point.y)
                ));
            }
            PathSegment::QuadTo(control, point) => {
                data.push_str(&format!(
                    "Q{} {} {} {}",
                    svg_logo_number(control.x),
                    svg_logo_number(control.y),
                    svg_logo_number(point.x),
                    svg_logo_number(point.y)
                ));
            }
            PathSegment::CubicTo(first, second, point) => {
                data.push_str(&format!(
                    "C{} {} {} {} {} {}",
                    svg_logo_number(first.x),
                    svg_logo_number(first.y),
                    svg_logo_number(second.x),
                    svg_logo_number(second.y),
                    svg_logo_number(point.x),
                    svg_logo_number(point.y)
                ));
            }
            PathSegment::Close => data.push('Z'),
        }
    }
    data
}

fn svg_logo_transform(value: usvg::Transform) -> Option<SvgTransform> {
    if value.is_identity() {
        return None;
    }
    Some(SvgTransform {
        a: svg_logo_number(value.sx),
        b: svg_logo_number(value.ky),
        c: svg_logo_number(value.kx),
        d: svg_logo_number(value.sy),
        e: svg_logo_number(value.tx),
        f: svg_logo_number(value.ty),
    })
}

fn svg_logo_number(value: f32) -> String {
    let value = if value.abs() < 0.000_001 { 0.0 } else { value };
    let mut output = format!("{value:.6}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}
