mod project;
mod source_ast;
mod source_config;
mod source_db;
mod source_environment;
mod source_i18n;
mod source_imports;
mod source_kv;
#[cfg(test)]
mod source_kv_tests;
mod source_parser;
mod source_queue;
#[cfg(test)]
mod source_queue_tests;
mod source_server;
mod source_stdlib;
mod source_types;
mod source_values;
mod source_vector;
#[cfg(test)]
mod source_vector_tests;
mod source_views;

pub use project::inspect_project_capabilities;
pub use source_views::validate_design_copilot_dowe;

pub(crate) use project::parse_project_for;
pub(crate) use source_ast::{SourceFile, SourceNode, SourceObjectEntry, SourceProp, SourceValue};
pub(crate) use source_config::{parse_config_file, parse_theme_file};
pub(crate) use source_environment::parse_environment_files;
pub(crate) use source_i18n::{parse_translation_catalog, validate_translation_source};
pub(crate) use source_imports::resolve_import;
pub(crate) use source_parser::parse_source_file;
pub(crate) use source_queue::queue_publish_result_type;
pub(crate) use source_server::{parse_server_source, validate_server_module_source};
pub(crate) use source_types::{
    TypeRegistry, reference_fields_for_type, type_from_source_value, validate_shared_type_source,
};
pub(crate) use source_views::{ViewModuleCache, parse_views_file};
pub(crate) use source_views::{validate_view_source, validate_view_store_source};

#[cfg(test)]
mod tests;
