//! Tasks file validation module.
//!
//! Validates tasks.md files for structural correctness and task organization.
//! This is a Spec Oxide feature not present in OpenSpec.

use std::fs;
use std::path::Path;

use super::ValidationReport;

/// Statistics about task completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStats {
    /// Total number of tasks (including subtasks).
    pub total: usize,
    /// Number of completed tasks.
    pub completed: usize,
}

impl TaskStats {
    /// Create new task statistics.
    pub fn new() -> Self {
        Self {
            total: 0,
            completed: 0,
        }
    }

    /// Calculate completion percentage (0-100).
    pub fn percentage(&self) -> u8 {
        if self.total == 0 {
            0
        } else {
            ((self.completed as f64 / self.total as f64) * 100.0).round() as u8
        }
    }
}

impl Default for TaskStats {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tasks: {}/{} completed ({}%)",
            self.completed,
            self.total,
            self.percentage()
        )
    }
}

/// Validate a tasks.md file at the given path.
///
/// # Arguments
/// * `path` - Path to the tasks.md file to validate
///
/// # Returns
/// A tuple of `(ValidationReport, TaskStats)` containing validation issues and task statistics.
pub fn validate_tasks(path: &Path) -> (ValidationReport, TaskStats) {
    let mut report = ValidationReport::new();
    let file_path = path.to_string_lossy().to_string();

    // Read file content
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            report.add_error(&file_path, Some(1), &format!("Failed to read file: {}", e));
            return (report, TaskStats::new());
        }
    };

    validate_tasks_content(&content, &file_path)
}

/// Validate tasks from content string (for unit testing).
///
/// # Arguments
/// * `content` - The content of the tasks.md file
/// * `file_name` - The file name to use in validation messages
///
/// # Returns
/// A tuple of `(ValidationReport, TaskStats)` containing validation issues and task statistics.
pub fn validate_tasks_content(content: &str, file_name: &str) -> (ValidationReport, TaskStats) {
    let mut report = ValidationReport::new();
    let mut stats = TaskStats::new();

    let lines: Vec<&str> = content.lines().collect();

    // Track parsed tasks for validation
    let mut tasks: Vec<ParsedTask> = Vec::new();
    let mut has_group_headers = false;
    let mut tasks_without_numbers: Vec<(usize, String)> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let line_number = idx + 1; // 1-indexed

        // Check for task group headers (## N. Group)
        if is_task_group_header(line) {
            has_group_headers = true;
        }

        // Check for checkbox items
        if let Some(task) = parse_checkbox_item(line) {
            stats.total += 1;
            if task.completed {
                stats.completed += 1;
            }

            // Check for numbered prefix
            if !task.has_number {
                tasks_without_numbers.push((line_number, task.description.clone()));
            }

            tasks.push(ParsedTask {
                line: line_number,
                completed: task.completed,
                description: task.description,
                has_number: task.has_number,
                indent_level: task.indent_level,
            });
        }
    }

    // Validation: Must have at least one task
    if tasks.is_empty() {
        report.add_error(
            file_name,
            Some(1),
            "tasks.md must have at least one checkbox item",
        );
    }

    // Validation: Tasks should use numbered prefixes
    for (line_number, desc) in tasks_without_numbers {
        let preview = if desc.len() > 30 {
            format!("{}...", &desc[..30])
        } else {
            desc.clone()
        };
        report.add_warning(
            file_name,
            Some(line_number),
            &format!(
                "Task \"{}\" does not use numbered prefix (e.g., 1.1)",
                preview
            ),
        );
    }

    // Info: Task group headers organization
    if has_group_headers && !tasks.is_empty() {
        report.add_info(file_name, None, "Tasks are organized with group headers");
    }

    (report, stats)
}

/// A parsed checkbox task item.
#[derive(Debug)]
#[allow(dead_code)] // Fields reserved for future validation features
struct ParsedTask {
    /// Line number where the task appears (1-indexed).
    line: usize,
    /// Whether the task is completed.
    completed: bool,
    /// The task description text.
    description: String,
    /// Whether the task has a numbered prefix (N.N format).
    has_number: bool,
    /// Indentation level (0 for top-level, 1+ for subtasks).
    indent_level: usize,
}

