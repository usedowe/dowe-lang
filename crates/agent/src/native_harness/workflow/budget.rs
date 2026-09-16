use super::*;
use crate::{AgentRequest, AgentUsage, agent_model_details, agent_response_usage};

#[derive(Clone)]
pub(super) struct WorkflowBudget {
    limit: u64,
    cost_limit: Option<f64>,
    tokens: u64,
    cost: f64,
    unknown_cost: bool,
    estimated: bool,
    attempts: u64,
    pub exhausted: bool,
}

pub(super) struct Reservation {
    tokens: u64,
    cost: f64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BudgetSnapshot {
    pub tokens: u64,
    pub cost: f64,
    pub unknown_cost: bool,
    pub estimated: bool,
    pub attempts: u64,
    pub exhausted: bool,
}

impl WorkflowBudget {
    pub fn restore(config: &HarnessConfig, summary: &WorkflowSummary) -> AgentResult<Self> {
        if !summary.cost_usd.is_finite() || summary.cost_usd < 0.0 {
            return Err(AgentError::new("invalid persisted workflow cost"));
        }
        let mut budget = Self::new(config);
        budget.tokens = summary.charged_tokens;
        budget.cost = summary.cost_usd;
        budget.unknown_cost = summary.cost_unknown;
        budget.estimated = summary.estimated;
        budget.attempts = summary.provider_attempts;
        budget.exhausted = summary.budget_exhausted;
        budget.check()?;
        Ok(budget)
    }
    pub fn new(config: &HarnessConfig) -> Self {
        Self {
            limit: config.token_budget,
            cost_limit: config.cost_budget_usd,
            tokens: 0,
            cost: 0.0,
            unknown_cost: false,
            estimated: false,
            attempts: 0,
            exhausted: false,
        }
    }

    pub fn check(&mut self) -> AgentResult<()> {
        if self.tokens >= self.limit
            || self
                .cost_limit
                .is_some_and(|limit| self.unknown_cost || self.cost >= limit)
        {
            self.exhausted = true;
        }
        if self.exhausted {
            Err(AgentError::new(
                "workflow budget exhausted or cost uncertain",
            ))
        } else {
            Ok(())
        }
    }

    pub fn reserve(&mut self, request: &mut AgentRequest) -> AgentResult<Reservation> {
        self.check()?;
        let output = request
            .extra
            .get("max_completion_tokens")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(8192)
            .clamp(1, 8192);
        request
            .extra
            .insert("max_completion_tokens".into(), json!(output));
        let input = super::super::task::estimate_request(request)?;
        let tokens = input.saturating_add(output);
        let price = request
            .provider
            .as_deref()
            .and_then(|p| agent_model_details(p, &request.model))
            .and_then(|d| {
                let prices = d.prices?;
                let long = d.long_context_prices.unwrap_or(prices);
                let input_rate = [prices[0], prices[2], prices[3], long[0], long[2], long[3]]
                    .into_iter()
                    .fold(0.0_f64, f64::max);
                let output_rate = prices[1].max(long[1]);
                Some((input as f64 * input_rate + output as f64 * output_rate) / 1_000_000.0)
            })
            .filter(|cost| cost.is_finite() && *cost >= 0.0);
        if self.tokens.saturating_add(tokens) > self.limit
            || self
                .cost_limit
                .is_some_and(|limit| price.is_none_or(|cost| self.cost + cost > limit))
        {
            self.exhausted = true;
            return Err(AgentError::new(
                "workflow cannot reserve the next provider attempt",
            ));
        }
        let cost = price.unwrap_or(0.0);
        self.tokens = self.tokens.saturating_add(tokens);
        self.cost += cost;
        self.attempts += 1;
        Ok(Reservation { tokens, cost })
    }

    pub fn settle(
        &mut self,
        request: &AgentRequest,
        reservation: Reservation,
        payload: Option<&serde_json::Value>,
    ) {
        let usage = payload.and_then(|payload| {
            agent_response_usage(
                request.provider.as_deref().unwrap_or(""),
                &request.model,
                payload,
            )
        });
        self.settle_usage(reservation, usage);
    }

    fn settle_usage(&mut self, reservation: Reservation, usage: Option<AgentUsage>) {
        if let Some(usage) = usage {
            self.tokens = self
                .tokens
                .saturating_sub(reservation.tokens)
                .saturating_add(usage.context_tokens());
            if let Some(cost) = usage.cost_usd {
                self.cost = (self.cost - reservation.cost).max(0.0) + cost;
                self.estimated |= usage.estimated_cost;
            } else {
                self.unknown_cost = true;
            }
        } else {
            self.estimated = true;
            self.unknown_cost = true;
        }
        let _ = self.check();
    }

    pub fn event(&self) -> serde_json::Value {
        json!({"event":"workflow_budget","charged_tokens":self.tokens,"reserved_or_reported_cost_usd":self.cost,
            "cost_unknown":self.unknown_cost,"estimated":self.estimated,"attempts":self.attempts,
            "exhausted":self.exhausted,"token_limit":self.limit,"cost_limit_usd":self.cost_limit})
    }

    pub fn interrupt(&mut self) {
        self.unknown_cost = true;
        self.estimated = true;
        self.exhausted = true;
    }

    pub fn snapshot(&self) -> BudgetSnapshot {
        BudgetSnapshot {
            tokens: self.tokens,
            cost: self.cost,
            unknown_cost: self.unknown_cost,
            estimated: self.estimated,
            attempts: self.attempts,
            exhausted: self.exhausted,
        }
    }
}
