pub fn highlight_code(language: CodeLanguage, source: &str) -> Vec<CodeToken> {
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < source.len() {
        let rest = &source[index..];
        if rest.starts_with("//") && language != CodeLanguage::Dowe {
            let end = rest.find('\n').unwrap_or(rest.len());
            push_code_token(&mut tokens, CodeTokenKind::Comment, &rest[..end]);
            index += end;
            continue;
        }
        let current = rest.chars().next().expect("code character");
        if is_code_quote(language, current) {
            let end = quoted_code_end(rest, current);
            push_code_token(&mut tokens, CodeTokenKind::String, &rest[..end]);
            index += end;
            continue;
        }
        if current.is_ascii_digit() {
            let end = code_number_end(rest);
            push_code_token(&mut tokens, CodeTokenKind::Number, &rest[..end]);
            index += end;
            continue;
        }
        if is_code_identifier_start(current) {
            let end = code_identifier_end(rest);
            let value = &rest[..end];
            let kind = code_identifier_kind(language, value, &rest[end..]);
            push_code_token(&mut tokens, kind, value);
            index += end;
            continue;
        }
        let kind = if current.is_whitespace() {
            CodeTokenKind::Plain
        } else {
            CodeTokenKind::Punctuation
        };
        push_code_token(&mut tokens, kind, &rest[..current.len_utf8()]);
        index += current.len_utf8();
    }

    tokens
}

fn push_code_token(tokens: &mut Vec<CodeToken>, kind: CodeTokenKind, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = tokens.last_mut()
        && last.kind == kind
    {
        last.text.push_str(text);
        return;
    }
    tokens.push(CodeToken {
        kind,
        text: text.to_string(),
    });
}

fn is_code_quote(language: CodeLanguage, value: char) -> bool {
    match language {
        CodeLanguage::Dowe => value == '"',
        CodeLanguage::TypeScript => matches!(value, '"' | '\'' | '`'),
        CodeLanguage::Go | CodeLanguage::Rust => matches!(value, '"' | '\'' | '`'),
    }
}

fn quoted_code_end(source: &str, delimiter: char) -> usize {
    let mut escaped = false;
    for (index, value) in source.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if value == '\\' {
            escaped = true;
        } else if value == delimiter {
            return index + value.len_utf8();
        }
    }
    source.len()
}

fn code_number_end(source: &str) -> usize {
    source
        .char_indices()
        .find_map(|(index, value)| {
            (!value.is_ascii_digit() && !matches!(value, '.' | '_')).then_some(index)
        })
        .unwrap_or(source.len())
}

fn is_code_identifier_start(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_'
}

fn code_identifier_end(source: &str) -> usize {
    source
        .char_indices()
        .find_map(|(index, value)| {
            (!value.is_ascii_alphanumeric() && value != '_').then_some(index)
        })
        .unwrap_or(source.len())
}

fn code_identifier_kind(language: CodeLanguage, value: &str, suffix: &str) -> CodeTokenKind {
    if suffix.starts_with(':') {
        return CodeTokenKind::Attribute;
    }
    if code_keywords(language).contains(&value) {
        return CodeTokenKind::Keyword;
    }
    if code_types(language).contains(&value) {
        return CodeTokenKind::Type;
    }
    if matches!(value, "true" | "false" | "null" | "nil") {
        return CodeTokenKind::Number;
    }
    CodeTokenKind::Plain
}

fn code_keywords(language: CodeLanguage) -> &'static [&'static str] {
    match language {
        CodeLanguage::Dowe => &[
            "action", "assign", "children", "column", "component", "config", "each", "else", "env",
            "handler", "if", "import", "layout", "main", "middleware", "page", "request",
            "reset", "return", "route", "server", "signal", "type", "views",
        ],
        CodeLanguage::TypeScript => &[
            "as", "async", "await", "class", "const", "else", "export", "extends", "from",
            "function", "if", "implements", "import", "interface", "let", "new", "return",
            "type", "var",
        ],
        CodeLanguage::Go => &[
            "break", "case", "chan", "const", "continue", "default", "defer", "else", "fallthrough",
            "for", "func", "go", "goto", "if", "import", "interface", "map", "package", "range",
            "return", "select", "struct", "switch", "type", "var",
        ],
        CodeLanguage::Rust => &[
            "as", "async", "await", "break", "const", "continue", "crate", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "trait",
            "true", "type", "unsafe", "use", "where", "while",
        ],
    }
}

fn code_types(language: CodeLanguage) -> &'static [&'static str] {
    match language {
        CodeLanguage::Dowe => &[
            "Alert", "AppBar", "BottomBar", "Box", "Button", "Candlestick", "Card", "Code",
            "Divider", "Drawer", "Flex", "Footer", "Grid", "Input", "Option", "Path", "Select",
            "NavMenu", "Scaffold", "SideNav", "Sidebar", "Svg", "Table", "Tabs", "Text",
            "Title", "Video",
        ],
        CodeLanguage::TypeScript => &[
            "any", "boolean", "never", "number", "object", "string", "unknown", "void",
        ],
        CodeLanguage::Go => &[
            "bool", "byte", "complex128", "complex64", "error", "float32", "float64", "int",
            "int16", "int32", "int64", "int8", "rune", "string", "uint", "uint16", "uint32",
            "uint64", "uint8", "uintptr",
        ],
        CodeLanguage::Rust => &[
            "bool", "char", "f32", "f64", "i128", "i16", "i32", "i64", "i8", "isize", "str",
            "u128", "u16", "u32", "u64", "u8", "usize",
        ],
    }
}
