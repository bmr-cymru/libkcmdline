use thiserror::Error;

#[derive(Error, Debug)]
pub enum KCmdlineError {
    #[error("Database loading failed: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("Hardware probe error: {0}")]
    ProbeError(#[from] ProbeError),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid parameter value: {0}")]
    InvalidValue(String),
    #[error("Unknown parameter: {0}")]
    UnknownParameter(String),
    #[error("Validator not found: {0}")]
    ValidatorNotFound(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Syntax error in parameter: {0}")]
    SyntaxError(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Missing required value for parameter: {0}")]
    MissingValue(String),
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Unknown parameter: {0}")]
    UnknownParameter(String),
    #[error("Invalid configuration for parameter {param}: {error}")]
    InvalidConfig { param: String, error: String },
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
}

#[derive(Error, Debug)]
pub enum ProbeError {
    #[error("I/O error during hardware probe: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse system information: {0}")]
    ParseError(String),
    #[error("Required system file not found: {0}")]
    MissingSystemFile(String),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("I/O error during database load: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to load parameter database: {0}")]
    LoadError(String),
    #[error("Database format error: {0}")]
    FormatError(String),
    #[error("Missing required parameter definition: {0}")]
    MissingDefinition(String),
}

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Name {0} is already in use")]
    NameError(String),
}
