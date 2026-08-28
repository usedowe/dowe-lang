use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn solar_public_name(name: &str, style: &str) -> String {
    let suffix = match style {
        "Linear" => "",
        "Broken" => "-broken",
        "Outline" => "-outline",
        "Bold" => "-bold",
        "LineDuotone" => "-line-duotone",
        "BoldDuotone" => "-bold-duotone",
        _ => panic!("unsupported Solar style {style}"),
    };
    format!("{name}{suffix}")
}

fn ir_struct_fields(
    manifest: &PathBuf,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> {
    let mut structs = std::collections::BTreeMap::new();
    let model_root = manifest.join("src/component/model");
    for entry in WalkDir::new(model_root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(path).unwrap();
        let mut current = None;
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix("pub struct ") {
                let name = name.split_whitespace().next().unwrap().to_string();
                structs
                    .entry(name.clone())
                    .or_insert_with(std::collections::BTreeMap::new);
                current = Some(name);
                continue;
            }
            if trimmed.starts_with("pub enum ") || trimmed == "}" {
                current = None;
                continue;
            }
            let Some(struct_name) = current.as_ref() else {
                continue;
            };
            let Some(field) = trimmed.strip_prefix("pub ") else {
                continue;
            };
            let Some((name, ty)) = field.split_once(':') else {
                continue;
            };
            structs.get_mut(struct_name).unwrap().insert(
                name.trim().to_string(),
                ty.trim().trim_end_matches(',').to_string(),
            );
        }
    }
    structs
}

fn ir_field_exists(
    structs: &std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
    path: &str,
) -> bool {
    let mut parts = path.split('.').peekable();
    let Some(mut type_name) = parts.next() else {
        return false;
    };
    if !structs.contains_key(type_name) {
        return false;
    }
    while let Some(field) = parts.next() {
        let Some(fields) = structs.get(type_name) else {
            return false;
        };
        let Some(field_type) = fields.get(field) else {
            return false;
        };
        type_name = field_type
            .trim_start_matches("Option<")
            .trim_start_matches("Box<")
            .trim_end_matches('>')
            .split('<')
            .next()
            .unwrap();
        if !structs.contains_key(type_name) && parts.peek().is_some() {
            return false;
        }
    }
    true
}

