use crate::{AgentError, AgentResult};
use dowe_components::{
    BuiltinComponent, PropValueKind, VIEW_PROP_INVENTORY, ViewPropOwner, component_prop_contract,
    prop_allowed_values, prop_validator_name,
};
use serde::{Deserialize, Serialize};

/// First lookup response. It is deliberately small so component selection does
/// not spend context on props the model did not ask for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSummary {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentProp {
    pub name: String,
    pub kind: String,
    pub validator: String,
    pub allowed_values: Vec<String>,
}

/// Second lookup response. This is the authoritative component contract used
/// before source generation: props and a minimal valid example are returned
/// together so the model cannot invent a component API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentContract {
    pub name: String,
    pub props: Vec<ComponentProp>,
    pub example: String,
}

/// Compact first-stage catalog. This is the complete authoritative inventory;
/// detailed props are intentionally withheld until the model selects names.
pub fn component_catalog() -> Vec<ComponentSummary> {
    BuiltinComponent::ALL
        .iter()
        .map(|component| ComponentSummary {
            name: component.as_str().into(),
            description: description(*component).into(),
        })
        .collect()
}

/// Resolve the selected names in one deterministic second-stage payload.
pub fn selected_component_contracts(names: &[String]) -> AgentResult<Vec<ComponentContract>> {
    if names.is_empty() || names.len() > 64 {
        return Err(AgentError::new(
            "view component selection must contain between 1 and 64 components",
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    let mut contracts = Vec::with_capacity(names.len());
    for name in names {
        if !unique.insert(name.to_ascii_lowercase()) {
            return Err(AgentError::new(
                "view component selection contains duplicates",
            ));
        }
        contracts.push(component_contract(name).ok_or_else(|| {
            AgentError::new(format!(
                "view component selection contains unknown component `{name}`"
            ))
        })?);
    }
    Ok(contracts)
}

pub fn component_summaries(query: &str, limit: usize) -> Vec<ComponentSummary> {
    let query = query.to_ascii_lowercase();
    BuiltinComponent::ALL
        .iter()
        .filter(|component| component.as_str().to_ascii_lowercase().contains(&query))
        .take(limit.clamp(1, 32))
        .map(|component| ComponentSummary {
            name: component.as_str().into(),
            description: description(*component).into(),
        })
        .collect()
}

pub fn component_contract(name: &str) -> Option<ComponentContract> {
    let component = BuiltinComponent::from_name(name)?;
    let mut names = vec![
        "variant", "scheme", "size", "rounded", "show", "p", "px", "py", "w", "h", "disabled",
        "loading", "src", "name", "label", "bind", "data", "items",
    ];
    names.extend(
        VIEW_PROP_INVENTORY
            .iter()
            .filter_map(|definition| match definition.owner {
                ViewPropOwner::Component(owner) if owner == component => Some(definition.prop),
                _ => None,
            }),
    );
    names.sort_unstable();
    names.dedup();
    let props = names
        .into_iter()
        .filter_map(|name| {
            let contract = component_prop_contract(component, name)?;
            Some(ComponentProp {
                name: name.into(),
                kind: kind_name(contract.kind).into(),
                validator: prop_validator_name(contract.validator).into(),
                allowed_values: prop_allowed_values(component, name)
                    .iter()
                    .map(|value| (*value).into())
                    .collect(),
            })
        })
        .collect();
    Some(ComponentContract {
        name: component.as_str().into(),
        props,
        example: example(component),
    })
}

fn kind_name(kind: PropValueKind) -> &'static str {
    match kind {
        PropValueKind::String => "string",
        PropValueKind::Number => "number",
        PropValueKind::Boolean => "boolean",
        PropValueKind::Any => "any",
    }
}

fn description(component: BuiltinComponent) -> &'static str {
    match component {
        BuiltinComponent::Button => "An actionable control for submitting or triggering an action.",
        BuiltinComponent::Text => "A text node that inherits foreground styling from its parent.",
        BuiltinComponent::Title => "A semantic heading with theme-controlled typography.",
        BuiltinComponent::Section => {
            "A semantic page band that owns layout and background context."
        }
        BuiltinComponent::Card => "A surfaced container for related content and actions.",
        BuiltinComponent::Image => "An image display component for project or remote media.",
        _ => "A built-in Dowe view component with a compiler-validated contract.",
    }
}

fn example(component: BuiltinComponent) -> String {
    match component {
        BuiltinComponent::Button => "Button \"Continue\"".into(),
        BuiltinComponent::Text => "Text \"Hello\"".into(),
        BuiltinComponent::Title => "Title \"Page title\"".into(),
        BuiltinComponent::Image => "Image src:\"/assets/example.png\"".into(),
        _ => format!("{}", component.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_is_small_and_detail_has_props_and_example() {
        let summary = component_summaries("but", 10);
        assert_eq!(summary[0].name, "Button");
        assert!(summary[0].description.contains("control"));
        let detail = component_contract("Button").unwrap();
        assert_eq!(detail.name, "Button");
        assert!(!detail.props.is_empty());
        assert_eq!(detail.example, "Button \"Continue\"");
    }

    #[test]
    fn first_stage_catalog_is_complete_and_second_stage_is_selected_only() {
        let catalog = component_catalog();
        assert_eq!(catalog.len(), BuiltinComponent::ALL.len());
        let selected = selected_component_contracts(&["Button".into(), "Card".into()]).unwrap();
        assert_eq!(
            selected
                .iter()
                .map(|contract| contract.name.as_str())
                .collect::<Vec<_>>(),
            ["Button", "Card"]
        );
        assert!(selected.iter().all(|contract| !contract.props.is_empty()));
        assert!(selected.iter().all(|contract| !contract.example.is_empty()));
        assert!(selected_component_contracts(&["NotAComponent".into()]).is_err());
        assert!(selected_component_contracts(&["Button".into(), "button".into()]).is_err());
    }
}
