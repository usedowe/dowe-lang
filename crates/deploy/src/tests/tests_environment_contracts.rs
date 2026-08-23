use super::*;

#[test]
fn non_live_deploy_rejects_live_only_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    write_environment(
        temp.path(),
        DeployEnvironment::Stage,
        "https://stage-password.example",
    );
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Static);
    options.environment = DeployEnvironment::Stage;

    let error = deploy(options).expect_err("live-only target");

    assert!(
        error
            .to_string()
            .contains("only available in the live environment")
    );
}
