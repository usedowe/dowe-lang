use super::*;
use crate::deploy_targets_for_surface;

#[test]
fn deploy_surfaces_follow_main_capabilities() {
    let fullstack = TempDir::new().expect("fullstack");
    write_fixture(fullstack.path(), "");
    let mut fullstack_surfaces = vec![
        DeploySurface::Server,
        DeploySurface::Web,
        DeploySurface::Android,
    ];
    if cfg!(target_os = "macos") {
        fullstack_surfaces.push(DeploySurface::Ios);
    }
    assert_eq!(
        available_deploy_surfaces(fullstack.path(), DeployEnvironment::Live).expect("surfaces"),
        fullstack_surfaces
    );
    assert_eq!(
        available_deploy_surfaces(fullstack.path(), DeployEnvironment::Stage).expect("surfaces"),
        [DeploySurface::Server, DeploySurface::Web]
    );

    let views_only = TempDir::new().expect("views only");
    fs::write(
        views_only.path().join("main.dowe"),
        "main\n  views:viewRoutes\n",
    )
    .expect("main");
    let mut views_surfaces = vec![DeploySurface::Web, DeploySurface::Android];
    if cfg!(target_os = "macos") {
        views_surfaces.push(DeploySurface::Ios);
    }
    assert_eq!(
        available_deploy_surfaces(views_only.path(), DeployEnvironment::Live).expect("surfaces"),
        views_surfaces
    );

    let server_only = TempDir::new().expect("server only");
    fs::write(
        server_only.path().join("main.dowe"),
        "main\n  server port:8080\n",
    )
    .expect("main");
    assert_eq!(
        available_deploy_surfaces(server_only.path(), DeployEnvironment::Live).expect("surfaces"),
        [DeploySurface::Server]
    );
}

#[test]
fn ssh_is_a_server_deploy_target() {
    assert_eq!(
        "ssh".parse::<DeployTarget>().expect("target"),
        DeployTarget::Ssh
    );
    assert_eq!(DeployTarget::Ssh.surface(), DeploySurface::Server);
    assert_eq!(
        deploy_targets_for_surface(DeploySurface::Server),
        [
            DeployTarget::Dowe,
            DeployTarget::Docker,
            DeployTarget::Ssh,
            DeployTarget::Cloudflare,
            DeployTarget::Vercel,
        ]
    );
}

#[test]
fn native_build_targets_follow_host_availability() {
    let targets = available_build_targets();

    assert!(targets.contains(&BuildTarget::Android));
    assert_eq!(
        targets.contains(&BuildTarget::Windows),
        cfg!(target_os = "windows")
    );
    assert_eq!(
        targets.contains(&BuildTarget::Linux),
        cfg!(target_os = "linux")
    );
    assert_eq!(
        targets.contains(&BuildTarget::Ios),
        cfg!(target_os = "macos")
    );
    assert_eq!(
        targets.contains(&BuildTarget::Macos),
        cfg!(target_os = "macos")
    );
}

#[test]
fn plans_android_release_apk_without_running_toolchains() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = BuildOptions::new(temp.path(), BuildTarget::Android);
    options.dry_run = true;

    let report = build(options).expect("Android build plan");

    assert!(!report.built);
    assert!(report.artifact.ends_with("DoweDev.apk"));
    assert!(report.commands.iter().any(|command| {
        command.first().map(String::as_str) == Some("java")
            && command
                .iter()
                .any(|argument| argument == "org.gradle.wrapper.GradleWrapperMain")
            && command
                .iter()
                .any(|argument| argument == ":app:assembleRelease")
    }));
    assert!(report.output_dir.join("build.json").is_file());
}

#[test]
fn plans_google_play_publication_without_exposing_access_token() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Android);
    options.publish = true;
    options.dry_run = true;
    options.track = Some("internal".into());

    let report = deploy(options).expect("Android deploy plan");

    assert!(!report.published);
    assert!(report.artifact.expect("artifact").ends_with("DoweDev.aab"));
    let command = report.command.expect("publish command");
    assert!(command.contains(&"$DOWE_GOOGLE_PLAY_ACCESS_TOKEN".to_string()));
    assert!(!command.join(" ").contains("Bearer"));
}

#[test]
fn rejects_unsafe_google_play_track_before_publication() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Android);
    options.publish = true;
    options.dry_run = true;
    options.track = Some("internal/../../production".into());

    let error = deploy(options).expect_err("invalid track");

    assert!(error.to_string().contains("track"));
}

#[test]
fn plans_ios_release_without_exposing_signing_values_on_macos() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = BuildOptions::new(temp.path(), BuildTarget::Ios);
    options.dry_run = true;

    let report = build(options).expect("iOS build plan");

    assert!(report.artifact.ends_with("DoweDev.ipa"));
    let commands = report
        .commands
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    assert!(commands.contains(&"<automatic-ios-signing-identity>".to_string()));
    assert!(commands.contains(&"<automatic-ios-provisioning-profile>".to_string()));
}

#[test]
fn plans_desktop_release_artifacts_without_running_toolchains() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let target = if cfg!(target_os = "windows") {
        Some((BuildTarget::Windows, "DoweDev.exe"))
    } else if cfg!(target_os = "linux") {
        Some((BuildTarget::Linux, "DoweDev"))
    } else {
        None
    };
    if let Some((target, artifact)) = target {
        let mut options = BuildOptions::new(temp.path(), target);
        options.dry_run = true;
        let report = build(options).expect("desktop build plan");

        assert!(report.artifact.ends_with(artifact));
        assert!(report.commands.is_empty());
        assert!(report.output_dir.join("app.dowe-bundle").is_file());
        assert!(fs::read(report.output_dir.join("app.dowe-bundle")).is_ok());
    }

    if cfg!(target_os = "macos") {
        let mut options = BuildOptions::new(temp.path(), BuildTarget::Macos);
        options.dry_run = true;
        let report = build(options).expect("macOS build plan");

        assert!(report.artifact.ends_with("DoweDev.dmg"));
        assert_eq!(report.commands[0][0], "swiftc");
        assert_eq!(report.commands[1][0], "hdiutil");
    }
}
