use crate::error::{DoweError, DoweResult};
use crate::language::model::{
    LanguageDiagnostic, LanguageDiagnosticSeverity, LanguageDocument, LanguageRange,
};
use crate::model::{DesignConfig, DoweType, DoweTypeField, EnvironmentConfig};
use crate::parser::{
    SourceFile, SourceNode, SourceObjectEntry, SourceValue, parse_config_file,
    parse_environment_files, parse_server_source, parse_source_file, parse_theme_file,
    parse_translation_catalog, parse_views_file, queue_publish_result_type,
    reference_fields_for_type, resolve_import, type_from_source_value,
    validate_server_module_source, validate_shared_type_source, validate_translation_source,
    validate_view_source, validate_view_store_source,
};
use crate::test_runner::validate_test_file;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};


include!("analysis_document_and_imports.rs");
include!("analysis_surfaces_and_validation.rs");
include!("analysis_store_fields.rs");
include!("analysis_signal_fields.rs");
