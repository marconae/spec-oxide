use std::fmt;

/// Error type for the Spox CLI.
#[derive(Debug)]
pub enum Error {
    /// Indicates a feature or functionality that has not been implemented yet.
    NotImplemented(String),
    /// Configuration file was not found at the expected path.
    ConfigNotFound(String),
    /// Failed to parse the configuration file.
    ConfigParse(String),
    /// A required configuration field is missing.
    ConfigMissingField(String),
    /// An error occurred during initialization.
    Init(String),
    /// A generic error with a custom message.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotImplemented(msg) => write!(f, "not implemented: {}", msg),
            Error::ConfigNotFound(path) => write!(f, "config not found: {}", path),
            Error::ConfigParse(msg) => write!(f, "config parse error: {}", msg),
            Error::ConfigMissingField(field) => write!(f, "config missing field: {}", field),
            Error::Init(msg) => write!(f, "initialization error: {}", msg),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// A specialized Result type for Spox operations.
pub type Result<T> = std::result::Result<T, Error>;
