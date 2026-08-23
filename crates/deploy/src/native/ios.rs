use super::helpers::native_command;
use super::helpers::{
    copy_ios_resources, release_build_number, release_version, safe_name, swift_sources,
};
use super::{NativeCommand, NativePlan, Requirement};
use crate::error::{DeployError, DeployResult};
use crate::files::copy_file;
use base64::Engine;
use dowe_compiler::CompiledProject;
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Debug)]
pub(super) struct IosSigning {
    pub(super) identity: String,
    pub(super) profile: PathBuf,
    pub(super) report_identity: String,
    pub(super) report_profile: String,
}

struct IosProfileCandidate {
    path: PathBuf,
    pub(super) identity: String,
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

pub(super) fn explicit_ios_signing(
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

pub(super) fn profile_bundle_match(application_identifier: &str, bundle: &str) -> bool {
    profile_bundle_is_exact(application_identifier, bundle)
        || application_identifier.ends_with(".*")
}

pub(super) fn profile_bundle_is_exact(application_identifier: &str, bundle: &str) -> bool {
    application_identifier
        .strip_suffix(bundle)
        .is_some_and(|prefix| prefix.ends_with('.') && !prefix[..prefix.len() - 1].contains('.'))
}

pub(super) fn plan(
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
