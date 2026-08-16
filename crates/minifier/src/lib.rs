#[derive(Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Word,
    Number,
    String,
    Template,
    Regex,
    Punctuator,
}

struct Token {
    kind: TokenKind,
    text: String,
    separated: bool,
    separated_by_newline: bool,
}

pub fn minify_js(input: &str) -> String {
    let mut scanner = JavascriptScanner::new(input);
    let mut output = String::with_capacity(input.len());
    let mut previous: Option<Token> = None;

    while let Some(token) = scanner.next(previous.as_ref()) {
        if let Some(previous) = previous.as_ref() {
            if token.separated {
                if requires_line_break(previous, &token, token.separated_by_newline) {
                    output.push('\n');
                } else if requires_space(previous, &token) {
                    output.push(' ');
                }
            }
        }
        output.push_str(&token.text);
        previous = Some(token);
    }

    output
}

pub fn minify_css(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut string_delimiter = None;
    let mut escaped = false;

    while let Some(current) = chars.next() {
        if let Some(delimiter) = string_delimiter {
            output.push(current);
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == delimiter {
                string_delimiter = None;
            }
            continue;
        }

        if current == '"' || current == '\'' {
            trim_space_before_punctuation(&mut output, current);
            output.push(current);
            string_delimiter = Some(current);
            continue;
        }

        if current == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut previous = '\0';
            for next in chars.by_ref() {
                if previous == '*' && next == '/' {
                    break;
                }
                previous = next;
            }
            continue;
        }

        if current.is_whitespace() {
            let previous = output.chars().last();
            let next = chars.peek().copied();
            if previous
                .zip(next)
                .is_some_and(|(previous, next)| !css_can_remove_space(previous, next))
                && !output.ends_with(' ')
            {
                output.push(' ');
            }
            continue;
        }

        trim_space_before_punctuation(&mut output, current);
        output.push(current);
    }

    output.trim().to_string()
}

struct JavascriptScanner {
    chars: Vec<char>,
    index: usize,
}

