use crate::error::{DoweError, DoweResult};
use crate::model::ViewTargetRoutes;
use crate::parser::source_ast::{SourceFile, SourceNode, SourceProp, SourceValue};
use crate::parser::source_parser::parse_source_file;
use dowe_components::{
    TranslationCatalog, TranslationLocale, TranslationValue, ViewNode, is_valid_i18n_key,
    is_valid_locale,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub fn parse_translation_catalog(root: &Path) -> DoweResult<TranslationCatalog> {
    let source_root = root.join("src/i18n");
    if !source_root.is_dir() {
        return Ok(TranslationCatalog::default());
    }
    let mut paths = fs::read_dir(&source_root)
        .map_err(|error| DoweError::at_path(&source_root, error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| DoweError::at_path(&source_root, error.to_string()))?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("dowe"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut default_locale = None;
    let mut locales = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path)
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        let file = parse_source_file(root, &path, source)?;
        let (locale, is_default, values) = parse_translation_file(&file)?;
        if is_default {
            if let Some(existing) = default_locale.as_deref() {
                return Err(DoweError::at_path(
                    &path,
                    format!(
                        "translation catalogs must declare exactly one default locale; `{existing}` is already default"
                    ),
                ));
            }
            default_locale = Some(locale.clone());
        }
        locales.push(TranslationLocale {
            locale,
            source_path: file.relative_path,
            values,
        });
    }

    if !locales.is_empty() && default_locale.is_none() {
        return Err(DoweError::at_path(
            &source_root,
            "translation catalogs must declare exactly one `translations default:true` locale",
        ));
    }

    Ok(TranslationCatalog {
        default_locale,
        locales,
    })
}

pub fn validate_translation_source(file: &SourceFile) -> DoweResult<()> {
    parse_translation_file(file).map(|_| ())
}

pub fn validate_view_i18n_keys(
    views_path: &Path,
    routes: &ViewTargetRoutes,
    catalog: &TranslationCatalog,
) -> DoweResult<()> {
    let mut keys = BTreeSet::new();
    for route in routes
        .web
        .iter()
        .chain(&routes.desktop)
        .chain(&routes.android)
        .chain(&routes.ios)
    {
        collect_i18n_keys(&route.layout_tree, &mut keys);
        collect_i18n_keys(&route.page_tree, &mut keys);
    }
    for key in keys {
        if catalog.locales.is_empty() {
            return Err(DoweError::at_path(
                views_path,
                format!("translation key `{key}` requires catalogs under `src/i18n/<locale>.dowe`"),
            ));
        }
        for locale in &catalog.locales {
            if locale.value(&key).is_none() {
                return Err(DoweError::at_path(
                    &locale.source_path,
                    format!(
                        "translation key `{key}` is missing for locale `{}`",
                        locale.locale
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn parse_translation_file(file: &SourceFile) -> DoweResult<(String, bool, Vec<TranslationValue>)> {
    let locale = locale_from_path(&file.path)?;
    if file.nodes.len() != 1 || file.nodes[0].name != "translations" {
        return Err(DoweError::at_path(
            &file.path,
            "translation catalog must declare one `translations` block",
        ));
    }
    let root = &file.nodes[0];
    if !root.args.is_empty() {
        return Err(node_error(root, "`translations` does not accept args"));
    }
    reject_unknown_props(root, &["default"])?;
    let is_default = optional_bool_prop(root, "default")?.unwrap_or(false);
    let mut values = BTreeMap::new();
    for node in &root.children {
        collect_translation_value(node, &mut Vec::new(), &mut values)?;
    }
    Ok((
        locale,
        is_default,
        values
            .into_iter()
            .map(|(key, value)| TranslationValue { key, value })
            .collect(),
    ))
}

fn collect_translation_value(
    node: &SourceNode,
    prefix: &mut Vec<String>,
    values: &mut BTreeMap<String, String>,
) -> DoweResult<()> {
    if node.name == "translation" {
        if !prefix.is_empty() {
            return Err(node_error(
                node,
                "`translation` entries must be direct children of `translations`",
            ));
        }
        return collect_explicit_translation_value(node, values);
    }

    if !is_valid_i18n_key_segment(&node.name) {
        return Err(node_error(
            node,
            "translation key groups must use non-empty alphanumeric, `_`, or `-` segments",
        ));
    }
    if !node.props.is_empty() {
        return Err(node_error(
            node,
            "translation key groups do not accept props",
        ));
    }
    if !node.args.is_empty() && !node.children.is_empty() {
        return Err(node_error(
            node,
            "translation values cannot have nested children",
        ));
    }

    prefix.push(node.name.clone());
    if node.children.is_empty() {
        let value = match node.args.as_slice() {
            [SourceValue::String(value)] if !value.is_empty() => value.clone(),
            [SourceValue::String(_)] => {
                return Err(node_error(node, "translation values cannot be empty"));
            }
            [_] => {
                return Err(node_error(
                    node,
                    "translation values must be quoted strings",
                ));
            }
            [] => {
                return Err(node_error(
                    node,
                    "translation key groups require children or one quoted string value",
                ));
            }
            _ => {
                return Err(node_error(
                    node,
                    "translation values accept one quoted string",
                ));
            }
        };
        let key = prefix.join(".");
        insert_translation_value(node, values, key, value)?;
    } else {
        if !node.args.is_empty() {
            return Err(node_error(
                node,
                "translation key groups cannot mix values and children",
            ));
        }
        for child in &node.children {
            collect_translation_value(child, prefix, values)?;
        }
    }
    prefix.pop();
    Ok(())
}

fn collect_explicit_translation_value(
    node: &SourceNode,
    values: &mut BTreeMap<String, String>,
) -> DoweResult<()> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`translation` only accepts `key` and `value` props",
        ));
    }
    reject_unknown_props(node, &["key", "value"])?;
    let key = required_string_prop(node, "key")?;
    if !is_valid_i18n_key(&key) {
        return Err(node_error(
            node,
            "`translation key` must use non-empty segments separated by dots",
        ));
    }
    let value = required_string_prop(node, "value")?;
    insert_translation_value(node, values, key, value)
}

fn insert_translation_value(
    node: &SourceNode,
    values: &mut BTreeMap<String, String>,
    key: String,
    value: String,
) -> DoweResult<()> {
    if values.insert(key.clone(), value).is_some() {
        return Err(node_error(
            node,
            format!("duplicate translation key `{key}`"),
        ));
    }
    Ok(())
}

fn is_valid_i18n_key_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
}

fn locale_from_path(path: &Path) -> DoweResult<String> {
    let locale = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    if is_valid_locale(&locale) {
        Ok(locale)
    } else {
        Err(DoweError::at_path(
            path,
            "translation locale file name must use two or three lowercase letters",
        ))
    }
}

fn collect_i18n_keys(node: &ViewNode, keys: &mut BTreeSet<String>) {
    match node {
        ViewNode::Scope { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Each { children, .. } => {
            for child in children {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => {
            for child in start.iter().chain(center).chain(end) {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_i18n_keys(child, keys);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_i18n_keys(child, keys);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_i18n_keys(child, keys);
                }
            }
        }
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_i18n_keys(child, keys);
                    }
                }
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Title { props, .. }
        | ViewNode::Text { props, .. }
        | ViewNode::RichText { props, .. } => {
            if let Some(key) = props.i18n.as_ref() {
                keys.insert(key.clone());
            }
        }
        ViewNode::Input { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Svg { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Children => {}
    }
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(prop_error(
                prop,
                format!("`{}` does not support `{}`", node.name, prop.name),
            ));
        }
    }
    Ok(())
}

fn required_string_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("`{}` requires `{name}`", node.name)))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(prop_error(
            prop,
            format!("`{name}` must be a non-empty quoted string"),
        )),
    }
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match prop.value {
            SourceValue::Boolean(value) => Ok(value),
            _ => Err(prop_error(prop, format!("`{name}` must be a boolean"))),
        })
        .transpose()
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_translation_catalog() {
        let (_, is_default, values) = parse_catalog(
            r#"translations default:true
  common
    welcome "Welcome, {{name}}!"
  messages
    items
      singular "You have {{count}} item"
      plural "You have {{count}} items"
"#,
        );

        assert!(is_default);
        assert_eq!(value(&values, "common.welcome"), Some("Welcome, {{name}}!"));
        assert_eq!(
            value(&values, "messages.items.singular"),
            Some("You have {{count}} item")
        );
        assert_eq!(
            value(&values, "messages.items.plural"),
            Some("You have {{count}} items")
        );
    }

    #[test]
    fn keeps_explicit_translation_entries_compatible() {
        let (_, _, values) = parse_catalog(
            r#"translations default:true
  translation key:"home.hero.title" value:"Dowe builds systems."
"#,
        );

        assert_eq!(
            value(&values, "home.hero.title"),
            Some("Dowe builds systems.")
        );
    }

    #[test]
    fn rejects_duplicate_nested_and_explicit_keys() {
        let root = Path::new("/project");
        let path = root.join("src/i18n/en.dowe");
        let file = parse_source_file(
            root,
            &path,
            r#"translations default:true
  home
    hero
      title "Dowe builds systems."
  translation key:"home.hero.title" value:"Duplicate"
"#
            .to_string(),
        )
        .expect("source");

        let error = parse_translation_file(&file).expect_err("duplicate");

        assert!(error.to_string().contains("duplicate translation key"));
    }

    fn parse_catalog(source: &str) -> (String, bool, Vec<TranslationValue>) {
        let root = Path::new("/project");
        let path = root.join("src/i18n/en.dowe");
        let file = parse_source_file(root, &path, source.to_string()).expect("translation source");
        parse_translation_file(&file).expect("translation catalog")
    }

    fn value<'a>(values: &'a [TranslationValue], key: &str) -> Option<&'a str> {
        values
            .iter()
            .find(|value| value.key == key)
            .map(|value| value.value.as_str())
    }
}
