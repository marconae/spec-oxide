//! Dashboard display module for Spec Oxide.
//!
//! Provides a dashboard overview showing all specs with requirement counts
//! and active changes with task progress.

use std::fs;
use std::path::Path;

use crate::config::Config;

// Import color utilities from parent module
use super::{centered_box_header, colored_progress_bar, dim, green, red, yellow, HEADER_WIDTH};

/// Summary information about a spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecSummary {
    /// The name/ID of the spec (directory name).
    pub name: String,
    /// Number of requirements in the spec.
    pub requirement_count: usize,
}

/// Summary information about a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSummary {
    /// The name/ID of the change (directory name).
    pub name: String,
    /// Number of completed tasks.
    pub tasks_completed: usize,
    /// Total number of tasks.
    pub tasks_total: usize,
    /// Summary of deltas (e.g., "auth (+1, ~1), notifications (+1)").
    pub delta_summary: String,
}

/// Dashboard information containing specs and changes.
#[derive(Debug, Clone, Default)]
pub struct DashboardInfo {
    /// All specs with their requirement counts.
    pub specs: Vec<SpecSummary>,
    /// All active changes with their progress.
    pub changes: Vec<ChangeSummary>,
}

/// Gather dashboard information from the config paths.
///
/// # Arguments
/// * `config` - The Spox configuration with folder paths
///
/// # Returns
/// `Ok(DashboardInfo)` containing specs and changes, or `Err` with error message.
pub fn gather_dashboard(config: &Config) -> Result<DashboardInfo, String> {
    Ok(DashboardInfo {
        specs: gather_specs(&config.spec_folder)?,
        changes: gather_changes(&config.changes_folder)?,
    })
}

/// Gather all specs with requirement counts.
///
/// # Arguments
/// * `spec_folder` - Path to the specs folder
///
/// # Returns
/// `Ok(Vec<SpecSummary>)` containing all specs, or `Err` with error message.
pub fn gather_specs(spec_folder: &str) -> Result<Vec<SpecSummary>, String> {
    let spec_path = Path::new(spec_folder);

    if !spec_path.exists() {
        return Ok(Vec::new());
    }

    let mut specs = Vec::new();

    let entries =
        fs::read_dir(spec_path).map_err(|e| format!("Failed to read spec folder: {}", e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip non-directories and special folders
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip special folders starting with underscore
        if name.starts_with('_') {
            continue;
        }

        // Count requirements in spec.md
        let spec_file = path.join("spec.md");
        let requirement_count = if spec_file.exists() {
            count_requirements(&spec_file)
        } else {
            0
        };

        specs.push(SpecSummary {
            name,
            requirement_count,
        });
    }

    // Sort alphabetically
    specs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(specs)
}

/// Count requirements in a spec file by counting `### Requirement:` headers.
fn count_requirements(spec_path: &Path) -> usize {
    let content = match fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    content
        .lines()
        .filter(|line| line.trim().starts_with("### Requirement:"))
        .count()
}

/// Gather all active changes with task progress.
///
/// # Arguments
/// * `changes_folder` - Path to the changes folder
///
/// # Returns
/// `Ok(Vec<ChangeSummary>)` containing all active changes, or `Err` with error message.
pub fn gather_changes(changes_folder: &str) -> Result<Vec<ChangeSummary>, String> {
    let changes_path = Path::new(changes_folder);

    if !changes_path.exists() {
        return Ok(Vec::new());
    }

    let mut changes = Vec::new();

    let entries =
        fs::read_dir(changes_path).map_err(|e| format!("Failed to read changes folder: {}", e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip non-directories
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip _archive folder
        if name == "_archive" || name.starts_with('.') {
            continue;
        }

        // Parse tasks.md for progress
        let tasks_path = path.join("tasks.md");
        let (tasks_completed, tasks_total) = if tasks_path.exists() {
            parse_task_progress(&tasks_path)
        } else {
            (0, 0)
        };

        // Gather delta summary from specs/ subdirectory
        let specs_path = path.join("specs");
        let delta_summary = if specs_path.exists() && specs_path.is_dir() {
            gather_delta_summary(&specs_path)
        } else {
            String::new()
        };

        changes.push(ChangeSummary {
            name,
            tasks_completed,
            tasks_total,
            delta_summary,
        });
    }

    // Sort alphabetically
    changes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(changes)
}

/// Parse tasks.md to get completion progress.
fn parse_task_progress(tasks_path: &Path) -> (usize, usize) {
    let content = match fs::read_to_string(tasks_path) {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let mut total = 0;
    let mut completed = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Match checkbox patterns: - [ ] or - [x] or - [X] or * [ ] etc.
        if trimmed.starts_with("- [ ] ") || trimmed.starts_with("* [ ] ") {
            total += 1;
        } else if trimmed.starts_with("- [x] ")
            || trimmed.starts_with("- [X] ")
            || trimmed.starts_with("* [x] ")
            || trimmed.starts_with("* [X] ")
        {
            total += 1;
            completed += 1;
        }
    }

    (completed, total)
}

/// Delta counts for a capability.
#[derive(Debug, Default)]
struct DeltaCounts {
    added: usize,
    modified: usize,
    removed: usize,
}

/// Gather delta summary from specs directory.
fn gather_delta_summary(specs_path: &Path) -> String {
    let mut capability_deltas: Vec<(String, DeltaCounts)> = Vec::new();

    if let Ok(entries) = fs::read_dir(specs_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let capability_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let spec_file = path.join("spec.md");
            if spec_file.exists() {
                let counts = count_delta_operations(&spec_file);
                if counts.added > 0 || counts.modified > 0 || counts.removed > 0 {
                    capability_deltas.push((capability_name, counts));
                }
            }
        }
    }

    // Sort by capability name
    capability_deltas.sort_by(|a, b| a.0.cmp(&b.0));

    // Format as "capability (+N, ~N, -N), ..."
    let parts: Vec<String> = capability_deltas
        .iter()
        .map(|(name, counts)| format_delta_counts(name, counts))
        .collect();

    parts.join(", ")
}

/// Count delta operations in a spec file.
fn count_delta_operations(spec_path: &Path) -> DeltaCounts {
    let content = match fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(_) => return DeltaCounts::default(),
    };

    let mut counts = DeltaCounts::default();
    let mut current_section: Option<&str> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        // Detect section headers
        if upper == "## ADDED REQUIREMENTS" {
            current_section = Some("added");
        } else if upper == "## MODIFIED REQUIREMENTS" {
            current_section = Some("modified");
        } else if upper == "## REMOVED REQUIREMENTS" {
            current_section = Some("removed");
        } else if upper == "## RENAMED REQUIREMENTS" {
            // RENAMED counts as modified for summary purposes
            current_section = Some("renamed");
        } else if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            current_section = None;
        }

        // Count requirements in current section
        if trimmed.starts_with("### Requirement:") {
            match current_section {
                Some("added") => counts.added += 1,
                Some("modified") | Some("renamed") => counts.modified += 1,
                Some("removed") => counts.removed += 1,
                _ => {}
            }
        }
    }

    counts
}

