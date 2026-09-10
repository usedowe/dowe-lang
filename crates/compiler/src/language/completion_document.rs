pub fn complete_document(
    root: &Path,
    document: &LanguageDocument,
    line: usize,
    column: usize,
) -> Vec<LanguageCompletion> {
    let root = document_workspace_root(root, &document.path);
    let root = root.as_path();
    let prefix = line_prefix(&document.source, line, column);
    let suite_owner = multiline_suite_owner(&document.source, line, &prefix);
    if document.path.ends_with("theme.dowe")
        && let Some(completions) = theme_color_completions(&prefix, suite_owner)
    {
        return completions;
    }
    if import_context(&prefix) {
        return import_completions(root, &document.path);
    }
    if prefix.ends_with("env.") {
        return env_completions(root);
    }
    if prop_value_context(&prefix, "provider")
        && prefix.split_whitespace().next() == Some("database")
    {
        return quoted_values(["postgres", "d1", "dowe"]);
    }
    if prop_value_context(&prefix, "provider") && prefix.split_whitespace().next() == Some("cache")
    {
        return quoted_values(["kv", "redis", "dowe"]);
    }
    if prop_value_context(&prefix, "provider") && prefix.split_whitespace().next() == Some("vector")
    {
        return quoted_values(["dowe"]);
    }
    if prop_value_context(&prefix, "provider") && prefix.split_whitespace().next() == Some("queue")
    {
        return quoted_values(["dowe", "rabbitmq"]);
    }
    if prop_value_context(&prefix, "source") {
        match prefix.split_whitespace().next() {
            Some(namespace) if dowe_stdlib::is_stdlib_namespace(namespace) => {
                return quoted_values(dowe_stdlib::functions(namespace).iter().copied());
            }
            Some("request") => {
                return quoted_values(["query", "rawQuery", "header", "cookie", "bytes"]);
            }
            Some("ws") => return quoted_values(["json"]),
            Some("agent") => return quoted_values(["chat"]),
            _ => {}
        }
    }
    if [
        "onClick",
        "onSend",
        "onLoadMore",
        "onStop",
        "onVoiceNote",
        "onFileAttach",
        "onCameraCapture",
        "onStart",
        "onPause",
        "onResume",
        "onDiscard",
        "onConfirm",
        "onChange",
        "onComplete",
        "onLocation",
        "onLocationError",
        "onRoute",
        "onPointer",
        "onKey",
        "onMotion",
    ]
    .iter()
    .any(|prop| prop_value_context(&prefix, prop))
    {
        return action_completions(root, document);
    }
    if middleware_context(&prefix) {
        return middleware_completions(root, document);
    }
    if prop_value_context(&prefix, "bind") {
        return signal_completions(root, document);
    }
    if ["data", "series", "items", "messages", "scene"]
        .iter()
        .any(|prop| prop_value_context(&prefix, prop))
    {
        return signal_completions(root, document);
    }
    if ["open", "loading", "sending", "streaming", "hasMore"]
        .iter()
        .any(|prop| prop_value_context(&prefix, prop))
    {
        return signal_completions(root, document);
    }
    if prop_value_context(&prefix, "show") {
        let mut completions = vec![
            completion("true", LanguageCompletionKind::Value, "boolean"),
            completion("false", LanguageCompletionKind::Value, "boolean"),
        ];
        completions.extend(signal_completions(root, document));
        return completions;
    }
    if prop_value_context(&prefix, "platform") {
        return quoted_values(["web", "desktop", "android", "ios"]);
    }
    if prefix.trim_start().starts_with("meta ") && prop_value_context(&prefix, "name") {
        return quoted_values(VIEW_META_NAMES.iter().copied());
    }
    if prop_value_context(&prefix, "i18n") {
        return i18n_completions(root);
    }
    if prefix.trim_start().starts_with("validate ") && prop_value_context(&prefix, "rule") {
        return quoted_values([
            "required",
            "email",
            "min:1",
            "max:100",
            "url",
            "phone",
            "pattern:^[A-Za-z]+$",
            "alphanumeric",
            "numeric",
            "alpha",
            "matches:form.confirmation",
            "strongPassword",
            "creditCard",
            "date",
            "minWords:1",
            "maxWords:100",
        ]);
    }
    if let Some(reference_root) = reference_completion_root(&prefix) {
        let mut fields = reference_fields(root, document, reference_root);
        if fields.is_empty() {
            let prefix = format!("{reference_root}.");
            fields = collect_line_signals(&document.source)
                .into_iter()
                .filter_map(|path| path.strip_prefix(&prefix).map(str::to_string))
                .collect();
        }
        if !fields.is_empty() {
            return fields
                .into_iter()
                .map(|field| completion(&field, LanguageCompletionKind::Property, "inferred field"))
                .collect();
        }
    }
    if let Some((component, prop)) = component_prop_value_context(&prefix)
        && let Some(completions) = project_component_value_completions(root, component, prop)
    {
        return completions;
    }
    if let Some(component) = suite_owner.and_then(BuiltinComponent::from_name)
        && let Some(prop) = prefix
            .split_whitespace()
            .last()
            .and_then(|token| token.split_once(':').map(|(prop, _)| prop))
        && let Some(completions) = project_component_value_completions(root, component, prop)
    {
        return completions;
    }
    if let Some(prop) = column_prop_value_context(&prefix)
        && let Some(completions) = column_value_completions(prop)
    {
        return completions;
    }
    if let Some(owner) = view_route_owner(&document.source, &prefix) {
        return view_route_prop_completions(owner);
    }
    if prefix.trim_start().starts_with("each ") {
        return each_prop_completions();
    }
    if prefix.trim_start().starts_with("meta ") {
        return view_meta_prop_completions();
    }
    if let Some(owner) = server_owner_before_cursor(&prefix) {
        let completions = server_prop_completions(owner);
        if !completions.is_empty() {
            return completions;
        }
    }
    if let Some(component) = component_before_cursor(&prefix) {
        return prop_completions(component);
    }
    if let Some(component) = suite_owner.and_then(BuiltinComponent::from_name) {
        return prop_completions(component.as_str());
    }
    base_completions()
}

