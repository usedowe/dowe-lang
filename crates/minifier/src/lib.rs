pub fn minify_js(input: &str) -> String {
    minify(input, true)
}

pub fn minify_css(input: &str) -> String {
    minify(input, false)
}

fn minify(input: &str, keep_template_strings: bool) -> String {
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

        if current == '"' || current == '\'' || keep_template_strings && current == '`' {
            trim_space_before_punctuation(&mut output, current);
            output.push(current);
            string_delimiter = Some(current);
            continue;
        }

        if current == '/' {
            match chars.peek().copied() {
                Some('/') if keep_template_strings => {
                    chars.next();
                    for next in chars.by_ref() {
                        if next == '\n' || next == '\r' {
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
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
                _ => {}
            }
        }

        if current.is_whitespace() {
            let previous = output.chars().last();
            let next = chars.peek().copied();
            if previous.is_some_and(is_identifier_char) && next.is_some_and(is_identifier_char) {
                output.push(' ');
            }
            continue;
        }

        trim_space_before_punctuation(&mut output, current);
        output.push(current);
    }

    output.trim().to_string()
}

fn trim_space_before_punctuation(output: &mut String, current: char) {
    if !is_identifier_char(current) && output.ends_with(' ') {
        output.pop();
    }
}

fn is_identifier_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_' || value == '$' || value == '-'
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
    fn minifies_css() {
        let input = r#"
            .box {
                color: red;
                padding: 8px;
            }
        "#;

        assert_eq!(minify_css(input), ".box{color:red;padding:8px;}");
    }
}
