fn validate_svg_spinner_source(source: &str) -> ComponentResult<()> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let tag = element.name();
                if !matches!(
                    tag.as_ref(),
                    b"svg"
                        | b"style"
                        | b"path"
                        | b"circle"
                        | b"ellipse"
                        | b"rect"
                        | b"g"
                        | b"defs"
                        | b"filter"
                        | b"feGaussianBlur"
                        | b"feColorMatrix"
                        | b"feBlend"
                ) {
                    return Err(ComponentError::invalid_prop(
                        "name",
                        "portable SVG Spinner elements",
                    ));
                }
                for (name, value) in xml_attrs(&element) {
                    if name.starts_with("on")
                        || value.to_ascii_lowercase().contains("javascript:")
                        || ((name == "href" || name == "xlink:href") && !value.starts_with('#'))
                    {
                        return Err(ComponentError::invalid_prop(
                            "name",
                            "safe bundled SVG Spinner source",
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
                        "portable bundled SVG Spinner CSS",
                    ));
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                return Err(ComponentError::invalid_prop(
                    "name",
                    "valid SVG Spinner source",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_spinner_svg(
    source: &str,
    fill: Option<ColorToken>,
    stroke: Option<ColorToken>,
) -> ComponentResult<(SvgViewBox, Vec<SvgPath>)> {
    let (view_box, paths) = parse_solar_svg(source, fill, stroke)?;
    if source.contains("r=\"0\"")
        || spinner_shapes_start_transparent(source)
        || spinner_first_frame_is_rotationally_symmetric(source)
    {
        return Ok((
            view_box,
            vec![SvgPath {
                data: "M12 3a9 9 0 1 1-6.364 2.636".to_string(),
                fill: SvgPathFill::Stroke {
                    color: stroke.or(fill),
                    opacity: 255,
                    width: 250,
                    line_cap: SvgLineCap::Round,
                    line_join: SvgLineJoin::Round,
                },
                transform: None,
            }],
        ));
    }
    Ok((view_box, paths))
}

fn spinner_shapes_start_transparent(source: &str) -> bool {
    let visible_shapes = [r#"<path "#, r#"<circle "#, r#"<ellipse "#, r#"<rect "#]
        .into_iter()
        .filter(|tag| source.contains(tag))
        .count();
    visible_shapes > 0
        && !source
            .split('<')
            .filter(|element| {
                element.starts_with("path ")
                    || element.starts_with("circle ")
                    || element.starts_with("ellipse ")
                    || element.starts_with("rect ")
            })
            .any(|element| !element.contains(r#"opacity="0""#))
}

fn spinner_first_frame_is_rotationally_symmetric(source: &str) -> bool {
    let mut reader = Reader::from_str(source);
    let mut shape_count = 0usize;
    let mut centered_circle = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let tag = element.name();
                if matches!(tag.as_ref(), b"path" | b"circle" | b"ellipse" | b"rect") {
                    shape_count += 1;
                    if tag.as_ref() == b"circle" {
                        let attrs = xml_attrs(&element);
                        centered_circle =
                            attr(&attrs, "cx") == Some("12") && attr(&attrs, "cy") == Some("12");
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return false,
            _ => {}
        }
    }
    shape_count == 1 && centered_circle
}
