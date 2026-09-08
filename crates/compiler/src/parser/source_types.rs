use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::SourceNode;

mod registry;
mod resolution;
mod values;

pub(crate) use registry::{TypeRegistry, is_shared_type_path, validate_shared_type_source};
pub(crate) use values::{
    reference_fields_for_type, type_from_source_value, type_from_store_literal,
    validate_reference_path, validate_source_value_type,
};

fn validate_identifier(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(node_error(node, format!("{label} name must not be empty")));
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
    {
        return Err(node_error(
            node,
            format!("{label} `{value}` must be an ASCII identifier"),
        ));
    }
    Ok(())
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{TypeRegistry, validate_shared_type_source};
    use crate::parser::source_parser::parse_source_file;
    use std::fs;
    use tempfile::TempDir;

    include!("source_types/tests.rs");
}
