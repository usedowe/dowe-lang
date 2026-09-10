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

