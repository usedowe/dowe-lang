fn country_flag_html(code: &str, context: &ReactiveRenderContext) -> String {
    phone_country_flag_icon(code)
        .map(|icon| render_svg_html(&icon.props, &icon.paths, context))
        .unwrap_or_else(|| "<span class=\"phone-flag-fallback\">--</span>".to_string())
}

