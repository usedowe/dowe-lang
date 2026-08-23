fn render_password_strength(props: &PasswordProps) -> String {
    if props.hide_strength {
        return String::new();
    }
    format!(
        r#"<span class="password-strength" data-dowe-password-strength data-dowe-weak-label="{}" data-dowe-medium-label="{}" data-dowe-strong-label="{}"><span class="password-strength-bars">{}</span><span class="password-strength-label"></span></span>"#,
        escape_attr(&props.weak_label),
        escape_attr(&props.medium_label),
        escape_attr(&props.strong_label),
        (0..6)
            .map(|_| r#"<span class="password-strength-bar"></span>"#)
            .collect::<String>()
    )
}

