mod curated_examples {
    use super::compile_dev;
    use crate::model::HttpMethod;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn compiles_curated_example_projects() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("repo root");
        let examples_root = repo_root.join("examples");

        for project_name in ["clinic-desk", "commerce-ops", "support-console"] {
            let temp = TempDir::new().expect("tempdir");
            copy_dir(
                &examples_root.join(project_name).join("src"),
                &temp.path().join("src"),
            );

            let project = compile_dev(temp.path())
                .unwrap_or_else(|error| panic!("{project_name} failed to compile: {error}"));

            assert!(
                project
                    .backend
                    .find_endpoint(&HttpMethod::Get, "/api/status")
                    .is_some()
            );
            assert!(!project.web.pages.is_empty());
        }
    }

    fn copy_dir(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).expect("destination");
        for entry in fs::read_dir(source).expect("source") {
            let entry = entry.expect("entry");
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            if source_path.is_dir() {
                copy_dir(&source_path, &destination_path);
            } else {
                fs::copy(&source_path, &destination_path).expect("copy");
            }
        }
    }
}
