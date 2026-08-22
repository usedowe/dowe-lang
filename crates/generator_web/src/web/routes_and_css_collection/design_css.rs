fn design_css_for_fonts(
    used_fonts: &BTreeSet<FontFamily>,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    compose_design_css(
        used_fonts,
        font_config,
        design_config,
        DesignCssFeatures::all(),
    )
}

#[cfg(test)]
fn design_css_for_trees<'a>(
    roots: impl IntoIterator<Item = &'a ViewNode>,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    let roots = roots.into_iter().collect::<Vec<_>>();
    let features = DesignCssFeatures::collect(roots.iter().copied());
    let mut fonts = BTreeSet::new();
    for root in roots {
        collect_node_font_families(root, &mut fonts);
    }
    compose_design_css(&fonts, font_config, design_config, features)
}

fn design_css_for_web(
    web: &WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    let roots = web
        .pages
        .iter()
        .flat_map(|page| [&page.layout_tree, &page.page_tree])
        .collect::<Vec<_>>();
    let features = DesignCssFeatures::collect(roots.iter().copied());
    let mut fonts = BTreeSet::new();
    for root in roots {
        collect_node_font_families(root, &mut fonts);
    }
    compose_design_base_css(
        &fonts,
        font_config,
        design_config,
        features.forms,
        features.section_center,
        features.box_center,
    )
}

fn design_css_file_name(css: &str) -> String {
    format!("design-{}.css", short_id("design", css))
}

fn compose_design_css(
    used_fonts: &BTreeSet<FontFamily>,
    font_config: &FontConfig,
    design_config: &DesignConfig,
    features: DesignCssFeatures,
) -> String {
    let mut css = compose_design_base_css(
        used_fonts,
        font_config,
        design_config,
        features.forms,
        features.section_center,
        features.box_center,
    );
    for chunk in design_css_chunks(features) {
        css.push_str(&chunk.content);
    }
    css
}

fn compose_design_base_css(
    used_fonts: &BTreeSet<FontFamily>,
    font_config: &FontConfig,
    design_config: &DesignConfig,
    include_form_metrics: bool,
    include_section_center_rules: bool,
    include_box_center_rules: bool,
) -> String {
    let fonts = font_config.effective_families(used_fonts);
    let mut css = String::new();
    append_root_css(
        &mut css,
        &fonts,
        font_config.default_family,
        design_config,
        include_form_metrics,
    );
    append_font_faces(&mut css, &fonts);
    append_css(&mut css, &[DESIGN_RESET_CSS]);
    append_visibility_base_css(&mut css);
    append_css(&mut css, &[DESIGN_FOUNDATION_CSS]);
    append_css(&mut css, &[DESIGN_MOTION_CSS]);
    append_responsive_visibility_css(&mut css);
    if include_section_center_rules {
        append_responsive_section_center_css(&mut css);
    }
    if include_box_center_rules {
        append_responsive_box_center_css(&mut css);
    }
    minify_css(&css)
}

fn append_root_css(
    css: &mut String,
    fonts: &BTreeSet<FontFamily>,
    default_font: FontFamily,
    design_config: &DesignConfig,
    include_form_metrics: bool,
) {
    css.push_str(":root{");
    append_theme_variables(css, design_config.default_theme());
    for font in fonts {
        append_custom_property(css, &format!("dowe-font-{}", font.as_str()), font_stack(*font));
    }
    for value in ContainerSize::all() {
        append_custom_property(
            css,
            &format!("container-{}", value.as_str()),
            value.css_rem(),
        );
    }
    append_custom_property(
        css,
        "dowe-font-default",
        &format!("var(--dowe-font-{})", default_font.as_str()),
    );
    if include_form_metrics {
        append_form_metrics(css);
    }
    css.push('}');

    for theme in &design_config.themes {
        if theme.name != design_config.default_theme {
            css.push_str(&format!(
                "[data-dowe-theme=\"{}\"]{{",
                escape_css_string(&theme.name)
            ));
            append_theme_variables(css, theme);
            css.push('}');
        }
    }
}

fn append_form_metrics(css: &mut String) {
    for (name, size) in [
        ("sm", ButtonSize::Sm),
        ("md", ButtonSize::Md),
        ("lg", ButtonSize::Lg),
    ] {
        append_custom_property(
            css,
            &format!("dowe-form-control-min-{name}"),
            &scale_rem(form_control_min_height(size, false)),
        );
        append_custom_property(
            css,
            &format!("dowe-form-control-text-{name}"),
            &text_size_css(form_control_text_size(size)),
        );
        append_custom_property(
            css,
            &format!("dowe-form-control-line-{name}"),
            text_line_css(form_control_text_size(size)),
        );
    }
    append_custom_property(
        css,
        "dowe-form-control-floating",
        &scale_rem(FORM_CONTROL_FLOATING_HEIGHT_INCREMENT),
    );
    append_custom_property(
        css,
        "dowe-form-control-padding",
        &scale_rem(INPUT_HORIZONTAL_PADDING),
    );
    append_custom_property(
        css,
        "dowe-form-control-padding-double",
        &scale_rem(ScaleValue::from_half_steps(INPUT_HORIZONTAL_PADDING.0 * 2)),
    );
    append_custom_property(
        css,
        "dowe-form-control-label-start",
        &scale_rem(ScaleValue::from_half_steps(INPUT_HORIZONTAL_PADDING.0 + 16)),
    );
    append_custom_property(
        css,
        "dowe-form-control-label-width",
        &scale_rem(ScaleValue::from_half_steps(
            INPUT_HORIZONTAL_PADDING.0 * 2 + 16,
        )),
    );
}

