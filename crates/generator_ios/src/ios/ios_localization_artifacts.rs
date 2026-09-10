fn ios_translation_artifacts(catalog: &TranslationCatalog) -> Vec<IosArtifact> {
    catalog
        .locales
        .iter()
        .map(|locale| IosArtifact {
            relative_path: PathBuf::from(format!(
                "apps/ios/{}.lproj/Localizable.strings",
                locale.locale
            )),
            content: ios_localizable_strings(locale),
            kind: IosArtifactKind::Localization,
            target: "ios",
        })
        .collect()
}

fn ios_phone_catalog_artifacts() -> Vec<IosArtifact> {
    const SHARD_SIZE: usize = 24;

    let countries = phone_countries()
        .iter()
        .filter_map(|country| {
            let icon = phone_country_flag_icon(country.code)?;
            Some(format!(
                "        DowePhoneCountry(code: {}, name: {}, dialCode: {}, flag: DoweControlIcon(viewBox: {}, paths: {}))",
                swift_string_literal(country.code),
                swift_string_literal(country.name),
                swift_string_literal(country.dial),
                swift_svg_view_box(&icon.props.view_box),
                swift_svg_paths(&icon.paths)
            ))
        })
        .collect::<Vec<_>>();
    let mut files = countries
        .chunks(SHARD_SIZE)
        .enumerate()
        .map(|(index, countries)| IosArtifact {
            relative_path: PathBuf::from(format!(
                "apps/ios/DowePhoneCatalogShard{index}.swift"
            )),
            content: format!(
                "import SwiftUI\n\nenum DowePhoneCatalogShard{index} {{\n    static let countries: [DowePhoneCountry] = [\n{}\n    ]\n}}\n",
                countries.join(",\n")
            ),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        })
        .collect::<Vec<_>>();
    let append_shards = files
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!("        countries.append(contentsOf: DowePhoneCatalogShard{index}.countries)")
        })
        .collect::<Vec<_>>()
        .join("\n");
    files.push(IosArtifact {
        relative_path: PathBuf::from("apps/ios/DowePhoneCatalog.swift"),
        content: format!(
            "import SwiftUI\n\nenum DowePhoneCatalog {{\n    static let countries: [DowePhoneCountry] = {{\n        var countries: [DowePhoneCountry] = []\n{append_shards}\n        return countries\n    }}()\n}}\n"
        ),
        kind: IosArtifactKind::GeneratedView,
        target: "ios",
    });
    files
}

fn ios_localizable_strings(locale: &dowe_components::TranslationLocale) -> String {
    locale
        .values
        .iter()
        .map(|value| {
            format!(
                "\"{}\" = \"{}\";",
                escape_swift(&value.key),
                escape_swift(&value.value)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn ios_environment(environment: &[(String, String)]) -> String {
    let mut values = environment
        .iter()
        .map(|(name, value)| format!("    static let {} = \"{}\"", name, escape_swift(value)))
        .collect::<Vec<_>>();
    if !environment.iter().any(|(name, _)| name == "BACKEND_URL") {
        values.push("    static let BACKEND_URL = \"\"".to_string());
    }
    let values = values.join("\n");
    format!(
        r#"import Foundation

enum DoweEnvironment {{
{values}
}}
"#
    )
}

