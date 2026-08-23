mod android;
mod helpers;
mod ios;
mod macos;

use crate::error::{DeployError, DeployResult};
use crate::files::reset_dir;
use crate::model::{BuildOptions, BuildReport, BuildTarget};
use dowe_compiler::{CompileEnvironment, CompiledProject, compile_for_environment};
use std::path::{Path, PathBuf};

const BUILD_WORKER_STACK_SIZE: usize = 64 * 1024 * 1024;

struct NativePlan {
    commands: Vec<NativeCommand>,
    requirements: Vec<Requirement>,
    copies: Vec<(PathBuf, PathBuf)>,
    artifact: PathBuf,
}

struct NativeCommand {
    program: String,
    args: Vec<String>,
    report_args: Vec<String>,
    cwd: Option<PathBuf>,
    stdout: Option<PathBuf>,
    env: Vec<(String, String)>,
}

impl NativeCommand {
    fn report(&self) -> Vec<String> {
        std::iter::once(self.program.clone())
            .chain(self.report_args.iter().cloned())
            .collect()
    }
}

enum Requirement {
    File(PathBuf, String),
}

pub fn build(options: BuildOptions) -> DeployResult<BuildReport> {
    std::thread::Builder::new()
        .name("dowe-build".to_string())
        .stack_size(BUILD_WORKER_STACK_SIZE)
        .spawn(move || build_on_current_thread(options))
        .map_err(|error| DeployError::new(format!("failed to start Dowe build worker: {error}")))?
        .join()
        .map_err(|_| DeployError::new("Dowe build worker panicked"))?
}

fn build_on_current_thread(options: BuildOptions) -> DeployResult<BuildReport> {
    let root = options.root.canonicalize()?;
    helpers::validate_host(options.target)?;
    let project = compile_for_environment(&root, CompileEnvironment::Live)?;
    if !project.capabilities.views {
        return Err(DeployError::new(format!(
            "build target `{}` requires `views` in main.dowe",
            options.target
        )));
    }
    let output = root.join(".dowe/dist/build").join(options.target.as_str());
    build_target(&project, options.target, &output, options.dry_run, false)
}

pub(crate) fn build_store(
    project: &CompiledProject,
    target: BuildTarget,
    output: &Path,
    dry_run: bool,
) -> DeployResult<BuildReport> {
    helpers::validate_host(target)?;
    build_target(project, target, output, dry_run, true)
}

fn build_target(
    project: &CompiledProject,
    target: BuildTarget,
    output: &Path,
    dry_run: bool,
    store_release: bool,
) -> DeployResult<BuildReport> {
    reset_dir(output)?;
    if matches!(target, BuildTarget::Windows | BuildTarget::Linux) {
        let report = crate::desktop_runtime::build(project, target, output, dry_run)?;
        helpers::write_build_manifest(project, target, output, &report.artifact)?;
        return Ok(report);
    }
    let plan = match target {
        BuildTarget::Android => android::plan(project, output, false, dry_run)?,
        BuildTarget::Ios => ios::plan(project, output, dry_run, store_release)?,
        BuildTarget::Macos => macos::plan(project, output)?,
        BuildTarget::Windows | BuildTarget::Linux => unreachable!(),
    };
    execute_plan(project, target, output, plan, dry_run)
}

pub(crate) fn android_store_bundle(
    project: &CompiledProject,
    output: &Path,
    dry_run: bool,
) -> DeployResult<BuildReport> {
    reset_dir(output)?;
    let plan = android::plan(project, output, true, dry_run)?;
    execute_plan(project, BuildTarget::Android, output, plan, dry_run)
}

fn execute_plan(
    project: &CompiledProject,
    target: BuildTarget,
    output: &Path,
    plan: NativePlan,
    dry_run: bool,
) -> DeployResult<BuildReport> {
    helpers::write_build_manifest(project, target, output, &plan.artifact)?;
    if !dry_run {
        helpers::validate_requirements(&plan)?;
        for command in &plan.commands {
            helpers::run(command)?;
        }
        helpers::finalize_artifact(&plan)?;
        if !plan.artifact.is_file() {
            return Err(DeployError::new(format!(
                "{} build completed without producing {}",
                target,
                plan.artifact.display()
            )));
        }
    }
    Ok(BuildReport {
        target,
        output_dir: output.to_path_buf(),
        artifact: plan.artifact,
        commands: plan.commands.iter().map(NativeCommand::report).collect(),
        built: !dry_run,
    })
}

#[cfg(test)]
mod tests {
    use super::{android, helpers, ios, macos};
    use crate::model::BuildTarget;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn native_artifact_names_are_path_safe() {
        assert_eq!(helpers::safe_name("Clinic Desk"), "ClinicDesk");
        assert_eq!(helpers::safe_name("***"), "DoweApp");
    }

