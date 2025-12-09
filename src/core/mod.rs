//! Core business logic for Spec Oxide.
//!
//! This module contains the core functionality that is independent of
//! any specific interface (CLI, MCP, etc.). It provides:
//!
//! - Configuration management
//! - Error types
//! - Validation logic
//! - Spec parsing
//!
//! All functions in this module return structured Rust types (structs, enums, Results)
//! without any formatting or I/O operations.

pub mod config;
pub mod error;
pub mod index;
pub mod spec;
pub mod validate;
