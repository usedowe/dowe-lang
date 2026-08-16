fn design_css_chunks_for_web(web: &WebOutput) -> Vec<GeneratedDesignCssChunk> {
    design_css_chunks(DesignCssFeatures::collect(
        web.pages
            .iter()
            .flat_map(|page| [&page.layout_tree, &page.page_tree]),
    ))
}

fn design_css_chunks(features: DesignCssFeatures) -> Vec<GeneratedDesignCssChunk> {
    let mut chunks = Vec::new();
    if features.content {
        chunks.push(design_css_chunk("content", DESIGN_CONTENT_CSS, false));
    }
    if features.forms {
        let mut sources = DESIGN_FORMS_BEFORE_CONTROL_CSS.to_vec();
        sources.push(include_str!("design_css/forms_control.css"));
        sources.extend_from_slice(DESIGN_FORMS_AFTER_CONTROL_CSS);
        chunks.push(design_css_chunk("forms", &sources, false));
    }
    if features.media {
        chunks.push(design_css_chunk("media", DESIGN_MEDIA_CSS, false));
    }
    if features.visualization {
        chunks.push(design_css_chunk(
            "visualization",
            DESIGN_VISUALIZATION_CSS,
            true,
        ));
    }
    if features.disclosure {
        chunks.push(design_css_chunk(
            "disclosure",
            DESIGN_DISCLOSURE_CSS,
            false,
        ));
    }
    if features.feedback {
        chunks.push(design_css_chunk("feedback", DESIGN_FEEDBACK_CSS, false));
    }
    if features.navigation {
        chunks.push(design_css_chunk(
            "navigation",
            DESIGN_NAVIGATION_CSS,
            false,
        ));
    }
    if features.overlays {
        chunks.push(design_css_chunk("overlays", DESIGN_OVERLAYS_CSS, true));
    }
    chunks
}

fn design_css_chunk(
    name: &'static str,
    sources: &[&str],
    include_responsive_rules: bool,
) -> GeneratedDesignCssChunk {
    let mut css = String::new();
    append_css(&mut css, sources);
    if include_responsive_rules {
        append_responsive_capability_css(&mut css, name);
    }
    GeneratedDesignCssChunk::new(name, minify_css(&css))
}

fn append_responsive_capability_css(css: &mut String, name: &str) {
    css.push_str(&format!(
        "@media (min-width:{}px){{",
        Breakpoint::Md.min_width()
    ));
    if name == "visualization" {
        css.push_str(".arc-chart-container.legend-left,.area-chart-container.legend-left,.bar-chart-container.legend-left,.line-chart-container.legend-left,.pie-chart-container.legend-left{flex-direction:row-reverse;}.arc-chart-container.legend-right,.area-chart-container.legend-right,.bar-chart-container.legend-right,.line-chart-container.legend-right,.pie-chart-container.legend-right{flex-direction:row;}");
    }
    if name == "overlays" {
        css.push_str(".command-kbd{display:flex;}");
    }
    css.push('}');
}
