include!("catalog_definitions.rs");
include!("catalog_validation.rs");
include!("catalog_selection.rs");
include!("catalog_paths.rs");
#[cfg(test)]
mod tests {
    include!("catalog_tests.rs");
}
