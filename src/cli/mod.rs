//! CLI interface for Spec Oxide.
//!
//! This module contains the command-line interface implementation:
//!
//! - `commands.rs` - Argument parsing with clap
//! - `render.rs` - Terminal output formatting (colors, boxes, progress bars)
//!
//! The CLI layer delegates to Core for business logic and formats output
//! for the terminal.

pub mod commands;
pub mod render;

// Re-export key types for convenient access from main
pub use commands::{ChangeCommands, Cli, Commands, ConfigCommands, McpCommands, SpecCommands};
