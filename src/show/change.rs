//! Change show module for displaying change proposals.
//!
//! This module provides functions to parse and display change directories
//! including proposal.md, tasks.md, and delta spec files.

use std::fs;
use std::path::Path;

use super::{blue, box_header, cyan_bold, dim, green, progress_bar, red, yellow};

/// Default box width for headers.
const BOX_WIDTH: usize = 61;

/// Delta operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaOp {
    Added,
    Modified,
    Removed,
    Renamed,
}

impl DeltaOp {
    /// Get the symbol for this operation.
    pub fn symbol(&self) -> &'static str {
        match self {
            DeltaOp::Added => "+",
            DeltaOp::Modified => "~",
            DeltaOp::Removed => "-",
            DeltaOp::Renamed => ">",
        }
    }

    /// Get the label for this operation.
    pub fn label(&self) -> &'static str {
        match self {
            DeltaOp::Added => "ADDED",
            DeltaOp::Modified => "MODIFIED",
            DeltaOp::Removed => "REMOVED",
            DeltaOp::Renamed => "RENAMED",
        }
    }

    /// Format the symbol with appropriate color.
    pub fn colored_symbol(&self) -> String {
        match self {
            DeltaOp::Added => green(self.symbol()),
            DeltaOp::Modified => yellow(self.symbol()),
            DeltaOp::Removed => red(self.symbol()),
            DeltaOp::Renamed => blue(self.symbol()),
        }
    }

    /// Format the label with appropriate color.
    pub fn colored_label(&self) -> String {
        match self {
            DeltaOp::Added => green(self.label()),
            DeltaOp::Modified => yellow(self.label()),
            DeltaOp::Removed => red(self.label()),
            DeltaOp::Renamed => blue(self.label()),
        }
    }
}

/// A single delta item (requirement change).
#[derive(Debug, Clone)]
pub struct DeltaItem {
    /// The type of operation.
    pub operation: DeltaOp,
    /// The requirement name.
    pub name: String,
    /// The requirement text (description).
    pub text: String,
    /// List of scenario names.
    pub scenarios: Vec<String>,
}

/// A group of delta items for a single capability.
#[derive(Debug, Clone)]
pub struct DeltaGroup {
    /// The capability name (directory name).
    pub capability: String,
    /// The delta items in this group.
    pub items: Vec<DeltaItem>,
}

/// Parsed change information.
#[derive(Debug, Clone)]
pub struct ChangeInfo {
    /// The change name (directory name).
    pub name: String,
    /// The Why section content.
    pub why: String,
    /// The What Changes section content.
    pub what_changes: String,
    /// Number of completed tasks.
    pub tasks_completed: usize,
    /// Total number of tasks.
    pub tasks_total: usize,
    /// Delta groups organized by capability.
    pub deltas: Vec<DeltaGroup>,
}

/// Parse a change directory and extract all information.
///
/// # Arguments
/// * `change_dir` - Path to the change directory
///
/// # Returns
/// A `ChangeInfo` struct or an error message.
pub fn parse_change(change_dir: &Path) -> Result<ChangeInfo, String> {
    // Get change name from directory
    let name = change_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid change directory path".to_string())?;

    // Check directory exists
    if !change_dir.exists() {
        return Err(format!(
            "Change directory does not exist: {}",
            change_dir.display()
        ));
    }

    if !change_dir.is_dir() {
        return Err(format!("Path is not a directory: {}", change_dir.display()));
    }

    // Parse proposal.md
    let proposal_path = change_dir.join("proposal.md");
    if !proposal_path.exists() {
        return Err("Missing proposal.md".to_string());
    }

    let proposal_content = fs::read_to_string(&proposal_path)
        .map_err(|e| format!("Failed to read proposal.md: {}", e))?;

    let (why, what_changes) = parse_proposal(&proposal_content);

    // Parse tasks.md
    let tasks_path = change_dir.join("tasks.md");
    let (tasks_completed, tasks_total) = if tasks_path.exists() {
        let tasks_content = fs::read_to_string(&tasks_path)
            .map_err(|e| format!("Failed to read tasks.md: {}", e))?;
        parse_tasks(&tasks_content)
    } else {
        (0, 0)
    };

    // Parse delta specs
    let specs_dir = change_dir.join("specs");
    let deltas = if specs_dir.exists() && specs_dir.is_dir() {
        parse_delta_specs(&specs_dir)?
    } else {
        Vec::new()
    };

    Ok(ChangeInfo {
        name,
        why,
        what_changes,
        tasks_completed,
        tasks_total,
        deltas,
    })
}

