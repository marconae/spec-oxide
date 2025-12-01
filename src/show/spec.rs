//! Spec display module.
//!
//! Provides functionality to parse and format spec files for display.

use std::fs;
use std::path::Path;

use super::{box_header, cyan_bold, dim, green, is_tty, yellow};

/// Default box width for headers.
const DEFAULT_BOX_WIDTH: usize = 63;

/// Information about a parsed spec.
#[derive(Debug, Clone)]
pub struct SpecInfo {
    /// The name of the spec (extracted from header).
    pub name: String,
    /// The purpose of the spec.
    pub purpose: String,
    /// List of requirements in the spec.
    pub requirements: Vec<RequirementInfo>,
}

/// Information about a single requirement.
#[derive(Debug, Clone)]
pub struct RequirementInfo {
    /// The name of the requirement.
    pub name: String,
    /// The description text of the requirement.
    pub text: String,
    /// List of scenario names.
    pub scenarios: Vec<String>,
}

/// Parse a spec file and return structured info.
///
/// # Arguments
/// * `path` - Path to the spec file
///
/// # Returns
/// A `SpecInfo` struct or an error string.
pub fn parse_spec(path: &Path) -> Result<SpecInfo, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read spec file: {}", e))?;

    parse_spec_content(&content)
}

/// Parse spec content string and return structured info.
///
/// # Arguments
/// * `content` - The spec file content as a string
///
/// # Returns
/// A `SpecInfo` struct or an error string.
pub fn parse_spec_content(content: &str) -> Result<SpecInfo, String> {
    let lines: Vec<&str> = content.lines().collect();

    // Extract spec name from header (# <name> Specification)
    let name = extract_spec_name(&lines)?;

    // Extract purpose section
    let purpose = extract_purpose(&lines)?;

    // Extract requirements
    let requirements = extract_requirements(&lines);

    Ok(SpecInfo {
        name,
        purpose,
        requirements,
    })
}

/// Extract the spec name from the first header line.
fn extract_spec_name(lines: &[&str]) -> Result<String, String> {
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            // Extract name from "# <name> Specification" or just "# <name>"
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
    Err("Missing spec header (expected '# <name> Specification')".to_string())
}

/// Extract the purpose section content.
fn extract_purpose(lines: &[&str]) -> Result<String, String> {
    let mut in_purpose = false;
    let mut purpose_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Check for Purpose header (case-insensitive)
        if trimmed.eq_ignore_ascii_case("## Purpose") {
            in_purpose = true;
            continue;
        }

        // Stop at next level-2 header
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

    // Trim and join purpose lines
    let purpose = purpose_lines
        .into_iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    Ok(purpose)
}

