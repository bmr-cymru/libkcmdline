use std::collections::HashMap;
use super::{ParameterValidator, ValidationResult};

// Only truly systemd-specific validators that can't be handled by common ones
#[derive(Clone)]
pub struct SystemdUnitValidator;

impl ParameterValidator for SystemdUnitValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        if value.is_empty() {
            return ValidationResult::Error("Unit name cannot be empty".to_string());
        }

        // Check for valid unit suffixes
        let valid_suffixes = [
            ".service", ".target", ".socket", ".timer", ".mount",
            ".automount", ".swap", ".path", ".slice", ".scope"
        ];

        if !valid_suffixes.iter().any(|suffix| value.ends_with(suffix)) {
            return ValidationResult::Warning(format!(
                "Unit '{}' doesn't have a recognized suffix", value
            ));
        }

        if value.contains('/') || value.contains('\\') {
            return ValidationResult::Error("Unit names cannot contain path separators".to_string());
        }

        ValidationResult::Valid
    }

    fn get_completion_suggestions(&self, partial: &str, _config: &HashMap<String, toml::Value>) -> Vec<String> {
        vec![
            "multi-user.target".to_string(),
            "graphical.target".to_string(),
            "rescue.target".to_string(),
            "emergency.target".to_string(),
        ].into_iter()
         .filter(|s| s.starts_with(partial))
         .collect()
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}