fn append_custom_property(css: &mut String, name: &str, value: &str) {
    css.push_str("--");
    css.push_str(name);
    css.push(':');
    css.push_str(value);
    css.push(';');
}

fn append_font_faces(css: &mut String, fonts: &BTreeSet<FontFamily>) {
    for font in fonts {
        let entry = font.catalog_entry();
        if entry.package_assets {
            for weight in entry.weights {
                css.push_str(&format!(
                    "@font-face{{font-family:\"Dowe {}\";font-style:normal;font-weight:{};src:url(\"/fonts/{}/{}.ttf\") format(\"truetype\");font-display:swap;}}",
                    entry.display_name,
                    weight.numeric_weight,
                    font.as_str(),
                    weight.asset_stem
                ));
            }
        } else if *font != FontFamily::System {
            css.push_str(&format!(
                "@font-face{{font-family:\"Dowe {}\";font-style:normal;font-weight:300 800;src:local(\"{}\");font-display:swap;}}",
                entry.display_name, entry.display_name
            ));
        }
    }
}

fn append_css(css: &mut String, sources: &[&str]) {
    for source in sources {
        css.push_str(source);
        css.push('\n');
    }
}

fn append_theme_variables(css: &mut String, theme: &DesignTheme) {
    for token in theme.ordered_color_tokens() {
        append_custom_property(
            css,
            &format!("dowe-{}", token.as_str()),
            theme.color_value(token),
        );
    }
    append_custom_property(css, "dowe-radius", &format!("{}px", theme.radius));
}

fn append_responsive_visibility_css(css: &mut String) {
    for breakpoint in [Breakpoint::Sm, Breakpoint::Md, Breakpoint::Lg, Breakpoint::Xl] {
        css.push_str(&format!("@media (min-width:{}px){{", breakpoint.min_width()));
        for value in [false, true] {
            css.push_str(&format!(
                ".{}\\:show-{value}:not([hidden]){{display:{};}}",
                breakpoint.as_str(),
                visibility_display(value)
            ));
        }
        css.push('}');
    }
}

fn append_responsive_box_center_css(css: &mut String) {
    for breakpoint in [Breakpoint::Sm, Breakpoint::Md, Breakpoint::Lg, Breakpoint::Xl] {
        css.push_str(&format!("@media (min-width:{}px){{", breakpoint.min_width()));
        css.push_str(&format!(".{}\\:box-center-x-true{{align-items:center;}}", breakpoint.as_str()));
        css.push_str(&format!(".{}\\:box-center-x-false{{align-items:flex-start;}}", breakpoint.as_str()));
        css.push_str(&format!(".{}\\:box-center-y-true{{justify-content:center;}}", breakpoint.as_str()));
        css.push_str(&format!(".{}\\:box-center-y-false{{justify-content:flex-start;}}", breakpoint.as_str()));
        css.push('}');
    }
}

fn append_responsive_section_center_css(css: &mut String) {
    for breakpoint in [Breakpoint::Sm, Breakpoint::Md, Breakpoint::Lg, Breakpoint::Xl] {
        css.push_str(&format!("@media (min-width:{}px){{", breakpoint.min_width()));
        css.push_str(&format!(
            ".{}\\:section-center-x-true{{align-items:center;}}",
            breakpoint.as_str()
        ));
        css.push_str(&format!(
            ".{}\\:section-center-x-false{{align-items:flex-start;}}",
            breakpoint.as_str()
        ));
        css.push_str(&format!(
            ".{}\\:section-center-y-true{{justify-content:center;}}",
            breakpoint.as_str()
        ));
        css.push_str(&format!(
            ".{}\\:section-center-y-false{{justify-content:flex-start;}}",
            breakpoint.as_str()
        ));
        css.push('}');
    }
}

fn append_visibility_base_css(css: &mut String) {
    for value in [false, true] {
        css.push_str(&format!(
            ".show-{value}:not([hidden]){{display:{};}}",
            visibility_display(value)
        ));
    }
}

fn visibility_display(value: bool) -> &'static str {
    if value {
        "var(--dowe-component-display,revert)"
    } else {
        "none"
    }
}

fn font_stack(value: FontFamily) -> &'static str {
    value.catalog_entry().web_stack
}
