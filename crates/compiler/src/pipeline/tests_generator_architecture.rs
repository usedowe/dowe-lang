#[test]
fn generators_do_not_depend_on_parser_or_source_nodes() {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for generator in ["generator_web", "generator_android", "generator_ios"] {
        let root = crates.join(generator);
        let cargo = std::fs::read_to_string(root.join("Cargo.toml")).expect("generator manifest");
        assert!(!cargo.contains("dowe_compiler"), "{generator} depends on compiler");
        assert!(!cargo.contains("dowe_parser"), "{generator} depends on parser");
        let mut sources = Vec::new();
        fn collect(path: &std::path::Path, sources: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(path) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect(&path, sources);
                } else if path.extension().is_some_and(|extension| extension == "rs") {
                    sources.push(std::fs::read_to_string(path).expect("generator source"));
                }
            }
        }
        collect(&root.join("src"), &mut sources);
        let source = sources.join("\n");
        assert!(!source.contains("SourceNode"), "{generator} reparses source nodes");
        assert!(!source.contains("parse_source_file"), "{generator} parses source files");
    }
}
