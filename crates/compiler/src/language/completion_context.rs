fn theme_color_completions(
    prefix: &str,
    suite_owner: Option<&str>,
) -> Option<Vec<LanguageCompletion>> {
    let inline_owner = prefix.trim_start().split_whitespace().next();
    let family_owner = inline_owner
        .filter(|owner| ColorFamily::from_theme_name(owner).is_some())
        .or_else(|| suite_owner.filter(|owner| ColorFamily::from_theme_name(owner).is_some()));
    if family_owner.is_none() && suite_owner == Some("colors") {
        return Some(
            ColorFamily::theme_names()
                .iter()
                .map(|name| completion(name, LanguageCompletionKind::Keyword, "theme color family"))
                .collect(),
        );
    }

    family_owner?;
    if prefix
        .split_whitespace()
        .last()
        .is_some_and(|token| token.contains(':'))
    {
        return Some(Vec::new());
    }
    Some(
        ["color", "text", "title"]
            .into_iter()
            .map(|role| completion(role, LanguageCompletionKind::Property, "theme color role"))
            .collect(),
    )
}

fn line_prefix(source: &str, line: usize, column: usize) -> String {
    let value = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or_default();
    value.chars().take(column.saturating_sub(1)).collect()
}

fn multiline_suite_owner<'a>(source: &'a str, line: usize, prefix: &str) -> Option<&'a str> {
    let current_indent = prefix.chars().take_while(|value| *value == ' ').count();
    let lines = source.lines().collect::<Vec<_>>();
    let end = line.saturating_sub(1).min(lines.len());

    for source_line in lines[..end].iter().rev() {
        if source_line.trim().is_empty() {
            continue;
        }
        let indent = source_line
            .chars()
            .take_while(|value| *value == ' ')
            .count();
        if indent > current_indent {
            continue;
        }
        let trimmed = source_line.trim();
        if indent == current_indent {
            let is_prop = trimmed.split_whitespace().count() == 1
                && trimmed
                    .split_once(':')
                    .is_some_and(|(name, value)| !name.is_empty() && !value.is_empty());
            if is_prop {
                continue;
            }
            return None;
        }
        let header = trimmed.strip_suffix(':')?.trim_end();
        return header.split_whitespace().next();
    }

    None
}

fn import_context(prefix: &str) -> bool {
    let quote_count = prefix.chars().filter(|value| *value == '"').count();
    prefix.trim_start().starts_with("import ") && quote_count % 2 == 1
}

fn prop_value_context(prefix: &str, name: &str) -> bool {
    let marker = format!("{name}:");
    prefix
        .split_whitespace()
        .last()
        .is_some_and(|value| value.starts_with(&marker))
}

fn reference_completion_root(prefix: &str) -> Option<&str> {
    let token = prefix
        .split(|value: char| value.is_whitespace() || matches!(value, ':' | '{' | '[' | ',' | '('))
        .next_back()?;
    let root = token.strip_suffix('.')?;
    (!root.is_empty() && !root.contains('.')).then_some(root)
}

fn component_prop_value_context(prefix: &str) -> Option<(BuiltinComponent, &str)> {
    let mut parts = prefix.split_whitespace();
    let component = BuiltinComponent::from_name(parts.next()?)?;
    let token = parts.last()?;
    let (prop, _) = token.split_once(':')?;
    (!prop.is_empty()).then_some((component, prop))
}

fn column_prop_value_context(prefix: &str) -> Option<&str> {
    let mut parts = prefix.split_whitespace();
    if parts.next()? != "column" {
        return None;
    }
    let token = parts.last()?;
    let (prop, _) = token.split_once(':')?;
    (!prop.is_empty()).then_some(prop)
}

fn component_before_cursor(prefix: &str) -> Option<&str> {
    let trimmed = prefix.trim_start();
    let mut parts = trimmed.split_whitespace();
    let name = parts.next()?;
    if (parts.next().is_some() && !trimmed.ends_with(':')) || trimmed.ends_with(' ') {
        Some(name)
    } else {
        None
    }
}

