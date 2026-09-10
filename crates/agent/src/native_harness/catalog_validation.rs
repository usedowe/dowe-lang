pub fn skill_index() -> String {
    format!(
        "Units: {}. Use get_skill with an id; provide resource for another declared bundle reference. Bundles core, server, views, theme, domain-modeling, native-ipc remain available. Load only necessary units. Core covers application .dowe source and docs; core/configuration covers root environment names and .gitignore; views units cover app public/assets media. Skills classify scope, never grant execution approval.",
        UNITS
            .iter()
            .map(|unit| unit.id)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn skill_unit(id: &str) -> AgentResult<SkillUnit> {
    let id = id.strip_prefix("dowe-").unwrap_or(id);
    let Some(unit) = UNITS.iter().find(|unit| unit.id == id) else {
        let doc = get_public_skill(id, false).map_err(|error| {
            let message = error.to_string();
            if message.starts_with("unknown public Dowe skill") {
                AgentError::new(format!(
                    "{message}; use an exact logical id such as core, theme, views, server, domain-modeling, or native-ipc"
                ))
            } else {
                error
            }
        })?;
        return Ok(SkillUnit {
            id: doc.id,
            hash: super::digest(doc.content.as_bytes()),
            dependencies: vec!["core".into()],
            resource: "SKILL.md".into(),
            content: doc.content,
        });
    };
    let doc = get_public_skill_resource(unit.bundle, unit.resource)?;
    let mut content = if unit.sections.is_empty() {
        doc.content
    } else {
        let mut selected = Vec::new();
        for section in unit.sections {
            let header = format!("## {section}\n");
            let start = doc.content.find(&header).ok_or_else(|| {
                AgentError::new(format!("missing section in {}: {section}", unit.id))
            })?;
            let rest = &doc.content[start + header.len()..];
            let end = rest.find("\n## ").unwrap_or(rest.len());
            selected.push(format!("{header}{}", &rest[..end]));
        }
        selected.join("\n\n")
    };
    if id == "core/configuration" {
        content.push_str("\nEnvironment files: .env, .env.example, .env.live, .env.stage, .env.uat belong at the application root. Only names and placeholders enter model context. Use local protected input for secrets. Never replace a hidden value with a placeholder. Keep private profiles ignored by Git. Root .gitignore and app docs may be edited for this workflow.\n");
    }
    Ok(SkillUnit {
        id: id.into(),
        hash: super::digest(content.as_bytes()),
        dependencies: if id == "core" {
            vec![]
        } else {
            vec!["core".into()]
        },
        resource: format!("{}/{}", unit.bundle, unit.resource),
        content,
    })
}

pub(super) fn catalog_fingerprint() -> AgentResult<String> {
        // cached independently from dynamic provider authority
    static FINGERPRINT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    if let Some(hash) = FINGERPRINT.get() {
        return Ok(hash.clone());
    }
    let mut bytes = Vec::new();
    for bundle in crate::public_skills() {
        bytes.extend_from_slice(
            crate::get_public_skill(&bundle.id, true)?
                .content
                .as_bytes(),
        );
    }
    bytes.extend_from_slice(skill_index().as_bytes());
    let hash = super::digest(&bytes);
    let _ = FINGERPRINT.set(hash.clone());
    Ok(hash)
}

pub fn authority_fingerprint_with_registry(registry: &crate::provider::ProviderRegistry) -> AgentResult<String> {
    use std::fmt::Write;
    let mut material = String::new();
    for provider_id in crate::builtin_provider_ids() {
        let definition = crate::provider_definition(provider_id)
            .ok_or_else(|| AgentError::new("embedded provider definition is missing"))?;
        writeln!(
            material,
            "provider|{}|{}|{}|{}|{}|{}|{}|{:?}|{:?}",
            definition.id,
            definition.name,
            definition.default_model,
            definition.protocol.as_str(),
            definition.supports_api_key,
            definition.supports_account,
            crate::catalog::builtin_models(provider_id).len(),
            definition.env_keys,
            definition.required_env,
        )
        .unwrap();
        for model in crate::catalog::builtin_models(provider_id) {
            writeln!(
                material,
                "model|{}|{}|{:?}|{:?}|{:?}",
                model.id, model.name, model.tools, model.images, model.context_window
            )
            .unwrap();
        }
    }
    material.push_str("dynamic-providers|");
        material.push_str(&registry.canonical_material()?);
        material.push_str("capabilities|");
    material.push_str(&String::from_utf8_lossy(
        &super::capability_catalog::authority_material(),
    ));
    material.push_str("skills|");
    material.push_str(&catalog_fingerprint()?);
    Ok(super::digest(material.as_bytes()))
}

pub fn validate_catalog() -> AgentResult<()> {
    let mut ids = BTreeSet::new();
    let mut graph = std::collections::BTreeMap::new();
    for unit in UNITS {
        if !ids.insert(unit.id) {
            return Err(AgentError::new("duplicate skill unit"));
        }
        let resolved = skill_unit(unit.id)?;
        graph.insert(unit.id.to_string(), resolved.dependencies.clone());
        for dependency in resolved.dependencies {
            if dependency == unit.id || !UNITS.iter().any(|unit| unit.id == dependency) {
                return Err(AgentError::new("invalid skill dependency"));
            }
        }
    }
    validate_dependencies(&graph)
}

