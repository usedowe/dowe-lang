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
    css.push_str(&icon_button_geometry_css());
    let motion_css = DESIGN_MOTION_CSS
        .replace(
            "__DOWE_PAGE_TRANSITION_DURATION__",
            &format!("{}ms", dowe_components::VIEW_PAGE_TRANSITION_DURATION_MS),
        )
        .replace(
            "__DOWE_PAGE_TRANSITION_EASING__",
            dowe_components::VIEW_PAGE_TRANSITION_EASING_CSS,
        );
    append_css(&mut css, &[&motion_css]);
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
    append_visual_contract_variables(css);
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

fn append_visual_contract_variables(css: &mut String) {
    let chip = dowe_components::ChipVisualContract::for_size(ButtonSize::Md);
    let skeleton = dowe_components::SkeletonVisualContract::standard();
    let empty = dowe_components::EmptyVisualContract::standard();
    let alert = dowe_components::AlertVisualContract::standard();
    let button = dowe_components::ButtonVisualContract::for_size(ButtonSize::Md);
    let tabs = dowe_components::TabsVisualContract::standard();
    let stepper = dowe_components::StepperVisualContract::standard();
    let accordion = dowe_components::AccordionVisualContract::standard();
    let form = dowe_components::FormControlVisualContract::standard();
    let feedback = dowe_components::FeedbackSurfaceVisualContract::standard();
    let card = dowe_components::CardVisualContract::standard();
    let table = dowe_components::TableVisualContract::standard();
    let avatar = dowe_components::AvatarVisualContract::standard();
    let loading = dowe_components::LoadingVisualContract::standard();
    let media = dowe_components::MediaVisualContract::standard();
    let visualization = dowe_components::VisualizationVisualContract::standard();
    let shell = dowe_components::NavigationShellVisualContract::standard();
    let menu = dowe_components::MenuVisualContract::standard();
    let date = dowe_components::DatePickerVisualContract::standard();
    let progress = dowe_components::ProgressVisualContract::standard();
    let banner = dowe_components::BannerVisualContract::standard();
    let state = dowe_components::InteractionStateVisualContract::standard();
    append_custom_property(css, "chip-height-md", &format!("{}px", chip.height));
    append_custom_property(css, "chip-padding-md", &format!("{}px", chip.horizontal_padding));
    append_custom_property(css, "chip-text-md", &format!("{}px", chip.text_size));
    append_custom_property(css, "divider-thickness", &format!("{}px", dowe_components::DividerVisualContract::standard().thickness));
    append_custom_property(css, "skeleton-text-height", &format!("{}px", skeleton.text_height));
    append_custom_property(css, "skeleton-pulse-alpha", &skeleton.pulse_alpha.to_string());
    append_custom_property(css, "skeleton-pulse-duration", &format!("{}ms", skeleton.pulse_duration_ms));
    append_custom_property(css, "empty-padding", &format!("{}px", empty.panel_padding));
    append_custom_property(css, "empty-gap", &format!("{}px", empty.content_gap));
    append_custom_property(css, "empty-icon-size", &format!("{}px", empty.icon_size));
    append_custom_property(css, "alert-panel-padding", &format!("{}px", alert.panel_padding));
    append_custom_property(css, "alert-content-gap", &format!("{}px", alert.content_gap));
    append_custom_property(css, "alert-close-button-size", &format!("{}px", alert.close_button_size));
    append_custom_property(css, "button-height-md", &format!("{}px", button.min_height));
    append_custom_property(css, "button-padding-x-md", &format!("{}px", button.horizontal_padding));
    append_custom_property(css, "button-padding-y-md", &format!("{}px", button.vertical_padding));
    append_custom_property(css, "button-text-md", &format!("{}px", button.text_size));
    append_custom_property(css, "tabs-list-gap", &format!("{}px", tabs.list_gap));
    append_custom_property(css, "tabs-padding-x", &format!("{}px", tabs.tab_horizontal_padding));
    append_custom_property(css, "tabs-padding-y", &format!("{}px", tabs.tab_vertical_padding));
    append_custom_property(css, "tabs-indicator-thickness", &format!("{}px", tabs.indicator_thickness));
    append_custom_property(css, "stepper-indicator-size", &format!("{}px", stepper.indicator_size));
    append_custom_property(css, "stepper-connector-thickness", &format!("{}px", stepper.connector_thickness));
    append_custom_property(css, "accordion-header-height", &format!("{}px", accordion.header_min_height));
    append_custom_property(css, "accordion-header-padding", &format!("{}px", accordion.header_padding));
    append_custom_property(css, "accordion-content-padding", &format!("{}px", accordion.content_padding));
    append_custom_property(css, "form-control-height", &format!("{}px", form.min_height));
    append_custom_property(css, "form-control-padding-x", &format!("{}px", form.horizontal_padding));
    append_custom_property(css, "form-control-text-size", &format!("{}px", form.text_size));
    append_custom_property(css, "form-control-radius", &format!("{}px", form.radius));
    append_custom_property(css, "selection-control-size", "18px");
    append_custom_property(css, "selection-disabled-opacity", "0.5");
    append_custom_property(css, "feedback-panel-padding", &format!("{}px", feedback.panel_padding));
    append_custom_property(css, "feedback-content-gap", &format!("{}px", feedback.content_gap));
    append_custom_property(css, "tooltip-padding-x", &format!("{}px", feedback.tooltip_padding_x));
    append_custom_property(css, "tooltip-padding-y", &format!("{}px", feedback.tooltip_padding_y));
    append_custom_property(css, "card-padding", &format!("{}px", card.padding));
    append_custom_property(css, "card-content-gap", &format!("{}px", card.content_gap));
    append_custom_property(css, "card-border-width", &format!("{}px", card.border_width));
    append_custom_property(css, "table-header-padding-y", &format!("{}px", table.header_padding_y));
    append_custom_property(css, "table-cell-padding", &format!("{}px", table.cell_padding));
    append_custom_property(css, "table-text-size", &format!("{}px", table.text_size));
    append_custom_property(css, "table-divider-width", &format!("{}px", table.divider_width));
    append_custom_property(css, "avatar-group-overlap", &format!("{}px", avatar.group_overlap));
    append_custom_property(css, "avatar-indicator-border", &format!("{}px", avatar.indicator_border));
    append_custom_property(css, "avatar-counter-border", &format!("{}px", avatar.counter_border));
    append_custom_property(css, "loading-spinner-size", &format!("{}px", loading.spinner_size));
    append_custom_property(css, "loading-spinner-stroke", &format!("{}px", loading.stroke_width));
    append_custom_property(css, "loading-disabled-opacity", &loading.disabled_alpha.to_string());
    append_custom_property(css, "media-radius", &format!("{}px", media.default_radius));
    append_custom_property(css, "media-control-size", &format!("{}px", media.control_size));
    append_custom_property(css, "media-overlay-alpha", &format!("{}%", media.overlay_alpha));
    append_custom_property(css, "visualization-padding", &format!("{}px", visualization.container_padding));
    append_custom_property(css, "visualization-grid-width", &format!("{}px", visualization.grid_line_width));
    append_custom_property(css, "visualization-empty-height", &format!("{}px", visualization.empty_min_height));
    append_custom_property(css, "navigation-item-height", &format!("{}px", shell.item_min_height));
    append_custom_property(css, "navigation-item-padding", &format!("{}px", shell.item_padding));
    append_custom_property(css, "navigation-rail-width", &format!("{}px", shell.rail_width));
    append_custom_property(css, "navigation-item-gap", &format!("{}px", shell.item_gap));
    append_custom_property(css, "menu-option-height", &format!("{}px", menu.option_min_height));
    append_custom_property(css, "menu-option-padding", &format!("{}px", menu.option_padding));
    append_custom_property(css, "menu-gap", &format!("{}px", menu.menu_gap));
    append_custom_property(css, "menu-min-width", &format!("{}px", menu.min_width));
    append_custom_property(css, "date-cell-size", &format!("{}px", date.cell_size));
    append_custom_property(css, "date-grid-gap", &format!("{}px", date.grid_gap));
    append_custom_property(css, "date-nav-size", &format!("{}px", date.nav_size));
    append_custom_property(css, "progress-track-height", &format!("{}px", progress.track_height));
    append_custom_property(css, "progress-radius", &format!("{}px", progress.radius));
    append_custom_property(css, "progress-disabled-opacity", &format!("0.{}", progress.disabled_alpha));
    append_custom_property(css, "banner-padding", &format!("{}px", banner.padding));
    append_custom_property(css, "banner-content-gap", &format!("{}px", banner.content_gap));
    append_custom_property(css, "banner-action-height", &format!("{}px", banner.action_height));
    append_custom_property(css, "banner-disabled-opacity", &format!("0.{}", banner.disabled_alpha));
    append_custom_property(css, "state-disabled-opacity", &state.disabled_alpha.to_string());
    append_custom_property(css, "state-focus-ring-width", &format!("{}px", state.focus_ring_width));
    append_custom_property(css, "state-focus-ring-alpha", &state.focus_ring_alpha.to_string());
    append_custom_property(css, "state-pressed-scale", &state.pressed_scale.to_string());
    append_custom_property(css, "state-error-ring-width", &format!("{}px", state.error_ring_width));
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
                    "@font-face{{font-family:\"Dowe {}\";font-style:normal;font-weight:{};src:url(\"/fonts/{}/{}.ttf\") format(\"truetype\");font-display:optional;}}",
                    entry.display_name,
                    weight.numeric_weight,
                    font.as_str(),
                    weight.asset_stem
                ));
            }
        } else if *font != FontFamily::System {
            css.push_str(&format!(
                "@font-face{{font-family:\"Dowe {}\";font-style:normal;font-weight:300 800;src:local(\"{}\");font-display:optional;}}",
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
