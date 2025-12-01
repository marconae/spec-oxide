//! Change proposal validation module.
//!
//! Validates change directories including proposal.md and delta specs
//! for structural correctness and business rules.

use std::fs;
use std::path::Path;

use super::parser::{extract_scenarios, has_then_clause, has_when_clause};
use super::tasks::{validate_tasks, TaskStats};
use super::ValidationReport;

/// Minimum character length for Why section to avoid warnings.
const MIN_WHY_LENGTH: usize = 50;

/// Valid delta operation headers.
const DELTA_HEADERS: &[&str] = &[
    "## ADDED Requirements",
    "## MODIFIED Requirements",
    "## REMOVED Requirements",
    "## RENAMED Requirements",
];

/// Result of change validation including task statistics.
#[derive(Debug)]
pub struct ChangeValidationResult {
    /// The validation report with all issues found.
    pub report: ValidationReport,
    /// Task statistics (if tasks.md was found and validated).
    pub task_stats: Option<TaskStats>,
}

/// Validate a change directory at the given path.
///
/// The path should be the change directory (e.g., specs/_changes/add-feature/).
///
/// # Arguments
/// * `change_dir` - Path to the change directory
///
/// # Returns
/// A `ChangeValidationResult` containing the validation report and task statistics.
pub fn validate_change(change_dir: &Path) -> ChangeValidationResult {
    let mut report = ValidationReport::new();
    let dir_path = change_dir.to_string_lossy().to_string();

    // Check if change directory exists
    if !change_dir.exists() {
        report.add_error(
            &dir_path,
            None,
            &format!("Change directory does not exist: {}", dir_path),
        );
        return ChangeValidationResult {
            report,
            task_stats: None,
        };
    }

    if !change_dir.is_dir() {
        report.add_error(
            &dir_path,
            None,
            &format!("Path is not a directory: {}", dir_path),
        );
        return ChangeValidationResult {
            report,
            task_stats: None,
        };
    }

    // Check for proposal.md
    let proposal_path = change_dir.join("proposal.md");
    let proposal_file = "proposal.md".to_string();

    if !proposal_path.exists() {
        report.add_error(&proposal_file, None, "Missing proposal.md");
        return ChangeValidationResult {
            report,
            task_stats: None,
        };
    }

    // Read and validate proposal.md
    let proposal_content = match fs::read_to_string(&proposal_path) {
        Ok(c) => c,
        Err(e) => {
            report.add_error(
                &proposal_file,
                Some(1),
                &format!("Failed to read proposal.md: {}", e),
            );
            return ChangeValidationResult {
                report,
                task_stats: None,
            };
        }
    };

    validate_proposal(&proposal_content, &proposal_file, &mut report);

    // Check for and validate tasks.md
    let tasks_path = change_dir.join("tasks.md");
    let tasks_file = "tasks.md".to_string();
    let task_stats = if tasks_path.exists() {
        let (tasks_report, stats) = validate_tasks(&tasks_path);
        report.merge(tasks_report);

        // Add info about task completion statistics
        report.add_info(
            &tasks_file,
            None,
            &format!("{}", stats),
        );

        Some(stats)
    } else {
        report.add_error(&tasks_file, None, "Missing tasks.md");
        None
    };

    // Check for delta specs in specs/ subdirectory
    let specs_dir = change_dir.join("specs");
    let mut total_delta_ops = 0;

    if specs_dir.exists() && specs_dir.is_dir() {
        // Iterate through subdirectories in specs/
        match fs::read_dir(&specs_dir) {
            Ok(entries) => {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        // Look for spec.md in each subdirectory
                        let spec_path = entry_path.join("spec.md");
                        if spec_path.exists() {
                            let delta_ops =
                                validate_delta_spec(&spec_path, &entry_path, &mut report);
                            total_delta_ops += delta_ops;
                        }
                    }
                }
            }
            Err(e) => {
                report.add_error(
                    "specs/",
                    None,
                    &format!("Failed to read specs directory: {}", e),
                );
            }
        }
    }

    // Check that at least one delta spec exists
    if total_delta_ops == 0 {
        report.add_error(
            &proposal_file,
            None,
            "Change must have at least one delta spec with ADDED/MODIFIED/REMOVED/RENAMED operations",
        );
    }

    ChangeValidationResult {
        report,
        task_stats,
    }
}

