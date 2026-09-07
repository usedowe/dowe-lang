use serde_json::Value;
use std::path::Path;

#[derive(Default, Clone)]
pub struct Redactor {
    secrets: Vec<String>,
}

impl Redactor {
    pub fn for_project(root: &Path) -> Self {
        let mut redactor = Self::default();
        for name in [".env", ".env.live", ".env.stage", ".env.uat"] {
            let path = root.join(name);
            if std::fs::symlink_metadata(&path).is_ok_and(|m| m.is_file() && m.len() <= 1048576)
                && let Ok(text) = std::fs::read_to_string(path)
            {
                for line in text.lines() {
                    if let Some((key, value)) = line.split_once('=') {
                        let value = value.trim().trim_matches(['\'', '"']);
                        if sensitive_key(key.trim()) {
                            redactor.add(value);
                        }
                        if let Ok(url) = reqwest::Url::parse(value)
                            && let Some(password) = url.password()
                        {
                            redactor.add(value);
                            redactor.add(password);
                        }
                    }
                }
            }
        }
        redactor
    }

    pub fn add(&mut self, secret: &str) {
        if !secret.is_empty() && !self.secrets.iter().any(|value| value == secret) {
            self.secrets.push(secret.into());
            self.secrets
                .sort_by_key(|value| std::cmp::Reverse(value.len()));
        }
    }

    pub fn text(&self, text: &str) -> String {
        if text.starts_with("data:image/") && text.contains(";base64,") {
            return text.into();
        }
        if text.contains("PRIVATE KEY-----") {
            return "[REDACTED PRIVATE KEY]".into();
        }
        let mut text = text.to_string();
        for secret in &self.secrets {
            text = replace_secret(&text, secret);
        }
        if let Ok(mut value @ (Value::Object(_) | Value::Array(_))) =
            serde_json::from_str::<Value>(&text)
        {
            let before = value.clone();
            self.value(&mut value);
            if value != before {
                return value.to_string();
            }
        }
        text.split_inclusive('\n')
            .map(|line| {
                if let Some((name, value)) = line.split_once('=')
                    && sensitive_key(name.trim())
                    && !placeholder(value)
                {
                    return format!(
                        "{name}=[REDACTED]{}",
                        if line.ends_with('\n') { "\n" } else { "" }
                    );
                }
                line.to_string()
            })
            .collect::<String>()
    }

    pub(crate) fn stream_safe(&self, text: &str) -> bool {
        !text.contains(['{', '['])
            && !sensitive_key(text)
            && !text.contains("PRIVATE KEY")
            && !self
                .secrets
                .iter()
                .any(|secret| secret.contains(['\n', '\r']))
    }

    pub fn value(&self, value: &mut Value) {
        match value {
            Value::String(text) => *text = self.text(text),
            Value::Array(values) => values.iter_mut().for_each(|value| self.value(value)),
            Value::Object(values) => {
                for (key, value) in values {
                    if value.as_str().is_some_and(|text| !placeholder(text)) && sensitive_key(key) {
                        *value = Value::String("[REDACTED]".into());
                    } else {
                        self.value(value);
                    }
                }
            }
            _ => {}
        }
    }
}

fn placeholder(value: &str) -> bool {
    let value = value.trim().trim_matches(['\'', '"']);
    value.is_empty()
        || value == "..."
        || value == "[REDACTED]"
        || (value.starts_with('<') && value.ends_with('>'))
        || (value.starts_with("${") && value.ends_with('}'))
}

fn sensitive_key(key: &str) -> bool {
    let key = key.to_lowercase();
    key == "token"
        || key == "pwd"
        || key.ends_with("_pass")
        || key.ends_with("_token")
        || [
            "password",
            "passwd",
            "passphrase",
            "secret",
            "api_key",
            "apikey",
            "private_key",
            "access_token",
            "refresh_token",
            "authorization",
        ]
        .iter()
        .any(|word| key.contains(word))
}

fn replace_secret(text: &str, secret: &str) -> String {
    if secret.len() >= 8 {
        return text.replace(secret, "[REDACTED]");
    }
    let mut output = String::new();
    let mut start = 0;
    for (index, _) in text.match_indices(secret) {
        let end = index + secret.len();
        let word = |ch: char| ch.is_alphanumeric() || ch == '_';
        if text[..index].chars().next_back().is_some_and(word)
            || text[end..].chars().next().is_some_and(word)
        {
            continue;
        }
        output.push_str(&text[start..index]);
        output.push_str("[REDACTED]");
        start = end;
    }
    output.push_str(&text[start..]);
    output
}

pub(crate) fn bounded_value(value: Value, max: usize) -> Value {
    let encoded = value.to_string();
    if encoded.len() <= max {
        return value;
    }
    let mut projection = serde_json::json!({"content":"","truncated":true,"instruction":"Request a smaller page or focused output"});
    let mut used = projection.to_string().len();
    let mut content = String::new();
    for character in encoded.chars() {
        let size = serde_json::to_string(&character.to_string()).unwrap().len() - 2;
        if used + size > max {
            break;
        }
        content.push(character);
        used += size;
    }
    projection["content"] = Value::String(content);
    projection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_envelopes_obey_the_budget_even_with_nested_json_escaping() {
        let value = serde_json::json!({"text":"\\\"\n\\".repeat(5000)});
        let bounded = bounded_value(value, 1024);
        assert!(bounded.to_string().len() <= 1024);
        assert_eq!(bounded["truncated"], true);
    }

    #[test]
    fn public_env_values_and_short_secrets_do_not_corrupt_session_ids_or_paths() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join(".env"),
            "APP_NAME=dowe\nPORT=3000\nPASSWORD=1\n",
        )
        .unwrap();
        let redactor = Redactor::for_project(root.path());
        assert_eq!(redactor.text("main.dowe"), "main.dowe");
        assert_eq!(redactor.text("a1bc3000def"), "a1bc3000def");
        assert_eq!(redactor.text("1"), "[REDACTED]");
        assert_eq!(redactor.text("PASSWORD=1\n"), "PASSWORD=[REDACTED]\n");
    }

    #[test]
    fn private_keys_json_credentials_and_url_passwords_are_redacted() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join(".env"),
            "DATABASE_URL=postgres://user:url-password@localhost/db\n",
        )
        .unwrap();
        let redactor = Redactor::for_project(root.path());
        assert!(!redactor.text("url-password").contains("url-password"));
        assert!(
            !redactor
                .text("{\"apiKey\":\"json-secret\"}")
                .contains("json-secret")
        );
        assert!(
            !redactor
                .text("-----BEGIN PRIVATE KEY-----\nprivate\n-----END PRIVATE KEY-----")
                .contains("private\n")
        );
        assert_eq!(
            redactor.text("data:image/png;base64,aGVsbG8="),
            "data:image/png;base64,aGVsbG8="
        );
    }
}
