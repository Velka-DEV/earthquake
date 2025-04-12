use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResultType {
    Hit,
    Free,
    Failed,
    Invalid,
    Banned,
    Retry,
    Custom(u8),
}

impl fmt::Display for ResultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultType::Hit => write!(f, "hit"),
            ResultType::Free => write!(f, "free"),
            ResultType::Failed => write!(f, "failed"),
            ResultType::Invalid => write!(f, "invalid"),
            ResultType::Banned => write!(f, "banned"),
            ResultType::Retry => write!(f, "retry"),
            ResultType::Custom(id) => write!(f, "custom_{}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub result_type: ResultType,
    pub message: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    pub retry_count: u32,
    pub captures: HashMap<String, String>,
    pub timestamp: u64,
}

impl CheckResult {
    pub fn new(result_type: ResultType) -> Self {
        Self {
            result_type,
            message: None,
            extra_data: None,
            retry_count: 0,
            captures: HashMap::new(),
            timestamp: Instant::now().elapsed().as_secs(),
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_extra_data(mut self, data: serde_json::Value) -> Self {
        self.extra_data = Some(data);
        self
    }

    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn with_capture(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.captures.insert(key.into(), value.into());
        self
    }

    pub fn add_capture(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.captures.insert(key.into(), value.into());
        self
    }

    pub fn get_capture(&self, key: &str) -> Option<&String> {
        self.captures.get(key)
    }

    pub fn has_capture(&self, key: &str) -> bool {
        self.captures.contains_key(key)
    }

    pub fn hit() -> Self {
        Self::new(ResultType::Hit)
    }

    pub fn free() -> Self {
        Self::new(ResultType::Free)
    }

    pub fn failed() -> Self {
        Self::new(ResultType::Failed)
    }

    pub fn invalid() -> Self {
        Self::new(ResultType::Invalid)
    }

    pub fn banned() -> Self {
        Self::new(ResultType::Banned)
    }

    pub fn retry() -> Self {
        Self::new(ResultType::Retry)
    }

    pub fn custom(id: u8) -> Self {
        Self::new(ResultType::Custom(id))
    }
}
