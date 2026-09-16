use dowe_agent_harness::coordinator::Intent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentDecision {
    pub intent: Intent,
    pub explicit: bool,
    pub reason: String,
}

pub fn route_intent(prompt: &str, requested: Option<Intent>) -> IntentDecision {
    if let Some(intent) = requested {
        return IntentDecision {
            intent,
            explicit: true,
            reason: "explicit invocation mode".into(),
        };
    }
    let normalized = prompt.trim().to_lowercase();
    let words: Vec<_> = normalized
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let starts =
        |prefix: &str| normalized == prefix || normalized.starts_with(&format!("{prefix} "));
    let has = |values: &[&str]| words.iter().any(|w| values.contains(w));
    let (intent, reason) = if [
        "no cambies",
        "no modifiques",
        "solo explica",
        "do not modify",
        "don't change",
        "explain only",
    ]
    .iter()
    .any(|p| normalized.contains(p))
    {
        (Intent::Ask, "explicit read-only constraint")
    } else if [
        "explica",
        "explícame",
        "explain",
        "what",
        "why",
        "cómo",
        "como",
        "por qué",
        "qué",
        "que",
        "review",
        "revisa",
        "audita",
    ]
    .iter()
    .any(|p| starts(p))
    {
        (Intent::Ask, "question or inspection request")
    } else if has(&[
        "plan",
        "planifica",
        "planea",
        "planear",
        "planning",
        "planifica",
        "spec",
        "specification",
    ]) && !has(&["implementa", "implement", "aplica", "apply"])
    {
        (Intent::Plan, "planning request")
    } else if has(&[
        "create",
        "build",
        "implement",
        "fix",
        "change",
        "delete",
        "remove",
        "rewrite",
        "add",
        "update",
        "crea",
        "crear",
        "crearlo",
        "construye",
        "implementa",
        "implementar",
        "corrige",
        "arregla",
        "cambia",
        "elimina",
        "eliminar",
        "reescribe",
        "añade",
        "actualiza",
        "run",
        "execute",
        "test",
    ]) {
        (Intent::Build, "requested implementation")
    } else {
        (Intent::Ask, "no explicit mutation request")
    };
    IntentDecision {
        intent,
        explicit: false,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn questions_about_changes_do_not_authorize_mutation() {
        for prompt in [
            "cómo puedo implementar login",
            "explain how to fix this",
            "no cambies nada, revisa login",
            "why does create fail?",
        ] {
            assert_eq!(route_intent(prompt, None).intent, Intent::Ask, "{prompt}");
        }
    }

    #[test]
    fn planning_build_and_explicit_modes_are_distinct() {
        assert_eq!(route_intent("planea el login", None).intent, Intent::Plan);
        assert_eq!(
            route_intent("puedes crearlo desde cero", None).intent,
            Intent::Build
        );
        assert_eq!(
            route_intent("fix login", Some(Intent::Ask)).intent,
            Intent::Ask
        );
        assert_eq!(
            route_intent("construction machinery", None).intent,
            Intent::Ask
        );
    }
}
