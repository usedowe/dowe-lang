use super::Redactor;

pub struct RedactedTextStream {
    redactor: Redactor,
    pending: String,
    remaining: usize,
    withheld: bool,
}

impl RedactedTextStream {
    pub fn new(redactor: Redactor, limit: usize) -> Self {
        Self {
            redactor,
            pending: String::new(),
            remaining: limit,
            withheld: false,
        }
    }

    pub fn push(&mut self, text: &str) -> Option<String> {
        if self.withheld {
            return None;
        }
        if text.len() > self.remaining {
            self.withheld = true;
            self.pending.clear();
            return None;
        }
        self.remaining -= text.len();
        self.pending.push_str(text);
        let end = self.pending.rfind('\n')? + 1;
        let text = self.pending.drain(..end).collect::<String>();
        if !self.redactor.stream_safe(&text) {
            self.withheld = true;
            self.pending.clear();
            return None;
        }
        Some(self.redactor.text(&text))
    }

    pub fn withheld(&self) -> bool {
        self.withheld
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secret_fragments_and_ambiguous_structures_are_not_previews() {
        let mut redactor = Redactor::default();
        redactor.add("hidden-value");
        let mut stream = RedactedTextStream::new(redactor, 1024);
        assert!(stream.push("hidden-").is_none());
        assert_eq!(stream.push("value\n"), Some("[REDACTED]\n".into()));
        assert!(stream.push("{\n").is_none());
        assert!(stream.push("\"api_key\":\"secret\"\n").is_none());
        assert!(stream.withheld());
    }
}
