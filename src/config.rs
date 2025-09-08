use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ParameterConfig {
    Boolean(bool),
    Integer(i64),
    String(String),
    List(Vec<String>),
    CpuList { cpus: Vec<u32>, flags: Vec<String> },
    Complex(HashMap<String, ParameterConfig>),
}

impl ParameterConfig {
    pub fn from_bool(value: bool) -> Self {
        Self::Boolean(value)
    }

    pub fn from_int(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn from_string<S: Into<String>>(value: S) -> Self {
        Self::String(value.into())
    }

    pub fn from_string_list(value: Vec<String>) -> Self {
        Self::List(value)
    }

    pub fn from_cpu_list(cpus: Vec<u32>) -> Self {
        Self::CpuList { cpus, flags: Vec::new() }
    }

    pub fn with_flags(mut self, flags: Vec<String>) -> Self {
        if let Self::CpuList { flags: ref mut cpu_flags, .. } = self {
            *cpu_flags = flags;
        }
        self
    }
}
