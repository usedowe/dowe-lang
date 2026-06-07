use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageDocument {
    pub path: PathBuf,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: LanguageDiagnosticSeverity,
    pub range: LanguageRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LanguageDiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanguageRange {
    pub start: LanguagePosition,
    pub end: LanguagePosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanguagePosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageCompletion {
    pub label: String,
    pub kind: LanguageCompletionKind,
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LanguageCompletionKind {
    Keyword,
    Component,
    Property,
    Value,
    Function,
    Variable,
    File,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageLocation {
    pub path: PathBuf,
    pub range: LanguageRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageDocumentSymbol {
    pub name: String,
    pub kind: LanguageSymbolKind,
    pub range: LanguageRange,
    pub selection_range: LanguageRange,
    pub children: Vec<LanguageDocumentSymbol>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LanguageSymbolKind {
    File,
    Module,
    Class,
    Function,
    Method,
    Property,
    Variable,
}

impl LanguageRange {
    pub fn single_line(line: usize, column: usize, length: usize) -> Self {
        Self {
            start: LanguagePosition { line, column },
            end: LanguagePosition {
                line,
                column: column + length.max(1),
            },
        }
    }
}
