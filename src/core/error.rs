//! Error types for Spec Oxide core functionality.

use std::fmt;

/// Error type for Spec Oxide core operations.
#[derive(Debug)]
pub enum Error {
    /// A generic error with a custom message.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// A specialized Result type for Spox operations.
pub type Result<T> = std::result::Result<T, Error>;
