use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResultStatus {
    Hit,
    Free,
    Error,
    Invalid,
    Banned,
    Retry,
    Unknown,
}

impl fmt::Display for ResultStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultStatus::Hit => write!(f, "hit"),
            ResultStatus::Free => write!(f, "free"),
            ResultStatus::Error => write!(f, "error"),
            ResultStatus::Invalid => write!(f, "invalid"),
            ResultStatus::Banned => write!(f, "banned"),
            ResultStatus::Retry => write!(f, "retry"),
            ResultStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: ResultStatus,
    pub message: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    pub retry_count: u32,
    pub captures: HashMap<String, String>,
    pub timestamp: u64,
}

impl CheckResult {
    pub fn new(status: ResultStatus) -> Self {
        Self {
            status,
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
        Self::new(ResultStatus::Hit)
    }

    pub fn free() -> Self {
        Self::new(ResultStatus::Free)
    }

    pub fn error() -> Self {
        Self::new(ResultStatus::Error)
    }

    pub fn invalid() -> Self {
        Self::new(ResultStatus::Invalid)
    }

    pub fn banned() -> Self {
        Self::new(ResultStatus::Banned)
    }

    pub fn retry() -> Self {
        Self::new(ResultStatus::Retry)
    }

    pub fn unknown() -> Self {
        Self::new(ResultStatus::Unknown)
    }
}
