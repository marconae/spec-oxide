//! Command handlers for the show subcommands.
//!
//! Provides `run_dashboard`, `run_spec_show`, and `run_change_show` functions
//! that display formatted spec and change information.

use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::show::{change, dashboard, spec};

/// Run the top-level `show` command to display the project dashboard.
///
/// # Returns
/// Returns `Ok(())` on success, or an error if config cannot be loaded
/// or dashboard cannot be generated.
pub fn run_dashboard() -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    let output = dashboard::show_dashboard(&config).map_err(Error::Other)?;
    println!("{}", output);
    Ok(())
}

/// Run the `spec show` command.
///
/// # Arguments
/// * `id` - Optional spec ID to show. If None, returns an error.
///
/// # Returns
/// Returns `Ok(())` on success, or an error if the spec is not found
/// or cannot be parsed.
pub fn run_spec_show(id: Option<String>) -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    match id {
        Some(spec_id) => {
            let spec_path = Path::new(config.spec_folder())
                .join(&spec_id)
                .join("spec.md");

            if !spec_path.exists() {
                return Err(Error::Other(format!("Spec '{}' not found", spec_id)));
            }

            let output = spec::show_spec(&spec_path).map_err(Error::Other)?;
            println!("{}", output);
        }
        None => {
            return Err(Error::Other(
                "Spec ID required. Use 'spox spec list' to see available specs.".to_string(),
            ));
        }
    }
    Ok(())
}

/// Run the `change show` command.
///
/// # Arguments
/// * `id` - Optional change ID to show. If None, returns an error.
/// * `deltas_only` - If true, only show delta requirements.
///
/// # Returns
/// Returns `Ok(())` on success, or an error if the change is not found
/// or cannot be parsed.
pub fn run_change_show(id: Option<String>, deltas_only: bool) -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    match id {
        Some(change_id) => {
            let change_path = Path::new(config.changes_folder()).join(&change_id);

            if !change_path.exists() {
                return Err(Error::Other(format!("Change '{}' not found", change_id)));
            }

            let output = if deltas_only {
                change::show_change_deltas_only(&change_path)
            } else {
                change::show_change(&change_path)
            }
            .map_err(Error::Other)?;

            println!("{}", output);
        }
        None => {
            return Err(Error::Other(
                "Change ID required. Use 'spox change list' to see active changes.".to_string(),
            ));
        }
    }
    Ok(())
}

// Note: Unit tests for show_cmd are not included here because they require
// changing the current directory, which causes race conditions in parallel tests.
// The functionality is fully tested by integration tests in tests/spec_show.rs,
// tests/change_show.rs, and tests/dashboard_show.rs.
