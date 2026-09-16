use crate::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MAX_ITEMS: usize = 256;
const MAX_FIELDS: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendArchitecturePlan {
    #[serde(default)]
    pub entities: Vec<BackendEntity>,
    #[serde(default)]
    pub handlers: Vec<BackendHandler>,
    #[serde(default)]
    pub routes: Vec<BackendRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendEntity {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendHandler {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendRoute {
    pub id: String,
    pub method: String,
    pub path: String,
    pub handler: String,
}

impl BackendArchitecturePlan {
    pub fn validate(&self) -> AgentResult<()> {
        if self.entities.len() > MAX_ITEMS
            || self.handlers.len() > MAX_ITEMS
            || self.routes.len() > MAX_ITEMS
        {
            return Err(AgentError::new("backend plan contains too many items"));
        }
        let entity_ids = self
            .entities
            .iter()
            .map(|entity| {
                validate_id(&entity.id, "entity")?;
                validate_text(&entity.name, "entity name", 256)?;
                if entity.fields.len() > MAX_FIELDS {
                    return Err(AgentError::new("backend entity contains too many fields"));
                }
                let mut fields = BTreeSet::new();
                for field in &entity.fields {
                    validate_text(field, "entity field", 128)?;
                    if !fields.insert(field) {
                        return Err(AgentError::new("backend entity fields must be unique"));
                    }
                }
                Ok(entity.id.as_str())
            })
            .collect::<AgentResult<BTreeSet<_>>>()?;
        let handler_ids = self
            .handlers
            .iter()
            .map(|handler| {
                validate_id(&handler.id, "handler")?;
                validate_text(&handler.name, "handler name", 256)?;
                if handler.operations.len() > MAX_FIELDS {
                    return Err(AgentError::new("backend handler has too many operations"));
                }
                for operation in &handler.operations {
                    validate_text(operation, "handler operation", 128)?;
                }
                Ok(handler.id.as_str())
            })
            .collect::<AgentResult<BTreeSet<_>>>()?;
        if entity_ids.len() != self.entities.len() || handler_ids.len() != self.handlers.len() {
            return Err(AgentError::new("backend plan IDs must be unique"));
        }
        let mut route_ids = BTreeSet::new();
        for route in &self.routes {
            validate_id(&route.id, "route")?;
            if !route_ids.insert(route.id.as_str()) {
                return Err(AgentError::new("backend route IDs must be unique"));
            }
            validate_method(&route.method)?;
            if route.path.len() > 512
                || !route.path.starts_with('/')
                || route.path.chars().any(char::is_control)
            {
                return Err(AgentError::new(
                    "backend route path must be a bounded absolute path",
                ));
            }
            validate_id(&route.handler, "route handler")?;
            if !handler_ids.contains(route.handler.as_str()) {
                return Err(AgentError::new(format!(
                    "backend route references unknown handler `{}`",
                    route.handler
                )));
            }
        }
        Ok(())
    }
}

fn validate_id(value: &str, kind: &str) -> AgentResult<()> {
    if value.is_empty()
        || value.len() > 128
        || value.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
        })
    {
        return Err(AgentError::new(format!("backend {kind} ID is invalid")));
    }
    Ok(())
}

fn validate_text(value: &str, kind: &str, max: usize) -> AgentResult<()> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(AgentError::new(format!("backend {kind} is invalid")));
    }
    Ok(())
}

fn validate_method(value: &str) -> AgentResult<()> {
    if !matches!(
        value,
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS" | "HEAD"
    ) {
        return Err(AgentError::new("backend route method is unsupported"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> BackendArchitecturePlan {
        BackendArchitecturePlan {
            entities: vec![BackendEntity {
                id: "user".into(),
                name: "User".into(),
                fields: vec!["email".into()],
            }],
            handlers: vec![BackendHandler {
                id: "list_users".into(),
                name: "List users".into(),
                operations: vec!["query user".into()],
            }],
            routes: vec![BackendRoute {
                id: "users".into(),
                method: "GET".into(),
                path: "/users".into(),
                handler: "list_users".into(),
            }],
        }
    }

    #[test]
    fn validates_entities_handlers_and_routes_as_one_contract() {
        plan().validate().expect("valid backend plan");
    }

    #[test]
    fn rejects_unknown_route_handlers_and_duplicate_fields() {
        let mut invalid = plan();
        invalid.routes[0].handler = "missing".into();
        assert!(invalid.validate().is_err());
        let mut duplicate = plan();
        duplicate.entities[0].fields.push("email".into());
        assert!(duplicate.validate().is_err());
    }
}
