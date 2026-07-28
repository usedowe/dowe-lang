fn parse_legacy_request_action(node: &SourceNode) -> DoweResult<ViewRequestAction> {
    if node.args.len() != 1 && node.args.len() != 2 {
        return Err(node_error(
            node,
            "`request` must use `request METHOD path` or `request METHOD route:\"/path\"`",
        ));
    }
    reject_legacy_request_unknown_props(node)?;
    let method_name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`request` method must be a name"))?;
    let method = ViewRequestMethod::from_name(&method_name).ok_or_else(|| {
        node_error(
            node,
            "`request` method must be GET, POST, PUT, PATCH or DELETE",
        )
    })?;
    let path = request_path(node)?;
    if !path.starts_with('/') {
        return Err(node_error(node, "`request` path must start with `/`"));
    }
    let result = legacy_request_result_blocks(node)?;
    let base_env = optional_env_ref_prop(node, "base")?
        .or_else(|| is_api_route(&path).then(|| "BACKEND_URL".to_string()));
    Ok(ViewRequestAction {
        method,
        path,
        base_env,
        headers: request_headers(node)?,
        body: optional_prop_string(node, "body")?,
        update: optional_prop_string(node, "update")?,
        reset: optional_prop_string(node, "reset")?,
        success_alert: result
            .success_alert
            .or(optional_prop_string(node, "successAlert")?),
        success_message: result
            .success_message
            .or(optional_static_string_prop(node, "successMessage")?),
        error_alert: result
            .error_alert
            .or(optional_prop_string(node, "errorAlert")?),
        error_message: result
            .error_message
            .or(optional_static_string_prop(node, "errorMessage")?),
        autoload: optional_prop_bool(node, "autoload")?.unwrap_or(false),
    })
}

fn reject_legacy_request_unknown_props(node: &SourceNode) -> DoweResult<()> {
    let allowed = [
        "base",
        "headers",
        "route",
        "path",
        "body",
        "update",
        "reset",
        "successAlert",
        "successMessage",
        "errorAlert",
        "errorMessage",
        "autoload",
    ];
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(node_error(
                node,
                format!("`request` does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

#[derive(Default)]
struct LegacyRequestResultBlocks {
    success_alert: Option<String>,
    success_message: Option<String>,
    error_alert: Option<String>,
    error_message: Option<String>,
}

fn legacy_request_result_blocks(node: &SourceNode) -> DoweResult<LegacyRequestResultBlocks> {
    let mut result = LegacyRequestResultBlocks::default();
    for child in &node.children {
        let outcome = legacy_request_outcome(child)?;
        match child.name.as_str() {
            "onSuccess" => {
                if node.prop("successAlert").is_some()
                    || node.prop("successMessage").is_some()
                    || result.success_alert.is_some()
                    || result.success_message.is_some()
                {
                    return Err(node_error(
                        child,
                        "`onSuccess` cannot be combined with inline success props",
                    ));
                }
                result.success_alert = Some(outcome.target);
                result.success_message = Some(outcome.message);
            }
            "onError" => {
                if node.prop("errorAlert").is_some()
                    || node.prop("errorMessage").is_some()
                    || result.error_alert.is_some()
                    || result.error_message.is_some()
                {
                    return Err(node_error(
                        child,
                        "`onError` cannot be combined with inline error props",
                    ));
                }
                result.error_alert = Some(outcome.target);
                result.error_message = Some(outcome.message);
            }
            _ => {
                return Err(node_error(
                    child,
                    "`request` children must be `onSuccess` or `onError`",
                ));
            }
        }
    }
    Ok(result)
}

struct LegacyRequestOutcome {
    target: String,
    message: String,
}

fn legacy_request_outcome(node: &SourceNode) -> DoweResult<LegacyRequestOutcome> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`onSuccess` and `onError` only accept props",
        ));
    }
    for prop in &node.props {
        if !matches!(prop.name.as_str(), "alert" | "message" | "target") {
            return Err(node_error(
                node,
                format!("`{}` is not valid in `{}`", prop.name, node.name),
            ));
        }
    }
    let message = optional_static_string_prop(node, "alert")?
        .or(optional_static_string_prop(node, "message")?)
        .ok_or_else(|| node_error(node, format!("`{}` must declare `alert`", node.name)))?;
    let target = optional_prop_string(node, "target")?.unwrap_or_else(|| "alert".to_string());
    Ok(LegacyRequestOutcome { target, message })
}

