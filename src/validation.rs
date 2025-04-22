use crate::Result;
use crate::combo::Combo;
use regex::Regex;

pub trait ComboValidator: Send + Sync {
    fn validate(&self, combo: &Combo) -> bool;
}

pub struct RegexValidator {
    pattern: Regex,
}

impl RegexValidator {
    pub fn new(pattern: &str) -> Result<Self> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
        })
    }
}

impl ComboValidator for RegexValidator {
    fn validate(&self, combo: &Combo) -> bool {
        self.pattern.is_match(&combo.raw)
    }
}

pub struct EmailUsernameValidator;

impl ComboValidator for EmailUsernameValidator {
    fn validate(&self, combo: &Combo) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(&combo.username)
    }
}

pub struct PasswordLengthValidator {
    min_length: usize,
    max_length: usize,
}

impl PasswordLengthValidator {
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            min_length,
            max_length,
        }
    }
}

impl ComboValidator for PasswordLengthValidator {
    fn validate(&self, combo: &Combo) -> bool {
        let password_len = combo.password.len();
        password_len >= self.min_length && password_len <= self.max_length
    }
}

pub struct Validators;

impl Validators {
    pub fn email() -> Box<dyn ComboValidator> {
        Box::new(EmailUsernameValidator)
    }

    pub fn password_length(min: usize, max: usize) -> Box<dyn ComboValidator> {
        Box::new(PasswordLengthValidator::new(min, max))
    }

    pub fn regex(pattern: &str) -> Result<Box<dyn ComboValidator>> {
        Ok(Box::new(RegexValidator::new(pattern)?))
    }

    pub fn all(validators: Vec<Box<dyn ComboValidator>>) -> Box<dyn ComboValidator> {
        Box::new(CombinedValidator {
            validators,
            require_all: true,
        })
    }

    pub fn any(validators: Vec<Box<dyn ComboValidator>>) -> Box<dyn ComboValidator> {
        Box::new(CombinedValidator {
            validators,
            require_all: false,
        })
    }
}

struct CombinedValidator {
    validators: Vec<Box<dyn ComboValidator>>,
    require_all: bool,
}

impl ComboValidator for CombinedValidator {
    fn validate(&self, combo: &Combo) -> bool {
        if self.validators.is_empty() {
            return true;
        }

        if self.require_all {
            self.validators.iter().all(|v| v.validate(combo))
        } else {
            self.validators.iter().any(|v| v.validate(combo))
        }
    }
}