/// Validate proposal.md content.
fn validate_proposal(content: &str, file_path: &str, report: &mut ValidationReport) {
    let lines: Vec<&str> = content.lines().collect();

    // Check for Why section
    let why_result = find_section(&lines, "Why");
    if why_result.is_none() {
        report.add_error(file_path, Some(1), "Missing Why section");
    } else {
        let (why_line, _) = why_result.unwrap();
        let why_text = extract_section_text(&lines, why_line);
        if why_text.len() < MIN_WHY_LENGTH {
            report.add_warning(
                file_path,
                Some(why_line + 1),
                &format!(
                    "Why section is too short ({} chars, minimum {} recommended)",
                    why_text.len(),
                    MIN_WHY_LENGTH
                ),
            );
        }
    }

    // Check for What Changes section
    let what_changes_result = find_section(&lines, "What Changes");
    if what_changes_result.is_none() {
        report.add_error(file_path, Some(1), "Missing What Changes section");
    }
}

/// Find a `## <header>` section and return its line index (0-indexed) and content.
fn find_section(lines: &[&str], header: &str) -> Option<(usize, String)> {
    let target = format!("## {}", header);

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case(&target) || trimmed == target {
            return Some((i, header.to_string()));
        }
    }
    None
}

/// Extract the text content of a section (from header to next same-level header).
fn extract_section_text(lines: &[&str], start_line: usize) -> String {
    let mut content = String::new();

    for line in lines.iter().skip(start_line + 1) {
        let trimmed = line.trim();
        // Stop at next ## header (but not ### or deeper)
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            break;
        }
        content.push_str(trimmed);
        content.push('\n');
    }

    content.trim().to_string()
}

/// Validate a delta spec file and return the count of delta operations found.
fn validate_delta_spec(spec_path: &Path, spec_dir: &Path, report: &mut ValidationReport) -> usize {
    let spec_name = spec_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let file_path = format!("specs/{}/spec.md", spec_name);

    let content = match fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(e) => {
            report.add_error(
                &file_path,
                Some(1),
                &format!("Failed to read delta spec: {}", e),
            );
            return 0;
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut delta_count = 0;

    // Find all delta operation headers
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        for delta_header in DELTA_HEADERS {
            if trimmed.eq_ignore_ascii_case(delta_header) {
                delta_count += 1;

                // For ADDED and MODIFIED requirements, validate they have proper structure
                if trimmed.to_uppercase().contains("ADDED")
                    || trimmed.to_uppercase().contains("MODIFIED")
                {
                    validate_delta_requirements(&lines, line_idx, trimmed, &file_path, report);
                }
            }
        }
    }

    // Check if there's at least one valid delta header
    if delta_count == 0 {
        // Check if there are any header-like patterns that might be malformed
        let has_invalid_headers = lines.iter().any(|line| {
            let upper = line.to_uppercase();
            (upper.contains("ADDED") || upper.contains("MODIFIED") || upper.contains("REMOVED") || upper.contains("RENAMED"))
                && line.trim().starts_with("##")
                && !DELTA_HEADERS.iter().any(|h| line.trim().eq_ignore_ascii_case(h))
        });

        if has_invalid_headers {
            report.add_error(
                &file_path,
                None,
                "Delta spec has invalid headers. Use: ## ADDED Requirements, ## MODIFIED Requirements, ## REMOVED Requirements, or ## RENAMED Requirements",
            );
        }
    }

    delta_count
}

