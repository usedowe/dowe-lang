use std::process::Command;

fn dowe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dowe"))
}

#[test]
fn rejects_uninstall_arguments_before_resolving_files() {
    let output = dowe()
        .args(["uninstall", "--unexpected"])
        .output()
        .expect("dowe uninstall");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("Usage: dowe"));
}
