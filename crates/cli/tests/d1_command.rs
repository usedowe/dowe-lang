use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn dowe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dowe"))
}

#[test]
fn generates_icon_catalog_migrations_from_the_d1_namespace() {
    let project = TempDir::new().expect("tempdir");
    fs::write(project.path().join("main.dowe"), "main\n").expect("main");
    let output = dowe()
        .args([
            "d1",
            "migrations",
            "icon-catalog",
            "--output",
            "server/migrations",
        ])
        .current_dir(project.path())
        .output()
        .expect("dowe d1 migrations icon-catalog");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let migrations = project.path().join("server/migrations");
    assert_eq!(fs::read_dir(&migrations).expect("migrations").count(), 39);
    assert!(
        fs::read_to_string(migrations.join("00001_icon_catalog.sql"))
            .expect("schema")
            .contains("CREATE TABLE IF NOT EXISTS icons")
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("39 D1 migrations with 7476 Solar icon variants")
    );
}