impl JavascriptScanner {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            index: 0,
        }
    }

    fn next(&mut self, previous: Option<&Token>) -> Option<Token> {
        let (separated, separated_by_newline) = self.skip_separators();
        let current = *self.chars.get(self.index)?;
        let can_start_regex = previous.is_none_or(regex_can_follow);
        let (kind, text) = if current == '"' || current == '\'' {
            (TokenKind::String, self.quoted(current))
        } else if current == '`' {
            (TokenKind::Template, self.template())
        } else if current == '/' && can_start_regex {
            (TokenKind::Regex, self.regex())
        } else if is_word_start(current) {
            (TokenKind::Word, self.word())
        } else if current.is_ascii_digit()
            || current == '.'
                && self
                    .chars
                    .get(self.index + 1)
                    .is_some_and(|value| value.is_ascii_digit())
        {
            (TokenKind::Number, self.number())
        } else {
            (TokenKind::Punctuator, self.punctuator())
        };

        Some(Token {
            kind,
            text,
            separated,
            separated_by_newline,
        })
    }

    fn skip_separators(&mut self) -> (bool, bool) {
        let mut separated = false;
        let mut newline = false;

        loop {
            while let Some(current) = self.chars.get(self.index).copied() {
                if !current.is_whitespace() {
                    break;
                }
                separated = true;
                newline |= current == '\n' || current == '\r';
                self.index += 1;
            }

            let first = self.chars.get(self.index).copied();
            let second = self.chars.get(self.index + 1).copied();
            if first == Some('/') && second == Some('/') {
                separated = true;
                self.index += 2;
                while let Some(current) = self.chars.get(self.index).copied() {
                    self.index += 1;
                    if current == '\n' || current == '\r' {
                        newline = true;
                        break;
                    }
                }
                continue;
            }
            if first == Some('/') && second == Some('*') {
                separated = true;
                self.index += 2;
                while self.index < self.chars.len() {
                    let current = self.chars[self.index];
                    newline |= current == '\n' || current == '\r';
                    if current == '*' && self.chars.get(self.index + 1) == Some(&'/') {
                        self.index += 2;
                        break;
                    }
                    self.index += 1;
                }
                continue;
            }
            return (separated, newline);
        }
    }

    fn quoted(&mut self, delimiter: char) -> String {
        let start = self.index;
        self.index += 1;
        let mut escaped = false;
        while let Some(current) = self.chars.get(self.index).copied() {
            self.index += 1;
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == delimiter {
                break;
            }
        }
        self.chars[start..self.index].iter().collect()
    }

    fn template(&mut self) -> String {
        let mut output = String::from("`");
        self.index += 1;
        let mut escaped = false;
        while let Some(current) = self.chars.get(self.index).copied() {
            if escaped {
                output.push(current);
                escaped = false;
                self.index += 1;
                continue;
            }
            if current == '\\' {
                output.push(current);
                escaped = true;
                self.index += 1;
                continue;
            }
            if current == '`' {
                output.push(current);
                self.index += 1;
                break;
            }
            if current == '$' && self.chars.get(self.index + 1) == Some(&'{') {
                output.push_str("${");
                self.index += 2;
                output.push_str(&minify_js(&self.template_expression()));
                output.push('}');
                continue;
            }
            output.push(current);
            self.index += 1;
        }
        output
    }

    fn template_expression(&mut self) -> String {
        let remaining = self.chars[self.index..].iter().collect::<String>();
        let mut scanner = JavascriptScanner::new(&remaining);
        let mut depth = 1usize;
        let mut previous = None;
        while let Some(token) = scanner.next(previous.as_ref()) {
            if token.kind == TokenKind::Punctuator && token.text == "{" {
                depth += 1;
            } else if token.kind == TokenKind::Punctuator && token.text == "}" {
                depth -= 1;
                if depth == 0 {
                    let expression = remaining
                        .chars()
                        .take(scanner.index.saturating_sub(1))
                        .collect();
                    self.index += scanner.index;
                    return expression;
                }
            }
            previous = Some(token);
        }
        self.index = self.chars.len();
        remaining
    }

    fn regex(&mut self) -> String {
        let start = self.index;
        self.index += 1;
        let mut escaped = false;
        let mut character_class = false;
        while let Some(current) = self.chars.get(self.index).copied() {
            self.index += 1;
            if escaped {
                escaped = false;
                continue;
            }
            if current == '\\' {
                escaped = true;
            } else if current == '[' {
                character_class = true;
            } else if current == ']' {
                character_class = false;
            } else if current == '/' && !character_class {
                break;
            }
        }
        while self
            .chars
            .get(self.index)
            .is_some_and(|current| is_word_continue(*current))
        {
            self.index += 1;
        }
        self.chars[start..self.index].iter().collect()
    }

    fn word(&mut self) -> String {
        let start = self.index;
        self.index += 1;
        while self
            .chars
            .get(self.index)
            .is_some_and(|current| is_word_continue(*current))
        {
            self.index += 1;
        }
        self.chars[start..self.index].iter().collect()
    }

    fn number(&mut self) -> String {
        let start = self.index;
        self.index += 1;
        while let Some(current) = self.chars.get(self.index).copied() {
            if current.is_ascii_alphanumeric()
                || current == '_'
                || current == '.'
                || matches!(current, '+' | '-')
                    && self
                        .chars
                        .get(self.index.wrapping_sub(1))
                        .is_some_and(|previous| matches!(previous, 'e' | 'E'))
            {
                self.index += 1;
            } else {
                break;
            }
        }
        self.chars[start..self.index].iter().collect()
    }

    fn punctuator(&mut self) -> String {
        const PUNCTUATORS: &[&str] = &[
            ">>>=", "===", "!==", ">>>", "**=", "&&=", "||=", "??=", "<<=", ">>=", "...", "=>",
            "==", "!=", "<=", ">=", "++", "--", "<<", ">>", "**", "&&", "||", "??", "?.", "+=",
            "-=", "*=", "/=", "%=", "&=", "|=", "^=",
        ];
        for punctuator in PUNCTUATORS {
            let values = punctuator.chars().collect::<Vec<_>>();
            if self.chars[self.index..].starts_with(&values) {
                self.index += values.len();
                return (*punctuator).to_string();
            }
        }
        let current = self.chars[self.index];
        self.index += 1;
        current.to_string()
    }
}

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

