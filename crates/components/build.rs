use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let solar_root = manifest.join("../../assets/icons/solar");
    let mut entries = Vec::new();
    for entry in WalkDir::new(&solar_root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("svg") {
            continue;
        }
        let relative = path.strip_prefix(&solar_root).unwrap();
        let parts = relative
            .iter()
            .map(|value| value.to_string_lossy())
            .collect::<Vec<_>>();
        if parts.len() != 3 {
            continue;
        }
        let category = parts[0].as_ref();
        let name = parts[2].strip_suffix(".svg").unwrap();
        let style = parts[1].as_ref();
        entries.push((
            category.to_string(),
            name.to_string(),
            style.to_string(),
            path.to_path_buf(),
        ));
    }
    entries.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then(left.2.cmp(&right.2))
            .then(left.0.cmp(&right.0))
    });
    let mut source = String::from("static SOLAR_ICONS: &[SolarIconSource] = &[\n");
    for (category, name, style, path) in entries {
        source.push_str(&format!(
            "SolarIconSource {{ category: {:?}, name: {:?}, style: {:?}, svg: include_str!({:?}) }},\n",
            category, name, style, path
        ));
    }
    source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("solar_icons.rs"),
        source,
    )
    .unwrap();
    println!("cargo:rerun-if-changed={}", solar_root.display());

    let country_root = manifest.join("../../assets/icons/country-flags");
    let mut country_entries = WalkDir::new(&country_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|value| value.to_str()) == Some("svg"))
                .then(|| {
                    path.file_stem().map(|stem| {
                        (
                            stem.to_string_lossy().to_ascii_uppercase(),
                            path.to_path_buf(),
                        )
                    })
                })
                .flatten()
        })
        .collect::<Vec<_>>();
    country_entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut country_source = String::from("static COUNTRY_FLAGS: &[CountryFlagSource] = &[\n");
    for (code, path) in country_entries {
        country_source.push_str(&format!(
            "CountryFlagSource {{ code: {:?}, svg: include_str!({:?}) }},\n",
            code, path
        ));
    }
    country_source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("country_flags.rs"),
        country_source,
    )
    .unwrap();
    println!("cargo:rerun-if-changed={}", country_root.display());

    let spinner_root = manifest.join("../../assets/icons/svg-spinners");
    let mut spinner_entries = WalkDir::new(&spinner_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|value| value.to_str()) == Some("svg"))
                .then(|| {
                    path.file_stem().map(|stem| {
                        (
                            stem.to_string_lossy().to_ascii_lowercase(),
                            path.to_path_buf(),
                        )
                    })
                })
                .flatten()
        })
        .collect::<Vec<_>>();
    spinner_entries.sort_by(|left, right| left.0.cmp(&right.0));
    for pair in spinner_entries.windows(2) {
        assert_ne!(
            pair[0].0, pair[1].0,
            "duplicate SVG Spinner icon name {}",
            pair[0].0
        );
    }
    let mut spinner_source = String::from("static SVG_SPINNERS: &[SvgSpinnerSource] = &[\n");
    for (name, path) in spinner_entries {
        spinner_source.push_str(&format!(
            "SvgSpinnerSource {{ name: {:?}, svg: include_str!({:?}) }},\n",
            name, path
        ));
    }
    spinner_source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("svg_spinners.rs"),
        spinner_source,
    )
    .unwrap();
    println!("cargo:rerun-if-changed={}", spinner_root.display());
}
