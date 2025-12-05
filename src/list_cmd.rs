//! Command handlers for the list subcommands.
//!
//! Provides `run_spec_list` and `run_change_list` functions
//! that display formatted lists of specs and changes without ANSI colors.

use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::show::dashboard::{gather_changes, gather_specs, ChangeSummary, SpecSummary};

/// Format the list of specs for display.
///
/// # Arguments
/// * `specs` - The list of specs to format
///
/// # Returns
/// A formatted string ready for terminal display (no ANSI colors).
///
/// # Example Output
/// ```text
/// Specs:
/// - auth    2 requirements
/// - config  3 requirements
/// ```
/// Or if empty: `No specs found.`
fn format_spec_list(specs: &[SpecSummary]) -> String {
    if specs.is_empty() {
        return "No specs found.".to_string();
    }

    let mut output = String::from("Specs:\n");

    // Calculate the maximum name length for alignment
    let max_name_len = specs.iter().map(|s| s.name.len()).max().unwrap_or(0);

    for spec in specs {
        let req_word = if spec.requirement_count == 1 {
            "requirement"
        } else {
            "requirements"
        };
        output.push_str(&format!(
            "- {:<width$}  {} {}\n",
            spec.name,
            spec.requirement_count,
            req_word,
            width = max_name_len
        ));
    }

    // Remove trailing newline
    output.trim_end().to_string()
}

/// Format the list of changes for display.
///
/// # Arguments
/// * `changes` - The list of changes to format
///
/// # Returns
/// A formatted string ready for terminal display (no ANSI colors).
///
/// # Example Output
/// ```text
/// Changes:
/// - add-2fa    2/5 tasks
/// - fix-login  0/3 tasks
/// ```
/// Or if empty: `No active changes.`
fn format_change_list(changes: &[ChangeSummary]) -> String {
    if changes.is_empty() {
        return "No active changes.".to_string();
    }

    let mut output = String::from("Changes:\n");

    // Calculate the maximum name length for alignment
    let max_name_len = changes.iter().map(|c| c.name.len()).max().unwrap_or(0);

    for change in changes {
        output.push_str(&format!(
            "- {:<width$}  {}/{} tasks\n",
            change.name,
            change.tasks_completed,
            change.tasks_total,
            width = max_name_len
        ));
    }

    // Remove trailing newline
    output.trim_end().to_string()
}

/// Run the `spec list` command.
///
/// Displays all specs with their requirement counts in a simple list format
/// without ANSI colors.
///
/// # Returns
/// Returns `Ok(())` on success, or an error if config cannot be loaded.
pub fn run_spec_list() -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    let specs = gather_specs(config.spec_folder()).map_err(Error::Other)?;
    let output = format_spec_list(&specs);
    println!("{}", output);

    Ok(())
}