/// Validate requirements within a delta section (ADDED or MODIFIED).
fn validate_delta_requirements(
    lines: &[&str],
    section_start: usize,
    section_header: &str,
    file_path: &str,
    report: &mut ValidationReport,
) {
    // Extract the section content
    let section_content = extract_delta_section(lines, section_start);

    // Find all requirements in this section
    let requirements = find_delta_requirements(&section_content);

    let is_added = section_header.to_uppercase().contains("ADDED");
    let is_modified = section_header.to_uppercase().contains("MODIFIED");

    for (req_line_offset, req_name, req_content) in requirements {
        let absolute_line = section_start + req_line_offset + 1; // +1 for 1-indexed

        // For ADDED requirements, check they have scenarios
        if is_added {
            let scenarios = extract_scenarios(&req_content);
            if scenarios.is_empty() {
                report.add_warning(
                    file_path,
                    Some(absolute_line),
                    &format!(
                        "ADDED requirement \"{}\" should have at least one scenario",
                        req_name
                    ),
                );
            } else {
                // Validate each scenario has WHEN/THEN
                for (scenario_offset, scenario_name, scenario_content) in &scenarios {
                    let scenario_line = absolute_line + scenario_offset;

                    if !has_when_clause(scenario_content) {
                        report.add_error(
                            file_path,
                            Some(scenario_line),
                            &format!(
                                "Scenario \"{}\" in requirement \"{}\" is missing WHEN clause",
                                scenario_name, req_name
                            ),
                        );
                    }

                    if !has_then_clause(scenario_content) {
                        report.add_error(
                            file_path,
                            Some(scenario_line),
                            &format!(
                                "Scenario \"{}\" in requirement \"{}\" is missing THEN clause",
                                scenario_name, req_name
                            ),
                        );
                    }
                }
            }
        }

        // For MODIFIED requirements, check they have full text
        if is_modified {
            // Check if requirement has substantive content (not just the header)
            let content_lines: Vec<&str> = req_content
                .lines()
                .skip(1) // Skip the requirement header
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("####"))
                .collect();

            if content_lines.is_empty() {
                report.add_warning(
                    file_path,
                    Some(absolute_line),
                    &format!(
                        "MODIFIED requirement \"{}\" should include the complete requirement text",
                        req_name
                    ),
                );
            }
        }
    }
}

/// Extract a delta section from start to next ## header.
fn extract_delta_section(lines: &[&str], start_line: usize) -> String {
    let mut section_lines = Vec::new();

    for line in lines.iter().skip(start_line + 1) {
        let trimmed = line.trim();
        // Stop at next ## header (but not ### or deeper)
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            break;
        }
        section_lines.push(*line);
    }

    section_lines.join("\n")
}

/// Find all `### Requirement:` blocks within delta section content.
/// Returns: Vec<(line_offset, name, content)>
fn find_delta_requirements(content: &str) -> Vec<(usize, String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut results = Vec::new();

    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.starts_with("### Requirement:") {
            let line_offset = idx;
            let name = trimmed
                .strip_prefix("### Requirement:")
                .unwrap_or("")
                .trim()
                .to_string();

            // Collect content until next ### header or ## header
            let mut section_lines = vec![line];
            idx += 1;

            while idx < lines.len() {
                let subsequent_line = lines[idx];
                let subsequent_trimmed = subsequent_line.trim();

                // Stop at next ### or ## header
                if subsequent_trimmed.starts_with("### ")
                    || (subsequent_trimmed.starts_with("## ")
                        && !subsequent_trimmed.starts_with("### "))
                {
                    break;
                }
                section_lines.push(subsequent_line);
                idx += 1;
            }

            results.push((line_offset, name, section_lines.join("\n")));
        } else {
            idx += 1;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create a temp change directory with files.
    fn create_temp_change(
        proposal_content: &str,
        delta_specs: &[(&str, &str)],
    ) -> (TempDir, std::path::PathBuf) {
        create_temp_change_with_tasks(proposal_content, delta_specs, None)
    }

    /// Helper to create a temp change directory with files including optional tasks.md.
    fn create_temp_change_with_tasks(
        proposal_content: &str,
        delta_specs: &[(&str, &str)],
        tasks_content: Option<&str>,
    ) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().to_path_buf();

        // Create proposal.md
        let proposal_path = change_dir.join("proposal.md");
        let mut file = fs::File::create(&proposal_path).unwrap();
        file.write_all(proposal_content.as_bytes()).unwrap();

        // Create tasks.md if provided
        if let Some(tasks) = tasks_content {
            let tasks_path = change_dir.join("tasks.md");
            let mut tasks_file = fs::File::create(&tasks_path).unwrap();
            tasks_file.write_all(tasks.as_bytes()).unwrap();
        }

        // Create specs/ directory and delta specs
        if !delta_specs.is_empty() {
            let specs_dir = change_dir.join("specs");
            fs::create_dir(&specs_dir).unwrap();

            for (spec_name, spec_content) in delta_specs {
                let spec_subdir = specs_dir.join(spec_name);
                fs::create_dir(&spec_subdir).unwrap();

                let spec_path = spec_subdir.join("spec.md");
                let mut spec_file = fs::File::create(&spec_path).unwrap();
                spec_file.write_all(spec_content.as_bytes()).unwrap();
            }
        }

        (temp_dir, change_dir)
    }

    // ==================== Basic validation tests ====================

    #[test]
    fn test_valid_change_passes() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
- Update database schema
- Modify frontend components
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Successful action

- **WHEN** user triggers the action
- **THEN** the action completes successfully
"#;

        let tasks = r#"# Tasks

## 1. Implementation
- [ ] 1.1 Create database schema
- [ ] 1.2 Implement API endpoint
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(
            result.report.is_valid(),
            "Expected valid change, got errors: {:?}",
            result.report.issues
        );
        assert!(result.task_stats.is_some());
        let stats = result.task_stats.unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 0);
    }

    #[test]
    fn test_missing_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().to_path_buf();

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("Missing proposal.md")));
    }

    #[test]
    fn test_missing_tasks_md() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Successful action

