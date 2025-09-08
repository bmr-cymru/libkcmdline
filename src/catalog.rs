use std::collections::HashMap;

use crate::parameter::Parameter;
use crate::query::QueryParameters;
use crate::probe::SystemProbe;

pub struct ParameterCatalog {
    parameters: HashMap<String, Parameter>,
    subparameter_index: HashMap<String, Vec<String>>, // parent -> children
}

#[derive(Debug, Clone)]
pub struct ParameterTree {
    pub root: Parameter,
    pub children: HashMap<String, ParameterTree>,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub parameter: Parameter,
    pub subparameters: Vec<Parameter>,
    pub applicable: bool,
    pub version_info: VersionInfo,
}

#[derive(Debug, Clone)]
pub struct VendorVersion {
    pub introduced: Option<String>,
    pub commit: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub introduced: Option<String>,
    pub commit: Option<String>,
    pub last_modified: Option<String>,
    pub last_modified_commit: Option<String>,
    pub vendors: HashMap<String, HashMap<String, VendorVersion>>,
}

impl ParameterCatalog {
    pub fn load_embedded() -> Result<Self, crate::error::KCmdlineError> {
        // Load from compiled database
        todo!()
    }

    pub fn parameters(&self) -> Vec<&Parameter> {
        self.parameters.values().collect()
    }

    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    pub fn query_parameters(&self, _query: &QueryParameters) -> Vec<&Parameter> {
        // Implementation for querying parameters
        todo!()
    }

    pub fn get_applicable_parameters(&self, probe: &SystemProbe) -> Vec<&Parameter> {
        self.parameters.values()
            .filter(|param| param.is_applicable(probe))
            .collect()
    }

    pub fn get_subparameters(&self, parent_name: &str) -> Vec<&Parameter> {
        self.subparameter_index.get(parent_name)
            .map(|children| {
                children.iter()
                    .filter_map(|name| self.parameters.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn build_parameter_tree(&self, _name: &str) -> Option<ParameterTree> {
        // Build recursive tree structure
        todo!()
    }
}
