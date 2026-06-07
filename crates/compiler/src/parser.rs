mod project;
mod source_ast;
mod source_config;
mod source_discovery;
mod source_i18n;
mod source_imports;
mod source_parser;
mod source_server;
mod source_store;
mod source_types;
mod source_values;
mod source_views;

pub use project::parse_project;
pub use source_views::validate_design_copilot_dowe;

pub(crate) use source_ast::{
    SourceFile, SourceImport, SourceNode, SourceObjectEntry, SourceProp, SourceValue,
};
pub(crate) use source_config::{parse_config_file, parse_project_config};
pub(crate) use source_i18n::{parse_translation_catalog, validate_translation_source};
pub(crate) use source_imports::resolve_import;
pub(crate) use source_parser::parse_source_file;
pub(crate) use source_server::{parse_server_source, validate_server_module_source};
pub(crate) use source_types::{TypeRegistry, reference_fields_for_type, type_from_source_value};
pub(crate) use source_views::parse_views_file;
pub(crate) use source_views::validate_view_source;

#[cfg(test)]
mod tests;