- **WHEN** user triggers the action
- **THEN** the action completes successfully
"#;

        // No tasks.md provided
        let (_temp_dir, change_dir) = create_temp_change(proposal, &[("feature-x", delta_spec)]);

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("Missing tasks.md")));
        assert!(result.task_stats.is_none());
    }

    #[test]
    fn test_missing_why_section() {
        let proposal = r#"# Add Feature X

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Successful action

- **WHEN** user triggers the action
- **THEN** the action completes successfully
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("Missing Why section")));
    }

    #[test]
    fn test_missing_what_changes_section() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Successful action

- **WHEN** user triggers the action
- **THEN** the action completes successfully
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("Missing What Changes section")));
    }

    #[test]
    fn test_short_why_section_warning() {
        let proposal = r#"# Add Feature X

## Why

Short why.

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Successful action

- **WHEN** user triggers the action
- **THEN** the action completes successfully
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        // Should be valid (warnings don't fail validation)
        assert!(result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("Why section is too short")));
    }

    #[test]
    fn test_no_delta_specs() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("at least one delta spec")));
    }

    #[test]
    fn test_delta_without_valid_headers() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## Some Other Section

This is not a valid delta header.

### Requirement: Test

Some content.
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("at least one delta spec")));
    }

    #[test]
    fn test_added_requirement_without_scenarios_warning() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        // Should be valid (warnings don't fail validation)
        assert!(result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("should have at least one scenario")));
    }

    #[test]
    fn test_added_requirement_scenario_missing_when() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Missing when

- **THEN** action completes
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("missing WHEN clause")));
    }

    #[test]
    fn test_added_requirement_scenario_missing_then() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Missing then

- **WHEN** user triggers the action
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("missing THEN clause")));
    }

    #[test]
    fn test_modified_requirement_without_full_text_warning() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Update existing requirement
"#;

        let delta_spec = r#"# Feature X Spec Delta

## MODIFIED Requirements

### Requirement: Existing feature

"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        // Should be valid (warnings don't fail validation)
        assert!(result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("should include the complete requirement text")));
    }

    // ==================== Multiple delta sections tests ====================

    #[test]
    fn test_multiple_delta_operations() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new requirements
- Modify existing requirements
- Remove obsolete requirements
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: New Feature

The system SHALL support the new feature.

#### Scenario: Feature works

- **WHEN** user uses the feature
- **THEN** it works correctly

## MODIFIED Requirements

### Requirement: Existing Feature

The system SHALL support the improved existing feature with enhanced functionality.

## REMOVED Requirements

### Requirement: Old Feature

