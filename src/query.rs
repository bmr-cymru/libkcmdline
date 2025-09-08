use regex::Regex;
use crate::parameter::ParameterProcessor;

#[derive(Debug, Clone, Default)]
pub struct QueryParameters {
    pub query_mode: QueryMode,
    pub name: Option<Regex>,
    pub processor: Option<ParameterProcessor>,
    pub pci_ids: Vec<(u16, u16)>,
    pub usb_ids: Vec<(u16, u16)>,
    pub arch: Option<String>,
    pub applicable: Option<bool>,
    pub distribution: Option<DistributionQuery>,
    pub deprecated: Option<bool>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryMode {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct DistributionQuery {
    pub id: String,
    pub version: Option<String>,
}

impl Default for QueryMode {
    fn default() -> Self {
        QueryMode::And
    }
}

impl QueryParameters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name_pattern(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.name = Some(Regex::new(pattern)?);
        Ok(self)
    }

    pub fn with_processor(mut self, processor: ParameterProcessor) -> Self {
        self.processor = Some(processor);
        self
    }

    pub fn applicable_only(mut self) -> Self {
        self.applicable = Some(true);
        self
    }
}
