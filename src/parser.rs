use crate::parameter::Parameter;
use crate::validators::ValidationResult;
use crate::catalog::ParameterCatalog;
use crate::probe::SystemProbe;

#[derive(Debug, Clone)]
pub struct ParsedParameter {
    pub name: String,
    pub value: Option<String>,
    pub subparameters: Vec<ParsedParameter>,
    pub validation: ValidationResult,
    pub parameter_def: Option<Parameter>,
}

#[derive(Debug, Clone)]
pub struct ParsedCommandLine {
    pub parameters: Vec<ParsedParameter>,
    pub unknown_parameters: Vec<String>,
    pub validation_summary: crate::validators::ValidationSummary,
}

pub struct CommandLineParser<'a> {
    catalog: &'a ParameterCatalog,
}

pub struct ParameterParser<'a> {
    catalog: &'a ParameterCatalog,
}

pub struct ParameterBuilder<'a> {
    parameter: &'a Parameter,
}

impl<'a> CommandLineParser<'a> {
    pub fn new(catalog: &'a ParameterCatalog) -> Self {
        Self { catalog }
    }

    pub fn parse(&self, _cmdline: &str) -> Result<ParsedCommandLine, crate::error::ParseError> {
        // Split command line and parse each parameter
        todo!()
    }

    pub fn validate(&self, _cmdline: &str, _probe: &SystemProbe) -> ValidationResult {
        // Parse and validate entire command line
        todo!()
    }
}

impl<'a> ParameterParser<'a> {
    pub fn new(catalog: &'a ParameterCatalog) -> Self {
        Self { catalog }
    }

    pub fn parse_single(&self, _input: &str) -> Result<ParsedParameter, crate::error::ParseError> {
        // Parse single parameter=value
        todo!()
    }
}

impl<'a> ParameterBuilder<'a> {
    pub fn new(parameter: &'a Parameter) -> Self {
        Self { parameter }
    }

    pub fn build(&self, _config: &crate::config::ParameterConfig) -> Result<String, crate::error::BuildError> {
        // Build parameter string from config
        todo!()
    }
}