/// Parse proposal.md to extract Why and What Changes sections.
fn parse_proposal(content: &str) -> (String, String) {
    let why = extract_section(content, "Why");
    let what_changes = extract_section(content, "What Changes");
    (why, what_changes)
}

/// Extract a `## <header>` section from markdown content.
fn extract_section(content: &str, header: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let target = format!("## {}", header);

    let mut in_section = false;
    let mut section_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case(&target) {
            in_section = true;
            continue;
        }

        if in_section {
            // Stop at next ## header (but not ### or deeper)
            if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
                break;
            }
            section_lines.push(line);
        }
    }

    // Trim leading/trailing empty lines and join
    let result: Vec<&str> = section_lines
        .iter()
        .copied()
        .skip_while(|l| l.trim().is_empty())
        .collect();

    let result: String = result.join("\n");
    result.trim_end().to_string()
}

/// Parse tasks.md to count completed and total tasks.
fn parse_tasks(content: &str) -> (usize, usize) {
    let mut completed = 0;
    let mut total = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Match checkbox patterns: - [ ] or - [x] or - [X] or * variants
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

/// Parse delta specs from the specs/ subdirectory.
fn parse_delta_specs(specs_dir: &Path) -> Result<Vec<DeltaGroup>, String> {
    let mut groups = Vec::new();

    let entries =
        fs::read_dir(specs_dir).map_err(|e| format!("Failed to read specs directory: {}", e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let capability = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let spec_path = entry_path.join("spec.md");
            if spec_path.exists() {
                let content = fs::read_to_string(&spec_path)
                    .map_err(|e| format!("Failed to read {}/spec.md: {}", capability, e))?;

                let items = parse_delta_content(&content);
                if !items.is_empty() {
                    groups.push(DeltaGroup { capability, items });
                }
            }
        }
    }

    // Sort groups by capability name for consistent output
    groups.sort_by(|a, b| a.capability.cmp(&b.capability));

    Ok(groups)
}

/// Parse delta content from a spec.md file.
fn parse_delta_content(content: &str) -> Vec<DeltaItem> {
    let mut items = Vec::new();

    // Find delta operation headers and their requirements
    let delta_headers = [
        ("## ADDED Requirements", DeltaOp::Added),
        ("## MODIFIED Requirements", DeltaOp::Modified),
        ("## REMOVED Requirements", DeltaOp::Removed),
        ("## RENAMED Requirements", DeltaOp::Renamed),
    ];

    let lines: Vec<&str> = content.lines().collect();

    for (header_text, op) in delta_headers {
        // Find the header (case-insensitive)
        let header_idx = lines
            .iter()
            .position(|line| line.trim().eq_ignore_ascii_case(header_text));

        if let Some(start_idx) = header_idx {
            // Extract section until next ## header
            let section_content = extract_section_from_index(&lines, start_idx);
            let requirements = parse_requirements(&section_content, op);
            items.extend(requirements);
        }
    }

    items
}

/// Extract section content from a line index until next ## header.
fn extract_section_from_index(lines: &[&str], start_idx: usize) -> String {
    let mut section_lines = Vec::new();

    for line in lines.iter().skip(start_idx + 1) {
        let trimmed = line.trim();
        // Stop at next ## header (but not ### or deeper)
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            break;
        }
        section_lines.push(*line);
    }

    section_lines.join("\n")
}

/// Parse requirements from a delta section.
fn parse_requirements(content: &str, op: DeltaOp) -> Vec<DeltaItem> {
    let mut items = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.starts_with("### Requirement:") {
            let name = trimmed
                .strip_prefix("### Requirement:")
                .unwrap_or("")
                .trim()
                .to_string();

            // Collect requirement content until next ### or ## header
            let mut req_lines = Vec::new();
            idx += 1;

            while idx < lines.len() {
                let subsequent_line = lines[idx];
                let subsequent_trimmed = subsequent_line.trim();

                if subsequent_trimmed.starts_with("### ")
                    || (subsequent_trimmed.starts_with("## ")
                        && !subsequent_trimmed.starts_with("### "))
                {
                    break;
                }
                req_lines.push(subsequent_line);
                idx += 1;
            }

            let req_content = req_lines.join("\n");

            // Extract the requirement text (first non-empty paragraph)
            let text = extract_requirement_text(&req_content);

            // Extract scenario names
            let scenarios = extract_scenario_names(&req_content);

            items.push(DeltaItem {
                operation: op,
                name,
                text,
                scenarios,
            });
        } else {
            idx += 1;
        }
    }

    items
}

