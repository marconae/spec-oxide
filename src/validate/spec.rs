//! Spec file validation module.
//!
//! Validates spec files for structural correctness and business rules.

use std::fs;
use std::path::Path;

use super::ValidationReport;

/// Minimum character length for Purpose section to avoid warnings.
const MIN_PURPOSE_LENGTH: usize = 50;

/// Validate a spec file at the given path.
///
/// # Arguments
/// * `path` - Path to the spec.md file to validate
///
/// # Returns
/// A `ValidationReport` containing all validation issues found.
pub fn validate_spec(path: &Path) -> ValidationReport {
    let mut report = ValidationReport::new();
    let file_path = path.to_string_lossy().to_string();

    // Read file content
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            report.add_error(
                &file_path,
                Some(1),
                &format!("Failed to read file: {}", e),
            );
            return report;
        }
    };

    validate_spec_content(&content, &file_path, &mut report);
    report
}

/// Validate spec content (used for both file-based and content-based validation).
fn validate_spec_content(content: &str, file_path: &str, report: &mut ValidationReport) {
    let lines: Vec<&str> = content.lines().collect();

    // Check for Purpose section
    let purpose_result = find_section(&lines, "## Purpose");
    if purpose_result.is_none() {
        report.add_error(file_path, Some(1), "Missing Purpose section");
    } else {
        // Check Purpose length
        let (purpose_line, _) = purpose_result.unwrap();
        let purpose_text = extract_section_text(&lines, purpose_line);
        if purpose_text.len() < MIN_PURPOSE_LENGTH {
            report.add_warning(
                file_path,
                Some(purpose_line + 1),
                &format!(
                    "Purpose section is too short ({} chars, minimum {} recommended)",
                    purpose_text.len(),
                    MIN_PURPOSE_LENGTH
                ),
            );
        }
    }

    // Check for Requirements section
    let requirements_result = find_section(&lines, "## Requirements");
    if requirements_result.is_none() {
        report.add_error(file_path, Some(1), "Missing Requirements section");
        return; // Cannot validate requirements if section is missing
    }

    let (requirements_line, _) = requirements_result.unwrap();

    // Parse and validate each requirement block
    let requirements = parse_requirements(&lines, requirements_line);

    if requirements.is_empty() {
        report.add_warning(
            file_path,
            Some(requirements_line + 1),
            "Requirements section has no requirement blocks",
        );
    }

    for req in requirements {
        validate_requirement(&req, file_path, report);
    }
}

/// Find a section header in the lines and return its line index and content.
fn find_section(lines: &[&str], header: &str) -> Option<(usize, String)> {
    for (i, line) in lines.iter().enumerate() {
        if line.trim().eq_ignore_ascii_case(header) {
            return Some((i, header.to_string()));
        }
    }
    None
}

/// Extract the text content of a section (from header to next same-level header).
fn extract_section_text(lines: &[&str], start_line: usize) -> String {
    let mut content = String::new();
    let header_prefix = "## ";

    for line in lines.iter().skip(start_line + 1) {
        // Stop at next level-2 header
        if line.starts_with(header_prefix) {
            break;
        }
        content.push_str(line);
        content.push('\n');
    }

    content.trim().to_string()
}

/// Represents a parsed requirement block.
#[derive(Debug)]
struct Requirement {
    /// Line number where the requirement starts (0-indexed).
    line: usize,
    /// The name/title of the requirement.
    name: String,
    /// The descriptive text of the requirement.
    text: String,
    /// List of scenarios in this requirement.
    scenarios: Vec<Scenario>,
}

/// Represents a parsed scenario block.
#[derive(Debug)]
struct Scenario {
    /// Line number where the scenario starts (0-indexed).
    line: usize,
    /// The name/title of the scenario.
    name: String,
    /// Whether the scenario has a WHEN clause.
    has_when: bool,
    /// Whether the scenario has a THEN clause.
    has_then: bool,
}

