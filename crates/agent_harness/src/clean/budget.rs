use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_micros: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_micros: Option<u64>,
    pub duration_ms: u64,
}

impl Budget {
    pub fn accepts(self, current: BudgetUsage, next: BudgetUsage) -> bool {
        let input = current.input_tokens.saturating_add(next.input_tokens);
        let output = current.output_tokens.saturating_add(next.output_tokens);
        let duration = current.duration_ms.saturating_add(next.duration_ms);
        let cost = match (current.cost_micros, next.cost_micros) {
            (Some(left), Some(right)) => Some(left.saturating_add(right)),
            _ => None,
        };
        input <= self.input_tokens
            && output <= self.output_tokens
            && duration <= self.duration_ms
            && cost.is_none_or(|value| value <= self.cost_micros)
    }
}

impl BudgetUsage {
    pub fn add(self, other: Self) -> Self {
        Self {
            input_tokens: self.input_tokens.saturating_add(other.input_tokens),
            output_tokens: self.output_tokens.saturating_add(other.output_tokens),
            cost_micros: match (self.cost_micros, other.cost_micros) {
                (Some(left), Some(right)) => Some(left.saturating_add(right)),
                _ => None,
            },
            duration_ms: self.duration_ms.saturating_add(other.duration_ms),
        }
    }
}
