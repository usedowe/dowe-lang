fn render_phone_html(props: &PhoneProps, context: &ReactiveRenderContext) -> String {
    let country = phone_country(props.country.as_deref()).unwrap_or_else(|| phone_countries()[0]);
    let selected_code = country.code;
    let options = ordered_phone_countries(Some(country.code), &props.priority_countries)
        .iter()
        .map(|country| {
            format!(
                r#"<button type="button" class="phone-country" data-dowe-phone-option data-dowe-country="{}" data-dowe-dial="{}" aria-selected="{}"><span class="phone-flag">{}</span><span class="phone-country-name">{}</span><span class="phone-dial">+{}</span></button>"#,
                escape_attr(country.code),
                escape_attr(country.dial),
                country.code == selected_code,
                country_flag_html(country.code, context),
                escape_html(country.name),
                escape_html(country.dial)
            )
        })
        .collect::<String>();
    let priority = props.priority_countries.join(",");
    let country_trigger = format!(
        r#"<button class="phone-country-trigger" type="button" data-dowe-phone-country aria-expanded="false" aria-haspopup="listbox"><span class="phone-flag">{}</span><span class="phone-dial">+{}</span>{}</button>"#,
        country_flag_html(country.code, context),
        escape_html(country.dial),
        select_arrow_svg()
    );
    let number_input = format!(
        r#"<span class="phone-input-shell">{}<input class="phone-input input" type="tel" inputmode="numeric" pattern="[0-9]*"{}{}{}{}{} data-dowe-phone-input data-dowe-validation-control></span>"#,
        floating_label_html(&props.style),
        input_placeholder_attr(&props.style),
        props
            .value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled { " disabled" } else { "" }
    );
    let input = format!(
        r#"<input type="hidden" name="{}" value="{}" data-dowe-phone-dial>{}{}<div class="phone-popover" data-dowe-phone-popover hidden><div class="phone-search-wrap">{}<input class="phone-search" type="search" placeholder="{}" data-dowe-phone-search></div><div class="phone-countries" data-dowe-phone-countries role="listbox">{options}</div><div class="phone-empty" hidden>{}</div><div class="phone-loading" hidden>{}</div></div>"#,
        escape_attr(&props.dial_code_name),
        escape_attr(country.dial),
        country_trigger,
        number_input,
        view_icon_svg(ViewIcon::Search, "phone-search-icon"),
        escape_attr(&props.search_placeholder),
        escape_html(&props.empty_text),
        escape_html(&props.loading_text)
    );
    let extra = format!(
        r#" data-dowe-phone data-dowe-country="{}" data-dowe-priority-countries="{}""#,
        escape_attr(country.code),
        escape_attr(&priority)
    );
    let mut control_classes = variant_classes("control", &props.style);
    control_classes.push("phone".to_string());
    control_classes.push(format!(
        "is-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.style.label_floating {
        control_classes.push("is-floating".to_string());
    }
    if props.error_text.is_some()
        || props
            .style
            .element
            .form_validation()
            .and_then(|validation| validation.error_text.as_ref())
            .is_some()
    {
        control_classes.push("is-error".to_string());
    }
    let control = format!(
        "<span{}>{}</span>",
        attrs(
            control_classes,
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        input
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &control,
        context,
    )
}

