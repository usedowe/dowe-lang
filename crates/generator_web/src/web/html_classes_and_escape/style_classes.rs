fn append_style_classes(classes: &mut Vec<String>, props: &StyleProps) {
    append_reactive_style_markers(classes, props);
    append_show_classes(classes, props.element.show.as_ref());
    append_responsive_classes(classes, "font", props.font.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(classes, "bg", props.bg.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(classes, "color", props.text.as_ref(), |value| {
        value.as_str().to_string()
    });
    if let Some(animation) = props.animation()
        && animation != ViewAnimation::None
    {
        classes.push(format!("animate-{}", animation.class_suffix()));
    }
    let motion = props.motion();
    let transformed = motion.rotate.is_some()
        || motion.scale.is_some()
        || motion.translate_x.is_some()
        || motion.translate_y.is_some();
    if transformed || !matches!(motion.gesture, None | Some(ViewGesture::None)) {
        classes.push("has-transform".to_string());
    }
    append_responsive_classes(classes, "rotate", motion.rotate.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "scale", motion.scale.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(
        classes,
        "translate-x",
        motion.translate_x.as_ref(),
        |value| value.class_suffix(),
    );
    append_responsive_classes(
        classes,
        "translate-y",
        motion.translate_y.as_ref(),
        |value| value.class_suffix(),
    );
    if let Some(transition) = motion.transition {
        classes.push(format!("transition-{}", transition.as_str()));
    }
    if let Some(gesture) = motion.gesture
        && gesture != ViewGesture::None
    {
        classes.push("has-gesture".to_string());
        classes.push(format!("gesture-{}", gesture.as_str()));
    }
    let position = props.position();
    if position.mode != BoxPosition::Static {
        classes.push(format!("position-{}", position.mode.as_str()));
    }
    if matches!(position.mode, BoxPosition::Absolute | BoxPosition::Fixed) {
        if position.top.is_none() && position.bottom.is_none() {
            classes.push("top-0".to_string());
        }
        if position.left.is_none() && position.right.is_none() {
            classes.push("left-0".to_string());
        }
    }
    append_responsive_classes(classes, "top", position.top.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "right", position.right.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "bottom", position.bottom.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "left", position.left.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "p", props.spacing.p.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "px", props.spacing.px.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "py", props.spacing.py.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pl", props.spacing.pl.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pr", props.spacing.pr.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pt", props.spacing.pt.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "pb", props.spacing.pb.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(classes, "w", props.sizing.w.as_ref(), size_suffix);
    append_height_classes(classes, props.sizing.h.as_ref());
    append_responsive_classes(classes, "min-w", props.sizing.min_w.as_ref(), size_suffix);
    append_responsive_classes(classes, "min-h", props.sizing.min_h.as_ref(), size_suffix);
    append_responsive_classes(classes, "max-w", props.sizing.max_w.as_ref(), size_suffix);
    append_responsive_classes(classes, "max-h", props.sizing.max_h.as_ref(), size_suffix);
    append_responsive_classes(classes, "flex", props.flex.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(classes, "rounded", props.rounded.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(classes, "border", props.border.as_ref(), |value| {
        value.0.to_string()
    });
    if let Some(value) = props.border_color {
        classes.push(format!("border-color-{}", value.as_str()));
    }
    append_responsive_classes(classes, "shadow", props.shadow.as_ref(), |value| {
        value.as_str().to_string()
    });
    if let Some(value) = props.shadow_color {
        classes.push(format!("shadow-color-{}", value.as_str()));
    }
    append_responsive_classes(
        classes,
        "col-span",
        props.grid_item().col_span.as_ref(),
        |value| value.0.to_string(),
    );
    append_responsive_classes(
        classes,
        "row-span",
        props.grid_item().row_span.as_ref(),
        |value| value.0.to_string(),
    );
}

fn append_reactive_style_markers(classes: &mut Vec<String>, props: &StyleProps) {
    for binding in props.bindings() {
        let name = match binding.property {
            dowe_components::StyleBindingProperty::BackgroundColor => "bg",
            dowe_components::StyleBindingProperty::TextColor => "color",
            dowe_components::StyleBindingProperty::Padding => "p",
            dowe_components::StyleBindingProperty::PaddingInline => "px",
            dowe_components::StyleBindingProperty::PaddingBlock => "py",
            dowe_components::StyleBindingProperty::PaddingLeft => "pl",
            dowe_components::StyleBindingProperty::PaddingRight => "pr",
            dowe_components::StyleBindingProperty::PaddingTop => "pt",
            dowe_components::StyleBindingProperty::PaddingBottom => "pb",
            dowe_components::StyleBindingProperty::Width => "w",
            dowe_components::StyleBindingProperty::Height => "h",
            dowe_components::StyleBindingProperty::MinWidth => "minW",
            dowe_components::StyleBindingProperty::MinHeight => "minH",
            dowe_components::StyleBindingProperty::MaxWidth => "maxW",
            dowe_components::StyleBindingProperty::MaxHeight => "maxH",
            dowe_components::StyleBindingProperty::BorderWidth => "border",
            dowe_components::StyleBindingProperty::BorderRadius => "rounded",
        };
        classes.push(format!("dowe-style-binding-{name}-{}", binding.binding.path));
    }
}

fn append_show_classes(classes: &mut Vec<String>, value: Option<&VisibilityCondition>) {
    if let Some(VisibilityCondition::Static(value)) = value {
        append_responsive_classes(classes, "show", Some(value), |value| value.to_string());
    }
}

fn append_container_visual_classes(classes: &mut Vec<String>, props: &StyleProps) {
    if let Some(background) = props.background.as_ref() {
        classes.push("has-background".to_string());
        append_responsive_classes(classes, "background", Some(background), |value| {
            value.as_str().to_string()
        });
    }
    if let Some(cover) = props.cover.as_ref() {
        classes.push("has-cover".to_string());
        append_responsive_classes(classes, "cover", Some(cover), |value| cover_suffix(value));
    }
    if let Some(overlay) = props.overlay.as_ref() {
        classes.push("has-overlay".to_string());
        append_responsive_classes(classes, "overlay", Some(overlay), |value| {
            overlay_suffix(value)
        });
    }
}

fn append_responsive_classes<T, F>(
    classes: &mut Vec<String>,
    prefix: &str,
    value: Option<&ResponsiveValue<T>>,
    suffix: F,
) where
    F: Fn(&T) -> String,
{
    let Some(value) = value else {
        return;
    };

    for entry in &value.entries {
        let class_name = format!("{prefix}-{}", suffix(&entry.value));
        if entry.breakpoint == Breakpoint::Xs {
            classes.push(class_name);
        } else {
            classes.push(format!("{}:{class_name}", entry.breakpoint.as_str()));
        }
    }
}

fn append_height_classes(classes: &mut Vec<String>, value: Option<&ResponsiveValue<SizeValue>>) {
    let Some(value) = value else {
        return;
    };

    for entry in &value.entries {
        let class_name = match entry.value {
            SizeValue::ViewportMinus(value) => format!("vh-{}", value.class_suffix()),
            _ => format!("h-{}", size_suffix(&entry.value)),
        };
        if entry.breakpoint == Breakpoint::Xs {
            classes.push(class_name);
        } else {
            classes.push(format!("{}:{class_name}", entry.breakpoint.as_str()));
        }
    }
}

fn size_suffix(value: &SizeValue) -> String {
    match value {
        SizeValue::Scale(value) => value.class_suffix(),
        SizeValue::Container(value) => value.as_str().to_string(),
        SizeValue::Percent(value) => format!("pct-{value}"),
        SizeValue::Full => "full".to_string(),
        SizeValue::Auto => "auto".to_string(),
        SizeValue::ViewportMinus(value) => format!("vh-{}", value.class_suffix()),
    }
}

