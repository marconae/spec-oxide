//! Validation module for spec and change validation.
//!
//! This module provides types and utilities for validating spec files
//! and change proposals against formatting rules and business constraints.

pub mod change;
pub mod parser;
pub mod spec;
pub mod tasks;

// Re-export key types for convenient access (used by external consumers)
#[allow(unused_imports)]
pub use change::ChangeValidationResult;
#[allow(unused_imports)]
pub use tasks::TaskStats;

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Must fix - affects validity in both normal and strict modes.
    Error,
    /// Should fix - affects validity only in strict mode.
    Warning,
    /// Suggestion - never affects validity.
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// A single validation issue found during validation.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// The severity of the issue.
    pub severity: Severity,
    /// The file where the issue was found.
    pub file: String,
    /// The line number where the issue was found (if available).
    pub line: Option<usize>,
    /// A description of the issue.
    pub message: String,
}

impl ValidationIssue {
    /// Create a new validation issue.
    pub fn new(severity: Severity, file: &str, line: Option<usize>, message: &str) -> Self {
        Self {
            severity,
            file: file.to_string(),
            line,
            message: message.to_string(),
        }
    }

    /// Create an error issue.
    pub fn error(file: &str, line: Option<usize>, message: &str) -> Self {
        Self::new(Severity::Error, file, line, message)
    }

    /// Create a warning issue.
    pub fn warning(file: &str, line: Option<usize>, message: &str) -> Self {
        Self::new(Severity::Warning, file, line, message)
    }

    /// Create an info issue.
    pub fn info(file: &str, line: Option<usize>, message: &str) -> Self {
        Self::new(Severity::Info, file, line, message)
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = match self.line {
            Some(line) => format!("{}:{}", self.file, line),
            None => self.file.clone(),
        };
        write!(f, "  {:5}  {:<20} {}", self.severity, location, self.message)
    }
}

/// A report containing all validation issues found.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether the validation passed (no errors).
    pub valid: bool,
    /// All issues found during validation.
    pub issues: Vec<ValidationIssue>,
    /// Count of error-level issues.
    pub errors: usize,
    /// Count of warning-level issues.
    pub warnings: usize,
}

impl ValidationReport {
    /// Create a new empty validation report.
    pub fn new() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            errors: 0,
            warnings: 0,
        }
    }

    /// Add an error issue to the report.
    pub fn add_error(&mut self, file: &str, line: Option<usize>, message: &str) {
        self.issues.push(ValidationIssue::error(file, line, message));
        self.errors += 1;
        self.valid = false;
    }

    /// Add a warning issue to the report.
    pub fn add_warning(&mut self, file: &str, line: Option<usize>, message: &str) {
        self.issues.push(ValidationIssue::warning(file, line, message));
        self.warnings += 1;
    }

    /// Add an info issue to the report.
    pub fn add_info(&mut self, file: &str, line: Option<usize>, message: &str) {
        self.issues.push(ValidationIssue::info(file, line, message));
    }

    /// Check if the report is valid (no errors).
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Check if the report is valid in strict mode (no errors or warnings).
    pub fn is_valid_strict(&self) -> bool {
        self.valid && self.warnings == 0
    }

    /// Merge another report into this one.
    pub fn merge(&mut self, other: ValidationReport) {
        self.issues.extend(other.issues);
        self.errors += other.errors;
        self.warnings += other.warnings;
        if !other.valid {
            self.valid = false;
        }
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for issue in &self.issues {
            writeln!(f, "{}", issue)?;
        }

        if self.errors > 0 || self.warnings > 0 {
            writeln!(f)?;
            let error_word = if self.errors == 1 { "error" } else { "errors" };
            let warning_word = if self.warnings == 1 {
                "warning"
            } else {
                "warnings"
            };
            writeln!(
                f,
                "{} {}, {} {}",
                self.errors, error_word, self.warnings, warning_word
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Error), "ERROR");
        assert_eq!(format!("{}", Severity::Warning), "WARN");
        assert_eq!(format!("{}", Severity::Info), "INFO");
    }

    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue::error("spec.md", Some(10), "Missing section");
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.file, "spec.md");
        assert_eq!(issue.line, Some(10));
        assert_eq!(issue.message, "Missing section");
    }

    #[test]
    fn test_validation_issue_display_with_line() {
        let issue = ValidationIssue::error("spec.md", Some(10), "Missing section");
        let display = format!("{}", issue);
        assert!(display.contains("ERROR"));
        assert!(display.contains("spec.md:10"));
        assert!(display.contains("Missing section"));
    }

    #[test]
    fn test_validation_issue_display_without_line() {
        let issue = ValidationIssue::warning("spec.md", None, "Missing description");
        let display = format!("{}", issue);
        assert!(display.contains("WARN"));
        assert!(display.contains("spec.md"));
        assert!(display.contains("Missing description"));
    }

    #[test]
    fn test_report_new_is_valid() {
        let report = ValidationReport::new();
        assert!(report.is_valid());
        assert!(report.is_valid_strict());
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 0);
    }

    #[test]
    fn test_report_add_error() {
        let mut report = ValidationReport::new();
        report.add_error("spec.md", Some(1), "Error message");

        assert!(!report.is_valid());
        assert!(!report.is_valid_strict());
        assert_eq!(report.errors, 1);
        assert_eq!(report.warnings, 0);
        assert_eq!(report.issues.len(), 1);
    }

    #[test]
    fn test_report_add_warning() {
        let mut report = ValidationReport::new();
        report.add_warning("spec.md", Some(1), "Warning message");

        assert!(report.is_valid());
        assert!(!report.is_valid_strict());
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 1);
        assert_eq!(report.issues.len(), 1);
    }

    #[test]
    fn test_report_add_info() {
        let mut report = ValidationReport::new();
        report.add_info("spec.md", Some(1), "Info message");

        assert!(report.is_valid());
        assert!(report.is_valid_strict());
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 0);
        assert_eq!(report.issues.len(), 1);
    }

    #[test]
    fn test_report_merge() {
        let mut report1 = ValidationReport::new();
        report1.add_error("file1.md", Some(1), "Error 1");
        report1.add_warning("file1.md", Some(2), "Warning 1");

        let mut report2 = ValidationReport::new();
        report2.add_error("file2.md", Some(1), "Error 2");
        report2.add_info("file2.md", Some(2), "Info 1");

        report1.merge(report2);

        assert!(!report1.is_valid());
        assert_eq!(report1.errors, 2);
        assert_eq!(report1.warnings, 1);
        assert_eq!(report1.issues.len(), 4);
    }

    #[test]
    fn test_report_merge_valid_into_valid() {
        let mut report1 = ValidationReport::new();
        report1.add_warning("file1.md", None, "Warning");

        let report2 = ValidationReport::new();

        report1.merge(report2);

        assert!(report1.is_valid());
        assert!(!report1.is_valid_strict());
    }

    #[test]
    fn test_report_merge_invalid_into_valid() {
        let mut report1 = ValidationReport::new();

        let mut report2 = ValidationReport::new();
        report2.add_error("file.md", None, "Error");

        report1.merge(report2);

        assert!(!report1.is_valid());
    }
}
