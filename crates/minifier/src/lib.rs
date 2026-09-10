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

include!("javascript_scanner.rs");
include!("javascript_syntax_helpers.rs");
include!("minifier_tests.rs");
