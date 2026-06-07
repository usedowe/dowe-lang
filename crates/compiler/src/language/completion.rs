use crate::language::analysis::{
    document_workspace_root, environment_config, normalize_path, reference_fields, signal_fields,
};
use crate::language::model::{LanguageCompletion, LanguageCompletionKind, LanguageDocument};
use crate::parser::{SourceNode, SourceValue, parse_source_file};
use dowe_components::{
    AlertKind, Align, AvatarStatus, BuiltinComponent, ButtonSize, CodeLanguage, ColorFamily,
    ColorToken, ComponentVariant, DividerOrientation, DrawerPosition, FontFamily, GridAlignment,
    Justify, NativeExternalMode, NavigationOperation, OverlayCornerPosition, OverlayPosition,
    RoundedSize, SectionBackground, SideNavSize, SkeletonAnimation, SkeletonVariant,
    TableColumnAlign, TableSize, TabsPosition, TabsVariant, TextSize, TextSpacing, TextWeight,
    ToastKind, VideoAspect, ViewAnimation, WebTarget,
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
    if import_context(&prefix) {
        return import_completions(root, &document.path);
    }
    if prefix.ends_with("env.") {
        return env_completions(root);
    }
    if prop_value_context(&prefix, "onClick") {
        return action_completions(root, document);
    }
    if middleware_context(&prefix) {
        return middleware_completions(root, document);
    }
    if prop_value_context(&prefix, "bind") {
        return signal_completions(root, document);
    }
    if prop_value_context(&prefix, "data") {
        return signal_completions(root, document);
    }
    if prop_value_context(&prefix, "open") {
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
        && let Some(completions) = component_value_completions(component, prop)
    {
        return completions;
    }
    if let Some(prop) = column_prop_value_context(&prefix)
        && let Some(completions) = column_value_completions(prop)
    {
        return completions;
    }
    if let Some(component) = component_before_cursor(&prefix) {
        return prop_completions(component);
    }
    base_completions()
}

fn line_prefix(source: &str, line: usize, column: usize) -> String {
    let value = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or_default();
    value.chars().take(column.saturating_sub(1)).collect()
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
    let mut parts = prefix.trim_start().split_whitespace();
    let component = BuiltinComponent::from_name(parts.next()?)?;
    let token = parts.last()?;
    let (prop, _) = token.split_once(':')?;
    (!prop.is_empty()).then_some((component, prop))
}

fn column_prop_value_context(prefix: &str) -> Option<&str> {
    let mut parts = prefix.trim_start().split_whitespace();
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
    if parts.next().is_some() && !trimmed.ends_with(':') {
        Some(name)
    } else if trimmed.ends_with(' ') {
        Some(name)
    } else {
        None
    }
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
        "component",
        "action",
        "middleware",
        "signal",
        "request",
        "assign",
        "reset",
        "if",
        "else",
        "each",
        "route",
        "method",
        "handler",
        "continue",
        "jwt.verify",
        "jwt.decrypt",
        "jwt.sign",
        "jwt.encrypt",
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
        "tab",
        "column",
    ];
    let components = [
        "Box",
        "Section",
        "Flex",
        "Grid",
        "Card",
        "Table",
        "AppBar",
        "Footer",
        "BottomBar",
        "SideNav",
        "Sidebar",
        "NavMenu",
        "Scaffold",
        "Tabs",
        "Drawer",
        "Avatar",
        "Badge",
        "Chip",
        "Skeleton",
        "Modal",
        "AlertDialog",
        "Tooltip",
        "Toast",
        "Dropdown",
        "Command",
        "Input",
        "Select",
        "Option",
        "Code",
        "Video",
        "Candlestick",
        "Divider",
        "Button",
        "Alert",
        "Svg",
        "Path",
        "Title",
        "Text",
    ];
    keywords
        .into_iter()
        .map(|label| completion(label, LanguageCompletionKind::Keyword, "Dowe keyword"))
        .chain(
            components.into_iter().map(|label| {
                completion(label, LanguageCompletionKind::Component, "Dowe component")
            }),
        )
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
                .map(|name| completion(&name, LanguageCompletionKind::Function, "view action"))
                .collect()
        })
        .unwrap_or_else(|_| {
            collect_line_declarations(&document.source, "action")
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Function, "view action"))
                .collect()
        })
}

