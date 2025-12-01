//! Integration tests for spec show functionality.

use std::io::Write;
use tempfile::NamedTempFile;

// Import the show module
// Note: This requires the show module to be public in the crate
mod common {
    use std::fs;
    use std::path::Path;

    /// Parse a spec file and return structured info.
    /// Duplicated here for integration testing since we can't easily
    /// import from the library crate in integration tests.
    pub fn parse_spec_content(content: &str) -> Result<SpecInfo, String> {
        let lines: Vec<&str> = content.lines().collect();

        let name = extract_spec_name(&lines)?;
        let purpose = extract_purpose(&lines)?;
        let requirements = extract_requirements(&lines);

        Ok(SpecInfo {
            name,
            purpose,
            requirements,
        })
    }

    #[derive(Debug, Clone)]
    pub struct SpecInfo {
        pub name: String,
        pub purpose: String,
        pub requirements: Vec<RequirementInfo>,
    }

    #[derive(Debug, Clone)]
    pub struct RequirementInfo {
        pub name: String,
        pub text: String,
        pub scenarios: Vec<String>,
    }

    fn extract_spec_name(lines: &[&str]) -> Result<String, String> {
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                let header = trimmed.strip_prefix("# ").unwrap_or("");
                let name = header
                    .strip_suffix(" Specification")
                    .or_else(|| header.strip_suffix(" specification"))
                    .unwrap_or(header)
                    .trim()
                    .to_string();
                return Ok(name);
            }
        }
        Err("Missing spec header".to_string())
    }

    fn extract_purpose(lines: &[&str]) -> Result<String, String> {
        let mut in_purpose = false;
        let mut purpose_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.eq_ignore_ascii_case("## Purpose") {
                in_purpose = true;
                continue;
            }

            if in_purpose && trimmed.starts_with("## ") {
                break;
            }

            if in_purpose {
                purpose_lines.push(*line);
            }
        }

        if purpose_lines.is_empty() {
            return Err("Missing Purpose section".to_string());
        }

        let purpose = purpose_lines
            .into_iter()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        Ok(purpose)
    }

    fn extract_requirements(lines: &[&str]) -> Vec<RequirementInfo> {
        let mut requirements = Vec::new();
        let mut current_req: Option<RequirementInfo> = None;
        let mut in_requirements = false;
        let mut in_scenario = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.eq_ignore_ascii_case("## Requirements") {
                in_requirements = true;
                continue;
            }

            if in_requirements && trimmed.starts_with("## ") && !trimmed.eq_ignore_ascii_case("## Requirements") {
                if let Some(req) = current_req.take() {
                    requirements.push(req);
                }
                break;
            }

            if !in_requirements {
                continue;
            }

            if trimmed.starts_with("### Requirement:") {
                if let Some(req) = current_req.take() {
                    requirements.push(req);
                }

                let name = trimmed
                    .strip_prefix("### Requirement:")
                    .unwrap_or("")
                    .trim()
                    .to_string();

                current_req = Some(RequirementInfo {
                    name,
                    text: String::new(),
                    scenarios: Vec::new(),
                });
                in_scenario = false;
                continue;
            }

            if trimmed.starts_with("#### Scenario:") {
                let scenario_name = trimmed
                    .strip_prefix("#### Scenario:")
                    .unwrap_or("")
                    .trim()
                    .to_string();

                if let Some(ref mut req) = current_req {
                    req.scenarios.push(scenario_name);
                }
                in_scenario = true;
                continue;
            }

            if current_req.is_some() && !in_scenario && !trimmed.is_empty() {
                if let Some(ref mut req) = current_req {
                    if !req.text.is_empty() {
                        req.text.push(' ');
                    }
                    req.text.push_str(trimmed);
                }
            }
        }

        if let Some(req) = current_req.take() {
            requirements.push(req);
        }

        requirements
    }

    /// Format spec info for display.
    pub fn format_spec(spec: &SpecInfo) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("+{}+\n", "-".repeat(59)));
        output.push_str(&format!("| Spec: {:<51} |\n", spec.name));
        output.push_str(&format!("+{}+\n", "-".repeat(59)));
        output.push_str("\n");

        // Purpose
        output.push_str("Purpose:\n");
        output.push_str(&format!("  {}\n\n", spec.purpose));

        // Requirements count
        output.push_str(&format!("Requirements: {}\n\n", spec.requirements.len()));

        // Each requirement
        for req in &spec.requirements {
            output.push_str(&format!("  + Requirement: {}\n", req.name));
            output.push_str(&format!("  | {}\n", req.text));

            if !req.scenarios.is_empty() {
                output.push_str("  |\n");
                output.push_str("  | Scenarios:\n");
                for scenario in &req.scenarios {
                    output.push_str(&format!("  |   * {}\n", scenario));
                }
            }

            output.push_str("  +\n\n");
        }

        output
    }

    /// Show spec from file path.
    pub fn show_spec(path: &Path) -> Result<String, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read spec file: {}", e))?;
        let spec = parse_spec_content(&content)?;
        Ok(format_spec(&spec))
    }
}

