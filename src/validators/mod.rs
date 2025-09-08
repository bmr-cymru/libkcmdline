use std::collections::HashMap;

use crate::parameter::ParameterProcessor;
use crate::error::RegistryError;

pub mod common;
pub mod kernel;
pub mod systemd;
pub mod dracut;
pub mod hardware;

pub use common::*;
pub use kernel::*;
pub use systemd::*;
pub use dracut::*;
pub use hardware::*;

pub trait ParameterValidator: Send + Sync {
    fn validate(&self, value: &str, config: &HashMap<String, toml::Value>) -> ValidationResult;
    fn get_completion_suggestions(&self, _partial: &str, _config: &HashMap<String, toml::Value>) -> Vec<String> {
        Vec::new()
    }
    fn clone_boxed(&self) -> Box<dyn ParameterValidator>;
}

pub trait ValidatorRegistry: Send + Sync {
    fn get_validator(&self, processor: &ParameterProcessor, name: &str) -> Option<Box<dyn ParameterValidator>>;
    fn register_validator(&mut self, name: String, validator: Box<dyn ParameterValidator>) -> Result<(), RegistryError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Warning(String),
    Error(String),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_parameters: usize,
    pub valid_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub unknown_count: usize,
    pub details: Vec<(String, ValidationResult)>,
}

pub struct StandardValidatorRegistry {
    common_validators: HashMap<String, Box<dyn ParameterValidator>>,
    processor_validators: HashMap<String, Box<dyn ParameterValidator>>, // Only for truly unique cases
}

impl StandardValidatorRegistry {
    pub fn new() -> Self {
        let mut common = HashMap::new();
        common.insert("boolean".to_string(), Box::new(common::BooleanValidator) as Box<dyn ParameterValidator>);
        common.insert("integer".to_string(), Box::new(common::IntegerValidator));
        common.insert("enum".to_string(), Box::new(common::EnumValidator));
        common.insert("size".to_string(), Box::new(common::SizeValidator));
        common.insert("hex".to_string(), Box::new(common::HexValidator));
        common.insert("key_value".to_string(), Box::new(common::KeyValueValidator));

        let mut processor_specific = HashMap::new();
        // Only truly unique validators that can't be handled by common ones
        processor_specific.insert("cpu_list".to_string(), Box::new(kernel::CpuListValidator) as Box<dyn ParameterValidator>);
        processor_specific.insert("memory_range".to_string(), Box::new(kernel::MemoryRangeValidator));
        processor_specific.insert("pci_device".to_string(), Box::new(hardware::PciDeviceSpecValidator));
        processor_specific.insert("dracut_luks_name".to_string(), Box::new(dracut::DracutLuksNameValidator));

        Self {
            common_validators: common,
            processor_validators: processor_specific,
        }
    }
}

impl ValidatorRegistry for StandardValidatorRegistry {
    fn get_validator(&self, _processor: &ParameterProcessor, name: &str) -> Option<Box<dyn ParameterValidator>> {
        // First try processor-specific validators
        if let Some(validator) = self.processor_validators.get(name) {
            return Some(validator.as_ref().clone_boxed());
        }
        // Fall back to common validators
        self.common_validators.get(name).map(|v| v.clone_boxed())
    }

    fn register_validator(&mut self, name: String, validator: Box<dyn ParameterValidator>) -> Result<(), RegistryError> {
        //if let Some(validator) = self.processor_validators.get::Box<dyn ParameterValidator>(name.as_ref()) {
        //    return Err(RegistryError::NameError(name));
        //}
        self.processor_validators.insert(name.to_string(), validator);
        Ok(())
    }
}
