use std::collections::HashMap;
use super::{ParameterValidator, ValidationResult};
use regex::Regex;

#[derive(Clone)]
pub struct BooleanValidator;

impl ParameterValidator for BooleanValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        match value {
            "" => ValidationResult::Valid, // Boolean parameters can be just present
            "0" | "1" | "true" | "false" | "on" | "off" | "yes" | "no" | "y" | "n" => {
                ValidationResult::Valid
            }
            _ => ValidationResult::Error(format!("Invalid boolean value: '{}'", value)),
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct IntegerValidator;

impl ParameterValidator for IntegerValidator {
    fn validate(&self, value: &str, config: &HashMap<String, toml::Value>) -> ValidationResult {
        let parsed = match value.parse::<i64>() {
            Ok(n) => n,
            Err(_) => return ValidationResult::Error(format!("Invalid integer: '{}'", value)),
        };

        // Check range if specified in config
        if let (Some(min), Some(max)) = (
            config.get("min").and_then(|v| v.as_integer()),
            config.get("max").and_then(|v| v.as_integer()),
        ) {
            if parsed < min || parsed > max {
                return ValidationResult::Error(format!(
                    "Integer {} out of range [{}, {}]", parsed, min, max
                ));
            }
        }

        ValidationResult::Valid
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct SizeValidator;

impl ParameterValidator for SizeValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        let size_regex = Regex::new(r"^(\d+)([KMG]?)$").unwrap();

        if let Some(captures) = size_regex.captures(value) {
            let _number: u64 = captures[1].parse().unwrap();
            let suffix = captures.get(2).map_or("", |m| m.as_str());

            match suffix {
                "" | "K" | "M" | "G" => ValidationResult::Valid,
                _ => ValidationResult::Error(format!("Invalid size suffix: '{}'", suffix)),
            }
        } else {
            ValidationResult::Error(format!("Invalid size format: '{}'", value))
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct EnumValidator;

impl ParameterValidator for EnumValidator {
    fn validate(&self, value: &str, config: &HashMap<String, toml::Value>) -> ValidationResult {
        let choices = match config.get("choices").and_then(|v| v.as_array()) {
            Some(arr) => arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>(),
            None => return ValidationResult::Error("No choices configured for enum".to_string()),
        };

        let allow_empty = config.get("allow_empty")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let allow_multiple = config.get("allow_multiple")

            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if value.is_empty() && allow_empty {
            return ValidationResult::Valid;
        }

        if allow_multiple {
            for part in value.split(',') {
                let trimmed = part.trim();
                if !choices.contains(&trimmed) {
                    return ValidationResult::Error(format!("Invalid choice: '{}'", trimmed));
                }
            }
        } else if !choices.contains(&value) {
            return ValidationResult::Error(format!("Invalid choice: '{}'. Valid: {:?}", value, choices));
        }

        ValidationResult::Valid
    }

    fn get_completion_suggestions(&self, partial: &str, config: &HashMap<String, toml::Value>) -> Vec<String> {
        config.get("choices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| s.starts_with(partial))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct HexValidator;

impl ParameterValidator for HexValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        let hex_regex = Regex::new(r"^0x[0-9a-fA-F]+$").unwrap();

        if hex_regex.is_match(value) {
            ValidationResult::Valid
        } else {
            ValidationResult::Error(format!("Invalid hex format: '{}'", value))
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct KeyValueValidator;

impl ParameterValidator for KeyValueValidator {
    fn validate(&self, value: &str, config: &HashMap<String, toml::Value>) -> ValidationResult {
        let separator = config.get("separator")
            .and_then(|v| v.as_str())
            .unwrap_or("=");

        if let Some((key, val)) = value.split_once(separator) {
            // Basic validation - could be enhanced with key/value validators
            if key.is_empty() {
                ValidationResult::Error("Empty key in key=value pair".to_string())
            } else if val.is_empty() {
                ValidationResult::Warning(format!("Empty value for key '{}'", key))
            } else {
                ValidationResult::Valid
            }
        } else {
            ValidationResult::Error(format!("Missing '{}' separator in key{}value pair", separator, separator))
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}
