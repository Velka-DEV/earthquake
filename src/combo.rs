use crate::Result;
use crate::error::Error;
use crate::validation::{
    ComboValidator, EmailUsernameValidator, PasswordLengthValidator, RegexValidator,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combo {
    pub username: String,
    pub password: String,
    pub raw: String,
}

impl Combo {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        let username = username.into();
        let password = password.into();
        let raw = format!("{}:{}", username, password);

        Self {
            username,
            password,
            raw,
        }
    }

    pub fn from_raw(raw: impl Into<String>, separator: Option<&str>) -> Result<Self> {
        let raw = raw.into();
        let separator = separator.unwrap_or(":");

        let parts: Vec<&str> = raw.split(separator).collect();

        if parts.len() != 2 {
            return Err(Error::InvalidCombo(format!(
                "Invalid combo format: {}",
                raw
            )));
        }

        Ok(Self {
            username: parts[0].to_string(),
            password: parts[1].to_string(),
            raw,
        })
    }
}

impl std::fmt::Display for Combo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.username, self.password)
    }
}

#[async_trait]
pub trait ComboProvider: Send + Sync {
    async fn next(&self) -> Option<Combo>;
    async fn len(&self) -> usize;
    async fn remaining(&self) -> usize;
    async fn reset(&self);
}

pub struct FileComboProvider {
    path: String,
    combos: Arc<parking_lot::RwLock<Vec<String>>>,
    position: Arc<parking_lot::RwLock<usize>>,
    validators: Vec<Box<dyn ComboValidator>>,
    separator: String,
}

impl FileComboProvider {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            combos: Arc::new(parking_lot::RwLock::new(Vec::new())),
            position: Arc::new(parking_lot::RwLock::new(0)),
            validators: Vec::new(),
            separator: ":".to_string(),
        }
    }

    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    pub fn with_regex_filter(mut self, pattern: &str) -> Result<Self> {
        self.validators
            .push(Box::new(RegexValidator::new(pattern)?));
        Ok(self)
    }

    pub fn with_validator(mut self, validator: Box<dyn ComboValidator>) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn with_email_validator(self) -> Self {
        self.with_validator(Box::new(EmailUsernameValidator))
    }

    pub fn with_password_length(self, min_length: usize, max_length: usize) -> Self {
        self.with_validator(Box::new(PasswordLengthValidator::new(
            min_length, max_length,
        )))
    }

    pub fn load(&self) -> Result<()> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut combos = Vec::new();

        for line in reader.lines() {
            let line = line;

            if line.is_err() {
                continue;
            }

            let line = line.unwrap();

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if let Ok(combo) = Combo::from_raw(line, Some(&self.separator)) {
                if self.validate_combo(&combo) {
                    combos.push(line.to_string());
                }
            }
        }

        *self.combos.write() = combos;
        *self.position.write() = 0;

        Ok(())
    }

    fn validate_combo(&self, combo: &Combo) -> bool {
        if self.validators.is_empty() {
            return true;
        }

        for validator in &self.validators {
            if !validator.validate(combo) {
                return false;
            }
        }

        true
    }

    pub fn save_remaining(&self, path: impl AsRef<Path>) -> Result<usize> {
        let combos = self.combos.read();
        let position = *self.position.read();

        if position >= combos.len() {
            return Ok(0);
        }

        let remaining = &combos[position..];
        let mut file = File::create(path)?;

        use std::io::Write;
        for combo in remaining {
            writeln!(file, "{}", combo)?;
        }

        Ok(remaining.len())
    }
}

#[async_trait]
impl ComboProvider for FileComboProvider {
    async fn next(&self) -> Option<Combo> {
        let position;
        let raw;

        {
            let combos = self.combos.read();
            let mut pos = self.position.write();

            if *pos >= combos.len() {
                return None;
            }

            position = *pos;
            *pos += 1;

            raw = combos[position].clone();
        }

        match Combo::from_raw(raw, Some(&self.separator)) {
            Ok(combo) => Some(combo),
            Err(_) => self.next().await,
        }
    }

    async fn len(&self) -> usize {
        self.combos.read().len()
    }

    async fn remaining(&self) -> usize {
        let combos = self.combos.read();
        let position = *self.position.read();

        if position >= combos.len() {
            0
        } else {
            combos.len() - position
        }
    }

    async fn reset(&self) {
        *self.position.write() = 0;
    }
}