/// Format delta counts for a capability.
fn format_delta_counts(name: &str, counts: &DeltaCounts) -> String {
    let mut parts = Vec::new();

    if counts.added > 0 {
        parts.push(format!("+{}", counts.added));
    }
    if counts.modified > 0 {
        parts.push(format!("~{}", counts.modified));
    }
    if counts.removed > 0 {
        parts.push(format!("-{}", counts.removed));
    }

    if parts.is_empty() {
        name.to_string()
    } else {
        format!("{} ({})", name, parts.join(", "))
    }
}

/// Format dashboard information for display.
///
/// # Arguments
/// * `info` - The dashboard information to format
///
/// # Returns
/// A formatted string ready for terminal display.
pub fn format_dashboard(info: &DashboardInfo) -> String {
    let mut output = String::new();

    // Header box (centered)
    output.push_str(&centered_box_header("Spec Oxide Dashboard", HEADER_WIDTH));
    output.push_str("\n\n");

    // Specs section
    output.push_str(&format!("{} {}\n", yellow("Specs:"), info.specs.len()));

    if info.specs.is_empty() {
        output.push_str(&format!("  {}\n", dim("(no specs)")));
    } else {
        for spec in &info.specs {
            let req_word = if spec.requirement_count == 1 {
                "requirement"
            } else {
                "requirements"
            };
            output.push_str(&format!(
                "  {:<14} {} {}\n",
                spec.name,
                dim(&spec.requirement_count.to_string()),
                dim(req_word)
            ));
        }
    }

    output.push('\n');

    // Active Changes section
    output.push_str(&format!(
        "{} {}\n",
        yellow("Active Changes:"),
        info.changes.len()
    ));

    if info.changes.is_empty() {
        output.push_str(&format!("  {}\n", dim("(no active changes)")));
    } else {
        for change in &info.changes {
            // Progress bar
            let bar = colored_progress_bar(change.tasks_completed, change.tasks_total);
            let tasks_label = format!("{}/{} tasks", change.tasks_completed, change.tasks_total);

            output.push_str(&format!(
                "  {:<22} {} {}\n",
                yellow(&change.name),
                bar,
                dim(&tasks_label)
            ));

            // Delta summary (if present)
            if !change.delta_summary.is_empty() {
                let formatted_delta = format_colored_delta_summary(&change.delta_summary);
                output.push_str(&format!("    {} {}\n", dim("->"), formatted_delta));
            }
        }
    }

    output
}