/// Result of parsing a checkbox item.
#[derive(Debug)]
struct CheckboxItem {
    /// Whether the checkbox is checked.
    completed: bool,
    /// The text after the checkbox.
    description: String,
    /// Whether the description starts with a numbered prefix.
    has_number: bool,
    /// Indentation level based on leading whitespace.
    indent_level: usize,
}

/// Parse a checkbox item from a line.
///
/// Recognizes patterns like:
/// - `- [ ] Task description`
/// - `- [x] Task description`
/// - `  - [ ] Nested task` (with indentation)
///
/// Returns `None` if the line is not a checkbox item.
fn parse_checkbox_item(line: &str) -> Option<CheckboxItem> {
    // Count leading whitespace for indent level
    let leading_spaces = line.len() - line.trim_start().len();
    let indent_level = leading_spaces / 2; // Each indent level is 2 spaces

    let trimmed = line.trim();

    // Check for checkbox patterns: - [ ] or - [x] or - [X]
    let (completed, rest) = if trimmed.starts_with("- [ ] ") {
        (false, trimmed.strip_prefix("- [ ] ").unwrap_or(""))
    } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
        (true, &trimmed[6..])
    } else if trimmed.starts_with("* [ ] ") {
        (false, trimmed.strip_prefix("* [ ] ").unwrap_or(""))
    } else if trimmed.starts_with("* [x] ") || trimmed.starts_with("* [X] ") {
        (true, &trimmed[6..])
    } else {
        return None;
    };

    let description = rest.trim().to_string();

    // Check if description starts with a numbered prefix (N.N or N.N.N format)
    let has_number = has_numbered_prefix(&description);

    Some(CheckboxItem {
        completed,
        description,
        has_number,
        indent_level,
    })
}

/// Check if a description starts with a numbered prefix like 1.1, 2.3.1, etc.
fn has_numbered_prefix(description: &str) -> bool {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Match pattern: digit(s).digit(s) optionally followed by more .digit(s)
    // Examples: 1.1, 2.3, 1.2.3, 10.20.30
    let mut chars = trimmed.chars().peekable();

    // Must start with a digit
    if !chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return false;
    }

    // Consume first number
    while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        chars.next();
    }

    // Must have at least one dot followed by digits
    if chars.peek() != Some(&'.') {
        return false;
    }
    chars.next(); // consume the dot

    // Must have digits after the dot
    if !chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return false;
    }

    // Continue consuming number.number patterns
    loop {
        // Consume digits
        while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            chars.next();
        }

        // Check if there's another dot
        if chars.peek() == Some(&'.') {
            chars.next(); // consume the dot

            // Must have digits after the dot
            if !chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                return false;
            }
        } else {
            // End of number sequence - must be followed by space or end
            match chars.peek() {
                None | Some(' ') | Some('\t') => return true,
                _ => return false,
            }
        }
    }
}

