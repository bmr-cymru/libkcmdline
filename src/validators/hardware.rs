use std::collections::HashMap;
use super::{ParameterValidator, ValidationResult};
use regex::Regex;

#[derive(Clone)]
pub struct PciDeviceSpecValidator;

impl ParameterValidator for PciDeviceSpecValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Format: [<domain>:]<bus>:<dev>.<func>[/<dev>.<func>]*
        // or pci:<vendor>:<device>[:<subvendor>:<subdevice>]
        if value.starts_with("pci:") {
            self.validate_pci_id_format(&value[4..])
        } else {
            self.validate_pci_address_format(value)
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

impl PciDeviceSpecValidator {
    fn validate_pci_id_format(&self, value: &str) -> ValidationResult {
        // Format: <vendor>:<device>[:<subvendor>:<subdevice>]
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 2 && parts.len() != 4 {
            return ValidationResult::Error("PCI ID must be vendor:device or vendor:device:subvendor:subdevice".to_string());
        }

        for part in parts {
            if part.len() != 4 {
                return ValidationResult::Error("PCI IDs must be 4-digit hex".to_string());
            }
            if u16::from_str_radix(part, 16).is_err() {
                return ValidationResult::Error(format!("Invalid hex PCI ID: '{}'", part));
            }
        }

        ValidationResult::Valid
    }

    fn validate_pci_address_format(&self, value: &str) -> ValidationResult {
        // Format: [<domain>:]<bus>:<dev>.<func>
        let pci_regex = Regex::new(r"^(?:([0-9a-fA-F]{1,4}):)?([0-9a-fA-F]{1,2}):([0-9a-fA-F]{1,2})\.([0-7])$").unwrap();
        if pci_regex.is_match(value) {
            ValidationResult::Valid
        } else {
            ValidationResult::Error(format!("Invalid PCI address format: '{}'", value))
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct UsbDeviceSpecValidator;

impl ParameterValidator for UsbDeviceSpecValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Format: usb:<vendor>:<product>
        if !value.starts_with("usb:") {
            return ValidationResult::Error("USB device spec must start with 'usb:'".to_string());
        }

        let parts: Vec<&str> = value[4..].split(':').collect();
        if parts.len() != 2 {
            return ValidationResult::Error("USB device spec must be usb:vendor:product".to_string());
        }

        for part in parts {
            if part.len() != 4 {
                return ValidationResult::Error("USB vendor/product IDs must be 4-digit hex".to_string());
            }
            if u16::from_str_radix(part, 16).is_err() {
                return ValidationResult::Error(format!("Invalid hex USB ID: '{}'", part));
            }
        }

        ValidationResult::Valid
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DmiSpecValidator;

impl ParameterValidator for DmiSpecValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Format: dmi:<field>:<value>
        if !value.starts_with("dmi:") {
            return ValidationResult::Error("DMI spec must start with 'dmi:'".to_string());
        }

        let parts: Vec<&str> = value[4..].splitn(2, ':').collect();
        if parts.len() != 2 {
            return ValidationResult::Error("DMI spec must be dmi:field:value".to_string());
        }

        let valid_fields = [
            "vendor", "product", "version", "serial", "uuid", "sku",
            "family", "board_vendor", "board_name", "chassis_vendor"
        ];

        let field = parts[0];
        if !valid_fields.contains(&field) {
            return ValidationResult::Warning(format!(
                "Unknown DMI field: '{}'. Valid: {:?}", field, valid_fields
            ));
        }

        if parts[1].is_empty() {
            return ValidationResult::Error("DMI value cannot be empty".to_string());
        }

        ValidationResult::Valid
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}
