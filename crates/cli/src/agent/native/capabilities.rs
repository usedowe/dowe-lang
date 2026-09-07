use super::*;
use dowe_agent::native_harness::{
    CAPABILITY_CATALOG_VERSION, ModelCapabilities, builtin_capability_catalog,
    builtin_capability_evidence, builtin_model_capabilities,
};

impl NativeSession {
    pub(super) fn capability_command(
        &mut self,
        argument: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let parts: Vec<_> = argument.split_whitespace().collect();
        if parts.is_empty() {
            let mut models: Vec<_> = builtin_capability_catalog()
                .iter()
                .flat_map(|entry| {
                    entry
                        .models
                        .iter()
                        .map(|model| format!("{}/{model}", entry.provider))
                })
                .collect();
            models.sort();
            return Ok(
                json!({"declared":self.config.capabilities,"catalog_version":CAPABILITY_CATALOG_VERSION,"builtin_models":models,"usage":"/capabilities provider/model [tools:true|false images:true|false] or provider/model inherit","policy":"Explicit capability declaration is not execution approval or proof of account availability"}),
            );
        }
        let (provider, model) = parts[0].split_once('/').ok_or("Use provider/model")?;
        let model = dowe_agent::normalize_model_id(provider, model);
        let selected = ModelSelection::new(provider, model);
        selected.validate()?;
        let key = format!("{provider}/{model}");
        match parts.as_slice() {
            [_] => {
                let declared = self.config.capabilities.get(&key).copied();
                let builtin = builtin_model_capabilities(provider, model);
                return Ok(json!({
                    "model":key, "declared":declared, "builtin":builtin,
                    "effective":declared.or(builtin),
                    "effective_source":if declared.is_some() { "local_declaration" } else if builtin.is_some() { "builtin_snapshot" } else { "unknown" },
                    "catalog_version":CAPABILITY_CATALOG_VERSION,
                    "builtin_evidence":builtin_capability_evidence(provider, model),
                    "execution_approval":false
                }));
            }
            [_, "inherit"] => {
                self.config.capabilities.remove(&key);
            }
            [_, tools, images] => {
                let tools = tools
                    .strip_prefix("tools:")
                    .unwrap_or(tools)
                    .parse::<bool>()?;
                let images = images
                    .strip_prefix("images:")
                    .unwrap_or(images)
                    .parse::<bool>()?;
                self.config
                    .capabilities
                    .insert(key.clone(), ModelCapabilities { tools, images });
            }
            _ => {
                return Err(
                    "Use /capabilities provider/model tools:true|false images:true|false".into(),
                );
            }
        }
        self.store.save_config(&self.config)?;
        Ok(
            json!({"model":key,"declared":self.config.capabilities.get(&key),"execution_approval":false}),
        )
    }
}