/// Extract all requirements from the spec content.
fn extract_requirements(lines: &[&str]) -> Vec<RequirementInfo> {
    let mut requirements = Vec::new();
    let mut current_req: Option<RequirementInfo> = None;
    let mut in_requirements = false;
    let mut in_scenario = false;

    for line in lines {
        let trimmed = line.trim();

        // Check for Requirements section
        if trimmed.eq_ignore_ascii_case("## Requirements") {
            in_requirements = true;
            continue;
        }

        // Stop at next level-2 header (outside Requirements)
        if in_requirements && trimmed.starts_with("## ") && !trimmed.eq_ignore_ascii_case("## Requirements") {
            // Save current requirement if any
            if let Some(req) = current_req.take() {
                requirements.push(req);
            }
            break;
        }

        if !in_requirements {
            continue;
        }

        // Check for requirement header (### Requirement: <name>)
        if trimmed.starts_with("### Requirement:") {
            // Save previous requirement
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

        // Check for scenario header (#### Scenario: <name>)
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

        // Accumulate requirement text (before scenarios)
        if current_req.is_some() && !in_scenario && !trimmed.is_empty() {
            if let Some(ref mut req) = current_req {
                if !req.text.is_empty() {
                    req.text.push(' ');
                }
                req.text.push_str(trimmed);
            }
        }
    }

    // Save last requirement
    if let Some(req) = current_req.take() {
        requirements.push(req);
    }

    requirements
}

/// Format spec info for display (with colors if TTY).
///
/// # Arguments
/// * `spec` - The spec info to format
///
/// # Returns
/// A formatted string ready for display.
pub fn format_spec(spec: &SpecInfo) -> String {
    let mut output = String::new();

    // Header box
    let header_title = if is_tty() {
        format!("Spec: {}", cyan_bold(&spec.name))
    } else {
        format!("Spec: {}", spec.name)
    };
    output.push_str(&box_header(&header_title, DEFAULT_BOX_WIDTH));
    output.push_str("\n\n");

    // Purpose section
    let purpose_label = if is_tty() {
        yellow("Purpose:")
    } else {
        "Purpose:".to_string()
    };
    output.push_str(&purpose_label);
    output.push('\n');
    output.push_str(&format!("  {}\n\n", spec.purpose));

    // Requirements count
    output.push_str(&format!("Requirements: {}\n\n", spec.requirements.len()));

    // Each requirement
    for req in &spec.requirements {
        output.push_str(&format_requirement(req));
        output.push('\n');
    }

    output
}

/// Format a single requirement for display.
fn format_requirement(req: &RequirementInfo) -> String {
    let mut output = String::new();

    // Requirement name
    let req_name = if is_tty() {
        green(&req.name)
    } else {
        req.name.clone()
    };
    output.push_str(&format!("  \u{250C} Requirement: {}\n", req_name));

    // Requirement text
    output.push_str(&format!("  \u{2502} {}\n", req.text));

    // Scenarios
    if !req.scenarios.is_empty() {
        output.push_str("  \u{2502}\n");
        output.push_str("  \u{2502} Scenarios:\n");
        for scenario in &req.scenarios {
            let scenario_text = if is_tty() {
                dim(scenario)
            } else {
                scenario.clone()
            };
            output.push_str(&format!("  \u{2502}   \u{2022} {}\n", scenario_text));
        }
    }

    output.push_str("  \u{2514}\n");

    output
}

/// Main entry point: parse and format a spec.
///
/// # Arguments
/// * `path` - Path to the spec file
///
/// # Returns
/// A formatted string or an error string.
pub fn show_spec(path: &Path) -> Result<String, String> {
    let spec = parse_spec(path)?;
    Ok(format_spec(&spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SPEC: &str = r#"# Auth Specification

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

### Requirement: Session Management

The system SHALL manage user sessions securely.

#### Scenario: Session creation

- **WHEN** user logs in
- **THEN** session is created

#### Scenario: Session expiry

- **WHEN** session times out
- **THEN** user must re-authenticate
"#;

    #[test]
    fn test_parse_spec_valid() {
        let result = parse_spec_content(VALID_SPEC);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.name, "Auth");
        assert!(spec.purpose.contains("authentication requirements"));
        assert_eq!(spec.requirements.len(), 2);
    }

    #[test]
    fn test_parse_spec_extracts_name() {
        let content = "# MySpec Specification\n\n## Purpose\n\nTest purpose.\n";
        let result = parse_spec_content(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "MySpec");
    }

    #[test]
    fn test_parse_spec_extracts_name_without_suffix() {
        let content = "# MySpec\n\n## Purpose\n\nTest purpose.\n";
        let result = parse_spec_content(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "MySpec");
    }

    #[test]
    fn test_parse_spec_missing_purpose() {
        let content = "# Test Specification\n\n## Requirements\n\n### Requirement: Test\n";
        let result = parse_spec_content(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Purpose"));
    }

    #[test]
    fn test_parse_spec_missing_header() {
        let content = "## Purpose\n\nTest purpose.\n";
        let result = parse_spec_content(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("header"));
    }

    #[test]
    fn test_parse_spec_requirements() {
        let result = parse_spec_content(VALID_SPEC).unwrap();

        assert_eq!(result.requirements.len(), 2);

        let first_req = &result.requirements[0];
        assert_eq!(first_req.name, "User Login");
        assert!(first_req.text.contains("SHALL"));
        assert_eq!(first_req.scenarios.len(), 2);
        assert_eq!(first_req.scenarios[0], "Valid credentials");
        assert_eq!(first_req.scenarios[1], "Invalid credentials");

        let second_req = &result.requirements[1];
        assert_eq!(second_req.name, "Session Management");
        assert_eq!(second_req.scenarios.len(), 2);
    }

    #[test]
    fn test_parse_spec_requirement_with_multiple_scenarios() {
        let content = r#"# Test Specification

## Purpose

Test purpose text here.

## Requirements

### Requirement: Multi-Scenario

The system SHALL support multiple scenarios.

#### Scenario: First

Test first.

#### Scenario: Second

Test second.

#### Scenario: Third

Test third.
"#;

        let result = parse_spec_content(content).unwrap();
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].scenarios.len(), 3);
        assert_eq!(result.requirements[0].scenarios[0], "First");
        assert_eq!(result.requirements[0].scenarios[1], "Second");
        assert_eq!(result.requirements[0].scenarios[2], "Third");
    }

    #[test]
    fn test_format_spec_structure() {
        let spec = SpecInfo {
            name: "test".to_string(),
            purpose: "Test purpose here.".to_string(),
            requirements: vec![RequirementInfo {
                name: "Test Requirement".to_string(),
                text: "The system SHALL test.".to_string(),
                scenarios: vec!["Scenario One".to_string()],
            }],
        };

        let output = format_spec(&spec);

        // Check structure elements are present
        assert!(output.contains("Spec:"));
        assert!(output.contains("test"));
        assert!(output.contains("Purpose:"));
        assert!(output.contains("Test purpose here."));
        assert!(output.contains("Requirements: 1"));
        assert!(output.contains("Requirement: Test Requirement"));
        assert!(output.contains("The system SHALL test."));
        assert!(output.contains("Scenarios:"));
        assert!(output.contains("Scenario One"));
    }

    #[test]
    fn test_format_spec_no_scenarios() {
        let spec = SpecInfo {
            name: "test".to_string(),
            purpose: "Test purpose.".to_string(),
            requirements: vec![RequirementInfo {
                name: "No Scenarios".to_string(),
                text: "The system SHALL work.".to_string(),
                scenarios: vec![],
            }],
        };

        let output = format_spec(&spec);

        // Should not contain Scenarios section
        assert!(!output.contains("Scenarios:"));
        assert!(output.contains("Requirement: No Scenarios"));
    }

    #[test]
    fn test_format_spec_multiple_requirements() {
        let spec = SpecInfo {
            name: "multi".to_string(),
            purpose: "Multiple requirements test.".to_string(),
            requirements: vec![
                RequirementInfo {
                    name: "First".to_string(),
                    text: "First requirement.".to_string(),
                    scenarios: vec!["A".to_string()],
                },
                RequirementInfo {
                    name: "Second".to_string(),
                    text: "Second requirement.".to_string(),
                    scenarios: vec!["B".to_string(), "C".to_string()],
                },
            ],
        };

        let output = format_spec(&spec);

        assert!(output.contains("Requirements: 2"));
        assert!(output.contains("Requirement: First"));
        assert!(output.contains("Requirement: Second"));
        assert!(output.contains("\u{2022} A"));
        assert!(output.contains("\u{2022} B"));
        assert!(output.contains("\u{2022} C"));
    }

    #[test]
    fn test_format_requirement_with_box_drawing() {
        let req = RequirementInfo {
            name: "Test".to_string(),
            text: "Description text.".to_string(),
            scenarios: vec!["Scenario A".to_string()],
        };

        let output = format_requirement(&req);

        // Check box drawing characters
        assert!(output.contains("\u{250C}")); // top-left corner
        assert!(output.contains("\u{2502}")); // vertical line
        assert!(output.contains("\u{2514}")); // bottom-left corner
        assert!(output.contains("\u{2022}")); // bullet
    }

    #[test]
    fn test_extract_purpose_multiline() {
        let content = r#"# Test Specification

## Purpose

This is a multi-line purpose.
It spans several lines.
And has different content.

## Requirements
"#;

        let result = parse_spec_content(content).unwrap();
        // Multi-line purpose should be joined
        assert!(result.purpose.contains("multi-line"));
        assert!(result.purpose.contains("several lines"));
        assert!(result.purpose.contains("different content"));
    }

    #[test]
    fn test_case_insensitive_headers() {
        let content = r#"# Test Specification

## PURPOSE

This is the purpose.

## REQUIREMENTS

### Requirement: Test

The system SHALL work.
"#;

        let result = parse_spec_content(content);
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(spec.purpose.contains("purpose"));
        assert_eq!(spec.requirements.len(), 1);
    }
}