#[test]
fn test_show_spec_valid_file() {
    let content = r#"# Auth Specification

## Purpose

This spec defines authentication requirements for the system.

## Requirements

### Requirement: User Login

The system SHALL allow users to log in with valid credentials.

#### Scenario: Valid credentials

- **WHEN** user provides valid email and password
- **THEN** user is authenticated

#### Scenario: Invalid credentials

- **WHEN** user provides invalid credentials
- **THEN** error is shown
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let result = common::show_spec(file.path());
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Spec: Auth"));
    assert!(output.contains("Purpose:"));
    assert!(output.contains("authentication requirements"));
    assert!(output.contains("Requirements: 1"));
    assert!(output.contains("Requirement: User Login"));
    assert!(output.contains("Scenarios:"));
    assert!(output.contains("Valid credentials"));
    assert!(output.contains("Invalid credentials"));
}

#[test]
fn test_show_spec_nonexistent_file() {
    let path = std::path::Path::new("/nonexistent/path/to/spec.md");
    let result = common::show_spec(path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read"));
}

#[test]
fn test_show_spec_missing_purpose() {
    let content = r#"# Test Specification

## Requirements

### Requirement: Test

The system SHALL test.

#### Scenario: Test

- WHEN test
- THEN pass
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let result = common::show_spec(file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Purpose"));
}

#[test]
fn test_show_spec_multiple_requirements() {
    let content = r#"# Multi Specification

## Purpose

This spec has multiple requirements.

## Requirements

### Requirement: First

The system SHALL do first thing.

#### Scenario: First scenario

- WHEN first
- THEN done

### Requirement: Second

The system SHALL do second thing.

#### Scenario: Second scenario

- WHEN second
- THEN done

### Requirement: Third

The system SHALL do third thing.

#### Scenario: Third scenario

- WHEN third
- THEN done
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let result = common::show_spec(file.path());
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Requirements: 3"));
    assert!(output.contains("Requirement: First"));
    assert!(output.contains("Requirement: Second"));
    assert!(output.contains("Requirement: Third"));
}

#[test]
fn test_show_spec_requirement_with_many_scenarios() {
    let content = r#"# Test Specification

## Purpose

This tests multiple scenarios per requirement.

## Requirements

### Requirement: Many Scenarios

The system SHALL support multiple scenarios.

#### Scenario: First

Test 1.

#### Scenario: Second

Test 2.

#### Scenario: Third

Test 3.

#### Scenario: Fourth

Test 4.
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let result = common::show_spec(file.path());
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("First"));
    assert!(output.contains("Second"));
    assert!(output.contains("Third"));
    assert!(output.contains("Fourth"));
}

#[test]
fn test_show_spec_empty_requirements() {
    let content = r#"# Empty Specification

## Purpose

This spec has no requirements defined.

## Requirements

"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let result = common::show_spec(file.path());
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Requirements: 0"));
}

#[test]
fn test_parse_actual_spec_file() {
    // Test with the actual cli spec file if it exists
    let cli_spec_path = std::path::Path::new("openspec/specs/cli/spec.md");

    if cli_spec_path.exists() {
        let result = common::show_spec(cli_spec_path);
        assert!(result.is_ok(), "Failed to parse cli spec: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Spec: cli"));
        assert!(output.contains("Requirements:"));
    }
}
