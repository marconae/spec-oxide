//! Terminal output utilities for show commands.
//!
//! Provides colors, box drawing, and progress bar formatting
//! with automatic TTY detection for graceful fallback.

pub mod change;
pub mod dashboard;
pub mod spec;

// Re-export rendering utilities from cli/render for backward compatibility
pub use crate::cli::render::{
    blue, box_header, centered_box_header, colored_progress_bar, cyan_bold, dim, green, is_tty,
    progress_bar, red, yellow, HEADER_WIDTH,
};