/// Run the `change list` command.
///
/// Displays all active changes with their task progress in a simple list format
/// without ANSI colors.
///
/// # Returns
/// Returns `Ok(())` on success, or an error if config cannot be loaded.
pub fn run_change_list() -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    let changes = gather_changes(config.changes_folder()).map_err(Error::Other)?;
    let output = format_change_list(&changes);
    println!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Unit Tests for format_spec_list ====================

    #[test]
    fn test_format_spec_list_with_multiple_specs() {
        let specs = vec![
            SpecSummary {
                name: "auth".to_string(),
                requirement_count: 2,
            },
            SpecSummary {
                name: "config".to_string(),
                requirement_count: 3,
            },
        ];

        let output = format_spec_list(&specs);

        assert!(output.starts_with("Specs:"));
        assert!(output.contains("- auth"));
        assert!(output.contains("2 requirements"));
        assert!(output.contains("- config"));
        assert!(output.contains("3 requirements"));
    }

    #[test]
    fn test_format_spec_list_with_no_specs() {
        let specs: Vec<SpecSummary> = vec![];

        let output = format_spec_list(&specs);

        assert_eq!(output, "No specs found.");
    }

    #[test]
    fn test_format_spec_list_singular_requirement() {
        let specs = vec![SpecSummary {
            name: "single".to_string(),
            requirement_count: 1,
        }];

        let output = format_spec_list(&specs);

        assert!(output.contains("1 requirement"));
        assert!(!output.contains("1 requirements"));
    }

    #[test]
    fn test_format_spec_list_zero_requirements() {
        let specs = vec![SpecSummary {
            name: "empty".to_string(),
            requirement_count: 0,
        }];

        let output = format_spec_list(&specs);

        assert!(output.contains("0 requirements"));
    }

    // ==================== Unit Tests for format_change_list ====================

    #[test]
    fn test_format_change_list_with_multiple_changes() {
        let changes = vec![
            ChangeSummary {
                name: "add-2fa".to_string(),
                tasks_completed: 2,
                tasks_total: 5,
                delta_summary: String::new(),
            },
            ChangeSummary {
                name: "fix-login".to_string(),
                tasks_completed: 0,
                tasks_total: 3,
                delta_summary: String::new(),
            },
        ];

        let output = format_change_list(&changes);

        assert!(output.starts_with("Changes:"));
        assert!(output.contains("- add-2fa"));
        assert!(output.contains("2/5 tasks"));
        assert!(output.contains("- fix-login"));
        assert!(output.contains("0/3 tasks"));
    }

    #[test]
    fn test_format_change_list_with_no_changes() {
        let changes: Vec<ChangeSummary> = vec![];

        let output = format_change_list(&changes);

        assert_eq!(output, "No active changes.");
    }

    #[test]
    fn test_format_change_list_completed_change() {
        let changes = vec![ChangeSummary {
            name: "done-feature".to_string(),
            tasks_completed: 5,
            tasks_total: 5,
            delta_summary: String::new(),
        }];

        let output = format_change_list(&changes);

        assert!(output.contains("5/5 tasks"));
    }

    #[test]
    fn test_format_change_list_zero_tasks() {
        let changes = vec![ChangeSummary {
            name: "no-tasks".to_string(),
            tasks_completed: 0,
            tasks_total: 0,
            delta_summary: String::new(),
        }];

        let output = format_change_list(&changes);

        assert!(output.contains("0/0 tasks"));
    }

    // ==================== Unit Tests for column alignment ====================

    #[test]
    fn test_format_spec_list_column_alignment() {
        let specs = vec![
            SpecSummary {
                name: "short".to_string(),
                requirement_count: 1,
            },
            SpecSummary {
                name: "much-longer-name".to_string(),
                requirement_count: 5,
            },
        ];

        let output = format_spec_list(&specs);
        let lines: Vec<&str> = output.lines().collect();

        // Skip the header line "Specs:"
        // The spec names should be aligned to the longest name
        // "short" should be padded to match "much-longer-name" length
        let short_line = lines.iter().find(|l| l.contains("short")).unwrap();
        let long_line = lines
            .iter()
            .find(|l| l.contains("much-longer-name"))
            .unwrap();

        // Find position of the requirement count in each line
        // Both should have the count at roughly the same position
        let short_count_pos = short_line.find("  1 ").unwrap();
        let long_count_pos = long_line.find("  5 ").unwrap();

        assert_eq!(
            short_count_pos, long_count_pos,
            "Requirement counts should be aligned"
        );
    }

    #[test]
    fn test_format_change_list_column_alignment() {
        let changes = vec![
            ChangeSummary {
                name: "ab".to_string(),
                tasks_completed: 1,
                tasks_total: 2,
                delta_summary: String::new(),
            },
            ChangeSummary {
                name: "very-long-change-name".to_string(),
                tasks_completed: 10,
                tasks_total: 20,
                delta_summary: String::new(),
            },
        ];

        let output = format_change_list(&changes);
        let lines: Vec<&str> = output.lines().collect();

        // Skip the header line "Changes:"
        let short_line = lines.iter().find(|l| l.contains("ab")).unwrap();
        let long_line = lines
            .iter()
            .find(|l| l.contains("very-long-change-name"))
            .unwrap();

        // Find position of the task count in each line
        let short_tasks_pos = short_line.find("1/2").unwrap();
        let long_tasks_pos = long_line.find("10/20").unwrap();

        // The shorter name should be padded so tasks start at similar position
        // Allow for digit width difference
        assert!(
            (short_tasks_pos as i32 - long_tasks_pos as i32).abs() <= 1,
            "Task counts should be roughly aligned"
        );
    }

    // ==================== Unit Tests for no ANSI colors ====================

    #[test]
    fn test_format_spec_list_no_ansi_colors() {
        let specs = vec![SpecSummary {
            name: "auth".to_string(),
            requirement_count: 4,
        }];

        let output = format_spec_list(&specs);

        // Should not contain ANSI escape sequences
        assert!(!output.contains("\x1b["));
        assert!(!output.contains("\x1b("));
    }

    #[test]
    fn test_format_change_list_no_ansi_colors() {
        let changes = vec![ChangeSummary {
            name: "add-feature".to_string(),
            tasks_completed: 2,
            tasks_total: 5,
            delta_summary: "auth (+1)".to_string(),
        }];

        let output = format_change_list(&changes);

        // Should not contain ANSI escape sequences
        assert!(!output.contains("\x1b["));
        assert!(!output.contains("\x1b("));
    }
}
