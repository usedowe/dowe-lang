fn render_field_block(
    props: &VariantProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    body_html: &str,
    context: &ReactiveRenderContext,
) -> String {
    render_field_block_kind(props, help_text, error_text, body_html, "string", context)
}