/// Extract the main text of a requirement (first non-empty lines before scenarios).
fn extract_requirement_text(content: &str) -> String {
    let mut text_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Stop at scenario headers
        if trimmed.starts_with("#### Scenario:") {
            break;
        }

        // Skip empty lines at the start
        if text_lines.is_empty() && trimmed.is_empty() {
            continue;
        }

        // Stop at empty line after collecting text
        if !text_lines.is_empty() && trimmed.is_empty() {
            break;
        }

        text_lines.push(trimmed);
    }

    text_lines.join(" ")
}

/// Extract scenario names from requirement content.
fn extract_scenario_names(content: &str) -> Vec<String> {
    let mut scenarios = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#### Scenario:") {
            let name = trimmed
                .strip_prefix("#### Scenario:")
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() {
                scenarios.push(name);
            }
        }
    }

    scenarios
}

/// Format change info for full display.
///
/// # Arguments
/// * `change` - The parsed change information
///
/// # Returns
/// A formatted string for terminal output.
pub fn format_change(change: &ChangeInfo) -> String {
    let mut output = String::new();

    // Box header
    let header = box_header(&format!("Change: {}", change.name), BOX_WIDTH);
    output.push_str(&header);
    output.push_str("\n\n");

    // Why section
    output.push_str(&yellow("Why:"));
    output.push('\n');
    for line in change.why.lines() {
        output.push_str("  ");
        output.push_str(line);
        output.push('\n');
    }
    output.push('\n');

    // What Changes section
    output.push_str(&yellow("What Changes:"));
    output.push('\n');
    for line in change.what_changes.lines() {
        output.push_str("  ");
        output.push_str(line);
        output.push('\n');
    }
    output.push('\n');

    // Tasks progress
    if change.tasks_total > 0 {
        let percentage = (change.tasks_completed * 100) / change.tasks_total;
        let bar = progress_bar(change.tasks_completed, change.tasks_total);
        output.push_str(&format!(
            "Tasks: {} {}/{} ({}%)\n\n",
            bar, change.tasks_completed, change.tasks_total, percentage
        ));
    }

    // Deltas section
    if !change.deltas.is_empty() {
        output.push_str(&yellow("Deltas:"));
        output.push_str("\n\n");

        for group in &change.deltas {
            output.push_str(&format!("  {}/\n", group.capability));

            for item in &group.items {
                output.push_str(&format!(
                    "    {} {}: {}\n",
                    item.operation.colored_symbol(),
                    item.operation.colored_label(),
                    item.name
                ));
            }
            output.push('\n');
        }
    }

    output.trim_end().to_string()
}

/// Format change info for deltas-only display.
///
/// # Arguments
/// * `change` - The parsed change information
///
/// # Returns
/// A formatted string showing only delta requirements.
pub fn format_deltas_only(change: &ChangeInfo) -> String {
    let mut output = String::new();

    output.push_str(&format!("Deltas for: {}\n\n", cyan_bold(&change.name)));

    for group in &change.deltas {
        output.push_str(&format!("{}/\n", group.capability));

        for item in &group.items {
            // Format: + Name or ~ Name (MODIFIED)
            let header = if item.operation == DeltaOp::Modified {
                format!(
                    "  {} {} ({})",
                    item.operation.colored_symbol(),
                    item.name,
                    item.operation.colored_label()
                )
            } else {
                format!("  {} {}", item.operation.colored_symbol(), item.name)
            };
            output.push_str(&header);
            output.push('\n');

            // Requirement text (indented)
            if !item.text.is_empty() {
                output.push_str(&format!("    {}\n", item.text));
            }

            // Scenarios (if any)
            if !item.scenarios.is_empty() {
                let scenarios_str = item.scenarios.join(", ");
                output.push_str(&format!(
                    "    {}\n",
                    dim(&format!("Scenarios: {}", scenarios_str))
                ));
            }

            output.push('\n');
        }
    }

    output.trim_end().to_string()
}

/// Show a change with full output.
///
/// # Arguments
/// * `change_dir` - Path to the change directory
///
/// # Returns
/// Formatted output string or error message.
pub fn show_change(change_dir: &Path) -> Result<String, String> {
    let change = parse_change(change_dir)?;
    Ok(format_change(&change))
}

