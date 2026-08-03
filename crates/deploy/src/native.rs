use crate::error::{DeployError, DeployResult};
use crate::files::{copy_file, copy_tree, reset_dir, write_file};
use crate::model::{BuildOptions, BuildReport, BuildTarget};
use base64::Engine;
use dowe_compiler::{CompileEnvironment, CompiledProject, compile_for_environment};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

const BUILD_WORKER_STACK_SIZE: usize = 64 * 1024 * 1024;
const MACOS_DMG_ICON_SWIFT: &str = r#"import AppKit
let arguments = CommandLine.arguments
if arguments.count != 3 { fatalError("Dowe DMG icon requires icon and artifact paths") }
guard let icon = NSImage(contentsOfFile: arguments[1]) else { fatalError("Dowe DMG icon is invalid") }
if !NSWorkspace.shared.setIcon(icon, forFile: arguments[2], options: []) { fatalError("Dowe could not apply the DMG icon") }"#;

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
    validate_host(options.target)?;
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
    validate_host(target)?;
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
        write_build_manifest(project, target, output, &report.artifact)?;
        return Ok(report);
    }
    let plan = match target {
        BuildTarget::Android => android_plan(project, output, false, dry_run)?,
        BuildTarget::Ios => ios_plan(project, output, dry_run, store_release)?,
        BuildTarget::Macos => macos_plan(project, output)?,
        BuildTarget::Windows | BuildTarget::Linux => unreachable!(),
    };
    write_build_manifest(project, target, output, &plan.artifact)?;
    if !dry_run {
        validate_requirements(&plan)?;
        for command in &plan.commands {
            run(command)?;
        }
        finalize_artifact(&plan)?;
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

pub(crate) fn android_store_bundle(
    project: &CompiledProject,
    output: &Path,
    dry_run: bool,
) -> DeployResult<BuildReport> {
    reset_dir(output)?;
    let plan = android_plan(project, output, true, dry_run)?;
    write_build_manifest(project, BuildTarget::Android, output, &plan.artifact)?;
    if !dry_run {
        validate_requirements(&plan)?;
        for command in &plan.commands {
            run(command)?;
        }
        finalize_artifact(&plan)?;
        if !plan.artifact.is_file() {
            return Err(DeployError::new(
                "Android bundle build did not produce an AAB",
            ));
        }
    }
    Ok(BuildReport {
        target: BuildTarget::Android,
        output_dir: output.to_path_buf(),
        artifact: plan.artifact,
        commands: plan.commands.iter().map(NativeCommand::report).collect(),
        built: !dry_run,
    })
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AndroidSigningFile {
    alias: String,
    store_password: String,
    key_password: String,
}

struct AndroidSigning {
    keystore: PathBuf,
    alias: String,
    store_password: String,
    key_password: String,
    keytool: Option<NativeCommand>,
}

fn android_plan(
    project: &CompiledProject,
    output: &Path,
    bundle: bool,
    dry_run: bool,
) -> DeployResult<NativePlan> {
    release_version()?;
    release_build_number()?;
    let root = project.root.join(".dowe/apps/android");
    let task = if bundle {
        ":app:bundleRelease"
    } else {
        ":app:assembleRelease"
    };
    let extension = if bundle { "aab" } else { "apk" };
    let source = root
        .join("app/build/outputs")
        .join(if bundle {
            "bundle/release"
        } else {
            "apk/release"
        })
        .join(format!("app-release.{extension}"));
    let artifact = output.join(format!(
        "{}.{}",
        safe_name(&project.app_config.name),
        extension
    ));
    let signing = android_signing(&project.root, dry_run)?;
    let toolchain = crate::gradle::prepare(&project.root, dry_run)?;
    let mut signing_env = android_signing_env(&signing);
    signing_env.push((
        "GRADLE_USER_HOME".into(),
        toolchain.user_home.to_string_lossy().to_string(),
    ));
    let mut commands = Vec::new();
    if let Some(command) = signing.keytool {
        commands.push(command);
    }
    let mut gradle = native_command(
        "java",
        [
            "-classpath",
            toolchain.wrapper_jar.to_string_lossy().as_ref(),
            "org.gradle.wrapper.GradleWrapperMain",
            task,
            "--console=plain",
        ],
        Some(root.clone()),
    );
    gradle.env = signing_env;
    commands.push(gradle);
    Ok(NativePlan {
        commands,
        requirements: vec![Requirement::File(
            root.join("settings.gradle.kts"),
            "generated Android project".into(),
        )],
        copies: vec![(source, artifact.clone())],
        artifact,
    })
}

fn android_signing(project_root: &Path, dry_run: bool) -> DeployResult<AndroidSigning> {
    let names = [
        "DOWE_ANDROID_KEYSTORE",
        "DOWE_ANDROID_KEY_ALIAS",
        "DOWE_ANDROID_KEYSTORE_PASSWORD",
        "DOWE_ANDROID_KEY_PASSWORD",
    ];
    let values = names.map(|name| env::var(name).ok().filter(|value| !value.trim().is_empty()));
    if let Some(signing) = explicit_android_signing(values, dry_run)? {
        return Ok(signing);
    }

    automatic_android_signing(project_root, dry_run)
}

fn explicit_android_signing(
    values: [Option<String>; 4],
    dry_run: bool,
) -> DeployResult<Option<AndroidSigning>> {
    let count = values.iter().filter(|value| value.is_some()).count();
    if count != 0 && count != values.len() {
        return Err(DeployError::new(
            "Android signing overrides require DOWE_ANDROID_KEYSTORE, DOWE_ANDROID_KEY_ALIAS, DOWE_ANDROID_KEYSTORE_PASSWORD, and DOWE_ANDROID_KEY_PASSWORD together",
        ));
    }
    if count == values.len() {
        let signing = AndroidSigning {
            keystore: PathBuf::from(values[0].clone().expect("Android keystore")),
            alias: values[1].clone().expect("Android alias"),
            store_password: values[2].clone().expect("Android store password"),
            key_password: values[3].clone().expect("Android key password"),
            keytool: None,
        };
        if !dry_run && !signing.keystore.is_file() {
            return Err(DeployError::new(format!(
                "DOWE_ANDROID_KEYSTORE does not exist: {}",
                signing.keystore.display()
            )));
        }
        return Ok(Some(signing));
    }
    Ok(None)
}

fn automatic_android_signing(project_root: &Path, dry_run: bool) -> DeployResult<AndroidSigning> {
    let directory = project_root.join(".dowe/credentials/android");
    let keystore = directory.join("release.p12");
    let credentials = directory.join("signing.json");
    if keystore.is_file() && !credentials.is_file() {
        return Err(DeployError::new(format!(
            "Android keystore exists without its Dowe credential file: {}",
            credentials.display()
        )));
    }
    let signing_file = if credentials.is_file() {
        serde_json::from_slice::<AndroidSigningFile>(&fs::read(&credentials)?)?
    } else {
        AndroidSigningFile {
            alias: "dowe-release".into(),
            store_password: random_android_password(),
            key_password: String::new(),
        }
    };
    let signing_file = AndroidSigningFile {
        key_password: if signing_file.key_password.is_empty() {
            signing_file.store_password.clone()
        } else {
            signing_file.key_password
        },
        ..signing_file
    };
    validate_android_signing_file(&signing_file)?;
    if !credentials.is_file() && !dry_run {
        fs::create_dir_all(&directory)?;
        write_private_file(&credentials, &serde_json::to_vec_pretty(&signing_file)?)?;
    }
    let mut signing = AndroidSigning {
        keystore,
        alias: signing_file.alias,
        store_password: signing_file.store_password,
        key_password: signing_file.key_password,
        keytool: None,
    };
    if !signing.keystore.is_file() {
        signing.keytool = Some(android_keytool_command(&signing));
    } else if !dry_run {
        set_private_permissions(&signing.keystore)?;
    }
    Ok(signing)
}

fn android_keytool_command(signing: &AndroidSigning) -> NativeCommand {
    let mut command = native_command(
        "keytool",
        [
            "-genkeypair",
            "-keystore",
            signing.keystore.to_string_lossy().as_ref(),
            "-storetype",
            "PKCS12",
            "-alias",
            signing.alias.as_str(),
            "-keyalg",
            "RSA",
            "-keysize",
            "2048",
            "-validity",
            "10000",
            "-dname",
            "CN=Dowe Android Release,OU=Dowe,O=Dowe,C=US",
            "-storepass:env",
            "DOWE_ANDROID_KEYSTORE_PASSWORD",
            "-keypass:env",
            "DOWE_ANDROID_KEY_PASSWORD",
        ],
        None,
    );
    command.env = android_signing_env(signing);
    command
}

fn android_signing_env(signing: &AndroidSigning) -> Vec<(String, String)> {
    vec![
        (
            "DOWE_ANDROID_KEYSTORE".into(),
            signing.keystore.to_string_lossy().to_string(),
        ),
        ("DOWE_ANDROID_KEY_ALIAS".into(), signing.alias.clone()),
        (
            "DOWE_ANDROID_KEYSTORE_PASSWORD".into(),
            signing.store_password.clone(),
        ),
        (
            "DOWE_ANDROID_KEY_PASSWORD".into(),
            signing.key_password.clone(),
        ),
    ]
}

fn validate_android_signing_file(signing: &AndroidSigningFile) -> DeployResult<()> {
    if signing.alias.is_empty()
        || signing
            .alias
            .chars()
            .any(|value| !(value.is_ascii_alphanumeric() || value == '-' || value == '_'))
    {
        return Err(DeployError::new(
            "Android signing alias contains invalid characters",
        ));
    }
    if signing.store_password.len() < 16 || signing.key_password.len() < 16 {
        return Err(DeployError::new(
            "Android signing passwords must contain at least 16 characters",
        ));
    }
    Ok(())
}

fn random_android_password() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn write_private_file(path: &Path, content: &[u8]) -> DeployResult<()> {
    write_file(path, content)?;
    set_private_permissions(path)
}

fn set_private_permissions(_path: &Path) -> DeployResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(_path, fs::Permissions::from_mode(0o600))?;
        if let Some(parent) = _path.parent() {
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct IosSigning {
    identity: String,
    profile: PathBuf,
    report_identity: String,
    report_profile: String,
}

struct IosProfileCandidate {
    path: PathBuf,
    identity: String,
    application_identifier: String,
    expiration: String,
    development: bool,
}

fn ios_signing(bundle: &str, dry_run: bool, store_release: bool) -> DeployResult<IosSigning> {
    let identity = env::var("DOWE_IOS_SIGNING_IDENTITY")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let profile = env::var("DOWE_IOS_PROVISIONING_PROFILE")
        .ok()
        .filter(|value| !value.trim().is_empty());
    if let Some(signing) = explicit_ios_signing(identity, profile)? {
        return Ok(signing);
    }
    if dry_run {
        return Ok(IosSigning {
            identity: "<automatic-ios-signing-identity>".to_string(),
            profile: PathBuf::from("<automatic-ios-provisioning-profile>"),
            report_identity: "<automatic-ios-signing-identity>".to_string(),
            report_profile: "<automatic-ios-provisioning-profile>".to_string(),
        });
    }
    automatic_ios_signing(bundle, store_release)
}

fn explicit_ios_signing(
    identity: Option<String>,
    profile: Option<String>,
) -> DeployResult<Option<IosSigning>> {
    match (identity, profile) {
        (None, None) => Ok(None),
        (Some(identity), Some(profile)) => Ok(Some(IosSigning {
            identity,
            profile: PathBuf::from(profile),
            report_identity: "$DOWE_IOS_SIGNING_IDENTITY".to_string(),
            report_profile: "$DOWE_IOS_PROVISIONING_PROFILE".to_string(),
        })),
        _ => Err(DeployError::new(
            "iOS signing overrides require both DOWE_IOS_SIGNING_IDENTITY and DOWE_IOS_PROVISIONING_PROFILE",
        )),
    }
}

fn automatic_ios_signing(bundle: &str, store_release: bool) -> DeployResult<IosSigning> {
    let identities = ios_signing_identities()?;
    let current = command_text("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"])?;
    let mut candidates = Vec::new();
    for path in ios_provisioning_profiles()? {
        if let Some(candidate) = ios_profile_candidate(&path, &identities)
            && candidate.expiration > current
            && profile_bundle_match(&candidate.application_identifier, bundle)
            && (!store_release
                || (!candidate.development
                    && profile_bundle_is_exact(&candidate.application_identifier, bundle)))
        {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| {
        let left_preferred = left.development != store_release;
        let right_preferred = right.development != store_release;
        let left_exact = profile_bundle_is_exact(&left.application_identifier, bundle);
        let right_exact = profile_bundle_is_exact(&right.application_identifier, bundle);
        (right_preferred, right_exact, &right.expiration).cmp(&(
            left_preferred,
            left_exact,
            &left.expiration,
        ))
    });
    let candidate = candidates.into_iter().next().ok_or_else(|| {
        DeployError::new(format!(
            "no compatible Apple-issued iOS signing identity and {} provisioning profile were found for bundle `{bundle}`; configure the Apple account and bundle in Xcode, or set both DOWE_IOS_SIGNING_IDENTITY and DOWE_IOS_PROVISIONING_PROFILE",
            if store_release { "distribution" } else { "development" }
        ))
    })?;
    Ok(IosSigning {
        identity: candidate.identity,
        profile: candidate.path,
        report_identity: "<automatic-ios-signing-identity>".to_string(),
        report_profile: "<automatic-ios-provisioning-profile>".to_string(),
    })
}

fn ios_signing_identities() -> DeployResult<HashSet<String>> {
    let output = Command::new("security")
        .args(["find-identity", "-v", "-p", "codesigning"])
        .output()
        .map_err(|error| {
            DeployError::new(format!("failed to inspect iOS signing identities: {error}"))
        })?;
    if !output.status.success() {
        return Err(DeployError::new(
            "failed to inspect iOS signing identities in the Keychain",
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .filter(|value| {
            value.len() == 40 && value.chars().all(|character| character.is_ascii_hexdigit())
        })
        .map(str::to_ascii_uppercase)
        .collect())
}

fn ios_provisioning_profiles() -> DeployResult<Vec<PathBuf>> {
    let home = env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        DeployError::new("HOME is required to locate Xcode provisioning profiles")
    })?;
    let directories = [
        home.join("Library/Developer/Xcode/UserData/Provisioning Profiles"),
        home.join("Library/MobileDevice/Provisioning Profiles"),
    ];
    let mut profiles = Vec::new();
    for directory in directories {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let path = entry?.path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(OsStr::to_str),
                    Some("mobileprovision" | "provisionprofile")
                )
            {
                profiles.push(path);
            }
        }
    }
    profiles.sort();
    profiles.dedup();
    Ok(profiles)
}

fn ios_profile_candidate(path: &Path, identities: &HashSet<String>) -> Option<IosProfileCandidate> {
    let decoded = Command::new("security")
        .args(["cms", "-D", "-i"])
        .arg(path)
        .output()
        .ok()?;
    if !decoded.status.success() {
        return None;
    }
    let mut plist = NamedTempFile::new().ok()?;
    plist.write_all(&decoded.stdout).ok()?;
    let application_identifier = plist_value(plist.path(), "Entitlements.application-identifier")?;
    let expiration = plist_value(plist.path(), "ExpirationDate")?;
    let development = plist_value(plist.path(), "Entitlements.get-task-allow")
        .map(|value| value == "true")
        .unwrap_or(false);
    let certificate_count = plist_value(plist.path(), "DeveloperCertificates")
        .and_then(|value| value.parse::<usize>().ok())?;
    let identity = (0..certificate_count).find_map(|index| {
        let certificate = plist_value(plist.path(), &format!("DeveloperCertificates.{index}"))?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(certificate.split_whitespace().collect::<String>())
            .ok()?;
        let fingerprint = format!("{:X}", Sha1::digest(bytes));
        identities.contains(&fingerprint).then_some(fingerprint)
    })?;
    Some(IosProfileCandidate {
        path: path.to_path_buf(),
        identity,
        application_identifier,
        expiration,
        development,
    })
}

fn plist_value(path: &Path, key: &str) -> Option<String> {
    let output = Command::new("plutil")
        .args(["-extract", key, "raw", "-o", "-"])
        .arg(path)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}

fn command_text(program: &str, args: &[&str]) -> DeployResult<String> {
    let output = Command::new(program).args(args).output().map_err(|error| {
        DeployError::new(format!(
            "failed to start `{program}` for iOS signing: {error}"
        ))
    })?;
    if !output.status.success() {
        return Err(DeployError::new(format!(
            "`{program}` failed while resolving iOS signing"
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn profile_bundle_match(application_identifier: &str, bundle: &str) -> bool {
    profile_bundle_is_exact(application_identifier, bundle)
        || application_identifier.ends_with(".*")
}

fn profile_bundle_is_exact(application_identifier: &str, bundle: &str) -> bool {
    application_identifier
        .strip_suffix(bundle)
        .is_some_and(|prefix| prefix.ends_with('.') && !prefix[..prefix.len() - 1].contains('.'))
}

fn ios_plan(
    project: &CompiledProject,
    output: &Path,
    dry_run: bool,
    store_release: bool,
) -> DeployResult<NativePlan> {
    let source = project.root.join(".dowe/apps/ios");
    let staging = output.join("staging");
    let app_name = safe_name(&project.app_config.name);
    let app = staging.join("Payload").join(format!("{app_name}.app"));
    let executable = app.join("DoweIosApp");
    let artifact = output.join(format!("{app_name}.ipa"));
    let version = release_version()?;
    let build_number = release_build_number()?;
    let swift_files = swift_sources(&source)?;
    let mut compile_args = vec![
        "--sdk".into(),
        "iphoneos".into(),
        "swiftc".into(),
        "-parse-as-library".into(),
        "-O".into(),
        "-target".into(),
        "arm64-apple-ios17.0".into(),
    ];
    compile_args.extend(
        swift_files
            .iter()
            .map(|path| path.to_string_lossy().to_string()),
    );
    compile_args.extend(["-o".into(), executable.to_string_lossy().to_string()]);
    fs::create_dir_all(&app)?;
    copy_file(&source.join("Info.plist"), &app.join("Info.plist"))?;
    copy_ios_resources(&source, &app)?;
    let signing = ios_signing(&project.app_config.bundle, dry_run, store_release)?;
    let profile = signing.profile;
    let identity = signing.identity;
    let requirements = vec![
        Requirement::File(source.join("Info.plist"), "generated iOS Info.plist".into()),
        Requirement::File(profile.clone(), "iOS provisioning profile".into()),
    ];
    let profile_destination = app.join("embedded.mobileprovision");
    let decoded_profile = staging.join("profile.plist");
    let entitlements = staging.join("entitlements.plist");
    let mut commands = vec![
        native_command(
            "plutil",
            [
                "-replace",
                "CFBundleShortVersionString",
                "-string",
                version.as_str(),
                app.join("Info.plist").to_string_lossy().as_ref(),
            ],
            None,
        ),
        native_command(
            "plutil",
            [
                "-replace",
                "CFBundleVersion",
                "-string",
                build_number.as_str(),
                app.join("Info.plist").to_string_lossy().as_ref(),
            ],
            None,
        ),
        NativeCommand {
            program: "xcrun".into(),
            report_args: compile_args.clone(),
            args: compile_args,
            cwd: Some(source.clone()),
            stdout: None,
            env: Vec::new(),
        },
        NativeCommand {
            program: "cp".into(),
            args: vec![
                profile.to_string_lossy().to_string(),
                profile_destination.to_string_lossy().to_string(),
            ],
            report_args: vec![
                signing.report_profile.clone(),
                profile_destination.to_string_lossy().to_string(),
            ],
            cwd: None,
            stdout: None,
            env: Vec::new(),
        },
        NativeCommand {
            program: "security".into(),
            args: vec![
                "cms".into(),
                "-D".into(),
                "-i".into(),
                profile.to_string_lossy().to_string(),
            ],
            report_args: vec![
                "cms".into(),
                "-D".into(),
                "-i".into(),
                signing.report_profile,
            ],
            cwd: None,
            stdout: Some(decoded_profile.clone()),
            env: Vec::new(),
        },
        NativeCommand {
            program: "/usr/libexec/PlistBuddy".into(),
            args: vec![
                "-x".into(),
                "-c".into(),
                "Print :Entitlements".into(),
                decoded_profile.to_string_lossy().to_string(),
            ],
            report_args: vec![
                "-x".into(),
                "-c".into(),
                "Print :Entitlements".into(),
                decoded_profile.to_string_lossy().to_string(),
            ],
            cwd: None,
            stdout: Some(entitlements.clone()),
            env: Vec::new(),
        },
        NativeCommand {
            program: "codesign".into(),
            args: vec![
                "--force".into(),
                "--sign".into(),
                identity,
                "--timestamp=none".into(),
                "--entitlements".into(),
                entitlements.to_string_lossy().to_string(),
                app.to_string_lossy().to_string(),
            ],
            report_args: vec![
                "--force".into(),
                "--sign".into(),
                signing.report_identity,
                "--timestamp=none".into(),
                "--entitlements".into(),
                entitlements.to_string_lossy().to_string(),
                app.to_string_lossy().to_string(),
            ],
            cwd: None,
            stdout: None,
            env: Vec::new(),
        },
        native_command(
            "ditto",
            [
                "-c",
                "-k",
                "--sequesterRsrc",
                "--keepParent",
                "Payload",
                artifact.to_string_lossy().as_ref(),
            ],
            Some(staging),
        ),
    ];
    if source.join("Assets.xcassets").is_dir() {
        commands.insert(
            3,
            native_command(
                "xcrun",
                [
                    "actool",
                    source.join("Assets.xcassets").to_string_lossy().as_ref(),
                    "--compile",
                    app.to_string_lossy().as_ref(),
                    "--platform",
                    "iphoneos",
                    "--minimum-deployment-target",
                    "17.0",
                    "--app-icon",
                    "AppIcon",
                ],
                None,
            ),
        );
    }
    Ok(NativePlan {
        commands,
        requirements,
        copies: Vec::new(),
        artifact,
    })
}

fn macos_plan(project: &CompiledProject, output: &Path) -> DeployResult<NativePlan> {
    let source = project.root.join(".dowe/apps/desktop/macos");
    let app_name = safe_name(&project.app_config.name);
    let app = output.join(format!("{app_name}.app"));
    let contents = app.join("Contents");
    let executable = contents.join("MacOS").join(&app_name);
    let resources = contents.join("Resources");
    let artifact = output.join(format!("{app_name}.dmg"));
    let version = release_version()?;
    let build_number = release_build_number()?;
    fs::create_dir_all(executable.parent().expect("executable parent"))?;
    fs::create_dir_all(&resources)?;
    copy_tree(
        &project.root.join(".dowe/apps/desktop/web"),
        &resources.join("web"),
    )?;
    let icon = source.join("icon.icns");
    if icon.is_file() {
        copy_file(&icon, &resources.join("AppIcon.icns"))?;
    }
    write_file(
        &contents.join("Info.plist"),
        macos_plist(project, &app_name, icon.is_file(), &version, &build_number),
    )?;
    let mut commands = vec![
        native_command(
            "swiftc",
            [
                "DoweMacOSApp.swift",
                "-O",
                "-o",
                executable.to_string_lossy().as_ref(),
            ],
            Some(source.clone()),
        ),
        native_command(
            "hdiutil",
            [
                "create",
                "-volname",
                project.app_config.name.as_str(),
                "-srcfolder",
                app.to_string_lossy().as_ref(),
                "-ov",
                "-format",
                "UDZO",
                artifact.to_string_lossy().as_ref(),
            ],
            None,
        ),
    ];
    if icon.is_file() {
        commands.push(macos_dmg_icon_command(&icon, &artifact));
    }
    Ok(NativePlan {
        commands,
        requirements: vec![Requirement::File(
            source.join("DoweMacOSApp.swift"),
            "generated macOS host".into(),
        )],
        copies: Vec::new(),
        artifact,
    })
}

fn macos_dmg_icon_command(icon: &Path, artifact: &Path) -> NativeCommand {
    let args = vec![
        "swift".to_string(),
        "-e".to_string(),
        MACOS_DMG_ICON_SWIFT.to_string(),
        icon.to_string_lossy().into_owned(),
        artifact.to_string_lossy().into_owned(),
    ];
    NativeCommand {
        program: "xcrun".to_string(),
        args,
        report_args: vec![
            "swift".to_string(),
            "<dowe-dmg-icon>".to_string(),
            icon.to_string_lossy().into_owned(),
            artifact.to_string_lossy().into_owned(),
        ],
        cwd: None,
        stdout: None,
        env: Vec::new(),
    }
}

fn validate_host(target: BuildTarget) -> DeployResult<()> {
    if target.requires_macos() && !cfg!(target_os = "macos") {
        return Err(DeployError::new(format!(
            "build target `{target}` is only available on macOS"
        )));
    }
    if target == BuildTarget::Windows && !cfg!(target_os = "windows") {
        return Err(DeployError::new(
            "build target `windows` is only available on Windows",
        ));
    }
    if target == BuildTarget::Linux && !cfg!(target_os = "linux") {
        return Err(DeployError::new(
            "build target `linux` is only available on Linux",
        ));
    }
    Ok(())
}

fn validate_requirements(plan: &NativePlan) -> DeployResult<()> {
    for requirement in &plan.requirements {
        let Requirement::File(path, label) = requirement;
        if !path.is_file() {
            return Err(DeployError::new(format!(
                "missing {label}: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn run(command: &NativeCommand) -> DeployResult<()> {
    let mut process = Command::new(&command.program);
    process.args(&command.args);
    if let Some(cwd) = &command.cwd {
        process.current_dir(cwd);
    }
    process.envs(command.env.iter().map(|(name, value)| (name, value)));
    if command
        .args
        .iter()
        .any(|argument| argument == "org.gradle.wrapper.GradleWrapperMain")
    {
        let status = process
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|error| {
                DeployError::new(format!("failed to start `{}`: {error}", command.program))
            })?;
        if !status.success() {
            return Err(DeployError::new(format!(
                "`{}` failed with status {status}",
                command.program
            )));
        }
        return Ok(());
    }
    let output = process.output().map_err(|error| {
        DeployError::new(format!("failed to start `{}`: {error}", command.program))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DeployError::new(format!(
            "`{}` failed{}",
            command.program,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        )));
    }
    if let Some(path) = &command.stdout {
        write_file(path, &output.stdout)?;
    }
    if command.program == "keytool"
        && let Some((_, path)) = command
            .env
            .iter()
            .find(|(name, _)| name == "DOWE_ANDROID_KEYSTORE")
    {
        set_private_permissions(Path::new(path))?;
    }
    Ok(())
}

fn finalize_artifact(plan: &NativePlan) -> DeployResult<()> {
    for (source, destination) in &plan.copies {
        if !source.is_file() {
            return Err(DeployError::new(format!(
                "expected build output is missing: {}",
                source.display()
            )));
        }
        copy_file(source, destination)?;
    }
    Ok(())
}

fn native_command<I, S>(program: &str, args: I, cwd: Option<PathBuf>) -> NativeCommand
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    NativeCommand {
        program: program.into(),
        report_args: args.clone(),
        args,
        cwd,
        stdout: None,
        env: Vec::new(),
    }
}

fn swift_sources(root: &Path) -> DeployResult<Vec<PathBuf>> {
    let mut files = fs::read_dir(root)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(OsStr::to_str) == Some("swift"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn copy_ios_resources(source: &Path, app: &Path) -> DeployResult<()> {
    copy_tree(&source.join("Fonts"), &app.join("Fonts"))?;
    if source.join("AppIcon.png").is_file() {
        copy_file(&source.join("AppIcon.png"), &app.join("AppIcon.png"))?;
    }
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        if path.extension().and_then(OsStr::to_str) == Some("lproj") {
            let name = path
                .file_name()
                .ok_or_else(|| DeployError::new("invalid iOS resource path"))?;
            copy_tree(&path, &app.join(name))?;
        }
    }
    Ok(())
}

fn safe_name(value: &str) -> String {
    let name = value
        .chars()
        .filter(|value| value.is_ascii_alphanumeric() || *value == '-' || *value == '_')
        .collect::<String>();
    if name.is_empty() {
        "DoweApp".into()
    } else {
        name
    }
}

fn write_build_manifest(
    project: &CompiledProject,
    target: BuildTarget,
    output: &Path,
    artifact: &Path,
) -> DeployResult<()> {
    let artifact_name = artifact
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default();
    write_file(
        &output.join("build.json"),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "target": target,
            "app": { "name": project.app_config.name, "bundle": project.app_config.bundle },
            "artifact": artifact_name
        }))?,
    )
}

fn macos_plist(
    project: &CompiledProject,
    executable: &str,
    has_icon: bool,
    version: &str,
    build_number: &str,
) -> String {
    let icon = if has_icon {
        "<key>CFBundleIconFile</key><string>AppIcon</string>"
    } else {
        ""
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"><plist version=\"1.0\"><dict><key>CFBundleDisplayName</key><string>{}</string><key>CFBundleExecutable</key><string>{}</string><key>CFBundleIdentifier</key><string>{}</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>{}</string><key>CFBundleVersion</key><string>{}</string>{icon}</dict></plist>",
        xml(&project.app_config.name),
        xml(executable),
        xml(&project.app_config.bundle),
        xml(version),
        xml(build_number)
    )
}

fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn release_version() -> DeployResult<String> {
    let value = env::var("DOWE_APP_VERSION").unwrap_or_else(|_| "0.1.0".into());
    if value.is_empty()
        || value.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || character == '.' || character == '-')
        })
    {
        return Err(DeployError::new(
            "DOWE_APP_VERSION must contain only letters, numbers, dots, or hyphens",
        ));
    }
    Ok(value)
}

fn release_build_number() -> DeployResult<String> {
    let value = env::var("DOWE_APP_BUILD_NUMBER").unwrap_or_else(|_| "1".into());
    if value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .is_none()
    {
        return Err(DeployError::new(
            "DOWE_APP_BUILD_NUMBER must be a positive integer",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{
        automatic_android_signing, explicit_android_signing, explicit_ios_signing,
        macos_dmg_icon_command, profile_bundle_is_exact, profile_bundle_match, safe_name,
        validate_host,
    };
    use crate::model::BuildTarget;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn native_artifact_names_are_path_safe() {
        assert_eq!(safe_name("Clinic Desk"), "ClinicDesk");
        assert_eq!(safe_name("***"), "DoweApp");
    }

    #[test]
    fn apple_targets_follow_host_availability() {
        let result = validate_host(BuildTarget::Ios);
        assert_eq!(result.is_ok(), cfg!(target_os = "macos"));
    }

    #[test]
    fn desktop_targets_follow_host_availability() {
        assert_eq!(
            validate_host(BuildTarget::Windows).is_ok(),
            cfg!(target_os = "windows")
        );
        assert_eq!(
            validate_host(BuildTarget::Linux).is_ok(),
            cfg!(target_os = "linux")
        );
    }

    #[test]
    fn macos_dmg_icon_command_uses_native_appkit_without_exposing_source() {
        let command = macos_dmg_icon_command(
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
        let first = automatic_android_signing(temp.path(), false).expect("first signing");
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
        let second = automatic_android_signing(temp.path(), false).expect("second signing");

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
        let result = explicit_android_signing([Some("release.p12".into()), None, None, None], true);
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("partial override must fail"),
        };

        assert!(error.to_string().contains("require DOWE_ANDROID_KEYSTORE"));
    }

    #[test]
    fn ios_signing_override_is_atomic_and_sanitized() {
        let error =
            explicit_ios_signing(Some("identity".into()), None).expect_err("partial iOS override");
        assert!(error.to_string().contains("require both DOWE_IOS"));

        let signing = explicit_ios_signing(
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
        assert!(profile_bundle_is_exact(
            "TEAM123.dev.dowe.blogs",
            "dev.dowe.blogs"
        ));
        assert!(profile_bundle_match("TEAM123.*", "dev.dowe.blogs"));
        assert!(!profile_bundle_is_exact("TEAM123.*", "dev.dowe.blogs"));
        assert!(!profile_bundle_match(
            "TEAM123.dev.other.app",
            "dev.dowe.blogs"
        ));
    }
}
