fn android_design_block(design: &DesignConfig) -> String {
    let theme = design.default_theme();
    let mut output = String::from("object DoweDesign {\n");
    output.push_str(&format!(
        "    var name by mutableStateOf(\"{}\")\n        private set\n",
        escape_kotlin(&design.default_theme)
    ));
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "    var {} by mutableStateOf({})\n        private set\n",
            token.as_str(),
            android_color_literal(theme.color_value(token))
        ));
    }
    output.push_str(&format!(
        "    var radius by mutableStateOf({}.dp)\n        private set\n",
        theme.radius
    ));
    output.push_str("    fun applyTheme(name: String) {\n        val theme = DoweThemeModule.themes.firstOrNull { it.name == name } ?: DoweThemeModule.themes.first { it.name == DoweThemeModule.defaultTheme }\n        this.name = theme.name\n");
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "        {} = theme.colors[\"{}\"] ?: {}\n",
            token.as_str(),
            token.as_str(),
            android_color_literal(theme.color_value(token))
        ));
    }
    output.push_str("        radius = theme.radius\n    }\n}\n");
    output
}

fn android_theme_module(design_config: &DesignConfig) -> String {
    let names = design_config
        .themes
        .iter()
        .map(|theme| format!("        \"{}\",", escape_kotlin(&theme.name)))
        .collect::<Vec<_>>()
        .join("\n");
    let themes = design_config
        .themes
        .iter()
        .map(android_theme_record)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"package dev.dowe.generated

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

data class DoweGeneratedTheme(
    val name: String,
    val colors: Map<String, Color>,
    val radius: Dp
)

object DoweThemeModule {{
    const val generated = true
    const val defaultTheme = "{}"
    val names = listOf(
{}
    )
    val themes = listOf(
{}
    )
}}
"#,
        escape_kotlin(&design_config.default_theme),
        names,
        themes
    )
}

fn android_theme_record(theme: &DesignTheme) -> String {
    let colors = theme
        .ordered_color_tokens()
        .into_iter()
        .map(|token| {
            format!(
                "            \"{}\" to {},",
                token.as_str(),
                android_color_literal(theme.color_value(token))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "        DoweGeneratedTheme(name = \"{}\", colors = mapOf(\n{}\n        ), radius = {}.dp),",
        escape_kotlin(&theme.name),
        colors,
        theme.radius
    )
}

fn android_color_literal(value: &str) -> String {
    let raw = value.trim_start_matches('#');
    let value = if raw.len() == 6 {
        format!("FF{}", raw.to_ascii_uppercase())
    } else {
        format!(
            "{}{}",
            raw[6..8].to_ascii_uppercase(),
            raw[0..6].to_ascii_uppercase()
        )
    };
    format!("Color(0x{value})")
}

fn android_java_color_literal(value: &str) -> String {
    let raw = value.trim_start_matches('#');
    let red = u8::from_str_radix(&raw[0..2], 16).expect("red color");
    let green = u8::from_str_radix(&raw[2..4], 16).expect("green color");
    let blue = u8::from_str_radix(&raw[4..6], 16).expect("blue color");
    let alpha = if raw.len() == 6 {
        255
    } else {
        u8::from_str_radix(&raw[6..8], 16).expect("alpha color")
    };
    format!("0x{alpha:02X}{red:02X}{green:02X}{blue:02X}")
}

fn font_display_name(value: FontFamily) -> &'static str {
    value.catalog_entry().android_family_name
}

fn dev_design_constants(design: &DesignConfig) -> String {
    let theme = design.default_theme();
    let mut output = String::new();
    output.push_str(&format!(
        "    private static final String DOWE_DEFAULT_THEME = \"{}\";\n",
        escape_java(&design.default_theme)
    ));
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "    private static int {};\n",
            java_color(token)
        ));
    }
    output.push_str("    private static float DOWE_RADIUS;\n");
    output.push_str("\n    private void doweApplyTheme(String name) {\n");
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "        {} = {};\n",
            java_color(token),
            android_java_color_literal(theme.color_value(token))
        ));
    }
    output.push_str(&format!("        DOWE_RADIUS = {}f;\n", theme.radius));
    for (index, theme) in design.themes.iter().enumerate() {
        output.push_str(&format!(
            "        {} (\"{}\".equals(name)) {{\n",
            if index == 0 { "if" } else { "else if" },
            escape_java(&theme.name)
        ));
        for token in design.default_theme().ordered_color_tokens() {
            output.push_str(&format!(
                "            {} = {};\n",
                java_color(token),
                android_java_color_literal(theme.color_value(token))
            ));
        }
        output.push_str(&format!(
            "            DOWE_RADIUS = {}f;\n        }}\n",
            theme.radius
        ));
    }
    output.push_str("    }\n");
    output
}

fn java_color(value: ColorToken) -> &'static str {
    if value.as_str() == "white" {
        return "Color.WHITE";
    }
    let mut output = String::from("DOWE_");
    for character in value.as_str().chars() {
        if character.is_ascii_uppercase() {
            output.push('_');
        }
        output.push(character.to_ascii_uppercase());
    }
    intern_generated_color_name(output)
}

fn intern_generated_color_name(value: String) -> &'static str {
    static NAMES: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();
    let names = NAMES.get_or_init(|| Mutex::new(BTreeSet::new()));
    let mut names = names.lock().expect("generated color name registry");
    if let Some(existing) = names.get(value.as_str()) {
        return existing;
    }
    let value = Box::leak(value.into_boxed_str());
    names.insert(value);
    value
}

fn family_color(value: ColorFamily) -> ColorToken {
    value.color_token()
}

fn family_text_color(value: ColorFamily) -> ColorToken {
    value.text_token()
}

fn family_title_color(value: ColorFamily) -> ColorToken {
    value.title_token()
}

fn compose_screen_name(route: &str) -> String {
    format!("{}Screen", pascal_route(route))
}

fn pascal_route(route: &str) -> String {
    let mut name = String::new();

    for segment in route.split(|value: char| !value.is_ascii_alphanumeric()) {
        if segment.is_empty() {
            continue;
        }

        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            name.push(first.to_ascii_uppercase());
            for value in chars {
                name.push(value.to_ascii_lowercase());
            }
        }
    }

    if name.is_empty() {
        "Index".to_string()
    } else {
        name
    }
}
