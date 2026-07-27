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

fn android_dev_font_support(font_families: &BTreeSet<FontFamily>) -> String {
    let resource_branches = font_families
        .iter()
        .filter(|font| font.catalog_entry().package_assets)
        .map(|font| {
            let entry = font.catalog_entry();
            let mut groups = Vec::<(u16, &str)>::new();
            for weight in entry.weights {
                if let Some((maximum, asset_stem)) = groups.last_mut()
                    && *asset_stem == weight.asset_stem
                {
                    *maximum = weight.numeric_weight;
                } else {
                    groups.push((weight.numeric_weight, weight.asset_stem));
                }
            }
            let last = groups.len().saturating_sub(1);
            let resources = groups
                .iter()
                .enumerate()
                .map(|(index, (maximum, asset_stem))| {
                    let resource = android_font_resource_name(asset_stem);
                    if index == last {
                        format!("            return R.font.{resource};")
                    } else {
                        format!(
                            "            if (weight <= {maximum}) {{\n                return R.font.{resource};\n            }}"
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "        if (\"{}\".equals(font)) {{\n{resources}\n        }}",
                entry.android_family_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let variable_families = font_families
        .iter()
        .filter(|font| {
            let weights = font.catalog_entry().weights;
            weights.len() > 1
                && weights
                    .iter()
                    .all(|weight| weight.asset_stem == weights[0].asset_stem)
        })
        .map(|font| format!("\"{}\".equals(font)", font.catalog_entry().android_family_name))
        .collect::<Vec<_>>();
    let variable_condition = if variable_families.is_empty() {
        "false".to_string()
    } else {
        variable_families.join(" || ")
    };

    format!(
        r#"    private int doweFontResource(String font, int weight) {{
{resource_branches}
        return 0;
    }}

    private boolean doweVariableFont(String font) {{
        return {variable_condition};
    }}

    private Typeface doweTypeface(String font, int weight) {{
        int resource = doweFontResource(font, weight);
        if (resource != 0) {{
            Typeface bundled = getResources().getFont(resource);
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {{
                return Typeface.create(bundled, weight, false);
            }}
            return bundled;
        }}
        Typeface system = Typeface.create(font, Typeface.NORMAL);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {{
            return Typeface.create(system, weight, false);
        }}
        return Typeface.create(system, weight >= 600 ? Typeface.BOLD : Typeface.NORMAL);
    }}

"#
    )
}
