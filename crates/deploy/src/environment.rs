use crate::access::ACCESS_PASSWORD_NAME;
use dowe_compiler::{CompiledProject, EnvironmentVisibility};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DeployEnvironmentValues {
    pub client: Vec<(String, String)>,
    pub server: Vec<(String, String)>,
    pub server_names: Vec<String>,
}

impl DeployEnvironmentValues {
    pub(crate) fn from_project(project: &CompiledProject) -> Self {
        let mut values = Self {
            client: project.environment_config.client_values(),
            ..Self::default()
        };
        for variable in &project.environment_config.variables {
            if variable.visibility != EnvironmentVisibility::Server
                || variable.name == ACCESS_PASSWORD_NAME
            {
                continue;
            }
            values.server_names.push(variable.name.clone());
            if let Some(value) = variable.resolved_value.clone() {
                values.server.push((variable.name.clone(), value));
            }
        }
        values
    }
}
