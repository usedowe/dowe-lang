use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn dowe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dowe"))
}

#[test]
fn runs_tests_from_any_project_directory_and_emits_json() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("verification/release")).expect("test directory");
    fs::write(
        temp.path().join("verification/release/metadata.dowe"),
        "test \"metadata\"\n  assert equal actual:\"dowe\" expected:\"dowe\"\n",
    )
    .expect("test file");

    let output = dowe()
        .args(["test", "verification", "--json"])
        .current_dir(temp.path())
        .output()
        .expect("test command");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json report");

    assert!(output.status.success());
    assert_eq!(report["discovered"], 1);
    assert_eq!(report["passed"], 1);
    assert_eq!(
        report["cases"][0]["path"],
        "verification/release/metadata.dowe"
    );
}

#[test]
fn returns_failure_status_for_a_failing_assertion() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(
        temp.path().join("checks.dowe"),
        "test \"mismatch\"\n  assert false value:true\n",
    )
    .expect("test file");

    let output = dowe()
        .args(["test"])
        .current_dir(temp.path())
        .output()
        .expect("test command");
    let stdout = String::from_utf8(output.stdout).expect("stdout");

    assert!(!output.status.success());
    assert!(stdout.contains("FAIL checks.dowe:1 mismatch"));
    assert!(stdout.contains("0 passed; 1 failed; 1 discovered"));
}

#[test]
fn non_interactive_init_rejects_an_existing_dowe_project() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "existing main").expect("main");

    let output = dowe()
        .args(["init", "--template", "blank"])
        .current_dir(temp.path())
        .output()
        .expect("init command");
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert!(!output.status.success());
    assert!(stderr.contains("main.dowe"));
    assert!(stderr.contains("interactive"));
    assert_eq!(
        fs::read_to_string(temp.path().join("main.dowe")).expect("main"),
        "existing main"
    );
    assert!(!temp.path().join("theme.dowe").exists());
}
