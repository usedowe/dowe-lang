fn apply_action_control_press_feedback(props: &mut VariantProps) {
    props
        .style
        .motion_mut()
        .gesture
        .get_or_insert(ViewGesture::Press);
}

fn apply_design_defaults_to_actions(actions: &mut [ViewAction], defaults: &DesignDefaults) {
    for action in actions {
        if let ViewActionKind::Sequence(statements) = &mut action.kind {
            apply_design_defaults_to_statements(statements, defaults);
        }
    }
}

fn apply_design_defaults_to_statements(
    statements: &mut [ViewFunctionStatement],
    defaults: &DesignDefaults,
) {
    for statement in statements {
        match statement {
            ViewFunctionStatement::Toast(toast) => {
                if toast.variant.is_none() {
                    toast.variant = defaults
                        .variant
                        .get(&DesignComponentSlot::Toast)
                        .or_else(|| defaults.variant.get(&DesignComponentSlot::Ui))
                        .map(|variant| variant.as_str().to_string());
                }
            }
            ViewFunctionStatement::If { success, error, .. } => {
                apply_design_defaults_to_statements(success, defaults);
                apply_design_defaults_to_statements(error, defaults);
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

fn apply_section_defaults(props: &mut StyleProps, defaults: &DesignDefaults) {
    if props.bg.is_none()
        && let Some(family) = defaults
            .scheme
            .get(&DesignComponentSlot::Section)
            .or_else(|| defaults.scheme.get(&DesignComponentSlot::Ui))
    {
        props.bg = Some(ResponsiveValue::scalar(family.color_token()));
    }
    apply_style_defaults(props, defaults, DesignComponentSlot::Section);
}

fn apply_text_defaults(
    props: &mut TextProps,
    defaults: &DesignDefaults,
    slot: DesignComponentSlot,
) {
    if props.style.font.is_none()
        && let Some(value) = defaults.font.get(&slot)
    {
        let font = ResponsiveValue::scalar(*value);
        props.style.element.font = Some(font.clone());
        props.style.font = Some(font);
    }
}

