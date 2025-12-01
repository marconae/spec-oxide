//! Command handlers for the validate subcommands.
//!
//! Provides `run_spec_validate` and `run_change_validate` functions
//! that handle validation of specs and changes respectively.

use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::error::Result;
use crate::validate::{change, spec, Severity, ValidationReport};

/// Run the `spec validate` command.
///
/// If `id` is provided, validates that single spec.
/// Otherwise, validates all specs in the configured spec folder.
///
/// # Arguments
/// * `id` - Optional spec ID to validate
/// * `strict` - If true, warnings also cause validation failure
///
/// # Returns
/// Returns `Ok(())` if validation passes, exits with code 1 if validation fails.
pub fn run_spec_validate(id: Option<String>, strict: bool) -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    if let Some(spec_id) = id {
        // Validate single spec
        let spec_path = Path::new(&config.spec_folder).join(&spec_id).join("spec.md");

        if !spec_path.exists() {
            eprintln!("Spec '{}' not found at {}", spec_id, spec_path.display());
            std::process::exit(1);
        }

        let report = spec::validate_spec(&spec_path);
        print_spec_report(&spec_id, &report, strict);
        exit_on_failure(&report, strict);
    } else {
        // Validate all specs
        run_bulk_spec_validation(&config.spec_folder, strict)?;
    }

    Ok(())
}

/// Run the `change validate` command.
///
/// If `id` is provided, validates that single change.
/// Otherwise, validates all changes in the configured changes folder.
///
/// # Arguments
/// * `id` - Optional change ID to validate
/// * `strict` - If true, warnings also cause validation failure
///
/// # Returns
/// Returns `Ok(())` if validation passes, exits with code 1 if validation fails.
pub fn run_change_validate(id: Option<String>, strict: bool) -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    if let Some(change_id) = id {
        // Validate single change
        let change_path = Path::new(&config.changes_folder).join(&change_id);

        if !change_path.exists() {
            eprintln!("Change '{}' not found at {}", change_id, change_path.display());
            std::process::exit(1);
        }

        let result = change::validate_change(&change_path);
        print_change_report(&change_id, &result.report, strict);
        exit_on_failure(&result.report, strict);

        // Print success message if valid
        if result.report.is_valid() && (!strict || result.report.is_valid_strict()) {
            println!("\nChange '{}' is valid", change_id);
        }
    } else {
        // Validate all changes
        run_bulk_change_validation(&config.changes_folder, strict)?;
    }

    Ok(())
}

/// Run bulk validation for all specs in the spec folder.
fn run_bulk_spec_validation(spec_folder: &str, strict: bool) -> Result<()> {
    let spec_path = Path::new(spec_folder);

    if !spec_path.exists() {
        eprintln!("Spec folder '{}' does not exist", spec_folder);
        std::process::exit(1);
    }

    println!("Validating specs...");

    let mut total_validated = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut any_failed = false;

    // Read all subdirectories in spec folder
    let entries: Vec<_> = match fs::read_dir(spec_path) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            eprintln!("Failed to read spec folder: {}", e);
            std::process::exit(1);
        }
    };

    // Sort entries for consistent output
    let mut sorted_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            // Skip directories starting with _ (like _changes, _archive)
            let name = e.file_name().to_string_lossy().to_string();
            !name.starts_with('_')
        })
        .collect();
    sorted_entries.sort_by_key(|e| e.file_name());

    for entry in sorted_entries {
        let spec_id = entry.file_name().to_string_lossy().to_string();
        let spec_md_path = entry.path().join("spec.md");

        if !spec_md_path.exists() {
            continue; // Skip directories without spec.md
        }

        let report = spec::validate_spec(&spec_md_path);
        total_validated += 1;
        total_errors += report.errors;
        total_warnings += report.warnings;

        let status = format_bulk_status(&report, strict);
        println!("  {:<20} {}", spec_id, status);

        if !report.is_valid() || (strict && !report.is_valid_strict()) {
            any_failed = true;
        }
    }

    println!();
    println!(
        "{} specs validated, {} errors, {} warnings",
        total_validated, total_errors, total_warnings
    );

    if any_failed {
        std::process::exit(1);
    }

    Ok(())
}

