use crate::language::analysis::{
    document_workspace_root, environment_config, normalize_path, reference_fields, signal_fields,
};
use crate::language::documentation::{
    VIEW_COMPONENTS, component_documentation, component_prop_documentation, server_documentation,
    server_names, server_owner_prop_documentation, server_props, stdlib_documentation,
};
use crate::language::model::{LanguageCompletion, LanguageCompletionKind, LanguageDocument};
use crate::parser::{SourceNode, SourceValue, parse_source_file};
use dowe_components::{
    AlertKind, Align, AvatarStatus, BarPosition, BoxPosition, BuiltinComponent, ButtonSize,
    CarouselIndicatorType, CarouselOrientation, CarouselVariant, ChartCurve, ChartLegendPosition,
    ChartPalette, ChartSize, ChatBoxMode, CodeLanguage, ColorFamily, ColorToken, ComponentVariant,
    CountdownSize, DividerOrientation, DrawerPosition, EmptyKind, FlexDirection, FontFamily,
    GridAlignment, ImageAspect, ImageLoading, ImageObjectFit, Justify, MarqueeOrientation,
    MarqueeSpeed, NativeExternalMode, NavigationOperation, OverlayCornerPosition, OverlayPosition,
    RoundedSize, SectionBackground, ShadowSize, SideNavSize, SkeletonAnimation, SkeletonVariant,
    TableColumnAlign, TableSize, TabsPosition, TabsVariant, TextSize, TextSpacing, TextWeight,
    ToastKind, VIEW_META_NAMES, VideoAspect, ViewAnimation, ViewGesture, ViewIcon, ViewTransition,
    WebTarget,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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
        "center",
        "end",
        "bottomBar",
        "overlays",
        "tab",
        "column",
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

fn env_completions(root: &Path) -> Vec<LanguageCompletion> {
    environment_config(root)
        .map(|environment| {
            environment
                .variables
                .into_iter()
                .map(|variable| {
                    completion(
                        &variable.name,
                        LanguageCompletionKind::Variable,
                        "env variable",
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn i18n_completions(root: &Path) -> Vec<LanguageCompletion> {
    crate::parser::parse_translation_catalog(root)
        .map(|catalog| {
            catalog
                .locales
                .iter()
                .flat_map(|locale| locale.values.iter().map(|value| value.key.as_str()))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(|value| {
                    completion(
                        &format!("\"{value}\""),
                        LanguageCompletionKind::Value,
                        "translation key",
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn action_completions(root: &Path, document: &LanguageDocument) -> Vec<LanguageCompletion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| {
            file.nodes
                .iter()
                .flat_map(collect_actions)
                .map(|name| completion(&name, LanguageCompletionKind::Function, "view fn"))
                .collect()
        })
        .unwrap_or_else(|_| {
            collect_line_declarations(&document.source, "fn")
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Function, "view fn"))
                .collect()
        })
}

fn signal_completions(root: &Path, document: &LanguageDocument) -> Vec<LanguageCompletion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| {
            let types = crate::parser::TypeRegistry::parse_file(root, &file).unwrap_or_default();
            let mut names = file
                .nodes
                .iter()
                .flat_map(|node| collect_signals(node, &types))
                .collect::<Vec<_>>();
            names.extend(imported_view_store_paths(root, &file));
            names
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Variable, "reactive path"))
                .collect()
        })
        .unwrap_or_else(|_| {
            collect_line_signals(&document.source)
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Variable, "signal path"))
                .collect()
        })
}

fn imported_view_store_paths(root: &Path, file: &crate::parser::SourceFile) -> Vec<String> {
    let mut output = Vec::new();
    for import in &file.imports {
        let Ok(path) = crate::parser::resolve_import(root, &file.path, import) else {
            continue;
        };
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(target) = parse_source_file(root, &path, source) else {
            continue;
        };
        let Some(store) = target.nodes.iter().find(|node| {
            node.name == "store"
                && node
                    .args
                    .first()
                    .and_then(SourceValue::as_required_string)
                    .is_some_and(|name| name == import.local)
        }) else {
            continue;
        };
        output.push(import.local.clone());
        let types = crate::parser::TypeRegistry::parse_file(root, &target).unwrap_or_default();
        let fields = signal_type_fields(store, &types)
            .or_else(|| store.prop("value").map(|prop| signal_fields(&prop.value)));
        output.extend(
            fields
                .unwrap_or_default()
                .into_iter()
                .map(|field| format!("{}.{field}", import.local)),
        );
    }
    output
}

fn middleware_context(prefix: &str) -> bool {
    prefix
        .split_whitespace()
        .last()
        .is_some_and(|value| value.starts_with("middleware:"))
}

fn middleware_completions(root: &Path, document: &LanguageDocument) -> Vec<LanguageCompletion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| {
            file.imports
                .iter()
                .filter_map(|import| {
                    let path = crate::parser::resolve_import(root, &file.path, import).ok()?;
                    let source = fs::read_to_string(&path).ok()?;
                    let target = parse_source_file(root, &path, source).ok()?;
                    target
                        .nodes
                        .iter()
                        .any(|node| {
                            node.name == "middleware"
                                && node
                                    .args
                                    .first()
                                    .and_then(SourceValue::as_required_string)
                                    .is_some_and(|name| name == import.local)
                        })
                        .then(|| {
                            completion(
                                &import.local,
                                LanguageCompletionKind::Function,
                                "server middleware",
                            )
                        })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn collect_actions(node: &SourceNode) -> Vec<String> {
    let mut output = Vec::new();
    if node.name == "fn"
        && let Some(name) = node.args.first().and_then(SourceValue::as_required_string)
    {
        output.push(name);
    }
    for child in &node.children {
        output.extend(collect_actions(child));
    }
    output
}

fn collect_signals(node: &SourceNode, types: &crate::parser::TypeRegistry) -> Vec<String> {
    let mut output = Vec::new();
    if matches!(node.name.as_str(), "signal" | "const")
        && let Some(name) = node.args.first().and_then(SourceValue::as_required_string)
    {
        output.push(name.clone());
        if let Some(schema) = signal_type_fields(node, types) {
            output.extend(schema.into_iter().map(|field| format!("{name}.{field}")));
        } else if let Some(value) = node.prop("value") {
            output.extend(
                signal_fields(&value.value)
                    .into_iter()
                    .map(|field| format!("{name}.{field}")),
            );
        }
    }
    for child in &node.children {
        output.extend(collect_signals(child, types));
    }
    output
}

fn signal_type_fields(
    node: &SourceNode,
    types: &crate::parser::TypeRegistry,
) -> Option<Vec<String>> {
    let type_name = node.prop("type")?.value.as_required_string()?;
    let schema = types.resolve(node, &type_name).ok()?;
    Some(crate::parser::reference_fields_for_type(&schema))
}

fn collect_line_declarations(source: &str, keyword: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? == keyword {
                parts.next().map(str::to_string)
            } else {
                None
            }
        })
        .collect()
}

fn collect_line_signals(source: &str) -> Vec<String> {
    let mut output = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        let mut parts = trimmed.split_whitespace();
        if !matches!(parts.next(), Some("signal" | "const")) {
            continue;
        }
        let Some(name) = parts.next() else {
            continue;
        };
        output.push(name.to_string());
        if let Some(body) = trimmed
            .split("value:{")
            .nth(1)
            .and_then(|value| value.split('}').next())
        {
            for token in body.split_whitespace() {
                if let Some((field, _)) = token.split_once(':')
                    && !field.is_empty()
                {
                    output.push(format!("{name}.{field}"));
                }
            }
        }
    }
    output
}

fn prop_completions(component: &str) -> Vec<LanguageCompletion> {
    props_for_component(component)
        .into_iter()
        .map(|label| {
            documented_completion(
                label,
                LanguageCompletionKind::Property,
                "component prop",
                component_prop_documentation(component, label),
            )
        })
        .collect()
}

fn server_prop_completions(owner: &str) -> Vec<LanguageCompletion> {
    server_props(owner)
        .into_iter()
        .map(|label| {
            documented_completion(
                label,
                LanguageCompletionKind::Property,
                "server prop",
                server_owner_prop_documentation(owner, label),
            )
        })
        .collect()
}

pub(super) fn component_value_completions(
    component: BuiltinComponent,
    prop: &str,
) -> Option<Vec<LanguageCompletion>> {
    if !props_for_component(component.as_str()).contains(&prop) {
        return None;
    }

    match (component, prop) {
        (BuiltinComponent::Icon, "name") => Some(quoted_values(dowe_components::all_icon_names())),
        (BuiltinComponent::IconButton, "icon")
        | (BuiltinComponent::SideNav | BuiltinComponent::RailNav, "icon")
        | (BuiltinComponent::Button | BuiltinComponent::Input, "iconStart" | "iconEnd") => {
            Some(quoted_values(dowe_components::solar_icon_names()))
        }
        (BuiltinComponent::Icon, "fill" | "stroke") => Some(quoted_values(
            ["currentColor"]
                .into_iter()
                .chain(ColorToken::all().iter().map(|value| value.as_str())),
        )),
        (
            BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart
            | BuiltinComponent::Table
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::Sidebar
            | BuiltinComponent::Drawer
            | BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::ComboBox
            | BuiltinComponent::CsvField
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::Button
            | BuiltinComponent::IconButton
            | BuiltinComponent::Alert
            | BuiltinComponent::ToggleTheme
            | BuiltinComponent::SelectTheme
            | BuiltinComponent::Dropzone
            | BuiltinComponent::ChatBox
            | BuiltinComponent::Empty
            | BuiltinComponent::ToggleGroup
            | BuiltinComponent::Collapsible
            | BuiltinComponent::Countdown
            | BuiltinComponent::Map
            | BuiltinComponent::Image
            | BuiltinComponent::Accordion
            | BuiltinComponent::Toast
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange
            | BuiltinComponent::Toggle,
            "variant",
        ) => Some(quoted_values(
            ComponentVariant::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Chip
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command,
            "variant",
        ) => Some(quoted_values(
            ComponentVariant::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Avatar
            | BuiltinComponent::AvatarGroup
            | BuiltinComponent::Badge
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Fab
            | BuiltinComponent::Record,
            "variant",
        ) => Some(solid_soft_values()),
        (BuiltinComponent::Audio, "variant") => Some(solid_soft_values()),
        (BuiltinComponent::Carousel, "variant") => Some(quoted_values(
            CarouselVariant::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Tabs, "variant") => Some(quoted_values(
            TabsVariant::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart
            | BuiltinComponent::Table
            | BuiltinComponent::Divider
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::Sidebar
            | BuiltinComponent::Tabs
            | BuiltinComponent::Stepper
            | BuiltinComponent::Drawer
            | BuiltinComponent::Avatar
            | BuiltinComponent::Badge
            | BuiltinComponent::Chip
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Toast
            | BuiltinComponent::Dropdown
            | BuiltinComponent::Command
            | BuiltinComponent::Dropzone
            | BuiltinComponent::ComboBox
            | BuiltinComponent::CsvField
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::AvatarGroup
            | BuiltinComponent::ChatBox
            | BuiltinComponent::Empty
            | BuiltinComponent::Collapsible
            | BuiltinComponent::Countdown
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::SelectTheme,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Button
            | BuiltinComponent::Alert
            | BuiltinComponent::ToggleTheme
            | BuiltinComponent::Fab
            | BuiltinComponent::FabAction
            | BuiltinComponent::Slider
            | BuiltinComponent::SideNav
            | BuiltinComponent::RailNav,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all()
                .iter()
                .filter(|value| {
                    **value != ColorFamily::Background && **value != ColorFamily::Surface
                })
                .map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Record
            | BuiltinComponent::ToggleGroup
            | BuiltinComponent::Map
            | BuiltinComponent::Audio
            | BuiltinComponent::Image
            | BuiltinComponent::Accordion
            | BuiltinComponent::Carousel
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange
            | BuiltinComponent::Toggle,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all()
                .iter()
                .filter(|value| {
                    **value != ColorFamily::Background && **value != ColorFamily::Surface
                })
                .map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::AvatarGroup
            | BuiltinComponent::Chip
            | BuiltinComponent::ToggleTheme
            | BuiltinComponent::SelectTheme
            | BuiltinComponent::Fab
            | BuiltinComponent::ToggleGroup,
            "size",
        ) => Some(quoted_values(
            ButtonSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Slider
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::Dropzone
            | BuiltinComponent::ComboBox
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea,
            "size",
        ) => Some(control_size_values()),
        (
            BuiltinComponent::Carousel
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange,
            "size",
        ) => Some(control_size_values()),
        (BuiltinComponent::CsvField | BuiltinComponent::ImageCropper, "size") => Some(
            quoted_values(ButtonSize::all().iter().map(|value| value.as_str())),
        ),
        (BuiltinComponent::DragDrop, "direction") => {
            Some(quoted_values(["horizontal", "vertical"]))
        }
        (BuiltinComponent::RadioGroup, "orientation") => {
            Some(quoted_values(["vertical", "horizontal"]))
        }
        (BuiltinComponent::Image, "aspect") => Some(quoted_values(
            ImageAspect::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Image, "objectFit") => Some(quoted_values(
            ImageObjectFit::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Image, "loading") => Some(quoted_values(
            ImageLoading::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Carousel, "orientation") => Some(quoted_values(
            CarouselOrientation::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::Carousel, "indicatorType") => Some(quoted_values(
            CarouselIndicatorType::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::ImageCropper, "shape") => Some(quoted_values(["circle", "square"])),
        (BuiltinComponent::Pin, "type") => Some(quoted_values(["text", "password", "number"])),
        (BuiltinComponent::Table, "size") => Some(quoted_values(
            TableSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "size",
        ) => Some(quoted_values(
            ChartSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "palette",
        ) => Some(quoted_values(
            ChartPalette::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "legendPosition",
        ) => Some(quoted_values(
            ChartLegendPosition::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::AreaChart | BuiltinComponent::LineChart, "curve") => Some(
            quoted_values(ChartCurve::all().iter().map(|value| value.as_str())),
        ),
        (BuiltinComponent::Code, "language") => Some(quoted_values(
            CodeLanguage::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Video, "aspect") => Some(quoted_values(
            VideoAspect::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Iframe, "loading") => Some(quoted_values(["lazy", "eager"])),
        (BuiltinComponent::Iframe, "allow") => Some(quoted_values([
            "fullscreen",
            "autoplay",
            "camera; microphone",
            "clipboard-read; clipboard-write",
        ])),
        (BuiltinComponent::Iframe, "sandbox") => Some(quoted_values([
            "",
            "scripts",
            "scripts same-origin",
            "scripts same-origin forms",
        ])),
        (BuiltinComponent::Device, "device") => {
            Some(quoted_values(["mobile", "tablet", "laptop", "monitor"]))
        }
        (BuiltinComponent::AppBar, "position") => Some(quoted_values(
            BarPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Canvas, "fit") => Some(quoted_values(["contain", "cover", "stretch"])),
        (BuiltinComponent::Canvas, "background") => Some(quoted_values(
            ColorToken::all()
                .iter()
                .map(|value| value.as_str())
                .chain(["transparent"]),
        )),
        (BuiltinComponent::Divider, "orientation") => Some(quoted_values(
            DividerOrientation::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::ChatBox, "mode") => Some(quoted_values(
            ChatBoxMode::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Empty, "type") => Some(quoted_values(
            EmptyKind::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Marquee, "speed") => Some(quoted_values(
            MarqueeSpeed::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Marquee, "orientation") => Some(quoted_values(
            MarqueeOrientation::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Countdown, "size") => Some(quoted_values(
            CountdownSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::SideNav | BuiltinComponent::RailNav | BuiltinComponent::NavMenu,
            "size",
        ) => Some(quoted_values(
            SideNavSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Box, "position") => Some(quoted_values(
            BoxPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Drawer, "position") => Some(quoted_values(
            DrawerPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Tabs, "position") => Some(quoted_values(
            TabsPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Stepper, "orientation") => {
            Some(quoted_values(["horizontal", "vertical"]))
        }
        (BuiltinComponent::Avatar, "status") => Some(quoted_values(
            AvatarStatus::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Badge | BuiltinComponent::Toast | BuiltinComponent::Fab, "position") => {
            Some(quoted_values(
                OverlayCornerPosition::all()
                    .iter()
                    .map(|value| value.as_str()),
            ))
        }
        (BuiltinComponent::Tooltip, "position") => Some(quoted_values(
            OverlayPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Skeleton, "variant") => Some(quoted_values(
            SkeletonVariant::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Skeleton, "animation") => Some(quoted_values(
            SkeletonAnimation::all().iter().map(|value| value.as_str()),
        )),
        (_, "animation") => Some(quoted_values(
            ViewAnimation::all().iter().map(|value| value.as_str()),
        )),
        (_, "transition") => Some(quoted_values(
            ViewTransition::all().iter().map(|value| value.as_str()),
        )),
        (_, "gesture") => Some(quoted_values(
            ViewGesture::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Section, "background") => Some(quoted_values(
            SectionBackground::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Section, "boxed") => Some(boolean_values()),
        (BuiltinComponent::RichText, "title") => Some(boolean_values()),
        (BuiltinComponent::Title | BuiltinComponent::Text | BuiltinComponent::RichText, "size") => {
            Some(quoted_values(
                TextSize::all().iter().map(|value| value.as_str()),
            ))
        }
        (
            BuiltinComponent::Title | BuiltinComponent::Text | BuiltinComponent::RichText,
            "weight",
        ) => Some(quoted_values(
            TextWeight::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Title | BuiltinComponent::Text | BuiltinComponent::RichText,
            "spacing",
        ) => Some(quoted_values(
            TextSpacing::all().iter().map(|value| value.as_str()),
        )),
        (_, "font") => Some(quoted_values(
            FontFamily::all().iter().map(|value| value.as_str()),
        )),
        (_, "bg" | "color" | "upColor" | "downColor" | "fadeColor") => Some(quoted_values(
            ColorToken::all().iter().map(|value| value.as_str()),
        )),
        (_, "borderColor" | "shadowColor") => Some(quoted_values(
            ColorFamily::all().iter().map(|value| value.as_str()),
        )),
        (_, "shadow") => Some(quoted_values(
            ShadowSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Path, "fill") => Some(quoted_values(
            ["none", "currentColor"]
                .into_iter()
                .chain(ColorToken::all().iter().map(|value| value.as_str())),
        )),
        (BuiltinComponent::Path, "fillRule") => Some(quoted_values(["nonzero", "evenodd"])),
        (_, "rounded") => Some(quoted_values(
            RoundedSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "justify") => Some(quoted_values(
            Justify::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "direction") => Some(quoted_values(
            FlexDirection::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "align") => Some(quoted_values(
            Align::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Grid, "justify" | "align") => Some(quoted_values(
            GridAlignment::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "navigate",
        ) => Some(quoted_values(
            NavigationOperation::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "history",
        ) => Some(quoted_values(["back"])),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "target",
        ) => Some(quoted_values(
            WebTarget::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "externalMode",
        ) => Some(quoted_values(
            NativeExternalMode::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Fab | BuiltinComponent::FabAction, "icon") => Some(quoted_values(
            ViewIcon::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::ToggleGroup, "icon") => Some(quoted_values(
            ViewIcon::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Alert, "type") => Some(quoted_values(
            AlertKind::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Toast, "type") => Some(quoted_values(
            ToastKind::all().iter().map(|value| value.as_str()),
        )),
        _ => None,
    }
}

fn project_component_value_completions(
    root: &Path,
    component: BuiltinComponent,
    prop: &str,
) -> Option<Vec<LanguageCompletion>> {
    let mut completions = component_value_completions(component, prop)?;
    if prop == "scheme" {
        completions.extend(custom_color_family_completions(root));
    }
    Some(completions)
}

fn custom_color_family_completions(root: &Path) -> Vec<LanguageCompletion> {
    let path = root.join("theme.dowe");
    let Ok(source) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(file) = parse_source_file(root, &path, source) else {
        return Vec::new();
    };
    let mut families = BTreeSet::new();
    for node in &file.nodes {
        collect_custom_color_families(node, &mut families);
    }
    quoted_values(families)
}

fn collect_custom_color_families(node: &SourceNode, families: &mut BTreeSet<String>) {
    if node.name == "colors" {
        for child in &node.children {
            if let Some((family, false)) = ColorFamily::from_theme_name(&child.name)
                && !family.is_builtin()
            {
                families.insert(family.as_str().to_string());
            }
        }
    }
    for child in &node.children {
        collect_custom_color_families(child, families);
    }
}

fn solid_soft_values() -> Vec<LanguageCompletion> {
    quoted_values(["solid", "soft"])
}

fn control_size_values() -> Vec<LanguageCompletion> {
    quoted_values(["sm", "md", "lg"])
}

fn column_value_completions(prop: &str) -> Option<Vec<LanguageCompletion>> {
    if !COLUMN_PROPS.contains(&prop) {
        return None;
    }
    match prop {
        "align" => Some(quoted_values(
            TableColumnAlign::all().iter().map(|value| value.as_str()),
        )),
        _ => None,
    }
}

fn quoted_values<I>(values: I) -> Vec<LanguageCompletion>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    values
        .into_iter()
        .map(|value| {
            let value = value.as_ref();
            completion(
                &format!("\"{value}\""),
                LanguageCompletionKind::Value,
                "quoted static value",
            )
        })
        .collect()
}

fn boolean_values() -> Vec<LanguageCompletion> {
    ["true", "false"]
        .into_iter()
        .map(|value| completion(value, LanguageCompletionKind::Value, "static boolean value"))
        .collect()
}

pub(super) fn props_for_component(component: &str) -> Vec<&'static str> {
    let mut props = match component {
        "Box" => BOX_PROPS.to_vec(),
        "Section" => SECTION_PROPS.to_vec(),
        "Flex" => LAYOUT_PROPS.to_vec(),
        "Grid" => GRID_PROPS.to_vec(),
        "Card" => VARIANT_PROPS.to_vec(),
        "AppBar" => APP_BAR_PROPS.to_vec(),
        "BottomBar" => FLOATING_BAR_PROPS.to_vec(),
        "Footer" => BAR_PROPS.to_vec(),
        "SideNav" => SIDE_NAV_PROPS.to_vec(),
        "RailNav" => RAIL_NAV_PROPS.to_vec(),
        "Sidebar" => SIDEBAR_PROPS.to_vec(),
        "NavMenu" => NAV_MENU_PROPS.to_vec(),
        "Scaffold" => SCAFFOLD_PROPS.to_vec(),
        "Splash" => vec!["bind"],
        "Tabs" => TABS_PROPS.to_vec(),
        "tab" => TAB_PROPS.to_vec(),
        "Stepper" => STEPPER_PROPS.to_vec(),
        "step" => STEP_PROPS.to_vec(),
        "Drawer" => DRAWER_PROPS.to_vec(),
        "Avatar" => AVATAR_PROPS.to_vec(),
        "Badge" => BADGE_PROPS.to_vec(),
        "Chip" => CHIP_PROPS.to_vec(),
        "Skeleton" => SKELETON_PROPS.to_vec(),
        "Modal" => MODAL_PROPS.to_vec(),
        "AlertDialog" => ALERT_DIALOG_PROPS.to_vec(),
        "Tooltip" => TOOLTIP_PROPS.to_vec(),
        "Toast" => TOAST_PROPS.to_vec(),
        "Dropdown" => DROPDOWN_PROPS.to_vec(),
        "Command" => COMMAND_PROPS.to_vec(),
        "AvatarGroup" => AVATAR_GROUP_PROPS.to_vec(),
        "ChatBox" => CHAT_BOX_PROPS.to_vec(),
        "Empty" => EMPTY_PROPS.to_vec(),
        "Marquee" => MARQUEE_PROPS.to_vec(),
        "TypeWriter" => TYPE_WRITER_PROPS.to_vec(),
        "RichText" => RICH_TEXT_PROPS.to_vec(),
        "mark" => RICH_TEXT_MARK_PROPS.to_vec(),
        "Record" => RECORD_PROPS.to_vec(),
        "ToggleGroup" => TOGGLE_GROUP_PROPS.to_vec(),
        "Collapsible" => COLLAPSIBLE_PROPS.to_vec(),
        "Countdown" => COUNTDOWN_PROPS.to_vec(),
        "Map" => MAP_PROPS.to_vec(),
        "marker" => MAP_MARKER_PROPS.to_vec(),
        "waypoint" => MAP_WAYPOINT_PROPS.to_vec(),
        "RadioGroup" => RADIO_GROUP_PROPS.to_vec(),
        "item" => ITEM_PROPS.to_vec(),
        "submenu" | "megamenu" => NAV_MENU_ENTRY_PROPS.to_vec(),
        "group" => COMMAND_GROUP_PROPS.to_vec(),
        "ToggleTheme" => TOGGLE_THEME_PROPS.to_vec(),
        "SelectTheme" => SELECT_THEME_PROPS.to_vec(),
        "Fab" => FAB_PROPS.to_vec(),
        "fabAction" => FAB_ACTION_PROPS.to_vec(),
        "Slider" => SLIDER_PROPS.to_vec(),
        "Dropzone" => DROPZONE_PROPS.to_vec(),
        "ComboBox" => COMBO_BOX_PROPS.to_vec(),
        "comboOption" => COMBO_OPTION_PROPS.to_vec(),
        "CsvField" => CSV_FIELD_PROPS.to_vec(),
        "csvColumn" => CSV_COLUMN_PROPS.to_vec(),
        "DragDrop" => DRAG_DROP_PROPS.to_vec(),
        "dragGroup" => DRAG_GROUP_PROPS.to_vec(),
        "dragItem" => DRAG_ITEM_PROPS.to_vec(),
        "Editor" => EDITOR_PROPS.to_vec(),
        "ImageCropper" => IMAGE_CROPPER_PROPS.to_vec(),
        "Password" => PASSWORD_PROPS.to_vec(),
        "Phone" => PHONE_PROPS.to_vec(),
        "Pin" => PIN_PROPS.to_vec(),
        "Textarea" => TEXTAREA_PROPS.to_vec(),
        "Input" => INPUT_PROPS.to_vec(),
        "Select" => SELECT_PROPS.to_vec(),
        "Option" => OPTION_PROPS.to_vec(),
        "Code" => CODE_PROPS.to_vec(),
        "Video" => VIDEO_PROPS.to_vec(),
        "Iframe" => IFRAME_PROPS.to_vec(),
        "Device" => DEVICE_PROPS.to_vec(),
        "Canvas" => CANVAS_PROPS.to_vec(),
        "Candlestick" => CANDLESTICK_PROPS.to_vec(),
        "ArcChart" => ARC_CHART_PROPS.to_vec(),
        "AreaChart" => AREA_CHART_PROPS.to_vec(),
        "BarChart" => BAR_CHART_PROPS.to_vec(),
        "LineChart" => LINE_CHART_PROPS.to_vec(),
        "PieChart" => PIE_CHART_PROPS.to_vec(),
        "Table" => TABLE_PROPS.to_vec(),
        "column" => COLUMN_PROPS.to_vec(),
        "Divider" => DIVIDER_PROPS.to_vec(),
        "Button" => BUTTON_PROPS.to_vec(),
        "Brand" => BRAND_PROPS.to_vec(),
        "Banner" => BANNER_PROPS.to_vec(),
        "IconButton" => ICON_BUTTON_PROPS.to_vec(),
        "Alert" => ALERT_PROPS.to_vec(),
        "Icon" => ICON_PROPS.to_vec(),
        "Svg" => SVG_PROPS.to_vec(),
        "Path" => PATH_PROPS.to_vec(),
        "Title" | "Text" => TEXT_PROPS.to_vec(),
        "Audio" => combined_props(&["src", "subtitle", "avatarSrc"], VARIANT_PROPS),
        "Image" => combined_props(
            &[
                "src",
                "alt",
                "aspect",
                "objectFit",
                "loading",
                "hideControls",
            ],
            VARIANT_PROPS,
        ),
        "Accordion" => combined_props(&["multiple"], VARIANT_PROPS),
        "Carousel" => combined_props(
            &[
                "autoplay",
                "autoplayInterval",
                "disableLoop",
                "hideControls",
                "hideIndicators",
                "showNavigation",
                "showCounter",
                "orientation",
                "size",
                "indicatorType",
                "title",
                "slideWidth",
                "slideHeight",
                "slidesPerView",
                "gap",
            ],
            VARIANT_PROPS,
        ),
        "Checkbox" => combined_props(&["checked", "disabled", "name"], VARIANT_PROPS),
        "Color" => combined_props(
            &[
                "value",
                "size",
                "name",
                "helpText",
                "errorText",
                "showHex",
                "showRgb",
                "showCmyk",
                "showOklch",
            ],
            VARIANT_PROPS,
        ),
        "Date" => combined_props(
            &[
                "value",
                "size",
                "name",
                "helpText",
                "errorText",
                "min",
                "max",
            ],
            VARIANT_PROPS,
        ),
        "DateRange" => combined_props(
            &[
                "start",
                "end",
                "startValue",
                "endValue",
                "size",
                "name",
                "helpText",
                "errorText",
                "min",
                "max",
            ],
            VARIANT_PROPS,
        ),
        "Toggle" => combined_props(
            &["checked", "disabled", "name", "labelLeft", "labelRight"],
            VARIANT_PROPS,
        ),
        _ => Vec::new(),
    };
    if BuiltinComponent::from_name(component).is_some_and(|component| {
        !matches!(
            component,
            BuiltinComponent::Option
                | BuiltinComponent::FabAction
                | BuiltinComponent::ComboOption
                | BuiltinComponent::CsvColumn
                | BuiltinComponent::DragGroup
                | BuiltinComponent::DragItem
                | BuiltinComponent::Svg
                | BuiltinComponent::Path
        )
    }) {
        for &prop in INTERACTIVE_STYLE_PROPS {
            if !props.contains(&prop) {
                props.push(prop);
            }
        }
    }
    props
}

fn combined_props(
    specific: &'static [&'static str],
    common: &'static [&'static str],
) -> Vec<&'static str> {
    specific.iter().chain(common).copied().collect()
}

fn import_completions(root: &Path, from: &Path) -> Vec<LanguageCompletion> {
    let source_root = normalize_path(root.to_path_buf());
    let from = normalize_path(from.to_path_buf());
    let files = dowe_files(&source_root);
    files
        .into_iter()
        .filter(|path| normalize_path(path.to_path_buf()) != from)
        .filter(|path| importable_project_path(&source_root, path))
        .filter_map(|path| project_root_import_path(&source_root, &path))
        .map(|label| completion(&label, LanguageCompletionKind::File, "Dowe source"))
        .collect()
}

pub(super) fn dowe_files(path: &Path) -> Vec<PathBuf> {
    let mut output = Vec::new();
    let Ok(entries) = fs::read_dir(path) else {
        return output;
    };
    for entry in entries.flatten() {
        if entry
            .file_type()
            .is_ok_and(|file_type| file_type.is_symlink())
        {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            output.extend(dowe_files(&path));
        } else if path.extension().and_then(|value| value.to_str()) == Some("dowe") {
            output.push(path);
        }
    }
    output.sort();
    output
}

pub(super) fn importable_project_path(project_root: &Path, target: &Path) -> bool {
    let normalized = normalize_path(target.to_path_buf());
    let Ok(relative) = normalized.strip_prefix(project_root) else {
        return false;
    };
    !matches!(
        relative.to_string_lossy().as_ref(),
        "config.dowe" | "theme.dowe" | "main.dowe" | "views.dowe"
    )
}

pub(super) fn project_root_import_path(project_root: &Path, target: &Path) -> Option<String> {
    let normalized = normalize_path(target.to_path_buf());
    let mut path = normalized.strip_prefix(project_root).ok()?.to_path_buf();
    path.set_extension("");
    let value = path.to_string_lossy().replace('\\', "/");
    Some(format!("@/{value}"))
}

fn completion(label: &str, kind: LanguageCompletionKind, detail: &str) -> LanguageCompletion {
    LanguageCompletion {
        label: label.to_string(),
        kind,
        detail: Some(detail.to_string()),
        documentation: None,
    }
}

fn documented_completion(
    label: &str,
    kind: LanguageCompletionKind,
    detail: &str,
    documentation: Option<String>,
) -> LanguageCompletion {
    LanguageCompletion {
        label: label.to_string(),
        kind,
        detail: Some(detail.to_string()),
        documentation,
    }
}

const BOX_PROPS: &[&str] = &[
    "id",
    "show",
    "font",
    "bg",
    "color",
    "cover",
    "overlay",
    "animation",
    "colSpan",
    "rowSpan",
    "position",
    "top",
    "right",
    "bottom",
    "left",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "borderColor",
    "shadow",
    "shadowColor",
    "onClick",
];
const SECTION_PROPS: &[&str] = &[
    "id",
    "show",
    "font",
    "bg",
    "color",
    "background",
    "boxed",
    "cover",
    "overlay",
    "animation",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
    "borderColor",
    "shadow",
    "shadowColor",
];
const LAYOUT_PROPS: &[&str] = &[
    "direction",
    "wrap",
    "justify",
    "align",
    "gap",
    "id",
    "show",
    "font",
    "bg",
    "color",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "borderColor",
    "shadow",
    "shadowColor",
];
const GRID_PROPS: &[&str] = &[
    "columns", "rows", "justify", "align", "gap", "id", "show", "font", "bg", "color", "cover",
    "overlay", "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH", "maxW", "maxH",
];
const VARIANT_PROPS: &[&str] = &[
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "animation",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "borderColor",
    "shadow",
    "shadowColor",
    "onClick",
];
const BAR_PROPS: &[&str] = &[
    "variant", "scheme", "bordered", "blurred", "boxed", "id", "show", "font", "p", "px", "py",
    "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH", "rounded", "border",
];
const FLOATING_BAR_PROPS: &[&str] = &[
    "variant", "scheme", "bordered", "blurred", "boxed", "floating", "id", "show", "font", "p",
    "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH", "rounded",
    "border",
];
const APP_BAR_PROPS: &[&str] = &[
    "variant",
    "scheme",
    "position",
    "bordered",
    "blurred",
    "boxed",
    "floating",
    "hideOnScroll",
    "dockOnScroll",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const SIDE_NAV_PROPS: &[&str] = &[
    "variant", "scheme", "size", "wide", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt",
    "pb", "w", "h", "minW", "minH", "maxW", "maxH", "rounded", "border",
];
const RAIL_NAV_PROPS: &[&str] = &[
    "variant",
    "scheme",
    "size",
    "showLabels",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const SIDEBAR_PROPS: &[&str] = &[
    "variant", "scheme", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h",
    "minW", "minH", "maxW", "maxH", "rounded", "border",
];
const NAV_MENU_PROPS: &[&str] = &[
    "variant", "scheme", "size", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb",
    "w", "h", "minW", "minH", "maxW", "maxH", "rounded", "border",
];
const SCAFFOLD_PROPS: &[&str] = &[
    "boxed", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH", "maxW", "maxH", "rounded", "border",
];
const TABS_PROPS: &[&str] = &[
    "variant", "scheme", "position", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb",
    "w", "h", "minW", "minH", "maxW", "maxH", "rounded", "border",
];
const TAB_PROPS: &[&str] = &[
    "id",
    "label",
    "i18n",
    "href",
    "navigate",
    "target",
    "externalMode",
    "featured",
];
const STEPPER_PROPS: &[&str] = &[
    "scheme",
    "orientation",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const STEP_PROPS: &[&str] = &["id", "label", "i18n"];
const DRAWER_PROPS: &[&str] = &[
    "open",
    "position",
    "variant",
    "scheme",
    "disableOverlayClose",
    "hideCloseButton",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const AVATAR_PROPS: &[&str] = &[
    "src",
    "name",
    "alt",
    "href",
    "navigate",
    "history",
    "target",
    "externalMode",
    "onClick",
    "variant",
    "scheme",
    "size",
    "status",
    "bordered",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "borderColor",
    "shadow",
    "shadowColor",
];
const BADGE_PROPS: &[&str] = &[
    "text", "variant", "scheme", "position", "id", "show", "font", "bg", "cover", "overlay",
    "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH",
    "maxW", "maxH", "rounded",
];
const CHIP_PROPS: &[&str] = &[
    "variant", "scheme", "size", "onClose", "onClick", "id", "show", "font", "bg", "cover",
    "overlay", "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH", "maxW", "maxH", "rounded",
];
const INTERACTIVE_STYLE_PROPS: &[&str] = &[
    "animation",
    "rotate",
    "scale",
    "translateX",
    "translateY",
    "transition",
    "gesture",
];
const SKELETON_PROPS: &[&str] = &[
    "variant",
    "animation",
    "id",
    "show",
    "font",
    "bg",
    "color",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "borderColor",
    "shadow",
    "shadowColor",
];
const MODAL_PROPS: &[&str] = &[
    "open",
    "onClose",
    "variant",
    "scheme",
    "disableOverlayClose",
    "hideCloseButton",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const ALERT_DIALOG_PROPS: &[&str] = &[
    "open",
    "title",
    "description",
    "confirmText",
    "cancelText",
    "onConfirm",
    "onCancel",
    "variant",
    "scheme",
    "loading",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const TOOLTIP_PROPS: &[&str] = &[
    "label", "position", "variant", "scheme", "id", "show", "font", "bg", "cover", "overlay",
    "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH",
    "maxW", "maxH", "rounded",
];
const TOAST_PROPS: &[&str] = &[
    "source",
    "type",
    "title",
    "description",
    "position",
    "variant",
    "scheme",
    "showIcon",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "borderColor",
    "shadow",
    "shadowColor",
];
const DROPDOWN_PROPS: &[&str] = &[
    "scheme", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH", "maxW", "maxH", "rounded", "border",
];
const COMMAND_PROPS: &[&str] = &[
    "open",
    "placeholder",
    "emptyText",
    "closeText",
    "navigateText",
    "selectText",
    "toggleText",
    "shortcut",
    "disableGlobalShortcut",
    "showFooter",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const AVATAR_GROUP_PROPS: &[&str] = &[
    "items", "variant", "scheme", "size", "max", "autoFit", "inline", "bordered", "id", "show",
    "font", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH",
    "rounded", "border",
];
const CHAT_BOX_PROPS: &[&str] = &[
    "messages",
    "mode",
    "currentUserId",
    "userName",
    "userAvatar",
    "userStatus",
    "assistantName",
    "assistantAvatar",
    "showHeader",
    "placeholder",
    "showAttachments",
    "showVoiceNote",
    "showCamera",
    "loading",
    "sending",
    "streaming",
    "hasMore",
    "onSend",
    "onLoadMore",
    "onStop",
    "onVoiceNote",
    "onFileAttach",
    "onCameraCapture",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const EMPTY_PROPS: &[&str] = &[
    "type",
    "title",
    "description",
    "href",
    "navigate",
    "history",
    "target",
    "externalMode",
    "onClick",
    "actionLabel",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const MARQUEE_PROPS: &[&str] = &[
    "speed",
    "pauseOnHover",
    "reverse",
    "orientation",
    "fade",
    "fadeColor",
    "gap",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const TYPE_WRITER_PROPS: &[&str] = &[
    "typeSpeed",
    "deleteSpeed",
    "afterTyped",
    "afterDeleted",
    "repeat",
    "font",
    "id",
    "show",
    "bg",
    "color",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const RICH_TEXT_MARK_PROPS: &[&str] = &["text", "style", "scheme"];
const RECORD_PROPS: &[&str] = &[
    "name",
    "url",
    "disabled",
    "maxDuration",
    "onStart",
    "onPause",
    "onResume",
    "onStop",
    "onDiscard",
    "onConfirm",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const TOGGLE_GROUP_PROPS: &[&str] = &[
    "value",
    "selected",
    "size",
    "wide",
    "vertical",
    "disabled",
    "ariaLabel",
    "onChange",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const COLLAPSIBLE_PROPS: &[&str] = &[
    "label",
    "defaultOpen",
    "disabled",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const COUNTDOWN_PROPS: &[&str] = &[
    "target",
    "showDays",
    "showHours",
    "showMinutes",
    "showSeconds",
    "size",
    "daysLabel",
    "hoursLabel",
    "minutesLabel",
    "secondsLabel",
    "onComplete",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const MAP_PROPS: &[&str] = &[
    "centerLat",
    "centerLng",
    "zoom",
    "height",
    "width",
    "showControls",
    "showScale",
    "showLocationControl",
    "interactive",
    "routeStartLat",
    "routeStartLng",
    "routeEndLat",
    "routeEndLng",
    "onLocation",
    "onLocationError",
    "onRoute",
    "variant",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const MAP_MARKER_PROPS: &[&str] = &["id", "lat", "lng", "label", "popup", "icon", "onClick"];
const MAP_WAYPOINT_PROPS: &[&str] = &["lat", "lng"];
const RADIO_GROUP_PROPS: &[&str] = &[
    "bind",
    "label",
    "name",
    "info",
    "error",
    "orientation",
    "scheme",
    "size",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const ITEM_PROPS: &[&str] = &[
    "id",
    "value",
    "text",
    "label",
    "i18n",
    "description",
    "descriptionI18n",
    "statusI18n",
    "src",
    "name",
    "alt",
    "icon",
    "href",
    "navigate",
    "history",
    "target",
    "externalMode",
    "onClick",
    "disabled",
];
const NAV_MENU_ENTRY_PROPS: &[&str] = &[
    "label",
    "i18n",
    "description",
    "descriptionI18n",
    "href",
    "navigate",
    "target",
    "externalMode",
    "onClick",
];
const COMMAND_GROUP_PROPS: &[&str] = &["label"];
const TOGGLE_THEME_PROPS: &[&str] = &[
    "variant",
    "scheme",
    "size",
    "lightLabel",
    "darkLabel",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
];
const SELECT_THEME_PROPS: &[&str] = &[
    "label",
    "placeholder",
    "variant",
    "scheme",
    "size",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
];
const FAB_PROPS: &[&str] = &[
    "position", "fixed", "offsetX", "offsetY", "icon", "label", "variant", "scheme", "size",
    "onClick", "id", "show", "font", "bg", "cover", "overlay", "colSpan", "rowSpan", "p", "px",
    "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH", "rounded",
];
const FAB_ACTION_PROPS: &[&str] = &[
    "label",
    "icon",
    "scheme",
    "href",
    "navigate",
    "history",
    "target",
    "externalMode",
    "onClick",
];
const SLIDER_PROPS: &[&str] = &[
    "bind",
    "value",
    "min",
    "max",
    "step",
    "label",
    "name",
    "hideLabel",
    "scheme",
    "size",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
];
const DROPZONE_PROPS: &[&str] = &[
    "accept",
    "multiple",
    "maxSize",
    "name",
    "label",
    "helpText",
    "errorText",
    "placeholder",
    "disabled",
    "variant",
    "scheme",
    "size",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
];
const COMBO_BOX_PROPS: &[&str] = &[
    "bind",
    "value",
    "variant",
    "scheme",
    "size",
    "name",
    "label",
    "placeholder",
    "labelFloating",
    "searchPlaceholder",
    "emptyText",
    "loadingText",
    "loadingMoreText",
    "clearable",
    "disabled",
    "helpText",
    "errorText",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const COMBO_OPTION_PROPS: &[&str] = &["value", "label", "description", "src", "icon", "disabled"];
const CSV_FIELD_PROPS: &[&str] = &[
    "buttonText",
    "modalTitle",
    "instructions",
    "cancelText",
    "confirmText",
    "clearText",
    "previewTitle",
    "multiple",
    "showPreview",
    "previewRows",
    "previewPageSize",
    "errorText",
    "variant",
    "scheme",
    "size",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const CSV_COLUMN_PROPS: &[&str] = &["name", "label"];
const DRAG_DROP_PROPS: &[&str] = &[
    "emptyText",
    "direction",
    "allowGroupTransfer",
    "disabled",
    "variant",
    "scheme",
    "size",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const DRAG_GROUP_PROPS: &[&str] = &["id", "title"];
const DRAG_ITEM_PROPS: &[&str] = &["id", "label", "description", "disabled"];
const EDITOR_PROPS: &[&str] = &[
    "bind",
    "value",
    "placeholder",
    "label",
    "helpText",
    "errorText",
    "minHeight",
    "hideToolbar",
    "disabled",
    "readonly",
    "variant",
    "scheme",
    "size",
    "name",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const IMAGE_CROPPER_PROPS: &[&str] = &[
    "bind",
    "src",
    "alt",
    "accept",
    "placeholder",
    "label",
    "helpText",
    "errorText",
    "aspectRatio",
    "minWidth",
    "minHeight",
    "maxWidth",
    "maxHeight",
    "shape",
    "disabled",
    "variant",
    "scheme",
    "size",
    "name",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const PASSWORD_PROPS: &[&str] = &[
    "bind",
    "value",
    "placeholder",
    "label",
    "labelFloating",
    "helpText",
    "errorText",
    "hideStrength",
    "weakLabel",
    "mediumLabel",
    "strongLabel",
    "disabled",
    "readonly",
    "variant",
    "scheme",
    "size",
    "name",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const PHONE_PROPS: &[&str] = &[
    "bind",
    "value",
    "country",
    "dialCodeName",
    "placeholder",
    "label",
    "labelFloating",
    "searchPlaceholder",
    "emptyText",
    "loadingText",
    "priorityCountries",
    "disabled",
    "helpText",
    "errorText",
    "variant",
    "scheme",
    "size",
    "name",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const PIN_PROPS: &[&str] = &[
    "bind",
    "value",
    "length",
    "type",
    "label",
    "helpText",
    "errorText",
    "variant",
    "scheme",
    "size",
    "name",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const TEXTAREA_PROPS: &[&str] = &[
    "bind",
    "value",
    "placeholder",
    "label",
    "labelFloating",
    "helpText",
    "errorText",
    "rows",
    "cols",
    "maxLength",
    "resize",
    "disabled",
    "readonly",
    "variant",
    "scheme",
    "size",
    "name",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const INPUT_PROPS: &[&str] = &[
    "iconStart",
    "iconEnd",
    "variant",
    "scheme",
    "size",
    "bind",
    "label",
    "placeholder",
    "labelFloating",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
];
const SELECT_PROPS: &[&str] = &[
    "variant",
    "scheme",
    "size",
    "bind",
    "label",
    "placeholder",
    "labelFloating",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
];
const OPTION_PROPS: &[&str] = &["value", "label", "description"];
const CODE_PROPS: &[&str] = &[
    "content",
    "template",
    "language",
    "variant",
    "scheme",
    "copyLabel",
    "copiedLabel",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const VIDEO_PROPS: &[&str] = &[
    "src", "poster", "autoplay", "aspect", "variant", "scheme", "id", "show", "font", "p", "px",
    "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH", "rounded", "border",
];
const IFRAME_PROPS: &[&str] = &[
    "src",
    "title",
    "loading",
    "allow",
    "sandbox",
    "allowFullscreen",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const DEVICE_PROPS: &[&str] = &[
    "device", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH", "maxW", "maxH", "rounded", "border",
];
const CANVAS_PROPS: &[&str] = &[
    "scene",
    "viewWidth",
    "viewHeight",
    "fit",
    "fps",
    "autoplay",
    "background",
    "pixelated",
    "label",
    "onPointer",
    "onKey",
    "onMotion",
    "motionRate",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const CANDLESTICK_PROPS: &[&str] = &[
    "data",
    "stream",
    "variant",
    "scheme",
    "upColor",
    "downColor",
    "emptyLabel",
    "maxPoints",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const ARC_CHART_PROPS: &[&str] = &[
    "data",
    "variant",
    "scheme",
    "size",
    "palette",
    "legendPosition",
    "emptyLabel",
    "loading",
    "hideLegend",
    "startAngle",
    "endAngle",
    "thickness",
    "gap",
    "centerText",
    "centerValue",
    "showInlineLabels",
    "hideValues",
    "showGlow",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const AREA_CHART_PROPS: &[&str] = &[
    "data",
    "series",
    "variant",
    "scheme",
    "size",
    "palette",
    "legendPosition",
    "emptyLabel",
    "loading",
    "hideLegend",
    "curve",
    "stacked",
    "strokeWidth",
    "showPoints",
    "hideLine",
    "fillOpacity",
    "hideGrid",
    "hideXAxis",
    "hideYAxis",
    "showGlow",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const BAR_CHART_PROPS: &[&str] = &[
    "data",
    "series",
    "variant",
    "scheme",
    "size",
    "palette",
    "legendPosition",
    "emptyLabel",
    "loading",
    "hideLegend",
    "stacked",
    "grouped",
    "showValues",
    "barRadius",
    "hideGrid",
    "showGlow",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const LINE_CHART_PROPS: &[&str] = &[
    "data",
    "series",
    "variant",
    "scheme",
    "size",
    "palette",
    "legendPosition",
    "emptyLabel",
    "loading",
    "hideLegend",
    "curve",
    "strokeWidth",
    "pointRadius",
    "hidePoints",
    "hideGrid",
    "hideXAxis",
    "hideYAxis",
    "showGradientFill",
    "showGlow",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const PIE_CHART_PROPS: &[&str] = &[
    "data",
    "variant",
    "scheme",
    "size",
    "palette",
    "legendPosition",
    "emptyLabel",
    "loading",
    "hideLegend",
    "donut",
    "donutWidth",
    "centerLabel",
    "centerValue",
    "startAngle",
    "padAngle",
    "hideLabels",
    "hideValues",
    "hidePercentages",
    "showGlow",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const TABLE_PROPS: &[&str] = &[
    "data",
    "variant",
    "scheme",
    "size",
    "striped",
    "bordered",
    "dividers",
    "emptyTitle",
    "emptyDescription",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const COLUMN_PROPS: &[&str] = &["field", "label", "align", "width"];
const DIVIDER_PROPS: &[&str] = &[
    "orientation",
    "scheme",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const BUTTON_PROPS: &[&str] = &[
    "iconStart",
    "iconEnd",
    "i18n",
    "loading",
    "variant",
    "scheme",
    "size",
    "href",
    "navigate",
    "history",
    "target",
    "externalMode",
    "onClick",
    "id",
    "show",
    "font",
    "bg",
    "cover",
    "overlay",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
];
const BRAND_PROPS: &[&str] = &[
    "href",
    "label",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
    "borderColor",
    "shadow",
    "shadowColor",
];
const BANNER_PROPS: &[&str] = &[
    "href",
    "label",
    "id",
    "show",
    "font",
    "bg",
    "color",
    "cover",
    "overlay",
    "animation",
    "colSpan",
    "rowSpan",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
    "borderColor",
    "shadow",
    "shadowColor",
];
const ICON_BUTTON_PROPS: &[&str] = &[
    "icon",
    "label",
    "variant",
    "scheme",
    "size",
    "href",
    "navigate",
    "history",
    "target",
    "externalMode",
    "onClick",
    "id",
    "show",
    "font",
    "p",
    "px",
    "py",
    "pl",
    "pr",
    "pt",
    "pb",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "rounded",
    "border",
];
const ALERT_PROPS: &[&str] = &[
    "type", "message", "visible", "onClose", "variant", "scheme", "id", "show", "font", "bg",
    "cover", "overlay", "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h",
    "rounded",
];
const TEXT_PROPS: &[&str] = &[
    "size", "weight", "spacing", "i18n", "font", "id", "show", "bg", "color", "p", "px", "py",
    "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH", "rounded",
];
const RICH_TEXT_PROPS: &[&str] = &[
    "title", "size", "weight", "spacing", "i18n", "font", "id", "show", "bg", "color", "p", "px",
    "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH", "rounded",
];
const SVG_PROPS: &[&str] = &["viewBox", "data", "color", "w", "h", "id", "show"];
const ICON_PROPS: &[&str] = &["name", "fill", "stroke", "w", "h", "id", "show"];
const PATH_PROPS: &[&str] = &["d", "fill", "fillRule", "transform"];
