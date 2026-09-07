use crate::{
    AgentProviderProtocol, agent_model_details, normalize_model_id, protocol_for_model,
    provider_definition,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub cost_usd: Option<f64>,
    pub estimated_cost: bool,
}

impl AgentUsage {
    pub fn context_tokens(self) -> u64 {
        self.input
            .saturating_add(self.output)
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_write)
    }
}

pub fn agent_response_usage(provider: &str, model: &str, payload: &Value) -> Option<AgentUsage> {
    use AgentProviderProtocol::*;
    let protocol = protocol_for_model(&provider_definition(provider)?, model);
    let payload = payload
        .get("message")
        .filter(|message| message.get("usage").is_some())
        .unwrap_or(payload);
    let usage = payload
        .get("usageMetadata")
        .or_else(|| payload.get("usage"))?;
    let (input, output, cache_read, cache_write) = match protocol {
        OpenAiResponses | OpenAiCompletions | MistralConversations => {
            let input = usage
                .get("input_tokens")
                .or_else(|| usage.get("prompt_tokens"))?
                .as_u64()?;
            let output = usage
                .get("output_tokens")
                .or_else(|| usage.get("completion_tokens"))?
                .as_u64()?;
            let details = usage
                .get("input_tokens_details")
                .or_else(|| usage.get("prompt_tokens_details"));
            let read = details
                .and_then(|v| v.get("cached_tokens"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let write = details
                .and_then(|v| v.get("cache_write_tokens"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            (
                input.checked_sub(read.checked_add(write)?)?,
                output,
                read,
                write,
            )
        }
        AnthropicMessages => (
            usage.get("input_tokens")?.as_u64()?,
            usage.get("output_tokens")?.as_u64()?,
            number(usage, "cache_read_input_tokens"),
            number(usage, "cache_creation_input_tokens"),
        ),
        GoogleGenerativeAi | GoogleVertex => {
            let input = usage.get("promptTokenCount")?.as_u64()?;
            let read = number(usage, "cachedContentTokenCount");
            (
                input.checked_sub(read)?,
                usage
                    .get("candidatesTokenCount")?
                    .as_u64()?
                    .saturating_add(number(usage, "thoughtsTokenCount")),
                read,
                0,
            )
        }
        BedrockConverse => (
            usage.get("inputTokens")?.as_u64()?,
            usage.get("outputTokens")?.as_u64()?,
            number(usage, "cacheReadInputTokens"),
            number(usage, "cacheWriteInputTokens"),
        ),
        PiMessages => (
            usage.get("input")?.as_u64()?,
            usage.get("output")?.as_u64()?,
            number(usage, "cacheRead"),
            number(usage, "cacheWrite"),
        ),
    };
    let reported_cost = usage
        .get("cost")
        .and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.get("total").and_then(Value::as_f64))
        })
        .filter(|cost| cost.is_finite() && *cost >= 0.0);
    let standard_tier = payload
        .get("service_tier")
        .and_then(Value::as_str)
        .is_none_or(|tier| matches!(tier, "default" | "auto"));
    let estimate = agent_model_details(provider, model)
        .filter(|_| standard_tier)
        .and_then(|details| {
            let prices = if input.saturating_add(cache_read).saturating_add(cache_write) > 272000 {
                details.long_context_prices.or(details.prices)?
            } else {
                details.prices?
            };
            Some(
                [input, output, cache_read, cache_write]
                    .iter()
                    .zip(prices)
                    .map(|(tokens, price)| *tokens as f64 * price / 1_000_000.0)
                    .sum(),
            )
        });
    Some(AgentUsage {
        input,
        output,
        cache_read,
        cache_write,
        cost_usd: reported_cost.or(estimate),
        estimated_cost: reported_cost.is_none() && estimate.is_some(),
    })
}

fn number(value: &Value, field: &str) -> u64 {
    value.get(field).and_then(Value::as_u64).unwrap_or(0)
}

#[derive(Debug, Default)]
pub struct AgentUsageTotals {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub cost_usd: f64,
    pub estimated_cost: bool,
    pub incomplete_cost: bool,
    pub incomplete_usage: bool,
    pub responses: u64,
    pub known_usage: u64,
    pub known_cost: u64,
    last_context: Option<(String, String, u64)>,
}

impl AgentUsageTotals {
    pub fn record(&mut self, provider: &str, model: &str, payload: &Value) {
        self.record_usage(
            provider,
            model,
            agent_response_usage(provider, model, payload),
        );
    }

    pub fn record_usage(&mut self, provider: &str, model: &str, usage: Option<AgentUsage>) {
        self.responses += 1;
        self.last_context = None;
        let Some(usage) = usage else {
            self.incomplete_usage = true;
            self.incomplete_cost = true;
            return;
        };
        self.known_usage += 1;
        self.input = self.input.saturating_add(usage.input);
        self.output = self.output.saturating_add(usage.output);
        self.cache_read = self.cache_read.saturating_add(usage.cache_read);
        self.cache_write = self.cache_write.saturating_add(usage.cache_write);
        self.last_context = Some((
            provider.to_string(),
            normalize_model_id(provider, model).to_string(),
            usage.context_tokens(),
        ));
        if let Some(cost) = usage
            .cost_usd
            .filter(|cost| cost.is_finite() && *cost >= 0.0)
        {
            self.cost_usd += cost;
            self.known_cost += 1;
            self.estimated_cost |= usage.estimated_cost;
        } else {
            self.incomplete_cost = true;
        }
    }

    pub fn clear_context(&mut self) {
        self.last_context = None;
    }

    pub fn context_tokens(&self, provider: &str, model: &str) -> Option<u64> {
        self.last_context
            .as_ref()
            .filter(|(p, m, _)| p == provider && m == normalize_model_id(provider, model))
            .map(|(_, _, tokens)| *tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn caches_and_reasoning_are_not_counted_twice() {
        let payload = json!({"usage":{"input_tokens":1000,"output_tokens":100,"input_tokens_details":{"cached_tokens":200,"cache_write_tokens":100},"output_tokens_details":{"reasoning_tokens":80}}});
        let usage = agent_response_usage("openai-codex", "gpt-5.5", &payload).unwrap();
        assert_eq!(
            (
                usage.input,
                usage.output,
                usage.cache_read,
                usage.cache_write,
                usage.context_tokens()
            ),
            (700, 100, 200, 100, 1100)
        );
        assert!((usage.cost_usd.unwrap() - 0.0066).abs() < 1e-10);
        assert!(usage.estimated_cost);
    }

    #[test]
    fn snapshots_cost_tiers_and_unknown_models() {
        let mut totals = AgentUsageTotals::default();
        let payload = json!({"usage":{"input_tokens":300000,"output_tokens":100}});
        totals.record("openai-codex", "gpt-5.5", &payload);
        assert!((totals.cost_usd - 3.0045).abs() < 1e-10);
        totals.record(
            "openai-codex",
            "gpt-5.5",
            &json!({"usage":{"input_tokens":100,"output_tokens":50}}),
        );
        assert_eq!(totals.context_tokens("openai-codex", "gpt-5.5"), Some(150));
        assert_eq!(totals.input, 300100);
        assert_eq!(totals.context_tokens("openai", "gpt-5.5"), None);
        totals.record("openai", "custom", &json!({}));
        assert!(totals.incomplete_cost && totals.incomplete_usage);
        assert_eq!(totals.context_tokens("openai-codex", "gpt-5.5"), None);
        let unknown = agent_response_usage("openai", "custom", &payload).unwrap();
        assert_eq!(unknown.cost_usd, None);
        let reported = agent_response_usage(
            "openai",
            "custom",
            &json!({"usage":{"input_tokens":1,"output_tokens":2,"cost":0.123}}),
        )
        .unwrap();
        assert_eq!(reported.cost_usd, Some(0.123));
        assert!(!reported.estimated_cost);
        assert_eq!(
            agent_response_usage(
                "openai",
                "gpt-5.5",
                &json!({"service_tier":"priority","usage":{"input_tokens":1,"output_tokens":2}})
            )
            .unwrap()
            .cost_usd,
            None
        );
    }

    #[test]
    fn normalizes_every_native_usage_family() {
        let cases = [
            (
                "anthropic",
                json!({"usage":{"input_tokens":10,"output_tokens":20,"cache_read_input_tokens":30,"cache_creation_input_tokens":40}}),
                100,
            ),
            (
                "google",
                json!({"usageMetadata":{"promptTokenCount":50,"candidatesTokenCount":20,"thoughtsTokenCount":10,"cachedContentTokenCount":30}}),
                80,
            ),
            (
                "google-vertex",
                json!({"usageMetadata":{"promptTokenCount":50,"candidatesTokenCount":20}}),
                70,
            ),
            (
                "amazon-bedrock",
                json!({"usage":{"inputTokens":10,"outputTokens":20,"cacheReadInputTokens":30,"cacheWriteInputTokens":40}}),
                100,
            ),
            (
                "mistral",
                json!({"usage":{"prompt_tokens":50,"completion_tokens":20}}),
                70,
            ),
            (
                "radius",
                json!({"message":{"usage":{"input":10,"output":20,"cacheRead":30,"cacheWrite":40,"cost":{"total":0.2}}}}),
                100,
            ),
            (
                "deepseek",
                json!({"usage":{"prompt_tokens":50,"completion_tokens":20,"prompt_tokens_details":{"cached_tokens":10}}}),
                70,
            ),
        ];
        for (provider, payload, context) in cases {
            assert_eq!(
                agent_response_usage(provider, "test", &payload)
                    .unwrap()
                    .context_tokens(),
                context,
                "{provider}"
            );
        }
    }
}