fn rust_variant(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let first = chars.next().unwrap().to_ascii_uppercase();
            format!("{}{}", first, chars.collect::<String>())
        })
        .collect()
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let prop_definition_path = manifest.join("view_props.def");
    let prop_definitions = fs::read_to_string(&prop_definition_path).unwrap();
    let structs = ir_struct_fields(&manifest);
    let mut roots = std::collections::BTreeSet::new();
    let mut fields = std::collections::BTreeSet::new();
    for line in prop_definitions
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let parts = line.split('|').collect::<Vec<_>>();
        let path = parts[2].split('.').collect::<Vec<_>>();
        roots.insert(path[0].to_string());
        for field in path.iter().skip(1) {
            fields.insert(field.to_string());
        }
    }
    let mut type_source =
        String::from("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum IrFieldRoot {\n");
    for root in &roots {
        type_source.push_str(&format!("    {},\n", rust_variant(root)));
    }
    type_source.push_str("}\n\nimpl IrFieldRoot {\n    pub const fn as_str(self) -> &'static str {\n        match self {\n");
    for root in &roots {
        type_source.push_str(&format!(
            "            Self::{} => {:?},\n",
            rust_variant(root),
            root
        ));
    }
    type_source.push_str("        }\n    }\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum IrFieldSegment {\n");
    for field in &fields {
        type_source.push_str(&format!("    {},\n", rust_variant(field)));
    }
    type_source.push_str("}\n\nimpl IrFieldSegment {\n    pub const fn as_str(self) -> &'static str {\n        match self {\n");
    for field in &fields {
        type_source.push_str(&format!(
            "            Self::{} => {:?},\n",
            rust_variant(field),
            field
        ));
    }
    type_source.push_str("        }\n    }\n}\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("ir_field_types.rs"),
        type_source,
    )
    .unwrap();
    let mut prop_source =
        String::from("pub const GENERATED_VIEW_PROP_INVENTORY: &[ViewPropDefinition] = &[\n");
    let mut prop_keys = std::collections::BTreeSet::new();
    for (line_number, line) in prop_definitions.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('|').collect::<Vec<_>>();
        assert_eq!(
            fields.len(),
            5,
            "invalid view prop definition at line {}",
            line_number + 1
        );
        let [owner, prop, ir_field, kind, reactive] = fields.as_slice() else {
            unreachable!();
        };
        assert!(
            prop_keys.insert((*owner, *prop)),
            "duplicate view prop definition {owner}.{prop}"
        );
        assert!(
            ir_field_exists(&structs, ir_field),
            "unknown IR field {ir_field} at line {}",
            line_number + 1
        );
        let kind = match *kind {
            "string" => "PropValueKind::String",
            "number" => "PropValueKind::Number",
            "boolean" => "PropValueKind::Boolean",
            "any" => "PropValueKind::Any",
            _ => panic!("invalid view prop kind at line {}", line_number + 1),
        };
        let reactive = match *reactive {
            "true" => "true",
            "false" => "false",
            _ => panic!("invalid reactive flag at line {}", line_number + 1),
        };
        let owner = if *owner == "CommonStyle" {
            "ViewPropOwner::CommonStyle".to_string()
        } else if let Some(item) = owner.strip_prefix("Item:") {
            format!("ViewPropOwner::Item(ViewItemKind::{item})")
        } else {
            format!("ViewPropOwner::Component(BuiltinComponent::{owner})")
        };
        let path_parts = ir_field.split('.').collect::<Vec<_>>();
        let root = rust_variant(path_parts[0]);
        let segments = path_parts
            .iter()
            .skip(1)
            .map(|field| format!("IrFieldSegment::{}", rust_variant(field)))
            .collect::<Vec<_>>()
            .join(", ");
        let ir_path = format!("IrFieldPath::new(IrFieldRoot::{root}, &[{segments}])");
        prop_source.push_str(&format!(
            "ViewPropDefinition {{ owner: {owner}, prop: {prop:?}, ir_field: {ir_path}, kind: {kind}, reactive: {reactive} }},\n"
        ));
    }
    prop_source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("view_prop_inventory.rs"),
        prop_source,
    )
    .unwrap();
    println!("cargo:rerun-if-changed={}", prop_definition_path.display());
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
        let public_name = solar_public_name(name, style);
        entries.push((
            category.to_string(),
            name.to_string(),
            style.to_string(),
            public_name,
            path.to_path_buf(),
        ));
    }
    entries.sort_by(|left, right| left.3.cmp(&right.3).then(left.0.cmp(&right.0)));
    for pair in entries.windows(2) {
        assert_ne!(
            pair[0].3, pair[1].3,
            "duplicate Solar public icon name {}",
            pair[0].3
        );
    }
    let mut source = String::from("static SOLAR_ICONS: &[SolarIconSource] = &[\n");
    for (category, name, style, public_name, path) in entries {
        source.push_str(&format!(
            "SolarIconSource {{ category: {:?}, name: {:?}, style: {:?}, public_name: {:?}, svg: include_str!({:?}) }},\n",
            category, name, style, public_name, path
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

    let logo_root = manifest.join("../../assets/icons/svg-logos");
    let mut logo_entries = WalkDir::new(&logo_root)
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
    logo_entries.sort_by(|left, right| left.0.cmp(&right.0));
    for pair in logo_entries.windows(2) {
        assert_ne!(
            pair[0].0, pair[1].0,
            "duplicate SVG Logos icon name {}",
            pair[0].0
        );
    }
    let mut logo_source = String::from("static SVG_LOGOS: &[SvgLogoSource] = &[\n");
    for (name, path) in logo_entries {
        logo_source.push_str(&format!(
            "SvgLogoSource {{ name: {:?}, svg: include_str!({:?}) }},\n",
            name, path
        ));
    }
    logo_source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("svg_logos.rs"),
        logo_source,
    )
    .unwrap();
    println!("cargo:rerun-if-changed={}", logo_root.display());
}
