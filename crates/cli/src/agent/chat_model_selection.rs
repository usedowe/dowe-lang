fn prompt_api_key(
    definition: &dowe_agent::AgentProviderDefinition,
) -> Result<AgentCredential, Box<dyn std::error::Error>> {
    let key = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter {} API key", definition.name))
        .allow_empty_password(false)
        .interact()?;
    let mut scoped_env = std::collections::BTreeMap::new();
    if definition.id == "cloudflare-ai-gateway" || definition.id == "cloudflare-workers-ai" {
        let account = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Cloudflare account ID")
            .allow_empty(false)
            .interact_text()?;
        scoped_env.insert("CLOUDFLARE_ACCOUNT_ID".to_string(), account);
        if definition.id == "cloudflare-ai-gateway" {
            let gateway = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Cloudflare AI Gateway ID")
                .allow_empty(false)
                .interact_text()?;
            scoped_env.insert("CLOUDFLARE_GATEWAY_ID".to_string(), gateway);
        }
    }
    Ok(AgentCredential::ApiKey {
        key: Some(key),
        env: scoped_env,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedModel {
    provider: String,
    model: String,
}

fn select_model_for_session(
    store: &AgentAuthStore,
    current_provider: Option<&str>,
    current_model: Option<&str>,
) -> Result<Option<SelectedModel>, Box<dyn std::error::Error>> {
    let configured = configured_model_providers(store)?;
    let provider = current_provider
        .or_else(|| configured.first().map(String::as_str))
        .ok_or("no agent provider is configured")?;

    if configured.len() <= 1 {
        return Ok(
            select_model(provider, current_model)?.map(|model| SelectedModel {
                provider: provider.to_string(),
                model,
            }),
        );
    }

    select_model_from_providers(&configured, current_provider, current_model)
}

fn select_model_from_providers(
    providers: &[String],
    current_provider: Option<&str>,
    current_model: Option<&str>,
) -> Result<Option<SelectedModel>, Box<dyn std::error::Error>> {
    let entries = model_menu_entries(providers)?;

    let labels = entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect::<Vec<_>>();
    let items = aligned_menu_items(&labels, menu_width());
    let default_index = entries
        .iter()
        .position(|entry| {
            entry.provider == current_provider.unwrap_or_default()
                && entry.model.as_deref() == current_model
        })
        .unwrap_or(0);
    let Some(index) = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select provider and model")
        .items(&items)
        .default(default_index)
        .interact_opt()?
    else {
        return Ok(None);
    };
    let entry = &entries[index];
    let model = match entry.model.as_deref() {
        Some(model) => model.to_string(),
        None => {
            let default = current_provider
                .filter(|provider| *provider == entry.provider)
                .and(current_model)
                .map(str::to_string)
                .unwrap_or(provider_default_model(&entry.provider)?.to_string());
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Model for {}", entry.provider))
                .default(default)
                .allow_empty(false)
                .interact_text()?
        }
    };
    Ok(Some(SelectedModel {
        provider: entry.provider.clone(),
        model,
    }))
}

fn configured_model_providers(
    store: &AgentAuthStore,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(builtin_provider_info(store)?
        .into_iter()
        .filter(|provider| provider.configured)
        .map(|provider| provider.id)
        .collect())
}

#[derive(Debug, Clone)]
struct ModelMenuEntry {
    provider: String,
    model: Option<String>,
    label: String,
}

fn model_menu_entries(
    providers: &[String],
) -> Result<Vec<ModelMenuEntry>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    for provider in providers {
        for model in models_with_local_overrides(provider)? {
            entries.push(ModelMenuEntry {
                provider: provider.clone(),
                model: Some(model.id.clone()),
                label: format!("{provider} • {} • {}", model.name, model.id),
            });
        }
        entries.push(ModelMenuEntry {
            provider: provider.clone(),
            model: None,
            label: format!("{provider} • Enter another model id"),
        });
    }
    Ok(entries)
}

pub(super) fn select_model(
    provider: &str,
    current: Option<&str>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let models = models_with_local_overrides(provider)?;
    let default = current
        .map(str::to_string)
        .unwrap_or(provider_default_model(provider)?.to_string());
    if models.is_empty() {
        return Ok(Some(
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Model")
                .default(default)
                .allow_empty(false)
                .interact_text()?,
        ));
    }
    let mut items = aligned_model_labels(&models, menu_width());
    let custom_index = items.len();
    items.push(menu_text("Enter another model id", menu_width()));
    let default_index = models
        .iter()
        .position(|model| model.id == default)
        .unwrap_or(custom_index);
    let Some(index) = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select {provider} model"))
        .items(&items)
        .default(default_index)
        .interact_opt()?
    else {
        return Ok(None);
    };
    if index == custom_index {
        return Ok(Some(
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Model")
                .default(default)
                .allow_empty(false)
                .interact_text()?,
        ));
    }
    Ok(Some(models[index].id.to_string()))
}

pub(super) fn select_provider(
    store: &AgentAuthStore,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let providers = builtin_provider_info(store)?;
    if providers.is_empty() {
        return Err("no agent providers are available".into());
    }
    let items = aligned_provider_labels(&providers, menu_width());
    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select provider to configure")
        .items(&items)
        .default(0)
        .interact_opt()?;
    Ok(index.map(|index| providers[index].id.clone()))
}
