use std::collections::HashMap;
use regex::Regex;

use super::{ParameterValidator, ValidationResult};

#[derive(Clone)]
pub struct CpuListValidator;

impl ParameterValidator for CpuListValidator {
    fn validate(&self, value: &str, config: &HashMap<String, toml::Value>) -> ValidationResult {
        let supports_flags = config.get("supports_flags")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let supports_exclusion = config.get("supports_exclusion")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Parse flags if present (e.g., "nohz:1,2,3-5")
        let (flags, cpu_part) = if supports_flags && value.contains(':') {
            let parts: Vec<&str> = value.splitn(2, ':').collect();
            (Some(parts[0]), parts[1])
        } else {
            (None, value)
        };

        // Validate flags if present
        if let Some(flag_str) = flags {
            let valid_flags = config.get("valid_flags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            for flag in flag_str.split(',') {
                if !valid_flags.is_empty() && !valid_flags.contains(&flag.trim()) {
                    return ValidationResult::Error(format!("Invalid CPU flag: '{}'", flag));
                }
            }
        }

        // Validate CPU list part
        self.validate_cpu_list(cpu_part, supports_exclusion)
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

impl CpuListValidator {
    fn validate_cpu_list(&self, cpu_list: &str, supports_exclusion: bool) -> ValidationResult {
        let cpu_regex = Regex::new(r"^(\^?)(\d+)(-(\d+))?$").unwrap();

        for part in cpu_list.split(',') {
            let trimmed = part.trim();

            if let Some(captures) = cpu_regex.captures(trimmed) {
                let exclusion = captures.get(1).map_or("", |m| m.as_str());
                let start: u32 = captures[2].parse().unwrap();

                if exclusion == "^" && !supports_exclusion {
                    return ValidationResult::Error("CPU exclusion (^) not supported".to_string());
                }

                if let Some(end_match) = captures.get(4) {
                    let end: u32 = end_match.as_str().parse().unwrap();
                    if end <= start {
                        return ValidationResult::Error(format!("Invalid CPU range: {}-{}", start, end));
                    }
                }
            } else {
                return ValidationResult::Error(format!("Invalid CPU specification: '{}'", trimmed));
            }
        }

        ValidationResult::Valid
    }
}

#[derive(Clone)]
pub struct MemoryRangeValidator;

impl ParameterValidator for MemoryRangeValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Pattern: nn[KMG]@ss[KMG] or nn[KMG]#ss[KMG] or nn[KMG]$ss[KMG]
        let memory_regex = Regex::new(r"^(\d+)([KMG]?)[@#$](\d+)([KMG]?)$").unwrap();

        if memory_regex.is_match(value) {
            ValidationResult::Valid
        } else {
            ValidationResult::Error(format!("Invalid memory range format: '{}'", value))
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct IoRangeValidator;

impl ParameterValidator for IoRangeValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Pattern: <base>,<size> or <base>-<end>
        if value.contains(',') {
            let parts: Vec<&str> = value.split(',').collect();
            if parts.len() != 2 {
                return ValidationResult::Error("I/O range must be base,size".to_string());
            }
            //
            // Validate base and size are valid numbers (hex or decimal)
            for part in parts {
                if part.starts_with("0x") {
                    if u64::from_str_radix(&part[2..], 16).is_err() {
                        return ValidationResult::Error(format!("Invalid hex number: '{}'", part));
                    }
                } else if part.parse::<u64>().is_err() {
                    return ValidationResult::Error(format!("Invalid number: '{}'", part));
                }
            }

            ValidationResult::Valid
        } else {
            ValidationResult::Error("I/O range must specify base,size".to_string())
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}
