#[derive(Clone, Debug)]
pub(crate) struct SshPackage {
    pub executable: PathBuf,
    server_environment: Vec<(String, String)>,
    service_name: String,
    binary_name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct SshDestination {
    host: String,
    user: String,
    key_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedSshMetadata {
    pub environment: DeployEnvironment,
    pub access_hash: Option<String>,
    pub bind: String,
    pub client_environment: Vec<(String, String)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecutableMetadata<'a> {
    environment: DeployEnvironment,
    access_hash: Option<&'a str>,
    bind: &'a str,
    client_environment: &'a [(String, String)],
}

impl SshDestination {
    pub(crate) fn resolve(
        host: Option<&str>,
        user: Option<&str>,
        key_file: Option<&Path>,
    ) -> DeployResult<Self> {
        let host = host
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| DeployError::new("SSH publish requires --host"))?;
        let user = user
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| DeployError::new("SSH publish requires --user"))?;
        validate_host(host)?;
        validate_user(user)?;
        let key_file = key_file
            .map(|path| {
                let metadata = fs::metadata(path).map_err(|_| {
                    DeployError::new(format!("SSH key file does not exist: {}", path.display()))
                })?;
                if !metadata.is_file() {
                    return Err(DeployError::new(format!(
                        "SSH key file is not a regular file: {}",
                        path.display()
                    )));
                }
                path.canonicalize().map_err(DeployError::from)
            })
            .transpose()?;
        Ok(Self {
            host: host.to_string(),
            user: user.to_string(),
            key_file,
        })
    }

    fn target(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }

    fn auth_args(&self) -> Vec<String> {
        match self.key_file.as_ref() {
            Some(path) => vec![
                "-i".into(),
                path.display().to_string(),
                "-o".into(),
                "IdentitiesOnly=yes".into(),
            ],
            None => vec![
                "-o".into(),
                "PreferredAuthentications=password,keyboard-interactive".into(),
                "-o".into(),
                "PubkeyAuthentication=no".into(),
            ],
        }
    }
}

pub(crate) fn generate_ssh(
    root: &Path,
    output: &Path,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    client_environment: &[(String, String)],
    server_environment: &[(String, String)],
    runtime: &[u8],
) -> DeployResult<SshPackage> {
    generate_ssh_with_runtime(
        root,
        output,
        environment,
        access,
        client_environment,
        server_environment,
        runtime,
    )
}


