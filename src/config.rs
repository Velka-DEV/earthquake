use crate::result::ResultStatus;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub save_hits: bool,
    pub save_free: bool,
    pub save_errors: bool,
    pub save_invalid: bool,
    pub save_banned: bool,
    pub save_retries: bool,
    pub save_unknown: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            save_hits: true,
            save_free: true,
            save_errors: true,
            save_invalid: false,
            save_banned: true,
            save_retries: false,
            save_unknown: true,
        }
    }
}

impl OutputConfig {
    pub fn should_save(&self, status: ResultStatus) -> bool {
        match status {
            ResultStatus::Hit => self.save_hits,
            ResultStatus::Free => self.save_free,
            ResultStatus::Error => self.save_errors,
            ResultStatus::Invalid => self.save_invalid,
            ResultStatus::Banned => self.save_banned,
            ResultStatus::Retry => self.save_retries,
            ResultStatus::Unknown => self.save_unknown,
        }
    }

    pub fn enable_all(mut self) -> Self {
        self.save_hits = true;
        self.save_free = true;
        self.save_errors = true;
        self.save_invalid = true;
        self.save_banned = true;
        self.save_retries = true;
        self.save_unknown = true;
        self
    }

    pub fn disable_all(mut self) -> Self {
        self.save_hits = false;
        self.save_free = false;
        self.save_errors = false;
        self.save_invalid = false;
        self.save_banned = false;
        self.save_retries = false;
        self.save_unknown = false;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub threads: usize,
    pub module_name: String,
    #[serde(with = "serde_duration")]
    pub proxy_cooldown: Duration,
    pub proxy_max_failures: u32,
    pub max_retries: u32,
    pub combo_separator: String,
    pub combo_regex_filter: Option<String>,
    pub proxies_path: Option<String>,
    pub proxies_url: Option<String>,
    pub random_proxies: bool,
    pub combos_path: Option<String>,
    pub save_dir: String,
    pub output_config: OutputConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threads: 100,
            module_name: "default".to_string(),
            proxy_cooldown: Duration::from_secs(0),
            proxy_max_failures: 3,
            max_retries: 3,
            combo_separator: ":".to_string(),
            combo_regex_filter: None,
            proxies_path: None,
            proxies_url: None,
            random_proxies: false,
            combos_path: None,
            save_dir: "results".to_string(),
            output_config: OutputConfig::default(),
        }
    }
}

impl Config {
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
            ..Default::default()
        }
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    pub fn with_proxy_cooldown(mut self, cooldown: Duration) -> Self {
        self.proxy_cooldown = cooldown;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_combo_separator(mut self, separator: impl Into<String>) -> Self {
        self.combo_separator = separator.into();
        self
    }

    pub fn with_combo_regex_filter(mut self, regex: impl Into<String>) -> Self {
        self.combo_regex_filter = Some(regex.into());
        self
    }

    pub fn with_proxies_path(mut self, path: impl Into<String>) -> Self {
        self.proxies_path = Some(path.into());
        self
    }

    pub fn with_proxies_url(mut self, url: impl Into<String>) -> Self {
        self.proxies_url = Some(url.into());
        self
    }

    pub fn with_random_proxies(mut self, random: bool) -> Self {
        self.random_proxies = random;
        self
    }

    pub fn with_combos_path(mut self, path: impl Into<String>) -> Self {
        self.combos_path = Some(path.into());
        self
    }

    pub fn with_save_dir(mut self, dir: impl Into<String>) -> Self {
        self.save_dir = dir.into();
        self
    }

    pub fn with_output_config(mut self, output_config: OutputConfig) -> Self {
        self.output_config = output_config;
        self
    }

    pub fn enable_saving_for(mut self, status: ResultStatus) -> Self {
        match status {
            ResultStatus::Hit => self.output_config.save_hits = true,
            ResultStatus::Free => self.output_config.save_free = true,
            ResultStatus::Error => self.output_config.save_errors = true,
            ResultStatus::Invalid => self.output_config.save_invalid = true,
            ResultStatus::Banned => self.output_config.save_banned = true,
            ResultStatus::Retry => self.output_config.save_retries = true,
            ResultStatus::Unknown => self.output_config.save_unknown = true,
        }
        self
    }

    pub fn disable_saving_for(mut self, status: ResultStatus) -> Self {
        match status {
            ResultStatus::Hit => self.output_config.save_hits = false,
            ResultStatus::Free => self.output_config.save_free = false,
            ResultStatus::Error => self.output_config.save_errors = false,
            ResultStatus::Invalid => self.output_config.save_invalid = false,
            ResultStatus::Banned => self.output_config.save_banned = false,
            ResultStatus::Retry => self.output_config.save_retries = false,
            ResultStatus::Unknown => self.output_config.save_unknown = false,
        }
        self
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string(self)
            .map_err(|e| Error::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content).map_err(Error::Io)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(Error::Io)?;
        toml::from_str(&content)
            .map_err(|e| Error::ConfigError(format!("Failed to parse config: {}", e)))
    }
}

mod serde_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
