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
    let source_root = root.join("i18n");
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
                format!("translation key `{key}` requires catalogs under `i18n/<locale>.dowe`"),
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

