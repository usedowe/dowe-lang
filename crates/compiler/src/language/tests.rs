use super::{
    LanguageCompletionKind, LanguageDiagnosticSeverity, LanguageDocument, LanguageRange,
    code_actions_at, complete_document, definition_at, document_symbols, format_document, hover_at,
};
use crate::language::analyze_document;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

include!("tests_foundations_and_formatting.rs");
include!("tests_imports_and_diagnostics.rs");
include!("tests_view_validation.rs");
include!("tests_server_and_hover.rs");
include!("tests_completion_context.rs");
include!("tests_component_completions.rs");
include!("tests_value_completions.rs");
include!("tests_display_completions.rs");
include!("tests_component_values.rs");
include!("tests_documentation.rs");
include!("tests_navigation_and_actions.rs");
include!("tests_validation_tail.rs");