/// Parse all requirement blocks from the lines starting after requirements_line.
fn parse_requirements(lines: &[&str], requirements_line: usize) -> Vec<Requirement> {
    let mut requirements = Vec::new();
    let mut current_req: Option<Requirement> = None;
    let mut current_scenario: Option<Scenario> = None;
    let mut in_requirements_section = false;

    for (i, line) in lines.iter().enumerate().skip(requirements_line) {
        let trimmed = line.trim();

        // Check if we've entered a new level-2 section (end of Requirements)
        if i > requirements_line && trimmed.starts_with("## ") {
            // Save any pending scenario
            if let (Some(ref mut req), Some(scenario)) =
                (&mut current_req, current_scenario.take())
            {
                req.scenarios.push(scenario);
            }
            // Save any pending requirement
            if let Some(req) = current_req.take() {
                requirements.push(req);
            }
            break;
        }

        // Mark that we're in the requirements section
        if trimmed.eq_ignore_ascii_case("## Requirements") {
            in_requirements_section = true;
            continue;
        }

        if !in_requirements_section {
            continue;
        }

        // Check for requirement header (### Requirement: Name)
        if trimmed.starts_with("### Requirement:") {
            // Save any pending scenario to previous requirement
            if let (Some(ref mut req), Some(scenario)) =
                (&mut current_req, current_scenario.take())
            {
                req.scenarios.push(scenario);
            }
            // Save previous requirement
            if let Some(req) = current_req.take() {
                requirements.push(req);
            }

            let name = trimmed
                .strip_prefix("### Requirement:")
                .unwrap_or("")
                .trim()
                .to_string();

            current_req = Some(Requirement {
                line: i,
                name,
                text: String::new(),
                scenarios: Vec::new(),
            });
            continue;
        }

        // Check for scenario header (#### Scenario: Name)
        if trimmed.starts_with("#### Scenario:") {
            // Save any pending scenario
            if let (Some(ref mut req), Some(scenario)) =
                (&mut current_req, current_scenario.take())
            {
                req.scenarios.push(scenario);
            }

            let name = trimmed
                .strip_prefix("#### Scenario:")
                .unwrap_or("")
                .trim()
                .to_string();

            current_scenario = Some(Scenario {
                line: i,
                name,
                has_when: false,
                has_then: false,
            });
            continue;
        }

        // Check for WHEN/THEN in scenario content
        if let Some(ref mut scenario) = current_scenario {
            let upper = trimmed.to_uppercase();
            if upper.contains("**WHEN**")
                || upper.starts_with("WHEN ")
                || upper.starts_with("- WHEN")
                || upper.contains("- **WHEN**")
            {
                scenario.has_when = true;
            }
            if upper.contains("**THEN**")
                || upper.starts_with("THEN ")
                || upper.starts_with("- THEN")
                || upper.contains("- **THEN**")
            {
                scenario.has_then = true;
            }
        }

        // Accumulate requirement text (before scenarios)
        if current_req.is_some() && current_scenario.is_none() {
            if let Some(ref mut req) = current_req {
                // Skip empty lines at the start
                if !req.text.is_empty() || !trimmed.is_empty() {
                    if !req.text.is_empty() {
                        req.text.push(' ');
                    }
                    req.text.push_str(trimmed);
                }
            }
        }
    }

    // Save any pending scenario and requirement at end of file
    if let (Some(ref mut req), Some(scenario)) = (&mut current_req, current_scenario.take()) {
        req.scenarios.push(scenario);
    }
    if let Some(req) = current_req.take() {
        requirements.push(req);
    }

    requirements
}

/// Validate a single requirement block.
fn validate_requirement(req: &Requirement, file_path: &str, report: &mut ValidationReport) {
    // Check requirement has text
    if req.text.trim().is_empty() {
        report.add_error(
            file_path,
            Some(req.line + 1),
            &format!("Requirement \"{}\" has no description text", req.name),
        );
    } else {
        // Check for normative language (SHALL/MUST)
        let upper_text = req.text.to_uppercase();
        if !upper_text.contains("SHALL") && !upper_text.contains("MUST") {
            report.add_warning(
                file_path,
                Some(req.line + 1),
                &format!(
                    "Requirement \"{}\" does not use normative language (SHALL/MUST)",
                    req.name
                ),
            );
        }
    }

    // Check requirement has at least one scenario
    if req.scenarios.is_empty() {
        report.add_warning(
            file_path,
            Some(req.line + 1),
            &format!("Requirement \"{}\" has no scenarios", req.name),
        );
    }

    // Validate each scenario
    for scenario in &req.scenarios {
        validate_scenario(scenario, &req.name, file_path, report);
    }
}

