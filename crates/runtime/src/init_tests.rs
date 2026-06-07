use super::{
    InitProjectOptions, ProjectTemplate, TemplateFile, available_project_examples,
    available_project_templates, init_project, write_project_files,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn project_templates_use_canonical_order() {
    let names = available_project_templates()
        .iter()
        .map(|template| template.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        ["blank", "clinic-desk", "commerce-ops", "support-console"]
    );
}

#[test]
fn project_examples_exclude_blank() {
    let names = available_project_examples()
        .iter()
        .map(|template| template.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, ["clinic-desk", "commerce-ops", "support-console"]);
}

#[test]
fn init_blank_project_writes_source_files() {
    let temp = TempDir::new().expect("tempdir");
    let report =
        init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank)).expect("init");

    assert_eq!(report.template(), ProjectTemplate::Blank);
    assert_eq!(report.created().len(), 7);
    assert_eq!(
        fs::read_to_string(temp.path().join(".gitignore")).expect("gitignore"),
        ".dowe\n"
    );
    assert!(temp.path().join("src/config.dowe").is_file());
    assert!(temp.path().join("src/main.dowe").is_file());
    assert!(temp.path().join("src/views.dowe").is_file());
    assert!(temp.path().join("src/layouts/app.dowe").is_file());
    assert!(temp.path().join("src/pages/home.dowe").is_file());
    assert!(temp.path().join("src/handlers/hello.dowe").is_file());
    assert!(!temp.path().join(".dowe").exists());

    dowe_compiler::compile_dev(temp.path()).expect("compile blank project");
}

#[test]
fn init_example_project_writes_embedded_sources() {
    let temp = TempDir::new().expect("tempdir");
    let report = init_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::ClinicDesk),
    )
    .expect("init");
    let main = fs::read_to_string(temp.path().join("src/main.dowe")).expect("main");

    assert_eq!(report.template(), ProjectTemplate::ClinicDesk);
    assert!(main.contains("clinic-desk ready"));
    assert!(temp.path().join("src/pages/dashboard.dowe").is_file());
    assert!(temp.path().join("src/handlers/appointments.dowe").is_file());
    assert_eq!(
        fs::read_to_string(temp.path().join(".gitignore")).expect("gitignore"),
        ".dowe\n"
    );

    dowe_compiler::compile_dev(temp.path()).expect("compile example project");
}

#[test]
fn init_rejects_conflicts_without_partial_writes() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join(".gitignore"), "user").expect("gitignore");

    let error = init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank))
        .expect_err("error");

    assert!(error.to_string().contains(".gitignore"));
    assert!(!temp.path().join("src/main.dowe").exists());
}

#[test]
fn init_rejects_unsafe_template_paths() {
    let temp = TempDir::new().expect("tempdir");
    let files = [TemplateFile::new("../outside.dowe", "bad")];
    let error =
        write_project_files(temp.path(), ProjectTemplate::Blank, &files).expect_err("error");

    assert!(error.to_string().contains("unsafe init template path"));
    assert!(!temp.path().join("../outside.dowe").exists());
}
