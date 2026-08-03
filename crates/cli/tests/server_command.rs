use std::process::Command;

fn dowe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dowe"))
}

#[test]
fn routes_server_arguments_to_the_production_subcommand() {
    let output = dowe().arg("server").output().expect("dowe server");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("Usage: dowe server (--root <path>|--artifact <path>)"));
}