/// Check if a line is a task group header (## N. Group Name).
fn is_task_group_header(line: &str) -> bool {
    let trimmed = line.trim();

    // Must start with ##
    if !trimmed.starts_with("## ") {
        return false;
    }

    let after_header = trimmed.strip_prefix("## ").unwrap_or("").trim();

    // Must start with a number followed by a dot
    let mut chars = after_header.chars().peekable();

    // Must start with a digit
    if !chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return false;
    }

    // Consume digits
    while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        chars.next();
    }

    // Must have a dot
    if chars.peek() != Some(&'.') {
        return false;
    }
    chars.next(); // consume the dot

    // Must have something after the dot (the group name)
    chars.peek().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TaskStats tests ====================

    #[test]
    fn test_task_stats_new() {
        let stats = TaskStats::new();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.completed, 0);
    }

    #[test]
    fn test_task_stats_percentage_empty() {
        let stats = TaskStats::new();
        assert_eq!(stats.percentage(), 0);
    }

    #[test]
    fn test_task_stats_percentage_partial() {
        let stats = TaskStats {
            total: 10,
            completed: 5,
        };
        assert_eq!(stats.percentage(), 50);
    }

    #[test]
    fn test_task_stats_percentage_complete() {
        let stats = TaskStats {
            total: 5,
            completed: 5,
        };
        assert_eq!(stats.percentage(), 100);
    }

    #[test]
    fn test_task_stats_display() {
        let stats = TaskStats {
            total: 24,
            completed: 5,
        };
        assert_eq!(format!("{}", stats), "Tasks: 5/24 completed (21%)");
    }

    // ==================== Checkbox parsing tests ====================

    #[test]
    fn test_parse_checkbox_unchecked() {
        let item = parse_checkbox_item("- [ ] 1.1 Task description");
        assert!(item.is_some());
        let item = item.unwrap();
        assert!(!item.completed);
        assert_eq!(item.description, "1.1 Task description");
        assert!(item.has_number);
        assert_eq!(item.indent_level, 0);
    }

    #[test]
    fn test_parse_checkbox_checked() {
        let item = parse_checkbox_item("- [x] 1.2 Completed task");
        assert!(item.is_some());
        let item = item.unwrap();
        assert!(item.completed);
        assert_eq!(item.description, "1.2 Completed task");
        assert!(item.has_number);
    }

    #[test]
    fn test_parse_checkbox_checked_uppercase() {
        let item = parse_checkbox_item("- [X] 2.1 Uppercase X");
        assert!(item.is_some());
        let item = item.unwrap();
        assert!(item.completed);
    }

    #[test]
    fn test_parse_checkbox_indented() {
        let item = parse_checkbox_item("  - [ ] 1.1.1 Subtask");
        assert!(item.is_some());
        let item = item.unwrap();
        assert!(!item.completed);
        assert_eq!(item.description, "1.1.1 Subtask");
        assert!(item.has_number);
        assert_eq!(item.indent_level, 1);
    }

    #[test]
    fn test_parse_checkbox_deeply_indented() {
        let item = parse_checkbox_item("    - [ ] 1.1.1.1 Deep subtask");
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.indent_level, 2);
    }

    #[test]
    fn test_parse_checkbox_no_number() {
        let item = parse_checkbox_item("- [ ] Task without number");
        assert!(item.is_some());
        let item = item.unwrap();
        assert!(!item.has_number);
        assert_eq!(item.description, "Task without number");
    }

    #[test]
    fn test_parse_checkbox_asterisk() {
        let item = parse_checkbox_item("* [ ] 1.1 Asterisk task");
        assert!(item.is_some());
        let item = item.unwrap();
        assert!(!item.completed);
        assert!(item.has_number);
    }

    #[test]
    fn test_parse_checkbox_not_checkbox() {
        assert!(parse_checkbox_item("Regular text").is_none());
        assert!(parse_checkbox_item("- Regular list item").is_none());
        assert!(parse_checkbox_item("## Header").is_none());
        assert!(parse_checkbox_item("").is_none());
    }

    // ==================== Number prefix tests ====================

    #[test]
    fn test_has_numbered_prefix_simple() {
        assert!(has_numbered_prefix("1.1 Task"));
        assert!(has_numbered_prefix("2.3 Task"));
        assert!(has_numbered_prefix("10.20 Task"));
    }

    #[test]
    fn test_has_numbered_prefix_deep() {
        assert!(has_numbered_prefix("1.2.3 Task"));
        assert!(has_numbered_prefix("1.2.3.4 Task"));
        assert!(has_numbered_prefix("10.20.30 Task"));
    }

    #[test]
    fn test_has_numbered_prefix_no_prefix() {
        assert!(!has_numbered_prefix("Task without number"));
        assert!(!has_numbered_prefix("Create the thing"));
        assert!(!has_numbered_prefix(""));
    }

    #[test]
    fn test_has_numbered_prefix_partial() {
        // Single number without dot should not match
        assert!(!has_numbered_prefix("1 Task"));
        assert!(!has_numbered_prefix("10 Task"));
    }

    #[test]
    fn test_has_numbered_prefix_trailing_dot() {
        // Trailing dot without number should not match
        assert!(!has_numbered_prefix("1. Task"));
    }

    #[test]
    fn test_has_numbered_prefix_letters() {
        // Letters mixed with numbers should not match
        assert!(!has_numbered_prefix("1a.2 Task"));
        assert!(!has_numbered_prefix("a.1 Task"));
    }

    // ==================== Task group header tests ====================

    #[test]
    fn test_is_task_group_header_valid() {
        assert!(is_task_group_header("## 1. Implementation"));
        assert!(is_task_group_header("## 2. Testing"));
        assert!(is_task_group_header("## 10. Large section"));
        assert!(is_task_group_header("  ## 1. With leading space"));
    }

    #[test]
    fn test_is_task_group_header_invalid() {
        assert!(!is_task_group_header("## Purpose"));
        assert!(!is_task_group_header("## Requirements"));
        assert!(!is_task_group_header("# 1. Single hash"));
        assert!(!is_task_group_header("### 1. Triple hash"));
        assert!(!is_task_group_header("## 1 No dot"));
        assert!(!is_task_group_header("## . No number"));
    }

    // ==================== Full validation tests ====================

    #[test]
    fn test_validate_valid_tasks_file() {
        let content = r#"# Tasks

## 1. Implementation
- [ ] 1.1 Create database schema
- [ ] 1.2 Implement API endpoint
  - [ ] 1.2.1 Add request validation
  - [x] 1.2.2 Add response formatting

## 2. Testing
- [x] 2.1 Write unit tests
- [ ] 2.2 Write integration tests
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        assert!(
            report.is_valid(),
            "Expected valid tasks file, got errors: {:?}",
            report.issues
        );
        assert_eq!(stats.total, 6);
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.percentage(), 33);
    }

    #[test]
    fn test_validate_empty_tasks_file() {
        let content = r#"# Tasks

No tasks here yet.
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        assert!(!report.is_valid());
        assert!(report
            .issues
            .iter()
            .any(|i| i.message.contains("must have at least one checkbox")));
        assert_eq!(stats.total, 0);
        assert_eq!(stats.completed, 0);
    }

    #[test]
    fn test_validate_tasks_without_numbers() {
        let content = r#"# Tasks

- [ ] Create database schema
- [ ] Implement API endpoint
- [x] Write tests
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        // Should be valid (warnings don't fail validation)
        assert!(report.is_valid());
        assert_eq!(stats.total, 3);
        assert_eq!(stats.completed, 1);

        // Should have warnings about missing numbers
        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == super::super::Severity::Warning)
            .collect();
        assert_eq!(warnings.len(), 3);
        assert!(warnings
            .iter()
            .all(|w| w.message.contains("does not use numbered prefix")));
    }

    #[test]
    fn test_validate_nested_subtasks_counted() {
        let content = r#"# Tasks

## 1. Section
- [ ] 1.1 Parent task
  - [ ] 1.1.1 Subtask one
  - [x] 1.1.2 Subtask two
    - [ ] 1.1.2.1 Deep subtask
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        assert!(report.is_valid());
        assert_eq!(stats.total, 4);
        assert_eq!(stats.completed, 1);
    }

    #[test]
    fn test_validate_mixed_checkbox_formats() {
        let content = r#"# Tasks

- [ ] 1.1 Dash unchecked
- [x] 1.2 Dash checked lowercase
- [X] 1.3 Dash checked uppercase
* [ ] 1.4 Asterisk unchecked
* [x] 1.5 Asterisk checked
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        assert!(report.is_valid());
        assert_eq!(stats.total, 5);
        assert_eq!(stats.completed, 3);
    }

    #[test]
    fn test_validate_all_completed() {
        let content = r#"# Tasks

- [x] 1.1 Task one
- [x] 1.2 Task two
- [x] 1.3 Task three
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        assert!(report.is_valid());
        assert_eq!(stats.total, 3);
        assert_eq!(stats.completed, 3);
        assert_eq!(stats.percentage(), 100);
    }

    #[test]
    fn test_validate_info_about_group_headers() {
        let content = r#"# Tasks

## 1. Implementation
- [ ] 1.1 Task

## 2. Testing
- [ ] 2.1 Test
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        assert!(report.is_valid());
        assert_eq!(stats.total, 2);

        // Should have info about group headers
        let infos: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == super::super::Severity::Info)
            .collect();
        assert!(infos
            .iter()
            .any(|i| i.message.contains("organized with group headers")));
    }

    #[test]
    fn test_validate_no_info_without_group_headers() {
        let content = r#"# Tasks

- [ ] 1.1 Task one
- [ ] 1.2 Task two
"#;

        let (report, _stats) = validate_tasks_content(content, "tasks.md");

        // Should NOT have info about group headers
        let infos: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == super::super::Severity::Info)
            .collect();
        assert!(infos.is_empty());
    }

    #[test]
    fn test_validate_long_task_description_truncated_in_warning() {
        let content = r#"# Tasks

- [ ] This is a very long task description that should be truncated in the warning message
"#;

        let (report, _stats) = validate_tasks_content(content, "tasks.md");

        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == super::super::Severity::Warning)
            .collect();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("..."));
    }

    #[test]
    fn test_validate_file_not_found() {
        let path = Path::new("/nonexistent/path/to/tasks.md");
        let (report, stats) = validate_tasks(path);

        assert!(!report.is_valid());
        assert!(report
            .issues
            .iter()
            .any(|i| i.message.contains("Failed to read")));
        assert_eq!(stats.total, 0);
    }

    #[test]
    fn test_completion_statistics_calculation() {
        // Test various completion scenarios
        let test_cases = vec![
            ("- [ ] 1.1 Task", 1, 0, 0),
            ("- [x] 1.1 Task", 1, 1, 100),
            ("- [ ] 1.1 Task\n- [x] 1.2 Task", 2, 1, 50),
            ("- [x] 1.1 Task\n- [x] 1.2 Task\n- [ ] 1.3 Task", 3, 2, 67),
        ];

        for (content, expected_total, expected_completed, expected_percentage) in test_cases {
            let (_report, stats) = validate_tasks_content(content, "tasks.md");
            assert_eq!(
                stats.total, expected_total,
                "Total mismatch for: {}",
                content
            );
            assert_eq!(
                stats.completed, expected_completed,
                "Completed mismatch for: {}",
                content
            );
            assert_eq!(
                stats.percentage(),
                expected_percentage,
                "Percentage mismatch for: {}",
                content
            );
        }
    }

    #[test]
    fn test_validate_realistic_tasks_file() {
        let content = r#"# Change: add-validation-commands

## 1. Foundation
- [x] 1.1 Create validation module structure
- [x] 1.2 Define ValidationReport type
- [x] 1.3 Define Severity enum

## 2. Spec Validation
- [x] 2.1 Implement spec.rs validate_spec()
- [x] 2.2 Add Purpose section validation
- [x] 2.3 Add Requirements section validation
- [x] 2.4 Add scenario validation

## 3. Change Validation
- [ ] 3.1 Implement change.rs validate_change()
- [ ] 3.2 Add proposal.md validation
- [ ] 3.3 Add delta spec validation

## 4. Tasks Validation
- [ ] 4.1 Create tasks.rs with validate_tasks()
- [ ] 4.2 Implement checkbox detection
  - [ ] 4.2.1 Parse unchecked items
  - [ ] 4.2.2 Parse checked items
- [ ] 4.3 Implement numbered prefix validation
- [ ] 4.4 Implement group header parsing
- [ ] 4.5 Calculate completion statistics
"#;

        let (report, stats) = validate_tasks_content(content, "tasks.md");

        assert!(
            report.is_valid(),
            "Expected valid tasks file, got: {:?}",
            report.issues
        );
        assert_eq!(stats.total, 17);
        assert_eq!(stats.completed, 7);
        assert_eq!(stats.percentage(), 41);
    }
}
