use super::helpers::native_command;
use super::helpers::{release_build_number, release_version, safe_name};
use super::{NativeCommand, NativePlan, Requirement};
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use dowe_compiler::CompiledProject;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AndroidSigningFile {
    pub(super) alias: String,
    pub(super) store_password: String,
    pub(super) key_password: String,
}

pub(super) struct AndroidSigning {
    pub(super) keystore: PathBuf,
    pub(super) alias: String,
    pub(super) store_password: String,
    pub(super) key_password: String,
    pub(super) keytool: Option<NativeCommand>,
}

pub(super) fn plan(
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

pub(super) fn explicit_android_signing(
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

pub(super) fn automatic_android_signing(
    project_root: &Path,
    dry_run: bool,
) -> DeployResult<AndroidSigning> {
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

pub(super) fn set_private_permissions(_path: &Path) -> DeployResult<()> {
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
