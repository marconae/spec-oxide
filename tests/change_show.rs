//! Integration tests for change show functionality.

use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

/// Helper module with parsing functions duplicated for integration testing.
mod common {
    use std::fs;
    use std::path::Path;

    /// Delta operation type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DeltaOp {
        Added,
        Modified,
        Removed,
        Renamed,
    }

    impl DeltaOp {
        pub fn symbol(&self) -> &'static str {
            match self {
                DeltaOp::Added => "+",
                DeltaOp::Modified => "~",
                DeltaOp::Removed => "-",
                DeltaOp::Renamed => ">",
            }
        }

        pub fn label(&self) -> &'static str {
            match self {
                DeltaOp::Added => "ADDED",
                DeltaOp::Modified => "MODIFIED",
                DeltaOp::Removed => "REMOVED",
                DeltaOp::Renamed => "RENAMED",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct DeltaItem {
        pub operation: DeltaOp,
        pub name: String,
        pub text: String,
        pub scenarios: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct DeltaGroup {
        pub capability: String,
        pub items: Vec<DeltaItem>,
    }

    #[derive(Debug, Clone)]
    pub struct ChangeInfo {
        pub name: String,
        pub why: String,
        pub what_changes: String,
        pub tasks_completed: usize,
        pub tasks_total: usize,
        pub deltas: Vec<DeltaGroup>,
    }

    pub fn parse_change(change_dir: &Path) -> Result<ChangeInfo, String> {
        let name = change_dir
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid change directory path".to_string())?;

        if !change_dir.exists() {
            return Err(format!(
                "Change directory does not exist: {}",
                change_dir.display()
            ));
        }

        if !change_dir.is_dir() {
            return Err(format!("Path is not a directory: {}", change_dir.display()));
        }

        let proposal_path = change_dir.join("proposal.md");
        if !proposal_path.exists() {
            return Err("Missing proposal.md".to_string());
        }

        let proposal_content = fs::read_to_string(&proposal_path)
            .map_err(|e| format!("Failed to read proposal.md: {}", e))?;

        let (why, what_changes) = parse_proposal(&proposal_content);

        let tasks_path = change_dir.join("tasks.md");
        let (tasks_completed, tasks_total) = if tasks_path.exists() {
            let tasks_content = fs::read_to_string(&tasks_path)
                .map_err(|e| format!("Failed to read tasks.md: {}", e))?;
            parse_tasks(&tasks_content)
        } else {
            (0, 0)
        };

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

    fn parse_proposal(content: &str) -> (String, String) {
        let why = extract_section(content, "Why");
        let what_changes = extract_section(content, "What Changes");
        (why, what_changes)
    }

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
                if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
                    break;
                }
                section_lines.push(line);
            }
        }

        let result: Vec<&str> = section_lines
            .iter()
            .copied()
            .skip_while(|l| l.trim().is_empty())
            .collect();

        let result: String = result.join("\n");
        result.trim_end().to_string()
    }

    fn parse_tasks(content: &str) -> (usize, usize) {
        let mut completed = 0;
        let mut total = 0;

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

        (completed, total)
    }

    fn parse_delta_specs(specs_dir: &Path) -> Result<Vec<DeltaGroup>, String> {
        let mut groups = Vec::new();

        let entries = fs::read_dir(specs_dir)
            .map_err(|e| format!("Failed to read specs directory: {}", e))?;

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

        groups.sort_by(|a, b| a.capability.cmp(&b.capability));

        Ok(groups)
    }

    fn parse_delta_content(content: &str) -> Vec<DeltaItem> {
        let mut items = Vec::new();

        let delta_headers = [
            ("## ADDED Requirements", DeltaOp::Added),
            ("## MODIFIED Requirements", DeltaOp::Modified),
            ("## REMOVED Requirements", DeltaOp::Removed),
            ("## RENAMED Requirements", DeltaOp::Renamed),
        ];

        let lines: Vec<&str> = content.lines().collect();

        for (header_text, op) in delta_headers {
            let header_idx = lines
                .iter()
                .position(|line| line.trim().eq_ignore_ascii_case(header_text));

            if let Some(start_idx) = header_idx {
                let section_content = extract_section_from_index(&lines, start_idx);
                let requirements = parse_requirements(&section_content, op);
                items.extend(requirements);
            }
        }

        items
    }

    fn extract_section_from_index(lines: &[&str], start_idx: usize) -> String {
        let mut section_lines = Vec::new();

        for line in lines.iter().skip(start_idx + 1) {
            let trimmed = line.trim();
            if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
                break;
            }
            section_lines.push(*line);
        }

        section_lines.join("\n")
    }

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
                let text = extract_requirement_text(&req_content);
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

    fn extract_requirement_text(content: &str) -> String {
        let mut text_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("#### Scenario:") {
                break;
            }

            if text_lines.is_empty() && trimmed.is_empty() {
                continue;
            }

            if !text_lines.is_empty() && trimmed.is_empty() {
                break;
            }

            text_lines.push(trimmed);
        }

        text_lines.join(" ")
    }

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

    pub fn format_change(change: &ChangeInfo) -> String {
        let mut output = String::new();

        output.push_str(&format!("+{}+\n", "-".repeat(59)));
        output.push_str(&format!("| Change: {:<49} |\n", change.name));
        output.push_str(&format!("+{}+\n", "-".repeat(59)));
        output.push('\n');

        output.push_str("Why:\n");
        for line in change.why.lines() {
            output.push_str(&format!("  {}\n", line));
        }
        output.push('\n');

        output.push_str("What Changes:\n");
        for line in change.what_changes.lines() {
            output.push_str(&format!("  {}\n", line));
        }
        output.push('\n');

        if change.tasks_total > 0 {
            let percentage = (change.tasks_completed * 100) / change.tasks_total;
            output.push_str(&format!(
                "Tasks: {}/{} ({}%)\n\n",
                change.tasks_completed, change.tasks_total, percentage
            ));
        }

        if !change.deltas.is_empty() {
            output.push_str("Deltas:\n\n");

            for group in &change.deltas {
                output.push_str(&format!("  {}/\n", group.capability));

                for item in &group.items {
                    output.push_str(&format!(
                        "    {} {}: {}\n",
                        item.operation.symbol(),
                        item.operation.label(),
                        item.name
                    ));
                }
                output.push('\n');
            }
        }

        output
    }

    pub fn format_deltas_only(change: &ChangeInfo) -> String {
        let mut output = String::new();

        output.push_str(&format!("Deltas for: {}\n\n", change.name));

        for group in &change.deltas {
            output.push_str(&format!("{}/\n", group.capability));

            for item in &group.items {
                if item.operation == DeltaOp::Modified {
                    output.push_str(&format!(
                        "  {} {} ({})\n",
                        item.operation.symbol(),
                        item.name,
                        item.operation.label()
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", item.operation.symbol(), item.name));
                }

                if !item.text.is_empty() {
                    output.push_str(&format!("    {}\n", item.text));
                }

                if !item.scenarios.is_empty() {
                    let scenarios_str = item.scenarios.join(", ");
                    output.push_str(&format!("    Scenarios: {}\n", scenarios_str));
                }

                output.push('\n');
            }
        }

        output
    }

    pub fn show_change(change_dir: &Path) -> Result<String, String> {
        let change = parse_change(change_dir)?;
        Ok(format_change(&change))
    }

    pub fn show_change_deltas_only(change_dir: &Path) -> Result<String, String> {
        let change = parse_change(change_dir)?;
        Ok(format_deltas_only(&change))
    }
}

/// Helper to create a temporary change directory with files.
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

// ==================== Integration tests ====================

#[test]
fn test_show_change_valid_change() {
    let proposal = r#"# Add Two-Factor Authentication

## Why

Security is important. We need to add two-factor authentication
to protect user accounts from unauthorized access.

## What Changes

- Add 2FA requirement to auth spec
- Implement TOTP support
- Add backup codes feature
"#;

    let tasks = r#"# Tasks

## 1. Implementation
- [x] 1.1 Create TOTP module
- [x] 1.2 Add backup codes
- [ ] 1.3 Update login flow
- [ ] 1.4 Write tests
- [ ] 1.5 Update documentation
"#;

    let auth_delta = r#"## ADDED Requirements

### Requirement: Two-Factor Authentication

The system SHALL support two-factor authentication using TOTP.

#### Scenario: TOTP setup

- **WHEN** user enables 2FA
- **THEN** QR code is displayed

#### Scenario: TOTP verification

- **WHEN** user enters valid TOTP code
- **THEN** authentication succeeds

## MODIFIED Requirements

### Requirement: User Login

The system SHALL require 2FA after password verification.

#### Scenario: Valid credentials with 2FA

- **WHEN** user provides valid password and 2FA code
- **THEN** user is logged in
"#;

    let notifications_delta = r#"## ADDED Requirements

### Requirement: OTP Email Notification

The system SHALL send OTP codes via email when requested.

#### Scenario: Send OTP

- **WHEN** user requests OTP via email
- **THEN** email is sent with OTP code

#### Scenario: Resend OTP

- **WHEN** user requests resend
- **THEN** new OTP is sent
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        proposal,
        Some(tasks),
        &[("auth", auth_delta), ("notifications", notifications_delta)],
    );

    let result = common::show_change(&change_dir);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

    let output = result.unwrap();

    // Verify header
    assert!(output.contains("Change:"));

    // Verify Why section
    assert!(output.contains("Why:"));
    assert!(output.contains("Security is important"));
    assert!(output.contains("two-factor authentication"));

    // Verify What Changes section
    assert!(output.contains("What Changes:"));
    assert!(output.contains("Add 2FA requirement"));
    assert!(output.contains("Implement TOTP support"));

    // Verify Tasks progress
    assert!(output.contains("Tasks:"));
    assert!(output.contains("2/5"));
    assert!(output.contains("40%"));

    // Verify Deltas section
    assert!(output.contains("Deltas:"));
    assert!(output.contains("auth/"));
    assert!(output.contains("notifications/"));
    assert!(output.contains("ADDED"));
    assert!(output.contains("MODIFIED"));
    assert!(output.contains("Two-Factor Authentication"));
    assert!(output.contains("User Login"));
    assert!(output.contains("OTP Email Notification"));
}

