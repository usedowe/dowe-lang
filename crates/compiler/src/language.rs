mod analysis;
mod completion;
mod formatting;
mod model;
mod navigation;
mod symbols;

pub use analysis::{analyze_document, find_workspace_root};
pub use completion::complete_document;
pub use formatting::format_document;
pub use model::{
    LanguageCompletion, LanguageCompletionKind, LanguageDiagnostic, LanguageDiagnosticSeverity,
    LanguageDocument, LanguageDocumentSymbol, LanguageLocation, LanguagePosition, LanguageRange,
    LanguageSymbolKind,
};
pub use navigation::{definition_at, hover_at};
pub use symbols::document_symbols;

#[cfg(test)]
mod tests;
