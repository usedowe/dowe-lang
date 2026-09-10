fn parse_scale_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<ScaleValue>> {
    parse_responsive(
        name,
        value,
        "Dowe scale value from 0 to 96",
        |scalar| match scalar {
            PropScalar::Number(value) => scale_value(value),
            PropScalar::String(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_size_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<SizeValue>> {
    let allow_viewport_height = matches!(name, "h" | "minH" | "maxH");
    let allow_container_width = matches!(name, "w" | "minW" | "maxW");
    let allow_percentage_width = matches!(name, "w" | "minW");
    let expected = if allow_viewport_height {
        "Dowe scale value, full, auto or vh-<scale>"
    } else if allow_percentage_width {
        "Dowe scale value, container size, percentage from 10% to 100% in 10% increments or full"
    } else if allow_container_width {
        "Dowe scale value, container size or full"
    } else {
        "Dowe scale value or full"
    };
    parse_responsive(name, value, expected, |scalar| match scalar {
        PropScalar::Number(value) => scale_value(value).map(SizeValue::Scale),
        PropScalar::String(value) if value == "full" => Some(SizeValue::Full),
        PropScalar::String(value) if allow_viewport_height && value == "auto" => {
            Some(SizeValue::Auto)
        }
        PropScalar::String(value) if allow_percentage_width && value.ends_with('%') => value
            .strip_suffix('%')
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|value| (10..=100).contains(value) && value % 10 == 0)
            .map(SizeValue::Percent),
        PropScalar::String(value) if allow_container_width => {
            ContainerSize::from_name(value).map(SizeValue::Container)
        }
        PropScalar::String(value) if allow_viewport_height => value
            .strip_prefix("vh-")
            .and_then(scale_value)
            .map(SizeValue::ViewportMinus),
        PropScalar::String(_) => None,
        PropScalar::Boolean(_) => None,
    })
}

fn parse_rounded_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<RoundedSize>> {
    parse_responsive(
        name,
        value,
        "xs, sm, md, lg, xl or full",
        |scalar| match scalar {
            PropScalar::String(value) => RoundedSize::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_border_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<BorderWidth>> {
    parse_responsive(name, value, "integer from 1 to 4", |scalar| match scalar {
        PropScalar::Number(value) => value
            .parse::<u8>()
            .ok()
            .filter(|value| (1..=4).contains(value))
            .map(BorderWidth),
        PropScalar::String(_) | PropScalar::Boolean(_) => None,
    })
}

fn parse_shadow_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<ShadowSize>> {
    parse_responsive(name, value, "xs, sm, md, lg or xl", |scalar| match scalar {
        PropScalar::String(value) => ShadowSize::from_name(value),
        PropScalar::Number(_) | PropScalar::Boolean(_) => None,
    })
}

fn parse_justify_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<Justify>> {
    parse_responsive(
        name,
        value,
        "start, center, end, between, around or evenly",
        |scalar| match scalar {
            PropScalar::String(value) => Justify::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_flex_direction_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<FlexDirection>> {
    parse_responsive(name, value, "row or column", |scalar| match scalar {
        PropScalar::String(value) => FlexDirection::from_name(value),
        PropScalar::Number(_) | PropScalar::Boolean(_) => None,
    })
}

fn parse_flex_item_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<FlexItem>> {
    parse_responsive(
        name,
        value,
        "initial, auto, none or 1",
        |scalar| match scalar {
            PropScalar::String(value) => FlexItem::from_name(value),
            PropScalar::Number(value) if value == "1" => Some(FlexItem::Fill),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_align_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<Align>> {
    parse_responsive(
        name,
        value,
        "start, center, end, stretch or baseline",
        |scalar| match scalar {
            PropScalar::String(value) => Align::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_text_align_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<TextAlign>> {
    parse_responsive(
        name,
        value,
        "start, center, end or justify",
        |scalar| match scalar {
            PropScalar::String(value) => TextAlign::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_grid_alignment_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<GridAlignment>> {
    let is_justify = name == "justify";
    parse_responsive(
        name,
        value,
        if is_justify {
            "start, end, end-safe, center, center-safe, between, around, evenly, stretch or normal"
        } else {
            "start, end, end-safe, center, center-safe, baseline, baseline-last or stretch"
        },
        |scalar| match scalar {
            PropScalar::String(value) => GridAlignment::from_name(value).filter(|value| {
                if is_justify {
                    matches!(
                        value,
                        GridAlignment::Start
                            | GridAlignment::End
                            | GridAlignment::EndSafe
                            | GridAlignment::Center
                            | GridAlignment::CenterSafe
                            | GridAlignment::Between
                            | GridAlignment::Around
                            | GridAlignment::Evenly
                            | GridAlignment::Stretch
                            | GridAlignment::Normal
                    )
                } else {
                    matches!(
                        value,
                        GridAlignment::Start
                            | GridAlignment::End
                            | GridAlignment::EndSafe
                            | GridAlignment::Center
                            | GridAlignment::CenterSafe
                            | GridAlignment::Baseline
                            | GridAlignment::BaselineLast
                            | GridAlignment::Stretch
                    )
                }
            }),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_gap_prop(
    name: &str,
    value: &PropValue,
    pair_allowed: bool,
) -> ComponentResult<ResponsiveValue<GapValue>> {
    parse_responsive(
        name,
        value,
        "Dowe scale value or px value",
        |scalar| match scalar {
            PropScalar::Number(value) => {
                scale_value(value).map(|value| GapValue::Single(GapSize::Scale(value)))
            }
            PropScalar::String(value) => parse_gap_value(value, pair_allowed),
            PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_grid_tracks_prop(
    name: &str,
    value: &PropValue,
    auto_allowed: bool,
    max_count: Option<u16>,
) -> ComponentResult<ResponsiveValue<GridTracks>> {
    parse_responsive(
        name,
        value,
        if max_count.is_some() {
            "positive integer from 1 to 12 or space-separated positive fr tracks"
        } else {
            "positive integer or auto"
        },
        |scalar| match scalar {
            PropScalar::Number(value) => value
                .parse::<u16>()
                .ok()
                .filter(|value| *value > 0 && max_count.is_none_or(|max| *value <= max))
                .map(GridTracks::Count),
            PropScalar::String(value) if max_count.is_some() => parse_fraction_tracks(value),
            PropScalar::String(value) if auto_allowed && value == "auto" => Some(GridTracks::Auto),
            PropScalar::String(_) => None,
            PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_fraction_tracks(value: &str) -> Option<GridTracks> {
    let tracks = value
        .split_whitespace()
        .map(|track| track.strip_suffix("fr")?.parse::<u16>().ok())
        .collect::<Option<Vec<_>>>()?;
    (!tracks.is_empty() && tracks.iter().all(|value| *value > 0))
        .then_some(GridTracks::Fractions(tracks))
}

fn parse_span_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<GridSpan>> {
    parse_responsive(name, value, "positive integer", |scalar| match scalar {
        PropScalar::Number(value) => value
            .parse::<u16>()
            .ok()
            .filter(|value| *value > 0)
            .map(GridSpan),
        PropScalar::String(_) | PropScalar::Boolean(_) => None,
    })
}