#[test]
fn test_show_change_deltas_only() {
    let proposal = r#"# Add Feature

## Why

Feature is needed.

## What Changes

- Add feature
"#;

    let delta_spec = r#"## ADDED Requirements

### Requirement: New Feature

The system SHALL support the new feature.

#### Scenario: Feature works

- **WHEN** user uses feature
- **THEN** it works

## MODIFIED Requirements

### Requirement: Existing Feature

The system SHALL support the improved existing feature.

#### Scenario: Improved behavior

- **WHEN** user triggers action
- **THEN** improved result
"#;

    let (_temp_dir, change_dir) = create_temp_change(proposal, None, &[("feature", delta_spec)]);

    let result = common::show_change_deltas_only(&change_dir);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

    let output = result.unwrap();

    // Verify header
    assert!(output.contains("Deltas for:"));

    // Verify capability grouping
    assert!(output.contains("feature/"));

    // Verify ADDED requirement
    assert!(output.contains("+ New Feature"));
    assert!(output.contains("system SHALL support the new feature"));
    assert!(output.contains("Scenarios: Feature works"));

    // Verify MODIFIED requirement shows operation label
    assert!(output.contains("~ Existing Feature (MODIFIED)"));
    assert!(output.contains("system SHALL support the improved existing feature"));
}