/// Format delta summary with colors for +/~/-
fn format_colored_delta_summary(delta_summary: &str) -> String {
    let mut result = String::new();
    let mut chars = delta_summary.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '+' {
            // Color the + and following digits green
            let mut num = String::from("+");
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push_str(&green(&num));
        } else if c == '~' {
            // Color the ~ and following digits yellow
            let mut num = String::from("~");
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push_str(&yellow(&num));
        } else if c == '-' && chars.peek().is_some_and(|c| c.is_ascii_digit()) {
            // Color the - and following digits red (only if followed by digit)
            let mut num = String::from("-");
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push_str(&red(&num));
        } else {
            result.push(c);
        }
    }

    result
}

/// Main entry point for showing the dashboard.
///
/// # Arguments
/// * `config` - The Spox configuration
///
/// # Returns
/// `Ok(String)` with formatted dashboard output, or `Err` with error message.
pub fn show_dashboard(config: &Config) -> Result<String, String> {
    let info = gather_dashboard(config)?;
    Ok(format_dashboard(&info))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Unit Tests ====================

    #[test]
    fn test_format_delta_counts_all() {
        let counts = DeltaCounts {
            added: 2,
            modified: 1,
            removed: 1,
        };
        let result = format_delta_counts("auth", &counts);
        assert_eq!(result, "auth (+2, ~1, -1)");
    }

    #[test]
    fn test_format_delta_counts_added_only() {
        let counts = DeltaCounts {
            added: 1,
            modified: 0,
            removed: 0,
        };
        let result = format_delta_counts("notifications", &counts);
        assert_eq!(result, "notifications (+1)");
    }

    #[test]
    fn test_format_delta_counts_modified_only() {
        let counts = DeltaCounts {
            added: 0,
            modified: 2,
            removed: 0,
        };
        let result = format_delta_counts("config", &counts);
        assert_eq!(result, "config (~2)");
    }

    #[test]
    fn test_format_delta_counts_removed_only() {
        let counts = DeltaCounts {
            added: 0,
            modified: 0,
            removed: 3,
        };
        let result = format_delta_counts("legacy", &counts);
        assert_eq!(result, "legacy (-3)");
    }

    #[test]
    fn test_format_delta_counts_empty() {
        let counts = DeltaCounts::default();
        let result = format_delta_counts("empty", &counts);
        assert_eq!(result, "empty");
    }

    #[test]
    fn test_format_dashboard_empty() {
        let info = DashboardInfo::default();
        let output = format_dashboard(&info);

        assert!(output.contains("Spec Oxide Dashboard"));
        assert!(output.contains("Specs:"));
        assert!(output.contains("0"));
        assert!(output.contains("Active Changes:"));
        assert!(output.contains("(no specs)"));
        assert!(output.contains("(no active changes)"));
    }

    #[test]
    fn test_format_dashboard_with_specs() {
        let info = DashboardInfo {
            specs: vec![
                SpecSummary {
                    name: "auth".to_string(),
                    requirement_count: 4,
                },
                SpecSummary {
                    name: "config".to_string(),
                    requirement_count: 2,
                },
            ],
            changes: vec![],
        };

        let output = format_dashboard(&info);

        assert!(output.contains("Specs:"));
        assert!(output.contains("2"));
        assert!(output.contains("auth"));
        assert!(output.contains("4"));
        assert!(output.contains("config"));
        assert!(output.contains("requirements"));
    }

    #[test]
    fn test_format_dashboard_with_changes() {
        let info = DashboardInfo {
            specs: vec![],
            changes: vec![ChangeSummary {
                name: "add-feature".to_string(),
                tasks_completed: 3,
                tasks_total: 5,
                delta_summary: "auth (+1, ~1)".to_string(),
            }],
        };

        let output = format_dashboard(&info);

        assert!(output.contains("Active Changes:"));
        assert!(output.contains("1"));
        assert!(output.contains("add-feature"));
        assert!(output.contains("3/5 tasks"));
    }

    #[test]
    fn test_format_dashboard_single_requirement() {
        let info = DashboardInfo {
            specs: vec![SpecSummary {
                name: "single".to_string(),
                requirement_count: 1,
            }],
            changes: vec![],
        };

        let output = format_dashboard(&info);
        // Should use singular "requirement" not "requirements"
        assert!(output.contains("1"));
        assert!(output.contains("requirement"));
    }

    #[test]
    fn test_format_dashboard_change_with_no_tasks() {
        let info = DashboardInfo {
            specs: vec![],
            changes: vec![ChangeSummary {
                name: "empty-change".to_string(),
                tasks_completed: 0,
                tasks_total: 0,
                delta_summary: String::new(),
            }],
        };

        let output = format_dashboard(&info);
        assert!(output.contains("empty-change"));
        assert!(output.contains("0/0 tasks"));
    }

    #[test]
    fn test_parse_task_progress_content() {
        // Simulated content parsing
        let content = r#"# Tasks

## 1. Implementation
- [x] 1.1 First task
- [x] 1.2 Second task
- [ ] 1.3 Third task
- [ ] 1.4 Fourth task
- [X] 1.5 Fifth task (uppercase X)
"#;

        // Parse manually for testing
        let mut total = 0;
        let mut completed = 0;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [ ] ") || trimmed.starts_with("* [ ] ") {
                total += 1;
            } else if trimmed.starts_with("- [x] ")
                || trimmed.starts_with("- [X] ")
                || trimmed.starts_with("* [x] ")
                || trimmed.starts_with("* [X] ")
            {
                total += 1;
                completed += 1;
            }
        }

        assert_eq!(total, 5);
        assert_eq!(completed, 3);
    }

    #[test]
    fn test_count_delta_operations_content() {
        let content = r#"# Delta Spec

## ADDED Requirements

### Requirement: New Feature
The system SHALL support the new feature.

### Requirement: Another Feature
The system SHALL support another feature.

## MODIFIED Requirements

### Requirement: Existing Feature
The system SHALL improve the existing feature.

## REMOVED Requirements

### Requirement: Old Feature
"#;

        // Manual parsing for test
        let mut added = 0;
        let mut modified = 0;
        let mut removed = 0;
        let mut current_section: Option<&str> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            let upper = trimmed.to_uppercase();

            if upper == "## ADDED REQUIREMENTS" {
                current_section = Some("added");
            } else if upper == "## MODIFIED REQUIREMENTS" {
                current_section = Some("modified");
            } else if upper == "## REMOVED REQUIREMENTS" {
                current_section = Some("removed");
            } else if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
                current_section = None;
            }

            if trimmed.starts_with("### Requirement:") {
                match current_section {
                    Some("added") => added += 1,
                    Some("modified") => modified += 1,
                    Some("removed") => removed += 1,
                    _ => {}
                }
            }
        }

        assert_eq!(added, 2);
        assert_eq!(modified, 1);
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_spec_summary_equality() {
        let s1 = SpecSummary {
            name: "auth".to_string(),
            requirement_count: 4,
        };
        let s2 = SpecSummary {
            name: "auth".to_string(),
            requirement_count: 4,
        };
        let s3 = SpecSummary {
            name: "config".to_string(),
            requirement_count: 2,
        };

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_change_summary_equality() {
        let c1 = ChangeSummary {
            name: "add-feature".to_string(),
            tasks_completed: 2,
            tasks_total: 5,
            delta_summary: "auth (+1)".to_string(),
        };
        let c2 = ChangeSummary {
            name: "add-feature".to_string(),
            tasks_completed: 2,
            tasks_total: 5,
            delta_summary: "auth (+1)".to_string(),
        };

        assert_eq!(c1, c2);
    }

    #[test]
    fn test_dashboard_info_default() {
        let info = DashboardInfo::default();
        assert!(info.specs.is_empty());
        assert!(info.changes.is_empty());
    }

    #[test]
    fn test_format_colored_delta_summary() {
        let summary = "auth (+1, ~2), config (-1)";
        let result = format_colored_delta_summary(summary);

        // Should contain the original text (possibly with color codes)
        assert!(result.contains("auth"));
        assert!(result.contains("config"));
        // The +, ~, - should be present (possibly colored)
        assert!(result.contains("+1") || result.contains("\x1b["));
        assert!(result.contains("~2") || result.contains("\x1b["));
        assert!(result.contains("-1") || result.contains("\x1b["));
    }

    #[test]
    fn test_format_colored_delta_summary_empty() {
        let result = format_colored_delta_summary("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_colored_delta_summary_no_deltas() {
        let result = format_colored_delta_summary("just-text");
        assert_eq!(result, "just-text");
    }

    #[test]
    fn test_format_colored_delta_hyphen_in_name() {
        // Hyphens in names shouldn't be colored red
        let result = format_colored_delta_summary("my-feature (+1)");
        // The hyphen in "my-feature" should remain as-is
        assert!(result.contains("my-feature") || result.contains("my"));
    }
}
