use crate::{AgentError, AgentResult, normalize_model_id};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingLevel {
    Off,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl ThinkingLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AgentModelDetails {
    pub context_window: Option<u64>,
    pub thinking_levels: &'static [ThinkingLevel],
    pub prices: Option<[f64; 4]>,
    pub long_context_prices: Option<[f64; 4]>,
}

pub fn agent_model_details(provider: &str, model: &str) -> Option<AgentModelDetails> {
    use ThinkingLevel::*;
    let model = normalize_model_id(provider, model);
    let catalog_context_window = crate::catalog::builtin_models(provider)
        .iter()
        .find(|definition| definition.id == model)
        .and_then(|definition| definition.context_window);
    let codex = provider == "openai-codex";
    let openai = provider == "openai";
    let (context_window, prices, long_context_prices, thinking_levels): (_, _, _, &'static [_]) =
        match (provider, model) {
            (_, "gpt-5.4") if codex || openai => (
                272000,
                [2.5, 15.0, 0.25, 0.0],
                Some([5.0, 22.5, 0.5, 0.0]),
                if codex {
                    &[Low, Medium, High, Xhigh]
                } else {
                    &[Off, Low, Medium, High, Xhigh]
                },
            ),
            (_, "gpt-5.4-mini") if codex || openai => (
                if codex { 272000 } else { 400000 },
                [0.75, 4.5, 0.075, 0.0],
                None,
                if codex {
                    &[Low, Medium, High, Xhigh]
                } else {
                    &[Off, Low, Medium, High, Xhigh]
                },
            ),
            (_, "gpt-5.5") if codex || openai => (
                272000,
                [5.0, 30.0, 0.5, 0.0],
                Some([10.0, 45.0, 1.0, 0.0]),
                if codex {
                    &[Low, Medium, High, Xhigh]
                } else {
                    &[Off, Low, Medium, High, Xhigh]
                },
            ),
            ("openai-codex", "gpt-5.3-codex-spark") => (
                128000,
                [1.75, 14.0, 0.175, 0.0],
                None,
                &[Low, Medium, High, Xhigh],
            ),
            ("openai-codex", "gpt-5.6-luna") => (
                272000,
                [0.2, 1.2, 0.02, 0.25],
                Some([0.4, 1.8, 0.04, 0.5]),
                &[Low, Medium, High, Xhigh, Max],
            ),
            ("openai-codex", "gpt-5.6-sol") => (
                272000,
                [5.0, 30.0, 0.5, 6.25],
                Some([10.0, 45.0, 1.0, 12.5]),
                &[Low, Medium, High, Xhigh, Max],
            ),
            ("openai-codex", "gpt-5.6-terra") => (
                272000,
                [2.0, 12.0, 0.2, 2.5],
                Some([4.0, 18.0, 0.4, 5.0]),
                &[Low, Medium, High, Xhigh, Max],
            ),
            ("openai-codex", "gpt-6-astra") => (
                272000,
                [10.0, 50.0, 1.0, 12.5],
                Some([20.0, 75.0, 2.0, 25.0]),
                &[Low, Medium, High, Xhigh, Max],
            ),
            ("openrouter", "deepseek/deepseek-v4-flash") => (
                1000000,
                [0.14, 0.28, 0.0, 0.0],
                None,
                &[High, Xhigh],
            ),
            ("anthropic", "claude-sonnet-4-6") => (
                1000000,
                [3.0, 15.0, 0.3, 3.75],
                None,
                &[Off, Low, Medium, High, Max],
            ),
            ("anthropic", "claude-opus-4-6") => (
                1000000,
                [5.0, 25.0, 0.5, 6.25],
                None,
                &[Off, Low, Medium, High, Max],
            ),
            _ => {
                return catalog_context_window.map(|context_window| AgentModelDetails {
                    context_window: Some(context_window),
                    thinking_levels: &[],
                    prices: None,
                    long_context_prices: None,
                });
            }
        };
    Some(AgentModelDetails {
        context_window: catalog_context_window.or(Some(context_window)),
        thinking_levels,
        prices: Some(prices),
        long_context_prices,
    })
}

pub fn validate_thinking(provider: &str, model: &str, level: ThinkingLevel) -> AgentResult<()> {
    if agent_model_details(provider, model)
        .is_some_and(|details| details.thinking_levels.contains(&level))
    {
        return Ok(());
    }
    Err(AgentError::new(format!(
        "thinking level `{}` is not supported for {provider}/{model}",
        level.as_str()
    )))
}

pub(crate) fn apply_thinking(
    provider: &str,
    model: &str,
    level: ThinkingLevel,
    body: &mut Value,
) -> AgentResult<()> {
    validate_thinking(provider, model, level)?;
    body.as_object_mut()
        .ok_or_else(|| AgentError::new("invalid native request"))?
        .remove("temperature");
    if provider == "anthropic" {
        body["thinking"] = if level == ThinkingLevel::Off {
            json!({"type":"disabled"})
        } else {
            json!({"type":"adaptive"})
        };
        if level != ThinkingLevel::Off {
            body["output_config"] = json!({"effort":level.as_str()});
        }
    } else {
        body["reasoning"] =
            json!({"effort": if level == ThinkingLevel::Off { "none" } else { level.as_str() }});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_contract_restricts_effort_and_normalizes_names() {
        assert!(
            validate_thinking("openai-codex", "openai-codex/gpt-5.5", ThinkingLevel::High).is_ok()
        );
        assert!(validate_thinking("openai-codex", "gpt-5.5", ThinkingLevel::Max).is_err());
        assert!(validate_thinking("openai-codex", "gpt-5.5", ThinkingLevel::Off).is_err());
        assert!(validate_thinking("openai-codex", "gpt-6-astra", ThinkingLevel::Max).is_ok());
        assert!(validate_thinking("other", "gpt-5.5", ThinkingLevel::High).is_err());
        assert!(agent_model_details("openai", "custom").is_none());
        assert_eq!(
            agent_model_details("openai-codex", "gpt-5.5")
                .unwrap()
                .context_window,
            Some(272000)
        );
            let details = agent_model_details("openrouter", "deepseek/deepseek-v4-flash")
                .expect("OpenRouter DeepSeek V4 Flash metadata");
            assert_eq!(details.context_window, Some(1_000_000));
            assert_eq!(
                details.thinking_levels,
                &[ThinkingLevel::High, ThinkingLevel::Xhigh]
            );
    }
}
