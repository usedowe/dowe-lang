fn render_color_values(props: &ColorProps) -> String {
    if !(props.show_hex || props.show_rgb || props.show_cmyk || props.show_oklch) {
        return String::new();
    }
    let mut html = String::from("<span class=\"color-picker-values\">");
    if props.show_hex {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="hex"></code>"#,
        );
    }
    if props.show_rgb {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="rgb"></code>"#,
        );
    }
    if props.show_cmyk {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="cmyk"></code>"#,
        );
    }
    if props.show_oklch {
        html.push_str(
            r#"<code class="color-picker-value-code" data-dowe-color-format="oklch"></code>"#,
        );
    }
    html.push_str("</span>");
    html
}
