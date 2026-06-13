fn replace_android_font_support(
    output: &mut String,
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
) {
    let start = output
        .find("private enum class DoweFont {")
        .expect("font support start");
    let end = output
        .find("private sealed class DoweSize {")
        .expect("font support end");
    output.replace_range(
        start..end,
        &android_font_support(font_config, font_families),
    );
}

fn android_font_support(font_config: &FontConfig, font_families: &BTreeSet<FontFamily>) -> String {
    let enum_cases = font_families
        .iter()
        .map(|font| format!("    {}", font_name(*font)))
        .collect::<Vec<_>>()
        .join(",\n");
    let font_objects = font_families
        .iter()
        .filter(|font| font.catalog_entry().package_assets)
        .map(|font| {
            let entry = font.catalog_entry();
            let fonts = entry
                .weights
                .iter()
                .map(|weight| {
                    format!(
                        "        Font(R.font.{}, {})",
                        android_font_resource_name(weight.asset_stem),
                        compose_text_weight(weight.weight)
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n");
            format!("    val {} = FontFamily(\n{fonts}\n    )", font.as_str())
        })
        .collect::<Vec<_>>()
        .join("\n");
    let branches = font_families
        .iter()
        .map(|font| {
            format!(
                "        DoweFont.{} -> {}",
                font_name(*font),
                compose_font_family_ref(*font)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"private enum class DoweFont {{
{enum_cases}
}}

private object DoweFonts {{
{font_objects}
}}

private fun doweFontFamily(value: DoweFont?): FontFamily {{
    return when (value) {{
{branches}
        null -> {}
    }}
}}

"#,
        compose_font_family_ref(font_config.default_family)
    )
}
