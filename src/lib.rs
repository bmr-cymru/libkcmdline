//! libkcmdline - Comprehensive Linux kernel command line parameter validation
//! This library provides parsing, validation, and documentation for kernel
//! command line parameters across multiple boot components including the
//! kernel, systemd, dracut, and others.

mod catalog;
mod config;
mod database;
mod error;
mod parameter;
mod parser;
mod probe;
mod query;
mod validators;
mod version;

pub use parameter::{Parameter, ParameterProcessor, SyntaxDefinition};
pub use query::{QueryParameters, QueryMode};
pub use catalog::{
    ParameterCatalog,
    ParameterTree,
    ParameterInfo,
    VendorVersion,
    VersionInfo
};
pub use parser::{CommandLineParser, ParsedParameter};
pub use validators::{ValidationResult, ParameterValidator, ValidationSummary};
pub use probe::{SystemProbe, HardwareProbe};
pub use error::{KCmdlineError, ValidationError, RegistryError};
pub use version::{KernelVersion, ComponentVersion};

/// Main library interface
pub struct KCmdline {
    catalog: ParameterCatalog,
    probe: SystemProbe,
}

impl KCmdline {
    /// Unified parameter querying interface
    pub fn query_parameters(&self, query: &QueryParameters) -> Vec<&Parameter> {
        self.catalog.parameters().into_iter()
            .filter(|param| self.matches_query(param, query))
            .collect()
    }

    pub fn check_name_condition(&self, _param: &Parameter, _query: &QueryParameters) -> bool {
        todo!()
    }

    pub fn check_processor_condition(&self, _param: &Parameter, _query: &QueryParameters) -> bool {
        todo!()
    }

    pub fn check_hardware_condition(&self, _param: &Parameter, _query: &QueryParameters) -> bool {
        todo!()
    }

    pub fn check_applicability_condition(&self, _param: &Parameter, _query: &QueryParameters) -> bool {
        todo!()
    }

    pub fn check_distribution_condition(&self, _param: &Parameter, _query: &QueryParameters) -> bool {
        todo!()
    }

    pub fn check_deprecated_condition(&self, _param: &Parameter, _query: &QueryParameters) -> bool {
        todo!()
    }

    pub fn check_flags_condition(&self, _param: &Parameter, _query: &QueryParameters) -> bool {
        todo!()
    }

    fn matches_query(&self, param: &Parameter, query: &QueryParameters) -> bool {
        let conditions = vec![
            self.check_name_condition(param, &query),
            self.check_processor_condition(param, &query),
            self.check_hardware_condition(param, query),
            self.check_applicability_condition(param, &query),
            self.check_distribution_condition(param, &query),
            self.check_deprecated_condition(param, &query),
            self.check_flags_condition(param, &query),
        ];

        match query.query_mode {
            QueryMode::And => conditions.iter().all(|&c| c),
            QueryMode::Or => conditions.iter().any(|&c| c),
        }
    }
    //
    // Convenience methods that build QueryParameters
    pub fn find_parameters(&self, pattern: &str) -> Vec<&Parameter> {
        let regex = regex::Regex::new(pattern).unwrap();
        self.query_parameters(&QueryParameters {
            name: Some(regex),
            ..Default::default()
        })
    }

    pub fn parameters_for_pci_device(&self, vendor_id: u16, device_id: u16) -> Vec<&Parameter> {
        self.query_parameters(&QueryParameters {
            pci_ids: vec![(vendor_id, device_id)],
            ..Default::default()
        })
    }
}
