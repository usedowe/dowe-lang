fn parse_agent_args(
    args: &[String],
    require_prompt: bool,
) -> Result<ParsedAgentChatArgs, Box<dyn std::error::Error>> {
    let mut options = AgentPrepareOptions::default();
    let mut server_url = default_llm_server_url().to_string();
    let mut uses_legacy_server = false;
    let mut json_output = false;
    let mut api_key = None;
    let mut provider = None;
    let mut prompt = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--image" => {
                let value = required_value(args, index, "--image")?;
                options.image_paths.push(PathBuf::from(value));
                index += 2;
            }
            "--request-type" => {
                let value = required_value(args, index, "--request-type")?;
                options.request_type = Some(
                    AgentRequestType::parse(value)
                        .ok_or_else(|| format!("invalid request type `{value}`"))?,
                );
                index += 2;
            }
            "--provider" => {
                let value = required_value(args, index, "--provider")?;
                if !provider_exists(value) {
                    return Err(format!("unknown agent provider `{value}`").into());
                }
                provider = Some(value.to_string());
                index += 2;
            }
            "--model" => {
                options.model = Some(required_value(args, index, "--model")?.to_string());
                index += 2;
            }
            "--api-key" => {
                api_key = Some(required_value(args, index, "--api-key")?.to_string());
                index += 2;
            }
            "--server" => {
                server_url = required_value(args, index, "--server")?.to_string();
                uses_legacy_server = true;
                index += 2;
            }
            "--json" => {
                json_output = true;
                index += 1;
            }
            "--stream" => {
                options.stream = true;
                index += 1;
            }
            value if value.starts_with("--") => return Err(USAGE.into()),
            value => {
                prompt.push(value.to_string());
                index += 1;
            }
        }
    }

    let mut prompt = prompt.join(" ");
    let root = env::current_dir()?;
    prompt = attach_prompt_references(&prompt, &root, &mut options);
    if require_prompt && prompt.trim().is_empty() {
        return Err(USAGE.into());
    }

    options.provider = provider.clone();
    if !uses_legacy_server && options.request_type.is_none() {
        options.request_type = Some(AgentRequestType::Conversation);
    }
    let explicit_model = options.model.is_some();
    Ok(ParsedAgentChatArgs {
        explicit_model,
        prompt,
        provider,
        api_key,
        server_url,
        uses_legacy_server,
        json_output,
        options,
    })
}

const MAX_LOCAL_TEXT_BYTES: u64 = 1_048_576;
const MAX_LOCAL_IMAGE_BYTES: u64 = 20 * 1_048_576;

fn attach_prompt_references(
    prompt: &str,
    root: &Path,
    options: &mut AgentPrepareOptions,
) -> String {
    let mut output = String::with_capacity(prompt.len());
    let mut contexts: Vec<(PathBuf, String)> = Vec::new();
    let mut i = 0;
    while i < prompt.len() {
        let rest = &prompt[i..];
        let boundary = i == 0
            || prompt[..i]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace);
        let (end, raw, explicit) = if boundary && rest.starts_with('@') {
            if rest.as_bytes().get(1) == Some(&b'"') {
                let Some(close) = rest[2..].find('"') else {
                    output.push('@');
                    i += 1;
                    continue;
                };
                (i + 3 + close, &rest[2..2 + close], true)
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                (i + end, &rest[1..end], true)
            }
        } else if boundary && (rest.starts_with('"') || rest.starts_with('\'')) {
            let quote = rest.as_bytes()[0] as char;
            let Some(close) = rest[1..].find(quote) else {
                output.push(rest.chars().next().unwrap());
                i += 1;
                continue;
            };
            (i + 2 + close, &rest[1..1 + close], false)
        } else {
            let ch = rest.chars().next().unwrap();
            output.push(ch);
            i += ch.len_utf8();
            continue;
        };
        let candidate = PathBuf::from(raw);
        let image = supported_image(&candidate);
        let text = supported_text(&candidate);
        let absolute = candidate.is_absolute();
        let shape_ok = explicit || (absolute && image);
        let path_like = raw.contains('/') || raw.contains('\\') || candidate.extension().is_some();
        if shape_ok && (image || (explicit && text)) && (absolute || path_like) {
            let path = if absolute {
                candidate
            } else {
                root.join(candidate)
            };
            if !absolute {
                if let (Ok(canonical), Ok(root)) = (fs::canonicalize(&path), fs::canonicalize(root))
                {
                    if !canonical.starts_with(root) {
                        output.push_str(&prompt[i..end]);
                        i = end;
                        continue;
                    }
                }
            }
            if let Some(path) = safe_attachment_path(&path, image) {
                if image {
                    if !options.image_paths.iter().any(|existing| existing == &path) {
                        options.image_paths.push(path);
                    }
                } else if let Some(content) = read_local_text(&path)
                    && !contexts.iter().any(|(existing, _)| existing == &path)
                {
                    contexts.push((path, content));
                }
                i = end;
                continue;
            }
        }
        output.push_str(&prompt[i..end]);
        i = end;
    }
    for (path, content) in contexts {
        output.push_str(&format!(
            "\n\n[Untrusted local file context: {}]\n```\n{}\n```",
            path.display(),
            content
        ));
    }
    output
}

fn supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "gif")
    )
}

fn supported_text(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(
            "md" | "txt"
                | "rs"
                | "dowe"
                | "toml"
                | "json"
                | "yaml"
                | "yml"
                | "js"
                | "jsx"
                | "ts"
                | "tsx"
                | "css"
                | "html"
                | "py"
                | "go"
                | "java"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "sql"
                | "sh"
                | "bash"
        )
    )
}

fn safe_attachment_path(path: &Path, image: bool) -> Option<PathBuf> {
    let metadata = fs::symlink_metadata(path).ok()?;
    let max = if image {
        MAX_LOCAL_IMAGE_BYTES
    } else {
        MAX_LOCAL_TEXT_BYTES
    };
    if !metadata.file_type().is_file() || metadata.len() > max {
        return None;
    }
    let canonical = fs::canonicalize(path).ok()?;
    if !image && is_credential_file(path) {
        return None;
    }
    Some(canonical)
}

fn is_credential_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    name == ".env"
        || ["credential", "secret", "token", "private", "id_rsa"]
            .iter()
            .any(|word| name.contains(word))
        || matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("pem" | "key")
        )
}

fn read_local_text(path: &Path) -> Option<String> {
    String::from_utf8(fs::read(path).ok()?).ok()
}

fn required_value<'a>(
    args: &'a [String],
    index: usize,
    name: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    args.get(index + 1)
        .map(String::as_str)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("{name} requires a value").into())
}