fn server_owner_before_cursor(prefix: &str) -> Option<&str> {
    prefix
        .split_whitespace()
        .rev()
        .map(|token| token.trim_matches(['[', ']', '(', ')']))
        .find(|token| server_names().any(|name| name == *token))
}

fn view_route_owner<'a>(source: &str, prefix: &'a str) -> Option<&'a str> {
    let is_view_graph = source.lines().any(|line| {
        !line.chars().next().is_some_and(char::is_whitespace)
            && line
                .split_whitespace()
                .next()
                .is_some_and(|name| name == "views")
    });
    let owner = prefix.split_whitespace().next()?;
    (is_view_graph && matches!(owner, "group" | "route")).then_some(owner)
}

fn view_route_prop_completions(owner: &str) -> Vec<LanguageCompletion> {
    let props = match owner {
        "group" => ["path", "layout", "platform"].as_slice(),
        "route" => ["path", "page", "platform"].as_slice(),
        _ => return Vec::new(),
    };
    props
        .iter()
        .map(|prop| {
            documented_completion(
                prop,
                LanguageCompletionKind::Property,
                "view route prop",
                Some(format!(
                    "`{owner}.{prop}` is validated by the shared Dowe view route compiler."
                )),
            )
        })
        .collect()
}

fn each_prop_completions() -> Vec<LanguageCompletion> {
    ["in", "as", "key"]
        .into_iter()
        .map(|prop| {
            documented_completion(
                prop,
                LanguageCompletionKind::Property,
                "each prop",
                Some(format!(
                    "`each.{prop}` is validated by the shared Dowe view compiler."
                )),
            )
        })
        .collect()
}

fn view_meta_prop_completions() -> Vec<LanguageCompletion> {
    ["name", "content"]
        .into_iter()
        .map(|prop| {
            documented_completion(
                prop,
                LanguageCompletionKind::Property,
                "web metadata prop",
                Some(
                    match prop {
                        "name" => {
                            "`meta.name` selects a supported web document metadata identifier."
                        }
                        _ => {
                            "`meta.content` is the static value emitted into the web document head."
                        }
                    }
                    .to_string(),
                ),
            )
        })
        .collect()
}

fn base_completions() -> Vec<LanguageCompletion> {
    let keywords = [
        "import",
        "type",
        "config",
        "main",
        "views",
        "translations",
        "translation",
        "layout",
        "page",
        "meta",
        "component",
        "test",
        "assert",
        "fn",
        "middleware",
        "database",
        "vector",
        "emb",
        "msg",
        "entity",
        "seeder",
        "store",
        "const",
        "signal",
        "request",
        "invoke",
        "ipc",
        "set",
        "reset",
        "redirect",
        "if",
        "else",
        "each",
        "route",
        "method",
        "get",
        "post",
        "put",
        "patch",
        "delete",
        "handler",
        "next",
        "bearer",
        "send",
        "bridge",
        "task",
        "cron",
        "http",
        "jwt",
        "websocket",
        "init",
        "header",
        "footer",
        "item",
        "divider",
        "trigger",
        "group",
        "submenu",
        "megamenu",
        "icon",
        "content",
        "appBar",
        "main",
        "start",
        "centerX",
        "end",
        "bottomBar",
        "overlays",
        "tab",
        "column",
        "validate",
    ];
    let keywords = keywords
        .into_iter()
        .chain(server_names())
        .collect::<BTreeSet<_>>();
    keywords
        .into_iter()
        .map(|label| {
            documented_completion(
                label,
                LanguageCompletionKind::Keyword,
                "Dowe keyword",
                server_documentation(label),
            )
        })
        .chain(VIEW_COMPONENTS.iter().copied().map(|label| {
            documented_completion(
                label,
                LanguageCompletionKind::Component,
                "Dowe component",
                component_documentation(label),
            )
        }))
        .chain(dowe_stdlib::signatures().into_iter().map(|signature| {
            let label = format!("{}.{}", signature.namespace, signature.function);
            documented_completion(
                &label,
                LanguageCompletionKind::Function,
                "portable standard library",
                stdlib_documentation(&label),
            )
        }))
        .collect()
}

