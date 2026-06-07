use crate::model::{FileMetrics, FunctionFingerprint};

pub(crate) fn metrics_for(content: &str) -> FileMetrics {
    let mut metrics = FileMetrics {
        total_lines: content.lines().count(),
        ..FileMetrics::default()
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            metrics.code_lines += 1;
        }
        if starts_function(trimmed) {
            metrics.functions += 1;
        }
        if starts_type(trimmed) {
            metrics.types += 1;
        }
        if trimmed.starts_with("pub ") {
            metrics.public_items += 1;
        }
        if trimmed == "#[test]" || trimmed.ends_with("::test]") {
            metrics.inline_tests += 1;
        }
        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            metrics.imports += 1;
        }
    }

    metrics.responsibilities = responsibilities_for(content);
    metrics
}

pub(crate) fn function_fingerprints(path: &str, content: &str) -> Vec<FunctionFingerprint> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut fingerprints = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        if !starts_function(lines[index].trim()) {
            index += 1;
            continue;
        }

        let name = function_name(lines[index]).unwrap_or_else(|| "function".to_string());
        let start = index;
        let mut end = index;
        let mut balance = brace_delta(lines[index]);

        while end + 1 < lines.len() && balance > 0 {
            end += 1;
            balance += brace_delta(lines[end]);
        }

        if end > start {
            let body = lines[start..=end].join("\n");
            let normalized = normalize_code(&body);
            if body.lines().count() >= 3 && normalized.len() > 24 {
                fingerprints.push(FunctionFingerprint {
                    path: path.to_string(),
                    name,
                    start_line: start + 1,
                    end_line: end + 1,
                    normalized,
                });
            }
        }

        index = end + 1;
    }

    fingerprints
}

pub(crate) fn fingerprint_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;

    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}

fn starts_function(trimmed: &str) -> bool {
    trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub(crate) fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("pub async fn ")
}

fn starts_type(trimmed: &str) -> bool {
    trimmed.starts_with("struct ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("pub type ")
}

fn responsibilities_for(content: &str) -> usize {
    [
        "parse",
        "validat",
        "generat",
        "render",
        "write",
        "read",
        "route",
        "spawn",
        "graph",
        "manifest",
        "diagnostic",
    ]
    .into_iter()
    .filter(|token| content.contains(token))
    .count()
}

fn function_name(line: &str) -> Option<String> {
    let marker = line.find("fn ")?;
    let after = &line[marker + 3..];
    let name = after
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    if name.is_empty() { None } else { Some(name) }
}

fn brace_delta(line: &str) -> i32 {
    let opens = line.chars().filter(|character| *character == '{').count() as i32;
    let closes = line.chars().filter(|character| *character == '}').count() as i32;
    opens - closes
}

fn normalize_code(content: &str) -> String {
    let mut normalized = String::new();
    let mut chars = content.chars().peekable();

    while let Some(character) = chars.next() {
        if character.is_ascii_alphabetic() || character == '_' {
            while let Some(next) = chars.peek() {
                if next.is_ascii_alphanumeric() || *next == '_' {
                    chars.next();
                } else {
                    break;
                }
            }
            normalized.push_str("id");
        } else if character.is_ascii_digit() {
            while let Some(next) = chars.peek() {
                if next.is_ascii_digit() {
                    chars.next();
                } else {
                    break;
                }
            }
            normalized.push_str("num");
        } else if !character.is_whitespace() {
            normalized.push(character);
        }
    }

    normalized
}
