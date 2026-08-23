fn render_combo_option_html(option: &ComboOption) -> String {
    let media = option
        .src
        .as_deref()
        .map(|src| {
            format!(
                r#"<img class="combo-box-option-avatar" src="{}" alt="">"#,
                escape_attr(src)
            )
        })
        .or_else(|| {
            option
                .icon
                .map(|icon| view_icon_svg(icon, "combo-box-option-icon"))
        })
        .unwrap_or_default();
    let description = option
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="combo-box-option-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<button type="button" class="combo-box-option" role="option" data-dowe-combo-value="{}" data-dowe-combo-label="{}"{}>{media}<span class="combo-box-option-copy"><span class="combo-box-option-label">{}</span>{description}</span></button>"#,
        escape_attr(&option.value),
        escape_attr(&option.label),
        if option.disabled { " disabled" } else { "" },
        escape_html(&option.label)
    )
}

