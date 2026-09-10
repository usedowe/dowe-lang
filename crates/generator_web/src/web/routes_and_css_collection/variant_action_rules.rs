fn push_form_variant_rules(
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
    base: &'static str,
    props: &VariantProps,
) {
    if props.reactive.variant.is_some() || props.reactive.scheme.is_some() {
        for variant in [
            ComponentVariant::Solid,
            ComponentVariant::Outlined,
            ComponentVariant::Ghost,
        ] {
            for color in [
                ColorFamily::Primary,
                ColorFamily::Secondary,
                ColorFamily::Accent,
                ColorFamily::Muted,
                ColorFamily::Success,
                ColorFamily::Info,
                ColorFamily::Warning,
                ColorFamily::Danger,
            ] {
                let mut reactive_props = props.clone();
                reactive_props.variant = Some(variant);
                reactive_props.color = Some(color);
                push_variant_rule(variants, base, &reactive_props);
            }
        }
    } else {
        push_variant_rule(variants, base, props);
    }
}

fn collect_action_toast_variant_rules(
    action: &ViewAction,
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
) {
    if let ViewActionKind::Sequence(statements) = &action.kind {
        collect_statement_toast_variant_rules(statements, variants);
    }
}

fn collect_statement_toast_variant_rules(
    statements: &[ViewFunctionStatement],
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
) {
    for statement in statements {
        match statement {
            ViewFunctionStatement::Toast(toast) => {
                let family = toast
                    .scheme
                    .as_deref()
                    .and_then(ColorFamily::from_name)
                    .or_else(|| ColorFamily::from_name(&toast.kind))
                    .unwrap_or(if toast.kind == "error" {
                        ColorFamily::Danger
                    } else {
                        ColorFamily::Info
                    });
                let variant = toast
                    .variant
                    .as_deref()
                    .and_then(ComponentVariant::from_name)
                    .unwrap_or(ComponentVariant::Solid);
                let rule = ("toast", family, variant);
                if !variants.contains(&rule) {
                    variants.push(rule);
                }
            }
            ViewFunctionStatement::If { success, error, .. } => {
                collect_statement_toast_variant_rules(success, variants);
                collect_statement_toast_variant_rules(error, variants);
            }
            ViewFunctionStatement::Request { .. }
            | ViewFunctionStatement::Validate { .. }
            | ViewFunctionStatement::Invoke { .. }
            | ViewFunctionStatement::Assign(_)
            | ViewFunctionStatement::Reset(_)
            | ViewFunctionStatement::Redirect { .. } => {}
        }
    }
}

fn collect_nav_menu_variant_rules(
    item: &NavMenuItem,
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
) {
    if let NavMenuItem::Megamenu { content, .. } = item {
        for child in content {
            collect_variant_rules(child, variants);
        }
    }
}