    #[test]
    fn copies_project_assets_into_ios_release_bundle() {
        let temp = TempDir::new().expect("temporary directory");
        let source = temp.path().join("ios");
        let app = temp.path().join("DoweApp.app");
        fs::create_dir_all(source.join("assets/img")).expect("assets");
        fs::create_dir_all(&app).expect("app");
        fs::write(source.join("assets/img/feature.webp"), "image").expect("image");

        helpers::copy_ios_resources(&source, &app).expect("resources");

        assert_eq!(
            fs::read(app.join("assets/img/feature.webp")).expect("bundle image"),
            b"image"
        );
    }

    #[test]
    fn apple_targets_follow_host_availability() {
        let result = helpers::validate_host(BuildTarget::Ios);
        assert_eq!(result.is_ok(), cfg!(target_os = "macos"));
    }

    #[test]
    fn desktop_targets_follow_host_availability() {
        assert_eq!(
            helpers::validate_host(BuildTarget::Windows).is_ok(),
            cfg!(target_os = "windows")
        );
        assert_eq!(
            helpers::validate_host(BuildTarget::Linux).is_ok(),
            cfg!(target_os = "linux")
        );
    }

    #[test]
    fn macos_dmg_icon_command_uses_native_appkit_without_exposing_source() {
        let command = macos::macos_dmg_icon_command(
            std::path::Path::new("/project/icon.icns"),
            std::path::Path::new("/project/App.dmg"),
        );

        assert_eq!(command.program, "xcrun");
        assert_eq!(command.args[0], "swift");
        assert!(command.args[2].contains("NSWorkspace.shared.setIcon"));
        assert_eq!(command.report_args[1], "<dowe-dmg-icon>");
        assert!(!command.report_args.join(" ").contains("NSWorkspace"));
    }

    #[test]
    fn automatic_android_signing_persists_and_reuses_private_credentials() {
        let temp = TempDir::new().expect("tempdir");
        let first = android::automatic_android_signing(temp.path(), false).expect("first signing");
        let command = first.keytool.as_ref().expect("keytool").report().join(" ");

        assert!(
            first
                .keystore
                .ends_with(".dowe/credentials/android/release.p12")
        );
        assert!(command.contains("DOWE_ANDROID_KEYSTORE_PASSWORD"));
        assert!(!command.contains(&first.store_password));
        assert!(
            temp.path()
                .join(".dowe/credentials/android/signing.json")
                .is_file()
        );

        fs::write(&first.keystore, b"keystore").expect("keystore");
        let second =
            android::automatic_android_signing(temp.path(), false).expect("second signing");

        assert_eq!(second.alias, first.alias);
        assert_eq!(second.store_password, first.store_password);
        assert_eq!(second.key_password, first.key_password);
        assert!(second.keytool.is_none());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(temp.path().join(".dowe/credentials/android/signing.json"))
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
            let keystore_mode = fs::metadata(&second.keystore)
                .expect("keystore metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(keystore_mode, 0o600);
        }
    }

    #[test]
    fn partial_android_signing_override_is_rejected() {
        let result =
            android::explicit_android_signing([Some("release.p12".into()), None, None, None], true);
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("partial override must fail"),
        };

        assert!(error.to_string().contains("require DOWE_ANDROID_KEYSTORE"));
    }

    #[test]
    fn ios_signing_override_is_atomic_and_sanitized() {
        let error = ios::explicit_ios_signing(Some("identity".into()), None)
            .expect_err("partial iOS override");
        assert!(error.to_string().contains("require both DOWE_IOS"));

        let signing = ios::explicit_ios_signing(
            Some("identity".into()),
            Some("/profiles/app.mobileprovision".into()),
        )
        .expect("override")
        .expect("signing");
        assert_eq!(signing.identity, "identity");
        assert_eq!(signing.report_identity, "$DOWE_IOS_SIGNING_IDENTITY");
        assert_eq!(signing.report_profile, "$DOWE_IOS_PROVISIONING_PROFILE");
    }

    #[test]
    fn ios_profiles_match_exact_and_wildcard_bundle_identifiers() {
        assert!(ios::profile_bundle_is_exact(
            "TEAM123.dev.dowe.blogs",
            "dev.dowe.blogs"
        ));
        assert!(ios::profile_bundle_match("TEAM123.*", "dev.dowe.blogs"));
        assert!(!ios::profile_bundle_is_exact("TEAM123.*", "dev.dowe.blogs"));
        assert!(!ios::profile_bundle_match(
            "TEAM123.dev.other.app",
            "dev.dowe.blogs"
        ));
    }
}
