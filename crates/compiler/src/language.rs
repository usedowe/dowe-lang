mod analysis;
mod code_actions;
mod completion;
mod documentation;
mod formatting;
mod model;
mod navigation;
mod symbols;

pub use analysis::{analyze_document, find_workspace_root};
pub use code_actions::code_actions_at;
pub use completion::complete_document;
pub use formatting::format_document;
pub use model::{
    LanguageCodeAction, LanguageCompletion, LanguageCompletionKind, LanguageDiagnostic,
    LanguageDiagnosticSeverity, LanguageDocument, LanguageDocumentSymbol, LanguageLocation,
    LanguagePosition, LanguageRange, LanguageSymbolKind, LanguageTextEdit,
};
pub use navigation::{definition_at, hover_at};
pub use symbols::document_symbols;

#[cfg(test)]
mod tests;
