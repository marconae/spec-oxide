//! Validation module for spec and change validation.
//!
//! This module provides types and utilities for validating spec files
//! and change proposals against formatting rules and business constraints.

pub mod change;
pub mod parser;
pub mod spec;
pub mod tasks;

// Re-export core validation types for backward compatibility and convenient access
pub use crate::core::validate::{Severity, ValidationReport};

// Re-export key types for convenient access (used by external consumers)
#[allow(unused_imports)]
pub use change::ChangeValidationResult;
#[allow(unused_imports)]
pub use tasks::TaskStats;
