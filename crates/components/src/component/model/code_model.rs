#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeToken {
    pub kind: CodeTokenKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeTokenKind {
    Plain,
    Keyword,
    Type,
    String,
    Number,
    Attribute,
    Comment,
    Punctuation,
}

impl CodeTokenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Keyword => "keyword",
            Self::Type => "type",
            Self::String => "string",
            Self::Number => "number",
            Self::Attribute => "attribute",
            Self::Comment => "comment",
            Self::Punctuation => "punctuation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeLanguage {
    Dowe,
    TypeScript,
    JavaScript,
    Go,
    Rust,
    Python,
}

impl CodeLanguage {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "dowe" => Some(Self::Dowe),
            "typescript" => Some(Self::TypeScript),
            "javascript" => Some(Self::JavaScript),
            "go" => Some(Self::Go),
            "rust" => Some(Self::Rust),
            "python" => Some(Self::Python),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dowe => "dowe",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Go => "go",
            Self::Rust => "rust",
            Self::Python => "python",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Dowe,
            Self::TypeScript,
            Self::JavaScript,
            Self::Go,
            Self::Rust,
            Self::Python,
        ]
    }
}

