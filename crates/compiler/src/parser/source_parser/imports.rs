use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::{SourceImport, SourceLocation, SourceValue};
use crate::parser::source_values::{parse_value, split_top_level_whitespace};
use std::path::Path;

pub(super) fn parse_imports(path: &Path, location: SourceLocation, source: &str) -> DoweResult<Vec<SourceImport>> {
    let tokens = split_top_level_whitespace(source, location.column.saturating_sub(1))?;
    let Some(from_index) = tokens.iter().position(|token| token.text == "from") else {
        return Err(DoweError::at_path(path, format!("{}:{}: invalid import syntax", location.line, location.column)));
    };
    if tokens.first().is_none_or(|token| token.text != "import") || from_index < 2 || tokens.len() != from_index + 2 {
        return Err(DoweError::at_path(path, format!("{}:{}: invalid import syntax", location.line, location.column)));
    }
    let names = tokens[1..from_index].iter().map(|token| token.text.as_str()).collect::<Vec<_>>().join(" ");
    let names = parse_import_names(path, &location, &names)?;
    let SourceValue::String(import_path) = parse_value(path, location.line, tokens[from_index + 1].column, &tokens[from_index + 1].text)? else {
        return Err(DoweError::at_path(path, format!("{}:{}: import path must be a string", location.line, tokens[from_index + 1].column)));
    };
    Ok(names.into_iter().map(|local| SourceImport { local, path: import_path.clone(), location: location.clone() }).collect())
}

fn parse_import_names(path: &Path, location: &SourceLocation, source: &str) -> DoweResult<Vec<String>> {
    let source = source.trim();
    let source = if source.starts_with('{') {
        source.strip_prefix('{').and_then(|value| value.strip_suffix('}'))
    } else if source.contains('{') || source.contains('}') { None } else { Some(source) }
    .map(str::trim).filter(|value| !value.is_empty()).ok_or_else(|| DoweError::at_path(path, format!("{}:{}: invalid import syntax", location.line, location.column)))?;
    let mut names = Vec::new();
    for name in source.split(',').map(str::trim) {
        if !is_import_name(name) || names.iter().any(|existing| existing == name) {
            return Err(DoweError::at_path(path, format!("{}:{}: invalid import syntax", location.line, location.column)));
        }
        names.push(name.to_string());
    }
    Ok(names)
}

fn is_import_name(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some(value) if value.is_ascii_alphabetic() || value == '_')
        && characters.all(|value| value.is_ascii_alphanumeric() || value == '_')
}