fn signal_completions(root: &Path, document: &LanguageDocument) -> Vec<LanguageCompletion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| {
            let types =
                crate::parser::TypeRegistry::parse(&file.path, &file.nodes).unwrap_or_default();
            file.nodes
                .iter()
                .flat_map(|node| collect_signals(node, &types))
                .map(|name| completion(&name, LanguageCompletionKind::Variable, "signal path"))
                .collect()
        })
        .unwrap_or_else(|_| {
            collect_line_signals(&document.source)
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Variable, "signal path"))
                .collect()
        })
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
    if node.name == "action"
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
    if node.name == "signal"
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
            let mut parts = line.trim_start().split_whitespace();
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
        if parts.next() != Some("signal") {
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
        .map(|label| completion(label, LanguageCompletionKind::Property, "component prop"))
        .collect()
}

fn component_value_completions(
    component: BuiltinComponent,
    prop: &str,
) -> Option<Vec<LanguageCompletion>> {
    if !props_for_component(component.as_str()).contains(&prop) {
        return None;
    }

    match (component, prop) {
        (
            BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::Table
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::SideNav
            | BuiltinComponent::Drawer
            | BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Button
            | BuiltinComponent::Alert,
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
            | BuiltinComponent::Badge
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Toast,
            "variant",
        ) => Some(solid_soft_values()),
        (BuiltinComponent::Tabs, "variant") => Some(quoted_values(
            TabsVariant::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::Table
            | BuiltinComponent::Divider
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::SideNav
            | BuiltinComponent::Tabs
            | BuiltinComponent::Drawer
            | BuiltinComponent::Avatar
            | BuiltinComponent::Badge
            | BuiltinComponent::Chip
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Toast
            | BuiltinComponent::Dropdown
            | BuiltinComponent::Command,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Button
            | BuiltinComponent::Alert,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all()
                .iter()
                .filter(|value| !matches!(value, ColorFamily::Background | ColorFamily::Surface))
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::Button, "size") => Some(quoted_values(
            ButtonSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Avatar | BuiltinComponent::Chip, "size") => Some(quoted_values(
            ButtonSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Table, "size") => Some(quoted_values(
            TableSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Code, "language") => Some(quoted_values(
            CodeLanguage::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Video, "aspect") => Some(quoted_values(
            VideoAspect::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Divider, "orientation") => Some(quoted_values(
            DividerOrientation::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::SideNav | BuiltinComponent::Sidebar | BuiltinComponent::NavMenu,
            "size",
        ) => Some(quoted_values(
            SideNavSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Drawer, "position") => Some(quoted_values(
            DrawerPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Tabs, "position") => Some(quoted_values(
            TabsPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Avatar, "status") => Some(quoted_values(
            AvatarStatus::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Badge | BuiltinComponent::Toast, "position") => Some(quoted_values(
            OverlayCornerPosition::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::Tooltip, "position") => Some(quoted_values(
            OverlayPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Skeleton, "variant") => Some(quoted_values(
            SkeletonVariant::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Skeleton, "animation") => Some(quoted_values(
            SkeletonAnimation::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Box | BuiltinComponent::Section | BuiltinComponent::Card,
            "animation",
        ) => Some(quoted_values(
            ViewAnimation::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Section, "background") => Some(quoted_values(
            SectionBackground::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Title | BuiltinComponent::Text, "size") => Some(quoted_values(
            TextSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Title | BuiltinComponent::Text, "weight") => Some(quoted_values(
            TextWeight::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Title | BuiltinComponent::Text, "spacing") => Some(quoted_values(
            TextSpacing::all().iter().map(|value| value.as_str()),
        )),
        (_, "font") => Some(quoted_values(
            FontFamily::all().iter().map(|value| value.as_str()),
        )),
        (_, "bg" | "color" | "upColor" | "downColor") => Some(quoted_values(
            ColorToken::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Path, "fill") => Some(quoted_values(
            ["none", "currentColor"]
                .into_iter()
                .chain(ColorToken::all().iter().map(|value| value.as_str())),
        )),
        (_, "rounded") => Some(quoted_values(
            RoundedSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "justify") => Some(quoted_values(
            Justify::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "align") => Some(quoted_values(
            Align::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Grid, "justify" | "align") => Some(quoted_values(
            GridAlignment::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Button | BuiltinComponent::Avatar, "navigate") => Some(quoted_values(
            NavigationOperation::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::Button | BuiltinComponent::Avatar, "history") => {
            Some(quoted_values(["back"]))
        }
        (BuiltinComponent::Button | BuiltinComponent::Avatar, "target") => Some(quoted_values(
            WebTarget::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Button | BuiltinComponent::Avatar, "externalMode") => Some(
            quoted_values(NativeExternalMode::all().iter().map(|value| value.as_str())),
        ),
        (BuiltinComponent::Alert, "type") => Some(quoted_values(
            AlertKind::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Toast, "type") => Some(quoted_values(
            ToastKind::all().iter().map(|value| value.as_str()),
        )),
        _ => None,
    }
}

fn solid_soft_values() -> Vec<LanguageCompletion> {
    quoted_values(["solid", "soft"])
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

fn quoted_values<'a>(values: impl IntoIterator<Item = &'a str>) -> Vec<LanguageCompletion> {
    values
        .into_iter()
        .map(|value| {
            completion(
                &format!("\"{value}\""),
                LanguageCompletionKind::Value,
                "quoted static value",
            )
        })
        .collect()
}

fn props_for_component(component: &str) -> &'static [&'static str] {
    match component {
        "Box" => &STYLE_PROPS,
        "Section" => &SECTION_PROPS,
        "Flex" => &LAYOUT_PROPS,
        "Grid" => &GRID_PROPS,
        "Card" => &VARIANT_PROPS,
        "AppBar" | "BottomBar" => &FLOATING_BAR_PROPS,
        "Footer" => &BAR_PROPS,
        "SideNav" => &SIDE_NAV_PROPS,
        "Sidebar" => &SIDE_NAV_PROPS,
        "NavMenu" => &NAV_MENU_PROPS,
        "Scaffold" => &SCAFFOLD_PROPS,
        "Tabs" => &TABS_PROPS,
        "tab" => &TAB_PROPS,
        "Drawer" => &DRAWER_PROPS,
        "Avatar" => &AVATAR_PROPS,
        "Badge" => &BADGE_PROPS,
        "Chip" => &CHIP_PROPS,
        "Skeleton" => &SKELETON_PROPS,
        "Modal" => &MODAL_PROPS,
        "AlertDialog" => &ALERT_DIALOG_PROPS,
        "Tooltip" => &TOOLTIP_PROPS,
        "Toast" => &TOAST_PROPS,
        "Dropdown" => &DROPDOWN_PROPS,
        "Command" => &COMMAND_PROPS,
        "item" => &OVERLAY_ITEM_PROPS,
        "group" => &COMMAND_GROUP_PROPS,
        "Input" => &INPUT_PROPS,
        "Select" => &SELECT_PROPS,
        "Option" => &OPTION_PROPS,
        "Code" => &CODE_PROPS,
        "Video" => &VIDEO_PROPS,
        "Candlestick" => &CANDLESTICK_PROPS,
        "Table" => &TABLE_PROPS,
        "column" => &COLUMN_PROPS,
        "Divider" => &DIVIDER_PROPS,
        "Button" => &BUTTON_PROPS,
        "Alert" => &ALERT_PROPS,
        "Svg" => &SVG_PROPS,
        "Path" => &PATH_PROPS,
        "Title" | "Text" => &TEXT_PROPS,
        _ => &[],
    }
}

fn import_completions(root: &Path, from: &Path) -> Vec<LanguageCompletion> {
    let src = root.join("src");
    let files = dowe_files(&src);
    files
        .into_iter()
        .filter_map(|path| relative_import_path(from, &path))
        .map(|label| completion(&label, LanguageCompletionKind::File, "Dowe source"))
        .collect()
}

fn dowe_files(path: &Path) -> Vec<PathBuf> {
    let mut output = Vec::new();
    let Ok(entries) = fs::read_dir(path) else {
        return output;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            output.extend(dowe_files(&path));
        } else if path.extension().and_then(|value| value.to_str()) == Some("dowe") {
            output.push(path);
        }
    }
    output.sort();
    output
}

fn relative_import_path(from: &Path, target: &Path) -> Option<String> {
    let parent = from.parent()?;
    let normalized = normalize_path(target.to_path_buf());
    let relative = normalized.strip_prefix(parent).ok().map(Path::to_path_buf);
    let mut path = relative.unwrap_or(normalized);
    path.set_extension("");
    let value = path.to_string_lossy().replace('\\', "/");
    if value.starts_with('.') {
        Some(value)
    } else {
        Some(format!("./{value}"))
    }
}

fn completion(label: &str, kind: LanguageCompletionKind, detail: &str) -> LanguageCompletion {
    LanguageCompletion {
        label: label.to_string(),
        kind,
        detail: Some(detail.to_string()),
    }
}

const STYLE_PROPS: &[&str] = &[
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
    "rounded",
];
const SECTION_PROPS: &[&str] = &[
    "id",
    "show",
    "font",
    "bg",
    "color",
    "background",
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
    "rounded",
    "border",
];
const LAYOUT_PROPS: &[&str] = &[
    "justify", "align", "gap", "id", "show", "font", "bg", "color", "cover", "overlay", "colSpan",
    "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "rounded",
];
const GRID_PROPS: &[&str] = &[
    "columns", "rows", "justify", "align", "gap", "id", "show", "font", "bg", "color", "cover",
    "overlay", "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH",
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
    "rounded",
];
const BAR_PROPS: &[&str] = &[
    "variant", "scheme", "bordered", "blurred", "boxed", "id", "show", "font", "p", "px", "py",
    "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "rounded", "border",
];
const FLOATING_BAR_PROPS: &[&str] = &[
    "variant", "scheme", "bordered", "blurred", "boxed", "floating", "id", "show", "font", "p",
    "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "rounded", "border",
];
const SIDE_NAV_PROPS: &[&str] = &[
    "variant", "scheme", "size", "wide", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt",
    "pb", "w", "h", "minW", "minH", "rounded", "border",
];
const NAV_MENU_PROPS: &[&str] = &[
    "variant", "scheme", "size", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb",
    "w", "h", "minW", "minH", "rounded", "border",
];
const SCAFFOLD_PROPS: &[&str] = &[
    "boxed", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH", "rounded", "border",
];
const TABS_PROPS: &[&str] = &[
    "variant", "scheme", "position", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb",
    "w", "h", "minW", "minH", "rounded", "border",
];
const TAB_PROPS: &[&str] = &["id", "label"];
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
    "rounded",
];
const BADGE_PROPS: &[&str] = &[
    "text", "variant", "scheme", "position", "id", "show", "font", "bg", "cover", "overlay",
    "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH",
    "rounded",
];
const CHIP_PROPS: &[&str] = &[
    "variant", "scheme", "size", "onClose", "id", "show", "font", "bg", "cover", "overlay",
    "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH",
    "rounded",
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
    "rounded",
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
    "rounded",
    "border",
];
const TOOLTIP_PROPS: &[&str] = &[
    "label", "position", "variant", "scheme", "id", "show", "font", "bg", "cover", "overlay",
    "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH",
    "rounded",
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
    "rounded",
];
const DROPDOWN_PROPS: &[&str] = &[
    "scheme", "id", "show", "font", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW",
    "minH", "rounded", "border",
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
    "rounded",
    "border",
];
const OVERLAY_ITEM_PROPS: &[&str] = &[
    "label",
    "description",
    "href",
    "navigate",
    "history",
    "target",
    "externalMode",
    "onClick",
    "disabled",
];
const COMMAND_GROUP_PROPS: &[&str] = &["label"];
const INPUT_PROPS: &[&str] = &[
    "variant",
    "scheme",
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
    "rounded",
];
const SELECT_PROPS: &[&str] = &[
    "variant",
    "scheme",
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
    "rounded",
];
const OPTION_PROPS: &[&str] = &["value", "label", "description"];
const CODE_PROPS: &[&str] = &[
    "lines",
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
    "rounded",
    "border",
];
const VIDEO_PROPS: &[&str] = &[
    "src", "poster", "autoplay", "aspect", "variant", "scheme", "id", "show", "font", "p", "px",
    "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "rounded", "border",
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
    "rounded",
    "border",
];
const BUTTON_PROPS: &[&str] = &[
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
    "rounded",
];
const ALERT_PROPS: &[&str] = &[
    "type", "message", "visible", "onClose", "variant", "scheme", "id", "show", "font", "bg",
    "cover", "overlay", "colSpan", "rowSpan", "p", "px", "py", "pl", "pr", "pt", "pb", "w", "h",
    "rounded",
];
const TEXT_PROPS: &[&str] = &[
    "size", "weight", "spacing", "i18n", "font", "id", "show", "bg", "color", "p", "px", "py",
    "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "rounded",
];
const SVG_PROPS: &[&str] = &["viewBox", "color", "w", "h", "id", "show"];
const PATH_PROPS: &[&str] = &["d", "fill"];