/// Show a change with deltas-only output.
///
/// # Arguments
/// * `change_dir` - Path to the change directory
///
/// # Returns
/// Formatted output string or error message.
pub fn show_change_deltas_only(change_dir: &Path) -> Result<String, String> {
    let change = parse_change(change_dir)?;
    Ok(format_deltas_only(&change))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create a temp change directory with files.
    fn create_temp_change(
        proposal_content: &str,
        tasks_content: Option<&str>,
        delta_specs: &[(&str, &str)],
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

    // ==================== parse_proposal tests ====================

    #[test]
    fn test_parse_proposal_extracts_why() {
        let content = r#"# Change: Test

## Why

This is the reason for the change.
It spans multiple lines.

## What Changes

- Change 1
- Change 2
"#;

        let (why, _) = parse_proposal(content);
        assert!(why.contains("This is the reason for the change."));
        assert!(why.contains("It spans multiple lines."));
    }

    #[test]
    fn test_parse_proposal_extracts_what_changes() {
        let content = r#"# Change: Test

## Why

Some reason.

## What Changes

- First change
- Second change
- Third change

## Impact

Some impact.
"#;

        let (_, what_changes) = parse_proposal(content);
        assert!(what_changes.contains("First change"));
        assert!(what_changes.contains("Second change"));
        assert!(what_changes.contains("Third change"));
        assert!(!what_changes.contains("Impact"));
    }

    #[test]
    fn test_parse_proposal_case_insensitive() {
        let content = r#"# Change: Test

## WHY

Reason here.

## WHAT CHANGES

Changes here.
"#;

        let (why, what_changes) = parse_proposal(content);
        assert!(why.contains("Reason here"));
        assert!(what_changes.contains("Changes here"));
    }

    // ==================== parse_tasks tests ====================

    #[test]
    fn test_parse_tasks_counts_correctly() {
        let content = r#"# Tasks

## 1. Implementation
- [ ] 1.1 Create schema
- [x] 1.2 Implement API
- [ ] 1.3 Write tests
- [X] 1.4 Add docs
"#;

        let (completed, total) = parse_tasks(content);
        assert_eq!(total, 4);
        assert_eq!(completed, 2);
    }

    #[test]
    fn test_parse_tasks_handles_asterisk() {
        let content = r#"# Tasks
* [ ] Task 1
* [x] Task 2
"#;

        let (completed, total) = parse_tasks(content);
        assert_eq!(total, 2);
        assert_eq!(completed, 1);
    }

    #[test]
    fn test_parse_tasks_empty_content() {
        let content = r#"# Tasks

No tasks yet.
"#;

        let (completed, total) = parse_tasks(content);
        assert_eq!(total, 0);
        assert_eq!(completed, 0);
    }

    // ==================== parse_delta_content tests ====================

    #[test]
    fn test_parse_delta_content_added() {
        let content = r#"## ADDED Requirements

### Requirement: New Feature

The system SHALL support the new feature.

#### Scenario: Feature works

- **WHEN** user uses feature
- **THEN** it works
"#;

        let items = parse_delta_content(content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].operation, DeltaOp::Added);
        assert_eq!(items[0].name, "New Feature");
        assert!(items[0].text.contains("system SHALL support"));
        assert_eq!(items[0].scenarios.len(), 1);
        assert_eq!(items[0].scenarios[0], "Feature works");
    }

    #[test]
    fn test_parse_delta_content_modified() {
        let content = r#"## MODIFIED Requirements

### Requirement: Existing Feature

The system SHALL support the improved feature.

#### Scenario: Improved behavior

- **WHEN** user triggers action
- **THEN** improved result
"#;

        let items = parse_delta_content(content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].operation, DeltaOp::Modified);
        assert_eq!(items[0].name, "Existing Feature");
    }

    #[test]
    fn test_parse_delta_content_removed() {
        let content = r#"## REMOVED Requirements

### Requirement: Old Feature

This feature is no longer needed.
"#;

        let items = parse_delta_content(content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].operation, DeltaOp::Removed);
        assert_eq!(items[0].name, "Old Feature");
    }

    #[test]
    fn test_parse_delta_content_multiple_operations() {
        let content = r#"## ADDED Requirements

### Requirement: New Feature

New feature description.

#### Scenario: Works

- **WHEN** triggered
- **THEN** works

## MODIFIED Requirements

### Requirement: Changed Feature

Changed description.

## REMOVED Requirements

### Requirement: Old Feature

Removed.
"#;

        let items = parse_delta_content(content);
        assert_eq!(items.len(), 3);

        let added: Vec<_> = items
            .iter()
            .filter(|i| i.operation == DeltaOp::Added)
            .collect();
        let modified: Vec<_> = items
            .iter()
            .filter(|i| i.operation == DeltaOp::Modified)
            .collect();
        let removed: Vec<_> = items
            .iter()
            .filter(|i| i.operation == DeltaOp::Removed)
            .collect();

        assert_eq!(added.len(), 1);
        assert_eq!(modified.len(), 1);
        assert_eq!(removed.len(), 1);
    }

    #[test]
    fn test_parse_delta_content_multiple_requirements_per_section() {
        let content = r#"## ADDED Requirements

### Requirement: Feature A

Description A.

#### Scenario: A works

- **WHEN** A
- **THEN** A

### Requirement: Feature B

Description B.

#### Scenario: B works

- **WHEN** B
- **THEN** B
"#;

        let items = parse_delta_content(content);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Feature A");
        assert_eq!(items[1].name, "Feature B");
    }

    // ==================== parse_change tests ====================

    #[test]
    fn test_parse_change_full() {
        let proposal = r#"# Add Feature

## Why

This feature is needed for users.

## What Changes

- Add new endpoint
- Update schema
"#;

        let tasks = r#"# Tasks

## 1. Implementation
- [x] 1.1 Create schema
- [ ] 1.2 Implement API
- [ ] 1.3 Write tests
"#;

        let delta_spec = r#"## ADDED Requirements

### Requirement: New Feature

The system SHALL support the new feature.

#### Scenario: Success

- **WHEN** user triggers
- **THEN** success
"#;

        let (_temp_dir, change_dir) =
            create_temp_change(proposal, Some(tasks), &[("feature", delta_spec)]);

        let change = parse_change(&change_dir).unwrap();

        assert!(change.why.contains("This feature is needed"));
        assert!(change.what_changes.contains("Add new endpoint"));
        assert_eq!(change.tasks_completed, 1);
        assert_eq!(change.tasks_total, 3);
        assert_eq!(change.deltas.len(), 1);
        assert_eq!(change.deltas[0].capability, "feature");
        assert_eq!(change.deltas[0].items.len(), 1);
    }

    #[test]
    fn test_parse_change_missing_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().to_path_buf();

        let result = parse_change(&change_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing proposal.md"));
    }

    #[test]
    fn test_parse_change_nonexistent_directory() {
        let result = parse_change(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    // ==================== format_change tests ====================

    #[test]
    fn test_format_change_includes_header() {
        let change = ChangeInfo {
            name: "add-feature".to_string(),
            why: "Feature needed.".to_string(),
            what_changes: "- Add feature".to_string(),
            tasks_completed: 2,
            tasks_total: 5,
            deltas: vec![],
        };

        let output = format_change(&change);
        assert!(output.contains("Change: add-feature"));
    }

    #[test]
    fn test_format_change_includes_why() {
        let change = ChangeInfo {
            name: "test".to_string(),
            why: "This is the reason.".to_string(),
            what_changes: "- Change".to_string(),
            tasks_completed: 0,
            tasks_total: 0,
            deltas: vec![],
        };

        let output = format_change(&change);
        assert!(output.contains("Why:"));
        assert!(output.contains("This is the reason."));
    }

    #[test]
    fn test_format_change_includes_what_changes() {
        let change = ChangeInfo {
            name: "test".to_string(),
            why: "Reason".to_string(),
            what_changes: "- First change\n- Second change".to_string(),
            tasks_completed: 0,
            tasks_total: 0,
            deltas: vec![],
        };

        let output = format_change(&change);
        assert!(output.contains("What Changes:"));
        assert!(output.contains("First change"));
        assert!(output.contains("Second change"));
    }

    #[test]
    fn test_format_change_includes_tasks_progress() {
        let change = ChangeInfo {
            name: "test".to_string(),
            why: "Reason".to_string(),
            what_changes: "- Change".to_string(),
            tasks_completed: 2,
            tasks_total: 5,
            deltas: vec![],
        };

        let output = format_change(&change);
        assert!(output.contains("Tasks:"));
        assert!(output.contains("2/5"));
        assert!(output.contains("40%"));
    }

    #[test]
    fn test_format_change_includes_deltas() {
        let change = ChangeInfo {
            name: "test".to_string(),
            why: "Reason".to_string(),
            what_changes: "- Change".to_string(),
            tasks_completed: 0,
            tasks_total: 0,
            deltas: vec![DeltaGroup {
                capability: "auth".to_string(),
                items: vec![DeltaItem {
                    operation: DeltaOp::Added,
                    name: "New Feature".to_string(),
                    text: "Description".to_string(),
                    scenarios: vec!["Test".to_string()],
                }],
            }],
        };

        let output = format_change(&change);
        assert!(output.contains("Deltas:"));
        assert!(output.contains("auth/"));
        assert!(output.contains("New Feature"));
    }

    // ==================== format_deltas_only tests ====================

    #[test]
    fn test_format_deltas_only_header() {
        let change = ChangeInfo {
            name: "my-change".to_string(),
            why: String::new(),
            what_changes: String::new(),
            tasks_completed: 0,
            tasks_total: 0,
            deltas: vec![],
        };

        let output = format_deltas_only(&change);
        assert!(output.contains("Deltas for:"));
        assert!(output.contains("my-change"));
    }

    #[test]
    fn test_format_deltas_only_shows_requirements() {
        let change = ChangeInfo {
            name: "test".to_string(),
            why: String::new(),
            what_changes: String::new(),
            tasks_completed: 0,
            tasks_total: 0,
            deltas: vec![DeltaGroup {
                capability: "auth".to_string(),
                items: vec![
                    DeltaItem {
                        operation: DeltaOp::Added,
                        name: "Two-Factor Auth".to_string(),
                        text: "The system SHALL support 2FA.".to_string(),
                        scenarios: vec!["TOTP setup".to_string(), "TOTP verify".to_string()],
                    },
                    DeltaItem {
                        operation: DeltaOp::Modified,
                        name: "User Login".to_string(),
                        text: "The system SHALL require 2FA.".to_string(),
                        scenarios: vec!["Valid credentials with 2FA".to_string()],
                    },
                ],
            }],
        };

        let output = format_deltas_only(&change);
        assert!(output.contains("auth/"));
        assert!(output.contains("Two-Factor Auth"));
        assert!(output.contains("system SHALL support 2FA"));
        assert!(output.contains("TOTP setup"));
        assert!(output.contains("User Login"));
        assert!(output.contains("MODIFIED"));
    }

    // ==================== show_change tests ====================

    #[test]
    fn test_show_change_success() {
        let proposal = r#"# Test Change

## Why

Test reason.

## What Changes

- Test change
"#;

        let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

        let delta_spec = r#"## ADDED Requirements

### Requirement: Test

Test description.

#### Scenario: Works

- **WHEN** triggered
- **THEN** works
"#;

        let (_temp_dir, change_dir) =
            create_temp_change(proposal, Some(tasks), &[("test", delta_spec)]);

        let result = show_change(&change_dir);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Change:"));
        assert!(output.contains("Why:"));
    }

    #[test]
    fn test_show_change_error() {
        let result = show_change(Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_show_change_deltas_only_success() {
        let proposal = r#"# Test Change

## Why

Test reason.

## What Changes

- Test change
"#;

        let delta_spec = r#"## ADDED Requirements

### Requirement: Test

Test description.

#### Scenario: Works

- **WHEN** triggered
- **THEN** works
"#;

        let (_temp_dir, change_dir) = create_temp_change(proposal, None, &[("test", delta_spec)]);

        let result = show_change_deltas_only(&change_dir);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Deltas for:"));
        assert!(output.contains("Test"));
    }

    // ==================== DeltaOp tests ====================

    #[test]
    fn test_delta_op_symbols() {
        assert_eq!(DeltaOp::Added.symbol(), "+");
        assert_eq!(DeltaOp::Modified.symbol(), "~");
        assert_eq!(DeltaOp::Removed.symbol(), "-");
        assert_eq!(DeltaOp::Renamed.symbol(), ">");
    }

    #[test]
    fn test_delta_op_labels() {
        assert_eq!(DeltaOp::Added.label(), "ADDED");
        assert_eq!(DeltaOp::Modified.label(), "MODIFIED");
        assert_eq!(DeltaOp::Removed.label(), "REMOVED");
        assert_eq!(DeltaOp::Renamed.label(), "RENAMED");
    }
}