#[cfg(test)]
mod tests {
    use super::{minify_css, minify_js};

    #[test]
    fn minifies_generated_javascript() {
        let input = r#"
            export const value = "Layout";
            export function render(children = "") {
                return "<div>" + children + "</div>";
            }
        "#;

        assert_eq!(
            minify_js(input),
            r#"export const value="Layout";export function render(children=""){return"<div>"+children+"</div>";}"#
        );
    }

    #[test]
    fn removes_javascript_comments() {
        let input = r#"
            const value = "http://localhost";
            const next = 1;
        "#;

        assert_eq!(
            minify_js(input),
            r#"const value="http://localhost";const next=1;"#
        );
    }

    #[test]
    fn preserves_escaped_slashes_in_regular_expressions() {
        let input = r#"
            const path = value.replace(/^web\//, "");
            const next = 1;
        "#;

        assert_eq!(
            minify_js(input),
            r#"const path=value.replace(/^web\//,"");const next=1;"#
        );
    }

    #[test]
    fn preserves_spaces_between_ambiguous_operators() {
        assert_eq!(minify_js("const x = a + +b;"), "const x=a+ +b;");
        assert_eq!(minify_js("const x = a - -b;"), "const x=a- -b;");
    }

    #[test]
    fn preserves_division_before_a_regular_expression() {
        assert_eq!(
            minify_js("const ok = value / /a/.test(text);"),
            "const ok=value/ /a/.test(text);"
        );
    }

    #[test]
    fn preserves_restricted_line_breaks() {
        assert_eq!(minify_js("return\n{ ok: true };"), "return\n{ok:true};");
        assert_eq!(minify_js("value\n++next;"), "value\n++next;");
    }

    #[test]
    fn minifies_template_interpolations_without_changing_literal_text() {
        assert_eq!(
            minify_js("const value = `Hello  ${ name || \"Dowe\" }`;"),
            "const value=`Hello  ${name||\"Dowe\"}`;"
        );
    }

    #[test]
    fn minifies_css() {
        let input = r#"
            .box {
                color: red;
                padding: 8px;
            }
        "#;

        assert_eq!(minify_css(input), ".box{color:red;padding:8px;}");
    }

    #[test]
    fn preserves_css_math_operator_whitespace() {
        let input = r#"
            .box {
                width: calc(100% - 2rem);
                height: min(50vh + 1rem, 40rem);
            }
        "#;

        assert_eq!(
            minify_css(input),
            ".box{width:calc(100% - 2rem);height:min(50vh + 1rem,40rem);}",
        );
    }

    #[test]
    fn preserves_css_strings_urls_and_custom_property_values() {
        let input = r#"
            :root {
                --dowe-font-stack: "Dowe Sans", system-ui, sans-serif;
                --dowe-offset: calc(100% - 2rem);
            }
            .icon {
                content: "a /* literal */ b";
                background: url("data:image/svg+xml,%3Csvg viewBox='0 0 2 2'%3E%3C/svg%3E");
            }
        "#;

        assert_eq!(
            minify_css(input),
            ":root{--dowe-font-stack:\"Dowe Sans\",system-ui,sans-serif;--dowe-offset:calc(100% - 2rem);}.icon{content:\"a /* literal */ b\";background:url(\"data:image/svg+xml,%3Csvg viewBox='0 0 2 2'%3E%3C/svg%3E\");}",
        );
    }

    #[test]
    fn removes_css_comments_without_joining_identifiers() {
        assert_eq!(
            minify_css(".box { font: 600 /* generated */ 1rem sans-serif; }"),
            ".box{font:600 1rem sans-serif;}",
        );
    }
}