This requirement is no longer needed.
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(
            result.report.is_valid(),
            "Expected valid change with multiple delta operations, got errors: {:?}",
            result.report.issues
        );
    }

    #[test]
    fn test_renamed_requirements() {
        let proposal = r#"# Rename Feature

## Why

The requirement naming needs to be updated for clarity and consistency with the
new naming conventions established by the team for better documentation.

## What Changes

- Rename requirement from old name to new name
"#;

        let delta_spec = r#"# Feature Rename Delta

## RENAMED Requirements

### Requirement: Old Name -> New Name

The requirement has been renamed for clarity.
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Update references
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("rename", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(
            result.report.is_valid(),
            "Expected valid change with RENAMED, got errors: {:?}",
            result.report.issues
        );
    }

    // ==================== Non-existent directory tests ====================

    #[test]
    fn test_nonexistent_directory() {
        let path = Path::new("/nonexistent/path/to/change");
        let result = validate_change(path);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("does not exist")));
    }

    #[test]
    fn test_path_is_file_not_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("not-a-dir");
        fs::File::create(&file_path).unwrap();

        let result = validate_change(&file_path);
        assert!(!result.report.is_valid());
        assert!(result.report.issues.iter().any(|i| i.message.contains("not a directory")));
    }

    // ==================== Case insensitivity tests ====================

    #[test]
    fn test_case_insensitive_headers() {
        let proposal = r#"# Add Feature X

## WHY

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## WHAT CHANGES

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED REQUIREMENTS

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Successful action

- **WHEN** user triggers the action
- **THEN** the action completes successfully
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(
            result.report.is_valid(),
            "Expected valid change with uppercase headers, got errors: {:?}",
            result.report.issues
        );
    }

    // ==================== Multiple spec files tests ====================

    #[test]
    fn test_multiple_delta_spec_files() {
        let proposal = r#"# Multi-Spec Change

## Why

This change affects multiple specifications and needs to update requirements
across different areas of the system for a comprehensive feature enhancement.

## What Changes

- Update auth spec
- Update user spec
"#;

        let auth_delta = r#"# Auth Spec Delta

## MODIFIED Requirements

### Requirement: Login

The system SHALL authenticate users with the new method and additional security.
"#;

        let user_delta = r#"# User Spec Delta

## ADDED Requirements

### Requirement: User Profile

The system SHALL allow users to manage their profile.

#### Scenario: View profile

- **WHEN** user opens profile page
- **THEN** profile information is displayed
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Update auth
- [ ] 1.2 Update user
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(
            proposal,
            &[("auth", auth_delta), ("user", user_delta)],
            Some(tasks),
        );

        let result = validate_change(&change_dir);
        assert!(
            result.report.is_valid(),
            "Expected valid change with multiple delta specs, got errors: {:?}",
            result.report.issues
        );
    }

    // ==================== Task stats integration tests ====================

    #[test]
    fn test_task_stats_included_in_result() {
        let proposal = r#"# Add Feature X

## Why

This feature is needed because it solves a critical user problem that has been
requested by many users. It will improve the overall user experience significantly.

## What Changes

- Add new API endpoint
"#;

        let delta_spec = r#"# Feature X Spec Delta

## ADDED Requirements

### Requirement: User can perform action

The system SHALL allow users to perform the new action.

#### Scenario: Successful action

- **WHEN** user triggers the action
- **THEN** the action completes successfully
"#;

        let tasks = r#"# Tasks

## 1. Implementation
- [x] 1.1 Create database schema
- [x] 1.2 Implement API endpoint
- [ ] 1.3 Add frontend component
- [ ] 1.4 Write tests
"#;

        let (_temp_dir, change_dir) = create_temp_change_with_tasks(proposal, &[("feature-x", delta_spec)], Some(tasks));

        let result = validate_change(&change_dir);
        assert!(result.report.is_valid());
        assert!(result.task_stats.is_some());

        let stats = result.task_stats.unwrap();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.percentage(), 50);

        // Check that task stats info was added to report
        assert!(result.report.issues.iter().any(|i| i.message.contains("Tasks: 2/4 completed")));
    }
}
