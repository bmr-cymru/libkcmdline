use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub processor: ParameterProcessor,
    pub description: String,
    pub deprecated: bool,
    pub selectors: Vec<String>,
    pub syntax: SyntaxDefinition,
    pub distributions: HashMap<String, DistributionSupport>,
    pub examples: Examples,
    pub documentation: Option<DocumentationLinks>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ParameterProcessor {
    Kernel,
    Systemd { min_version: String },
    Dracut { min_version: String },
    InitramfsTools,
    Plymouth,
    Grub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxDefinition {
    pub validator_type: String,
    pub format: String,
    pub config: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSupport {
    pub min_version: Option<String>,
    pub max_version: Option<String>,
    pub component_version: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Examples {
    pub valid: Vec<String>,
    pub invalid: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationLinks {
    pub kernel_org: Option<String>,
    pub man_pages: Vec<String>,
    pub distribution_docs: HashMap<String, String>,
}

impl Parameter {
    pub fn is_applicable(&self, _probe: &crate::probe::SystemProbe) -> bool {
        // Implementation for checking applicability
        todo!()
    }

    pub fn is_available_in_distribution(&self, _distro: &str, _version: &str) -> bool {
        // Implementation for distribution checking
        todo!()
    }
}
