fn validate_node_structure_and_content(
    path: &Path,
    node: &ViewNode,
    signals: &HashMap<String, ViewSignalValue>,
    writable_signals: &HashSet<String>,
    actions: &HashSet<String>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> Option<DoweResult<()>> {
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => Some((|| -> DoweResult<()> {
            validate_typed_path(
                path,
                signals,
                locals,
                binding,
                "Splash bind",
                ViewPathExpectation::Bool,
            )?;
            for child in content.iter().chain(children) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
            Ok(())
        })()),
        ViewNode::Scope { children, .. } => Some((|| -> DoweResult<()> {
            for child in children {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
            Ok(())
        })()),
        ViewNode::Each {
            item,
            collection,
            key,
            children,
        } => Some((|| -> DoweResult<()> {
            let collection_type =
                resolve_each_collection(collection, signals, locals).ok_or_else(|| {
                    DoweError::at_path(
                        path,
                        format!("unknown view value `{collection}` in `collection`"),
                    )
                })?;
            let ViewSignalValue::Array(items) = collection_type else {
                return Err(DoweError::at_path(
                    path,
                    format!("view value `{collection}` in `collection` must be an array"),
                ));
            };
            if path_root(key) != item {
                return Err(DoweError::at_path(
                    path,
                    format!("`each` key `{key}` must start with `{item}`"),
                ));
            }
            let mut scoped = locals.clone();
            scoped.insert(item.clone(), items.first().cloned());
            validate_typed_path(path, signals, &scoped, key, "key", ViewPathExpectation::Any)?;
            for child in children {
                validate_node_references(path, child, signals, writable_signals, actions, &scoped)?;
            }
            Ok(())
        })()),
        ViewNode::Select {
            option_each: Some(option_each),
            ..
        } => Some((|| -> DoweResult<()> {
            let Some(ViewSignalValue::Array(items)) = signals.get(&option_each.collection) else {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "view value `{}` in Select `each` must be an array",
                        option_each.collection
                    ),
                ));
            };
            if path_root(&option_each.key) != option_each.item {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "`each` key `{}` must start with `{}`",
                        option_each.key, option_each.item
                    ),
                ));
            }
            let mut scoped = locals.clone();
            scoped.insert(option_each.item.clone(), items.first().cloned());
            for (name, value) in [
                ("key", option_each.key.as_str()),
                ("value", option_each.value.as_str()),
                ("label", option_each.label.as_str()),
            ] {
                validate_typed_path(
                    path,
                    signals,
                    &scoped,
                    value,
                    name,
                    if name == "key" {
                        ViewPathExpectation::Any
                    } else {
                        ViewPathExpectation::String
                    },
                )?;
            }
            if let Some(description) = option_each.description.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    &scoped,
                    description,
                    "description",
                    ViewPathExpectation::String,
                )?;
            }
            Ok(())
        })()),
        ViewNode::Collapsible { props, children } => Some((|| -> DoweResult<()> {
            if let Some(binding) = props.label.strip_prefix("@signal:") {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    binding,
                    "label",
                    ViewPathExpectation::String,
                )?;
            }
            for child in children {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
            Ok(())
        })()),
        ViewNode::Title { props, value } | ViewNode::Text { props, value } => {
            Some((|| -> DoweResult<()> {
                let binding = text_binding_path(value);
                let template_bindings =
                    dowe_components::text_template_bindings(value).collect::<Vec<_>>();
                if props.i18n.is_some() && (binding.is_some() || !template_bindings.is_empty()) {
                    return Err(DoweError::at_path(
                        path,
                        "`i18n` requires a static fallback text child",
                    ));
                }
                if let Some(binding) = binding {
                    validate_typed_path(
                        path,
                        signals,
                        locals,
                        binding,
                        "text",
                        ViewPathExpectation::String,
                    )?;
                } else {
                    for binding in template_bindings {
                        validate_typed_path(
                            path,
                            signals,
                            locals,
                            &binding,
                            "text",
                            ViewPathExpectation::String,
                        )?;
                    }
                }
                Ok(())
            })())
        }
        ViewNode::Alert { props } => Some((|| -> DoweResult<()> {
            if is_dynamic_reference(&props.message) {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    &props.message,
                    "message",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(visible) = props.visible.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    visible,
                    "visible",
                    ViewPathExpectation::Bool,
                )?;
            }
            if let Some(action) = props.on_close.as_ref()
                && !actions.contains(action)
            {
                return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
            }
            Ok(())
        })()),
        ViewNode::Svg { props, .. } => Some((|| -> DoweResult<()> {
            if let Some(data) = props.data.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    data,
                    "Svg data",
                    ViewPathExpectation::Any,
                )?;
            }
            if let Some(fill) = props.icon_fill_binding.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    fill,
                    "Icon fill",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(stroke) = props.icon_stroke_binding.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    stroke,
                    "Icon stroke",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(name) = props.icon_name.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    name,
                    "Icon name",
                    ViewPathExpectation::String,
                )?;
                if let Some(ViewSignalValue::String(value)) =
                    signal_path_value(path, signals, locals, name, "Icon name")?
                {
                    if !dowe_components::all_icon_names()
                        .iter()
                        .any(|name| name == &value)
                    {
                        return Err(DoweError::at_path(
                            path,
                            format!("invalid initial icon name `{value}` for `Icon name`"),
                        ));
                    }
                }
            }
            Ok(())
        })()),
        ViewNode::ToggleGroup { props, .. } => Some((|| -> DoweResult<()> {
            if let Some(value) = props.value.as_deref() {
                if signals.contains_key(path_root(value))
                    && !writable_signals.contains(path_root(value))
                {
                    return Err(DoweError::at_path(
                        path,
                        format!("constant path `{value}` cannot be used in `bind`"),
                    ));
                }
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    value,
                    if props.kind == ToggleGroupKind::Pagination {
                        "Pagination bind"
                    } else {
                        "ToggleGroup value"
                    },
                    if props.kind == ToggleGroupKind::Pagination {
                        ViewPathExpectation::Any
                    } else {
                        ViewPathExpectation::String
                    },
                )?;
            }
            if let Some(dowe_components::PaginationProps {
                total: dowe_components::PaginationTotal::Signal(total),
                ..
            }) = props.pagination.as_ref()
            {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    total,
                    "Pagination total",
                    ViewPathExpectation::Number,
                )?;
            }
            validate_optional_action(path, actions, props.on_change.as_deref())?;
            Ok(())
        })()),
        _ => None,
    }
}
