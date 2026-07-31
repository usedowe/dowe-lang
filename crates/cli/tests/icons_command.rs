use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn dowe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dowe"))
}

#[test]
fn generates_selected_icons_without_an_interactive_terminal() {
    let project = fixture();
    let output = dowe()
        .args([
            "icons",
            "--source",
            "assets/icon.svg",
            "--background",
            "#112233",
            "--rounded",
            "sm",
            "--target",
            "web",
        ])
        .current_dir(project.path())
        .output()
        .expect("dowe icons");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(project.path().join("icons/web/favicon.ico").is_file());
    assert!(!project.path().join("icons/desktop").exists());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Generated 7 web icon files"));
}

#[test]
fn requires_explicit_rendering_options_without_a_terminal() {
    let project = fixture();
    let output = dowe()
        .args(["icons", "--source", "assets/icon.svg"])
        .current_dir(project.path())
        .output()
        .expect("dowe icons");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("requires --source, --background and --rounded")
    );
    assert!(!project.path().join("icons").exists());
}

#[test]
fn rejects_database_catalog_generation_from_the_icons_command() {
    let project = fixture();
    let output = dowe()
        .args(["icons", "catalog-d1", "--output", "server/migrations"])
        .current_dir(project.path())
        .output()
        .expect("dowe icons catalog-d1");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage: dowe"));
    assert!(!project.path().join("server/migrations").exists());
}

fn fixture() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("assets")).expect("assets");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(
        temp.path().join("assets/icon.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><circle cx="50" cy="50" r="45" fill="#ffffff"/></svg>"##,
    )
    .expect("svg");
    temp
}
