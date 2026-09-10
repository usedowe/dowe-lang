fn regex_can_follow(token: &Token) -> bool {
    if token.kind == TokenKind::Word {
        return matches!(
            token.text.as_str(),
            "await"
                | "case"
                | "delete"
                | "do"
                | "else"
                | "in"
                | "instanceof"
                | "new"
                | "of"
                | "return"
                | "throw"
                | "typeof"
                | "void"
                | "yield"
        );
    }
    token.kind == TokenKind::Punctuator
        && matches!(
            token.text.as_str(),
            "(" | "["
                | "{"
                | ","
                | ";"
                | ":"
                | "?"
                | "="
                | "=="
                | "==="
                | "!="
                | "!=="
                | "!"
                | "&&"
                | "||"
                | "??"
                | "+"
                | "-"
                | "*"
                | "%"
                | "&"
                | "|"
                | "^"
                | "~"
                | "<"
                | ">"
                | "<="
                | ">="
                | "=>"
        )
}

fn requires_line_break(previous: &Token, current: &Token, had_newline: bool) -> bool {
    if !had_newline {
        return false;
    }
    previous.kind == TokenKind::Word
        && matches!(
            previous.text.as_str(),
            "break" | "continue" | "return" | "throw" | "yield"
        )
        || current.kind == TokenKind::Punctuator && matches!(current.text.as_str(), "++" | "--")
        || previous.kind == TokenKind::Punctuator && matches!(previous.text.as_str(), "++" | "--")
}

fn requires_space(previous: &Token, current: &Token) -> bool {
    let previous_last = previous.text.chars().last().unwrap_or('\0');
    let current_first = current.text.chars().next().unwrap_or('\0');
    if is_word_continue(previous_last) && is_word_continue(current_first) {
        return true;
    }
    if previous.kind == TokenKind::Number && current_first == '.' {
        return true;
    }
    if previous_last == '+' && current_first == '+'
        || previous_last == '-' && current_first == '-'
        || previous_last == '/' && matches!(current_first, '/' | '*')
    {
        return true;
    }
    if previous.kind == TokenKind::Punctuator && current.kind == TokenKind::Punctuator {
        let combined = format!("{}{}", previous.text, current.text);
        return matches!(
            combined.as_str(),
            "==" | "==="
                | "=>"
                | "!="
                | "!=="
                | "<="
                | ">="
                | "++"
                | "--"
                | "<<"
                | ">>"
                | ">>>"
                | "**"
                | "&&"
                | "||"
                | "??"
                | "?."
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "&="
                | "|="
                | "^="
                | "..."
        );
    }
    false
}

fn trim_space_before_punctuation(output: &mut String, current: char) {
    if matches!(current, '{' | '}' | ';' | ',' | ')' | ']' | '=') && output.ends_with(' ') {
        output.pop();
    }
}

fn css_can_remove_space(previous: char, current: char) -> bool {
    matches!(
        previous,
        '{' | '}' | ':' | ';' | ',' | '(' | '[' | '>' | '~' | '='
    ) || matches!(current, '{' | '}' | ';' | ',' | ')' | ']' | '>' | '~' | '=')
}

fn is_word_start(value: char) -> bool {
    value.is_alphabetic() || matches!(value, '_' | '$' | '\\')
}

fn is_word_continue(value: char) -> bool {
    value.is_alphanumeric() || matches!(value, '_' | '$' | '\\')
}