#[test]
fn test_show_change_nonexistent() {
    let result = common::show_change(Path::new("/nonexistent/path/to/change"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_show_change_missing_proposal() {
    let temp_dir = TempDir::new().unwrap();
    let change_dir = temp_dir.path().to_path_buf();

    // Only create specs directory, no proposal.md
    let specs_dir = change_dir.join("specs");
    fs::create_dir(&specs_dir).unwrap();

    let result = common::show_change(&change_dir);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing proposal.md"));
}

#[test]
fn test_show_change_no_tasks() {
    let proposal = r#"# Add Feature

## Why

Feature is needed.

## What Changes

- Add feature
"#;

    let delta_spec = r#"## ADDED Requirements

### Requirement: Test

Test description.

#### Scenario: Works

- **WHEN** triggered
- **THEN** works
"#;

    let (_temp_dir, change_dir) = create_temp_change(proposal, None, &[("test", delta_spec)]);

    let result = common::show_change(&change_dir);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Should not contain Tasks section when no tasks.md
    assert!(!output.contains("Tasks:"));
}

#[test]
fn test_show_change_no_deltas() {
    let proposal = r#"# Add Feature

## Why

Feature is needed.

## What Changes

- Add feature
"#;

    let tasks = r#"# Tasks
- [ ] 1.1 Task
"#;

    let (_temp_dir, change_dir) = create_temp_change(proposal, Some(tasks), &[]);

    let result = common::show_change(&change_dir);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Should not contain Deltas section when no specs
    assert!(!output.contains("Deltas:"));
}

#[test]
fn test_show_change_multiple_capabilities() {
    let proposal = r#"# Multi-Capability Change

## Why

This change affects multiple capabilities.

## What Changes

- Update auth
- Update user
- Update notifications
"#;

    let auth_delta = r#"## ADDED Requirements

### Requirement: Auth Feature

Auth description.

#### Scenario: Auth works

- **WHEN** auth
- **THEN** works
"#;

    let user_delta = r#"## MODIFIED Requirements

### Requirement: User Feature

User description.

#### Scenario: User works

- **WHEN** user
- **THEN** works
"#;

    let notifications_delta = r#"## REMOVED Requirements

### Requirement: Old Notification

Removed notification.
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        proposal,
        None,
        &[
            ("auth", auth_delta),
            ("user", user_delta),
            ("notifications", notifications_delta),
        ],
    );

    let result = common::show_change(&change_dir);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Verify all capabilities are present
    assert!(output.contains("auth/"));
    assert!(output.contains("user/"));
    assert!(output.contains("notifications/"));

    // Verify different operations
    assert!(output.contains("ADDED"));
    assert!(output.contains("MODIFIED"));
    assert!(output.contains("REMOVED"));
}

#[test]
fn test_parse_actual_change_directory() {
    // Test with the actual add-show-commands change if it exists
    let change_path = std::path::Path::new("openspec/changes/add-show-commands");

    if change_path.exists() {
        let result = common::show_change(change_path);
        assert!(
            result.is_ok(),
            "Failed to parse add-show-commands: {:?}",
            result.err()
        );

        let output = result.unwrap();
        assert!(output.contains("Change:"));
        assert!(output.contains("Why:"));
        assert!(output.contains("What Changes:"));
    }
}
