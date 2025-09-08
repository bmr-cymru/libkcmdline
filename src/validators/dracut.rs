use std::collections::HashMap;
use super::{ParameterValidator, ValidationResult};
use regex::Regex;

#[derive(Clone)]
pub struct DracutLvmLvValidator;

impl ParameterValidator for DracutLvmLvValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Format: rd.lvm.lv=<volume_group>/<logical_volume>
        if let Some((vg, lv)) = value.split_once('/') {
            if vg.is_empty() {
                ValidationResult::Error("Volume group name cannot be empty".to_string())
            } else if lv.is_empty() {
                ValidationResult::Error("Logical volume name cannot be empty".to_string())
            } else if vg.starts_with('/') {
                ValidationResult::Error("Volume group name cannot start with '/'".to_string())
            } else {
                ValidationResult::Valid
            }
        } else {
            ValidationResult::Error("Must be in format volume_group/logical_volume".to_string())
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DracutLuksNameValidator;

impl ParameterValidator for DracutLuksNameValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Format: rd.luks.name=<uuid>=<name>
        if let Some((uuid_part, name_part)) = value.split_once('=') {
            // Basic UUID validation
            let uuid_regex = Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$").unwrap();

            if !uuid_regex.is_match(uuid_part) {
                ValidationResult::Warning(format!("'{}' doesn't look like a valid UUID", uuid_part))
            } else if name_part.is_empty() {
                ValidationResult::Error("LUKS device name cannot be empty".to_string())
            } else if name_part.contains('/') {
                ValidationResult::Error("LUKS device name cannot contain '/'".to_string())
            } else {
                ValidationResult::Valid
            }
        } else {
            ValidationResult::Error("Must be in format uuid=name".to_string())
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DracutBreakValidator;

impl ParameterValidator for DracutBreakValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        let valid_breakpoints = [
            "cmdline", "pre-udev", "pre-trigger", "initqueue",
            "pre-mount", "mount", "pre-pivot", "cleanup"
        ];

        if value.is_empty() {
            // rd.break with no value breaks at initqueue
            ValidationResult::Valid
        } else if valid_breakpoints.contains(&value) {
            ValidationResult::Valid
        } else {
            ValidationResult::Error(format!(
                "Invalid break point: '{}'. Valid: {:?}", value, valid_breakpoints
            ))
        }
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DracutNetworkValidator;

impl ParameterValidator for DracutNetworkValidator {
    fn validate(&self, value: &str, _config: &HashMap<String, toml::Value>) -> ValidationResult {
        // Basic ip= parameter validation for dracut
        // Format is complex: ip=<client-IP>:[<peer>]:<gateway-IP>:<netmask>:<hostname>:<interface>:{none|off|dhcp|on|any|dhcp6|auto6|ibft}[:[<mtu>][:<macaddr>]]
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() < 7 {
            return ValidationResult::Error("IP configuration requires at least 7 colon-separated fields".to_string());
        }

        // Validate the boot protocol field (7th field)
        let boot_proto = parts[6];
        let valid_protos = ["none", "off", "dhcp", "on", "any", "dhcp6", "auto6", "ibft"];
        if !valid_protos.contains(&boot_proto) {
            return ValidationResult::Error(format!(
                "Invalid boot protocol: '{}'. Valid: {:?}", boot_proto, valid_protos
            ));
        }

        ValidationResult::Valid
    }

    fn clone_boxed(&self) -> Box<dyn ParameterValidator> {
        Box::new(self.clone())
    }
}
