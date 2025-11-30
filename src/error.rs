use std::fmt;

/// Error type for the Spox CLI.
#[derive(Debug)]
pub enum Error {
    /// Indicates a feature or functionality that has not been implemented yet.
    NotImplemented(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotImplemented(msg) => write!(f, "not implemented: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// A specialized Result type for Spox operations.
pub type Result<T> = std::result::Result<T, Error>;
