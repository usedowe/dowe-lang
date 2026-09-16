use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    Transient,
    Model,
    StaleSource,
    Conflict,
    Permission,
    UncertainEffect,
    Requirement,
    Budget,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub class: FailureClass,
    pub message: String,
    pub retryable: bool,
    pub attempt: u16,
}

impl Failure {
    pub fn new(class: FailureClass, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            class,
            message: message.into(),
            retryable,
            attempt: 0,
        }
    }

    pub fn next_attempt(mut self) -> Self {
        self.attempt = self.attempt.saturating_add(1);
        self
    }
}