/// Run bulk validation for all changes in the changes folder.
fn run_bulk_change_validation(changes_folder: &str, strict: bool) -> Result<()> {
    let changes_path = Path::new(changes_folder);

    if !changes_path.exists() {
        eprintln!("Changes folder '{}' does not exist", changes_folder);
        std::process::exit(1);
    }

    println!("Validating changes...");

    let mut total_validated = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut any_failed = false;

    // Read all subdirectories in changes folder
    let entries: Vec<_> = match fs::read_dir(changes_path) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            eprintln!("Failed to read changes folder: {}", e);
            std::process::exit(1);
        }
    };

    // Sort entries for consistent output
    let mut sorted_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            // Skip archive directory
            let name = e.file_name().to_string_lossy().to_string();
            name != "archive"
        })
        .collect();
    sorted_entries.sort_by_key(|e| e.file_name());

    for entry in sorted_entries {
        let change_id = entry.file_name().to_string_lossy().to_string();
        let change_path = entry.path();

        // Check if it looks like a change directory (has proposal.md)
        if !change_path.join("proposal.md").exists() {
            continue;
        }

        let result = change::validate_change(&change_path);
        total_validated += 1;
        total_errors += result.report.errors;
        total_warnings += result.report.warnings;

        let status = format_bulk_status(&result.report, strict);
        println!("  {:<20} {}", change_id, status);

        if !result.report.is_valid() || (strict && !result.report.is_valid_strict()) {
            any_failed = true;
        }
    }

    println!();
    println!(
        "{} changes validated, {} errors, {} warnings",
        total_validated, total_errors, total_warnings
    );

    if any_failed {
        std::process::exit(1);
    }

    Ok(())
}

/// Format the status string for bulk validation output.
fn format_bulk_status(report: &ValidationReport, strict: bool) -> String {
    if !report.is_valid() {
        if report.errors == 1 {
            "FAIL (1 error)".to_string()
        } else {
            format!("FAIL ({} errors)", report.errors)
        }
    } else if strict && !report.is_valid_strict() {
        if report.warnings == 1 {
            "FAIL (1 warning)".to_string()
        } else {
            format!("FAIL ({} warnings)", report.warnings)
        }
    } else if report.warnings > 0 {
        if report.warnings == 1 {
            "PASS (1 warning)".to_string()
        } else {
            format!("PASS ({} warnings)", report.warnings)
        }
    } else {
        "PASS".to_string()
    }
}

/// Print a validation report for a single spec.
fn print_spec_report(spec_id: &str, report: &ValidationReport, _strict: bool) {
    println!("Validating spec: {}", spec_id);
    print_report_issues(report);
}

/// Print a validation report for a single change.
fn print_change_report(change_id: &str, report: &ValidationReport, _strict: bool) {
    println!("Validating change: {}", change_id);
    print_report_issues(report);
}

/// Print the issues from a validation report.
fn print_report_issues(report: &ValidationReport) {
    for issue in &report.issues {
        let location = match issue.line {
            Some(line) => format!("{}:{}", issue.file, line),
            None => issue.file.clone(),
        };

        let severity_str = match issue.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
            Severity::Info => "INFO",
        };

        println!("  {:5}  {:<20} {}", severity_str, location, issue.message);
    }

    if report.errors > 0 || report.warnings > 0 {
        println!();
        let error_word = if report.errors == 1 { "error" } else { "errors" };
        let warning_word = if report.warnings == 1 { "warning" } else { "warnings" };
        println!("{} {}, {} {}", report.errors, error_word, report.warnings, warning_word);
    }
}

/// Exit with code 1 if the report indicates failure.
fn exit_on_failure(report: &ValidationReport, strict: bool) {
    let failed = if strict {
        !report.is_valid_strict()
    } else {
        !report.is_valid()
    };

    if failed {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests for run_spec_validate and run_change_validate require
    // changing the current directory, which is problematic in parallel test execution.
    // Integration tests should be used for end-to-end command testing.
    // Here we test the helper functions that don't require directory changes.

    #[test]
    fn test_format_bulk_status_pass() {
        let report = ValidationReport::new();
        assert_eq!(format_bulk_status(&report, false), "PASS");
        assert_eq!(format_bulk_status(&report, true), "PASS");
    }

    #[test]
    fn test_format_bulk_status_pass_with_warnings() {
        let mut report = ValidationReport::new();
        report.add_warning("test.md", None, "Test warning");

        assert_eq!(format_bulk_status(&report, false), "PASS (1 warning)");
        assert_eq!(format_bulk_status(&report, true), "FAIL (1 warning)");
    }

    #[test]
    fn test_format_bulk_status_pass_with_multiple_warnings() {
        let mut report = ValidationReport::new();
        report.add_warning("test.md", None, "Warning 1");
        report.add_warning("test.md", None, "Warning 2");

        assert_eq!(format_bulk_status(&report, false), "PASS (2 warnings)");
        assert_eq!(format_bulk_status(&report, true), "FAIL (2 warnings)");
    }

    #[test]
    fn test_format_bulk_status_fail() {
        let mut report = ValidationReport::new();
        report.add_error("test.md", None, "Test error");

        assert_eq!(format_bulk_status(&report, false), "FAIL (1 error)");
        assert_eq!(format_bulk_status(&report, true), "FAIL (1 error)");
    }

    #[test]
    fn test_format_bulk_status_fail_multiple_errors() {
        let mut report = ValidationReport::new();
        report.add_error("test.md", None, "Error 1");
        report.add_error("test.md", None, "Error 2");

        assert_eq!(format_bulk_status(&report, false), "FAIL (2 errors)");
        assert_eq!(format_bulk_status(&report, true), "FAIL (2 errors)");
    }
}
