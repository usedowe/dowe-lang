mod icon_artifacts;

include!("pipeline/compile_and_artifacts.rs");

#[cfg(test)]
mod tests {
    include!("pipeline/tests_config.rs");
    include!("pipeline/tests_web_output.rs");
    include!("pipeline/tests_native_output.rs");
    include!("pipeline/tests_security.rs");
    include!("pipeline/tests_generated_sync.rs");
    include!("pipeline/tests_icon_output.rs");
}
