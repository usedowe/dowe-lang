fn validate_node_references(
    path: &Path,
    node: &ViewNode,
    signals: &HashMap<String, ViewSignalValue>,
    writable_signals: &HashSet<String>,
    actions: &HashSet<String>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> DoweResult<()> {
    if let Some(props) = node_element_props(node) {
        if let Some(VisibilityCondition::Signal(show)) = props.show.as_ref() {
            validate_typed_path(
                path,
                signals,
                locals,
                show,
                "show",
                ViewPathExpectation::Bool,
            )?;
        }
        if let Some(VisibilityCondition::NumberComparison { path: show, .. }) = props.show.as_ref()
        {
            validate_typed_path(
                path,
                signals,
                locals,
                show,
                "show.when",
                ViewPathExpectation::Number,
            )?;
        }
        if let Some(binding) = props.bind.as_ref() {
            if signals.contains_key(path_root(binding))
                && !writable_signals.contains(path_root(binding))
            {
                return Err(DoweError::at_path(
                    path,
                    format!("constant path `{binding}` cannot be used in `bind`"),
                ));
            }
            let expectation = match node {
                ViewNode::Checkbox { .. } | ViewNode::Toggle { .. } => ViewPathExpectation::Bool,
                ViewNode::Slider { .. } => ViewPathExpectation::Number,
                _ => ViewPathExpectation::String,
            };
            validate_typed_path(path, signals, locals, binding, "bind", expectation)?;
        }
        if let Some(action) = props.on_click.as_ref()
            && !actions.contains(action)
        {
            return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
        }
    }

    if let ViewNode::Button { props, .. } = node {
        if let Some(binding) = props.reactive.loading.as_deref() {
            validate_typed_path(
                path,
                signals,
                locals,
                binding,
                "loading",
                ViewPathExpectation::Bool,
            )?;
        }
        for (name, binding) in [
            ("variant", props.reactive.variant.as_deref()),
            ("scheme", props.reactive.scheme.as_deref()),
            ("size", props.reactive.size.as_deref()),
            ("rounded", props.reactive.rounded.as_deref()),
        ] {
            if let Some(binding) = binding {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    binding,
                    name,
                    ViewPathExpectation::String,
                )?;
                if let Some(ViewSignalValue::String(value)) =
                    signal_path_value(path, signals, locals, binding, name)?
                {
                    let allowed: &[&str] = match name {
                        "variant" => &["solid", "soft", "outlined", "ghost"],
                        "scheme" => &[
                            "primary",
                            "secondary",
                            "tertiary",
                            "muted",
                            "success",
                            "info",
                            "warning",
                            "danger",
                        ],
                        "size" => &["xs", "sm", "md", "lg", "xl"],
                        "rounded" => &["xs", "sm", "md", "lg", "xl", "full"],
                        _ => &[],
                    };
                    if !allowed.contains(&value.as_str()) {
                        return Err(DoweError::at_path(
                            path,
                            format!(
                                "invalid initial value `{value}` for reactive Button prop `{name}`"
                            ),
                        ));
                    }
                }
            }
        }
        for (name, binding, numeric) in [
            (
                "iconStart.when",
                props.reactive.icon_start_when.as_deref(),
                props.reactive.icon_start_comparison.is_some(),
            ),
            (
                "iconEnd.when",
                props.reactive.icon_end_when.as_deref(),
                props.reactive.icon_end_comparison.is_some(),
            ),
        ] {
            if let Some(binding) = binding {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    binding,
                    name,
                    if numeric {
                        ViewPathExpectation::Number
                    } else {
                        ViewPathExpectation::Bool
                    },
                )?;
            }
        }
    }

    if let ViewNode::SideNav { props, .. } = node {
        for (name, binding) in [
            ("variant", props.style.reactive.variant.as_deref()),
            ("scheme", props.style.reactive.scheme.as_deref()),
            ("size", props.style.reactive.size.as_deref()),
        ] {
            if let Some(binding) = binding {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    binding,
                    name,
                    ViewPathExpectation::String,
                )?;
                if let Some(ViewSignalValue::String(value)) =
                    signal_path_value(path, signals, locals, binding, name)?
                {
                    let allowed: &[&str] = match name {
                        "variant" => &["solid", "soft", "outlined", "ghost"],
                        "scheme" => &[
                            "primary",
                            "secondary",
                            "tertiary",
                            "muted",
                            "success",
                            "info",
                            "warning",
                            "danger",
                        ],
                        "size" => &["sm", "md", "lg"],
                        _ => &[],
                    };
                    if !allowed.contains(&value.as_str()) {
                        return Err(DoweError::at_path(
                            path,
                            format!(
                                "invalid initial value `{value}` for reactive SideNav prop `{name}`"
                            ),
                        ));
                    }
                }
            }
        }
        if let Some(binding) = props.reactive_wide.as_deref() {
            validate_typed_path(
                path,
                signals,
                locals,
                binding,
                "wide",
                ViewPathExpectation::Bool,
            )?;
        }
    }
    if let ViewNode::Code { props } = node {
        for segment in &props.template_segments {
            if let CodeTemplateSegment::Binding(binding) = segment {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    binding,
                    "Code template",
                    ViewPathExpectation::Any,
                )?;
            }
        }
    }
    validate_node_variant_references(
        path,
        node,
        signals,
        writable_signals,
        actions,
        locals,
    )
}