/// Validate a single scenario block.
fn validate_scenario(
    scenario: &Scenario,
    req_name: &str,
    file_path: &str,
    report: &mut ValidationReport,
) {
    if !scenario.has_when {
        report.add_error(
            file_path,
            Some(scenario.line + 1),
            &format!(
                "Scenario \"{}\" in requirement \"{}\" is missing WHEN clause",
                scenario.name, req_name
            ),
        );
    }

    if !scenario.has_then {
        report.add_error(
            file_path,
            Some(scenario.line + 1),
            &format!(
                "Scenario \"{}\" in requirement \"{}\" is missing THEN clause",
                scenario.name, req_name
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::Severity;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a temp file with content and validate it.
    fn validate_content(content: &str) -> ValidationReport {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        validate_spec(file.path())
    }

    #[test]
    fn test_valid_spec_passes() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context for readers to understand the
goals and scope of this document.

## Requirements

### Requirement: User Authentication

The system SHALL authenticate users with valid credentials before granting access.

#### Scenario: Successful login

- **WHEN** a user provides valid credentials
- **THEN** the system grants access
"#;

        let report = validate_content(content);
        assert!(
            report.is_valid(),
            "Expected valid spec, got errors: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_missing_purpose_section() {
        let content = r#"# My Spec

## Requirements

### Requirement: Test

The system SHALL do something.

#### Scenario: Test case

- **WHEN** something happens
- **THEN** something else happens
"#;

        let report = validate_content(content);
        assert!(!report.is_valid());

        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.iter().any(|i| i.message.contains("Missing Purpose")));
    }

    #[test]
    fn test_missing_requirements_section() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.
"#;

        let report = validate_content(content);
        assert!(!report.is_valid());

        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors
            .iter()
            .any(|i| i.message.contains("Missing Requirements")));
    }

    #[test]
    fn test_short_purpose_warning() {
        let content = r#"# My Spec

## Purpose

Short purpose.

## Requirements

### Requirement: Test

The system SHALL test something important.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

        let report = validate_content(content);
        // Should be valid (warnings don't fail validation)
        assert!(report.is_valid());

        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .collect();
        assert!(warnings
            .iter()
            .any(|i| i.message.contains("Purpose section is too short")));
    }

    #[test]
    fn test_requirement_without_scenarios() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.

## Requirements

### Requirement: Test Requirement

The system SHALL do something without any scenarios defined.
"#;

        let report = validate_content(content);
        // Should be valid (warnings don't fail validation)
        assert!(report.is_valid());

        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .collect();
        assert!(warnings
            .iter()
            .any(|i| i.message.contains("has no scenarios")));
    }

    #[test]
    fn test_scenario_without_when() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.

## Requirements

### Requirement: Test

The system SHALL do something.

#### Scenario: Missing when

- **THEN** something happens
"#;

        let report = validate_content(content);
        assert!(!report.is_valid());

        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.iter().any(|i| i.message.contains("missing WHEN")));
    }

    #[test]
    fn test_scenario_without_then() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.

## Requirements

### Requirement: Test

The system SHALL do something.

#### Scenario: Missing then

- **WHEN** something happens
"#;

        let report = validate_content(content);
        assert!(!report.is_valid());

        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.iter().any(|i| i.message.contains("missing THEN")));
    }

    #[test]
    fn test_requirement_without_description() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.

## Requirements

### Requirement: Empty Requirement

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

        let report = validate_content(content);
        assert!(!report.is_valid());

        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors
            .iter()
            .any(|i| i.message.contains("has no description")));
    }

    #[test]
    fn test_requirement_without_normative_language() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.

## Requirements

### Requirement: Non-normative

The system provides a feature that does something useful for users.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

        let report = validate_content(content);
        // Should be valid (warnings don't fail validation)
        assert!(report.is_valid());

        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .collect();
        assert!(
            warnings
                .iter()
                .any(|i| i.message.contains("normative language")),
            "Expected warning about normative language, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_multiple_requirements_and_scenarios() {
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail for understanding.

## Requirements

### Requirement: First Requirement

The system SHALL do the first thing.

#### Scenario: First scenario

- **WHEN** action one
- **THEN** result one

#### Scenario: Second scenario

- **WHEN** action two
- **THEN** result two

### Requirement: Second Requirement

The system MUST do the second thing.

#### Scenario: Third scenario

- **WHEN** action three
- **THEN** result three
"#;

        let report = validate_content(content);
        assert!(
            report.is_valid(),
            "Expected valid spec, got errors: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_file_not_found() {
        let path = Path::new("/nonexistent/path/to/spec.md");
        let report = validate_spec(path);
        assert!(!report.is_valid());

        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.iter().any(|i| i.message.contains("Failed to read")));
    }

    #[test]
    fn test_case_insensitive_headers() {
        let content = r#"# My Spec

## PURPOSE

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.

## REQUIREMENTS

### Requirement: Test

The system SHALL test something.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

        let report = validate_content(content);
        assert!(
            report.is_valid(),
            "Expected valid spec with uppercase headers, got errors: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_when_then_variations() {
        // Test various WHEN/THEN formats that should be recognized
        let content = r#"# My Spec

## Purpose

This is a comprehensive purpose section that explains what this specification
is all about. It provides sufficient context and detail.

## Requirements

### Requirement: Test

The system SHALL test various formats.

#### Scenario: Bold format

- **WHEN** bold when clause
- **THEN** bold then clause
"#;

        let report = validate_content(content);
        assert!(
            report.is_valid(),
            "Expected valid spec, got errors: {:?}",
            report.issues
        );
    }
}
