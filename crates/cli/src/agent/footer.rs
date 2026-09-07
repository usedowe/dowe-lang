use dialoguer::console::{measure_text_width, style, truncate_str};
use dowe_agent::{AgentUsageTotals, ThinkingLevel, agent_model_details};

pub(super) struct Footer<'a> {
    pub provider: Option<&'a str>,
    pub model: Option<&'a str>,
    pub thinking: Option<ThinkingLevel>,
    pub usage: &'a AgentUsageTotals,
}

impl Footer<'_> {
    pub fn lines(&self, width: usize) -> [String; 2] {
        let provider = self.provider.unwrap_or("no-provider");
        let model = self.model.unwrap_or("no-model");
        let details = agent_model_details(provider, model);
        let thinking = self
            .thinking
            .map(ThinkingLevel::as_str)
            .unwrap_or(if details.is_some() { "default" } else { "n/a" });
        let right = format!(
            "({provider}) {model} • {thinking}{}",
            if provider == "openai-codex" {
                " (sub)"
            } else {
                ""
            }
        );
        let right: String = right.chars().filter(|ch| !ch.is_control()).collect();
        let tokens = if self.usage.known_usage == 0 && self.usage.responses > 0 {
            "tokens n/a".to_string()
        } else {
            format!(
                "↑{} ↓{} R{} W{}{}",
                compact(self.usage.input),
                compact(self.usage.output),
                compact(self.usage.cache_read),
                compact(self.usage.cache_write),
                if self.usage.incomplete_usage {
                    " +?"
                } else {
                    ""
                }
            )
        };
        let cost = if self.usage.known_cost == 0 && self.usage.responses > 0 {
            "cost n/a".to_string()
        } else {
            format!(
                "${:.3}{}{}",
                self.usage.cost_usd,
                if self.usage.estimated_cost {
                    " est."
                } else {
                    ""
                },
                if self.usage.incomplete_cost {
                    " +?"
                } else {
                    ""
                }
            )
        };
        let context = match (
            self.usage.context_tokens(provider, model),
            details.and_then(|d| d.context_window),
        ) {
            (Some(tokens), Some(window)) => format!(
                "ctx {:.1}%/{}",
                tokens as f64 * 100.0 / window as f64,
                compact(window)
            ),
            (None, Some(window)) => format!("ctx ?/{}", compact(window)),
            (Some(tokens), None) => format!("ctx {}/?", compact(tokens)),
            _ => "ctx n/a".to_string(),
        };
        let left = format!("{tokens} {cost} {context}");
        let right_width = measure_text_width(&right);
        let text = if right_width + 2 < width {
            let left = truncate_str(&left, width - right_width - 2, "…");
            format!(
                "{left}{}{right}",
                " ".repeat(width - right_width - measure_text_width(&left))
            )
        } else {
            truncate_str(&right, width, "").into_owned()
        };
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let cwd: String = cwd.chars().filter(|ch| !ch.is_control()).collect();
        [
            style(truncate_str(&cwd, width, "…")).dim().to_string(),
            style(text).dim().to_string(),
        ]
    }
}

fn compact(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1000 {
        format!("{:.0}k", tokens as f64 / 1000.0)
    } else {
        tokens.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dialoguer::console::strip_ansi_codes;
    use serde_json::json;

    #[test]
    fn footer_reports_actual_usage_and_handles_narrow_terminals() {
        let mut usage = AgentUsageTotals::default();
        usage.record(
            "openai-codex",
            "gpt-5.5",
            &json!({"usage":{"input_tokens":1000,"output_tokens":100}}),
        );
        let footer = Footer {
            provider: Some("openai-codex"),
            model: Some("gpt-5.5"),
            thinking: Some(ThinkingLevel::High),
            usage: &usage,
        };
        let line = footer.lines(160)[1].clone();
        let text = strip_ansi_codes(&line);
        assert!(text.contains("(openai-codex) gpt-5.5 • high (sub)"));
        assert!(text.contains("↑1k ↓100"));
        assert!(text.contains("$0.008 est."));
        assert!(text.contains("ctx 0.4%/272k"));
        assert!(!text.contains("auto"));
        for width in [1, 8, 30, 80, 160] {
            for line in footer.lines(width) {
                assert!(measure_text_width(&line) <= width);
            }
        }
    }

    #[test]
    fn missing_metrics_are_not_reported_as_zero() {
        let mut usage = AgentUsageTotals::default();
        usage.record("openai", "custom", &json!({}));
        let footer = Footer {
            provider: Some("openai"),
            model: Some("custom"),
            thinking: None,
            usage: &usage,
        };
        let lines = footer.lines(160);
        assert!(lines[1].contains("cost n/a"));
        assert!(lines[1].contains("tokens n/a"));
        assert!(lines[1].contains("ctx n/a"));
        assert!(!lines[1].contains("$0.000"));
    }
}
