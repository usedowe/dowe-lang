struct SolarIconSource {
    category: &'static str,
    name: &'static str,
    style: &'static str,
    public_name: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/solar_icons.rs"));

struct CountryFlagSource {
    code: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/country_flags.rs"));

struct SvgSpinnerSource {
    name: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/svg_spinners.rs"));

struct SvgLogoSource {
    name: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/svg_logos.rs"));

pub fn solar_icon_names() -> Vec<&'static str> {
    let mut names = SOLAR_ICONS.iter().map(|icon| icon.name).collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

fn solar_component_icon_names() -> Vec<&'static str> {
    SOLAR_ICONS.iter().map(|icon| icon.public_name).collect()
}

pub fn all_icon_names() -> Vec<String> {
    let mut names = solar_component_icon_names()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    names.extend(
        COUNTRY_FLAGS
            .iter()
            .map(|flag| format!("country-flags:{}", flag.code)),
    );
    names.extend(
        SVG_SPINNERS
            .iter()
            .map(|spinner| format!("svg-spinners:{}", spinner.name)),
    );
    names.extend(
        SVG_LOGOS
            .iter()
            .map(|logo| format!("svg-logos:{}", logo.name)),
    );
    names.sort_unstable();
    names.dedup();
    names
}

fn country_flag_svg(code: &str) -> Option<&'static str> {
    let code = code.to_ascii_uppercase();
    COUNTRY_FLAGS
        .binary_search_by(|flag| flag.code.cmp(code.as_str()))
        .ok()
        .map(|index| COUNTRY_FLAGS[index].svg)
}

fn svg_spinner_svg(name: &str) -> Option<&'static str> {
    SVG_SPINNERS
        .binary_search_by(|spinner| spinner.name.cmp(name))
        .ok()
        .map(|index| SVG_SPINNERS[index].svg)
}

fn svg_logo_svg(name: &str) -> Option<&'static str> {
    SVG_LOGOS
        .binary_search_by(|logo| logo.name.cmp(name))
        .ok()
        .map(|index| SVG_LOGOS[index].svg)
}

pub fn country_flag_icon(code: &str) -> Option<SideNavIcon> {
    let svg = country_flag_svg(code)?;
    let (view_box, paths) = parse_country_flag_svg(svg).ok()?;
    let props = parse_svg_props(
        BuiltinComponent::Icon,
        &[ComponentProp {
            name: "viewBox".to_string(),
            value: PropValue::String(view_box.as_str()),
        }],
    )
    .ok()?;
    Some(SideNavIcon { props, paths })
}

pub fn validate_solar_icon_catalog() -> ComponentResult<usize> {
    for icon in SOLAR_ICONS {
        let (_, paths) = parse_solar_svg(icon.svg, None, None).map_err(|_| {
            ComponentError::invalid_prop_combination(format!(
                "invalid Solar geometry for {} {}",
                icon.name, icon.style
            ))
        })?;
        if paths.is_empty() {
            return Err(ComponentError::invalid_prop(
                "name",
                "Solar icon with visible vector geometry",
            ));
        }
    }
    Ok(SOLAR_ICONS.len())
}
