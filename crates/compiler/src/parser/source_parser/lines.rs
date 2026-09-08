use crate::error::{DoweError, DoweResult};
use std::path::Path;

#[derive(Clone)]
pub(super) struct LogicalLine {
    pub(super) line: usize,
    pub(super) indent_spaces: usize,
    pub(super) source: String,
}

#[derive(Default)]
struct DelimiterState {
    brace_depth: usize,
    bracket_depth: usize,
}

pub(super) fn logical_lines(path: &Path, source: &str) -> DoweResult<Vec<LogicalLine>> {
    let physical_lines = source.lines().collect::<Vec<_>>();
    let mut logical_lines = Vec::new();
    let mut index = 0usize;

    while index < physical_lines.len() {
        let physical = physical_lines[index];
        if physical.trim().is_empty() || physical.trim_start().starts_with("//") {
            index += 1;
            continue;
        }
        let line = index + 1;
        let indent_spaces = leading_indent(path, line, physical)?;
        let value = strip_line_comment(physical[indent_spaces..].trim_end());
        if value.trim().is_empty() {
            index += 1;
            continue;
        }
        let mut value = value.to_string();
        if value.contains("\"\"\"") {
            let opening_count = value.matches("\"\"\"").count();
            if opening_count != 1 || !value.ends_with("\"\"\"") {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "{line}:{}: multiline strings must open after a prop value",
                        indent_spaces + 1
                    ),
                ));
            }
            loop {
                index += 1;
                let Some(next) = physical_lines.get(index).copied() else {
                    return Err(DoweError::at_path(
                        path,
                        format!(
                            "{line}:{}: missing multiline string closing delimiter",
                            indent_spaces + 1
                        ),
                    ));
                };
                let next_indent = leading_indent(path, index + 1, next)?;
                if next.trim() == "\"\"\"" {
                    if next_indent != indent_spaces {
                        return Err(DoweError::at_path(
                            path,
                            format!(
                                "{}:{}: multiline string closing delimiter must align with its prop",
                                index + 1,
                                next_indent + 1
                            ),
                        ));
                    }
                    value.push('\n');
                    value.push_str("\"\"\"");
                    break;
                }
                value.push('\n');
                value.push_str(next);
            }
            logical_lines.push(LogicalLine { line, indent_spaces, source: value });
            index += 1;
            continue;
        }
        let mut delimiters = DelimiterState::default();
        delimiters.scan(path, line, &value)?;

        while delimiters.is_open() {
            index += 1;
            let Some(physical) = physical_lines.get(index).copied() else {
                return Err(DoweError::at_path(
                    path,
                    format!("{line}:{}: unclosed structured value", indent_spaces + 1),
                ));
            };
            if !physical.trim().is_empty() {
                leading_indent(path, index + 1, physical)?;
            }
            let next_value = strip_line_comment(physical.trim());
            value.push('\n');
            value.push_str(next_value);
            delimiters.scan(path, index + 1, next_value)?;
        }

        logical_lines.push(LogicalLine { line, indent_spaces, source: value });
        index += 1;
    }

    Ok(logical_lines)
}

fn strip_line_comment(source: &str) -> &str {
    let mut escaped = false;
    let mut in_string = false;
    let bytes = source.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped { escaped = false; }
            else if byte == b'\\' { escaped = true; }
            else if byte == b'"' { in_string = false; }
            index += 1;
            continue;
        }
        if byte == b'"' { in_string = true; index += 1; }
        else if byte == b'/' && bytes.get(index + 1) == Some(&b'/') { return source[..index].trim_end(); }
        else { index += 1; }
    }
    source
}

impl DelimiterState {
    fn is_open(&self) -> bool { self.brace_depth > 0 || self.bracket_depth > 0 }

    fn scan(&mut self, path: &Path, line: usize, source: &str) -> DoweResult<()> {
        let mut string_delimiter = None;
        let mut escaped = false;
        for (column, value) in source.char_indices() {
            if let Some(delimiter) = string_delimiter {
                if escaped { escaped = false; }
                else if value == '\\' { escaped = true; }
                else if value == delimiter { string_delimiter = None; }
                continue;
            }
            match value {
                '"' => string_delimiter = Some(value),
                '{' => self.brace_depth += 1,
                '}' if self.brace_depth == 0 => return Err(DoweError::at_path(path, format!("{line}:{}: unexpected `}}`", column + 1))),
                '}' => self.brace_depth -= 1,
                '[' => self.bracket_depth += 1,
                ']' if self.bracket_depth == 0 => return Err(DoweError::at_path(path, format!("{line}:{}: unexpected `]`", column + 1))),
                ']' => self.bracket_depth -= 1,
                _ => {}
            }
        }
        if string_delimiter.is_some() {
            return Err(DoweError::at_path(path, format!("{line}:1: strings cannot continue across lines")));
        }
        Ok(())
    }
}

fn leading_indent(path: &Path, line: usize, source: &str) -> DoweResult<usize> {
    let mut count = 0usize;
    for value in source.chars() {
        match value {
            ' ' => count += 1,
            '\t' => return Err(DoweError::at_path(path, format!("{line}:1: tabs are not valid indentation in Dowe Source Format"))),
            _ => break,
        }
    }
    if count % 2 != 0 {
        return Err(DoweError::at_path(path, format!("{line}:1: indentation must use two spaces per level")));
    }
    Ok(count)
}
