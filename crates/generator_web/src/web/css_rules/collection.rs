fn push_custom_rule(rules: &mut Vec<String>, breakpoint: Breakpoint, rule: &str) {
    let rule = if breakpoint == Breakpoint::Xs {
        rule.to_string()
    } else {
        format!("@media (min-width:{}px){{{rule}}}", breakpoint.min_width())
    };
    if !rules.contains(&rule) {
        rules.push(rule);
    }
}

fn responsive_custom_class(breakpoint: Breakpoint, base: &str) -> String {
    if breakpoint == Breakpoint::Xs {
        base.to_string()
    } else {
        format!("{}:{base}", breakpoint.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssRuleFragment {
    breakpoint: Breakpoint,
    content: String,
}

fn push_css_rule_fragment(rules: &mut Vec<CssRuleFragment>, fragment: String) {
    for breakpoint in [
        Breakpoint::Xs,
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
    ] {
        let prefix = format!("@media (min-width:{}px){{", breakpoint.min_width());
        if let Some(content) = fragment
            .strip_prefix(&prefix)
            .and_then(|content| content.strip_suffix('}'))
        {
            rules.push(CssRuleFragment {
                breakpoint,
                content: content.to_string(),
            });
            return;
        }
    }
    rules.push(CssRuleFragment {
        breakpoint: Breakpoint::Xs,
        content: fragment,
    });
}

fn append_css_rule_fragments(css: &mut String, rules: &mut [CssRuleFragment]) {
    rules.sort_by_key(|rule| rule.breakpoint);
    for breakpoint in [
        Breakpoint::Xs,
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
    ] {
        let has_rules = rules.iter().any(|rule| rule.breakpoint == breakpoint);
        if !has_rules {
            continue;
        }
        if breakpoint != Breakpoint::Xs {
            css.push_str(&format!(
                "@media (min-width:{}px){{",
                breakpoint.min_width()
            ));
        }
        for rule in rules.iter().filter(|rule| rule.breakpoint == breakpoint) {
            css.push_str(&rule.content);
        }
        if breakpoint != Breakpoint::Xs {
            css.push('}');
        }
    }
}

fn push_variant_rule(
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
    base: &'static str,
    props: &VariantProps,
) {
    let default_variant = if base == "accordion" {
        ComponentVariant::Ghost
    } else {
        ComponentVariant::Solid
    };
    let rule = (
        base,
        props.color.unwrap_or(ColorFamily::Primary),
        props.variant.unwrap_or(default_variant),
    );
    if !variants.contains(&rule) {
        variants.push(rule);
    }
}

fn append_class_css(css: &mut String, class_name: &str) {
    if let Some((breakpoint, base)) = responsive_class(class_name) {
        if let Some(body) = class_body(base) {
            css.push_str(&format!(
                "@media (min-width:{}px){{",
                breakpoint.min_width()
            ));
            append_responsive_rule(css, breakpoint, base, &body);
            css.push('}');
        }
    } else if let Some(body) = class_body(class_name) {
        append_rule(css, class_name, &body);
    }
}

