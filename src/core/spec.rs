//! Spec parsing for Spec Oxide Core.
//!
//! This module provides functions to parse spec files into structured data.
//! It contains no formatting or I/O beyond file reading - only structured data.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::{Error, Result};

/// A parsed specification with all its requirements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedSpec {
    /// The spec ID (folder name).
    pub id: String,
    /// The spec title (from # heading).
    pub title: String,
    /// The spec purpose (from ## Purpose section).
    pub purpose: String,
    /// All requirements in the spec.
    pub requirements: Vec<ParsedRequirement>,
}

/// A parsed requirement with its scenarios.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedRequirement {
    /// The requirement name (from ### Requirement: heading).
    pub name: String,
    /// The requirement description (text after heading, before scenarios).
    pub description: String,
    /// All scenarios for this requirement.
    pub scenarios: Vec<ParsedScenario>,
}

/// A parsed scenario with its clauses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedScenario {
    /// The scenario name (from #### Scenario: heading).
    pub name: String,
    /// The WHEN clause(s).
    pub when_clause: String,
    /// All THEN/AND clauses.
    pub then_clauses: Vec<String>,
}

/// Parse a single spec file and return structured data.
///
/// # Arguments
///
/// * `path` - Path to the spec.md file
///
/// # Returns
///
/// A `ParsedSpec` containing the spec's structured data.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn parse_spec_file(path: &Path) -> Result<ParsedSpec> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::Other(format!("Failed to read spec file: {}", e)))?;

    // Extract spec ID from parent directory name
    let id = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    parse_spec_content(&content, &id)
}

/// Parse all specs in a folder.
///
/// # Arguments
///
/// * `folder` - Path to the specs folder
///
/// # Returns
///
/// A list of all parsed specs in the folder.
///
/// # Errors
///
/// Returns an error if the folder cannot be read.
pub fn parse_all_specs(folder: &Path) -> Result<Vec<ParsedSpec>> {
    let mut specs = Vec::new();

    if !folder.exists() {
        return Err(Error::Other(format!(
            "Specs folder does not exist: {}",
            folder.display()
        )));
    }

    let entries = fs::read_dir(folder)
        .map_err(|e| Error::Other(format!("Failed to read specs folder: {}", e)))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| Error::Other(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();

        // Skip non-directories
        if !path.is_dir() {
            continue;
        }

        // Skip directories starting with underscore (like _changes, _archive)
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if dir_name.starts_with('_') {
            continue;
        }

        // Look for spec.md in the directory
        let spec_file = path.join("spec.md");
        if spec_file.exists() {
            match parse_spec_file(&spec_file) {
                Ok(spec) => specs.push(spec),
                Err(_) => continue, // Skip invalid specs
            }
        }
    }

    Ok(specs)
}

/// Get a spec by its ID.
///
/// # Arguments
///
/// * `folder` - Path to the specs folder
/// * `id` - The spec ID (folder name)
///
/// # Returns
///
/// The parsed spec matching the ID.
///
/// # Errors
///
/// Returns an error if the spec is not found.
pub fn get_spec_by_id(folder: &Path, id: &str) -> Result<ParsedSpec> {
    let spec_dir = folder.join(id);
    let spec_file = spec_dir.join("spec.md");

    if !spec_file.exists() {
        return Err(Error::Other(format!("Spec not found: {}", id)));
    }

    parse_spec_file(&spec_file)
}

// =============================================================================
// Internal parsing functions
// =============================================================================

/// Parse spec content string and return structured data.
fn parse_spec_content(content: &str, id: &str) -> Result<ParsedSpec> {
    let lines: Vec<&str> = content.lines().collect();

    // Extract spec title from header
    let title = extract_spec_title(&lines)?;

    // Extract purpose section
    let purpose = extract_purpose(&lines)?;

    // Extract requirements with scenarios
    let requirements = extract_requirements_with_scenarios(&lines);

    Ok(ParsedSpec {
        id: id.to_string(),
        title,
        purpose,
        requirements,
    })
}

/// Extract the spec title from the first header line.
fn extract_spec_title(lines: &[&str]) -> Result<String> {
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            // Return the full header text (including "Specification" suffix)
            return Ok(trimmed.strip_prefix("# ").unwrap_or("").trim().to_string());
        }
    }
    Err(Error::Other(
        "Missing spec header (expected '# <title>')".to_string(),
    ))
}

/// Extract the purpose section content.
fn extract_purpose(lines: &[&str]) -> Result<String> {
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

        if in_purpose && !trimmed.is_empty() {
            purpose_lines.push(trimmed);
        }
    }

    if purpose_lines.is_empty() {
        return Err(Error::Other("Missing Purpose section".to_string()));
    }

    Ok(purpose_lines.join(" "))
}

/// Extract all requirements with their scenarios from the spec content.
fn extract_requirements_with_scenarios(lines: &[&str]) -> Vec<ParsedRequirement> {
    let mut requirements = Vec::new();
    let mut current_req: Option<ParsedRequirement> = None;
    let mut current_scenario: Option<ParsedScenario> = None;
    let mut in_requirements = false;
    let mut in_scenario = false;
    let mut collecting_then = false;

    for line in lines {
        let trimmed = line.trim();

        // Check for Requirements section
        if trimmed.eq_ignore_ascii_case("## Requirements") {
            in_requirements = true;
            continue;
        }

        // Stop at next level-2 header (outside Requirements)
        if in_requirements
            && trimmed.starts_with("## ")
            && !trimmed.eq_ignore_ascii_case("## Requirements")
        {
            // Save current scenario if any
            if let Some(scenario) = current_scenario.take() {
                if let Some(ref mut req) = current_req {
                    req.scenarios.push(scenario);
                }
            }
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
            // Save current scenario if any
            if let Some(scenario) = current_scenario.take() {
                if let Some(ref mut req) = current_req {
                    req.scenarios.push(scenario);
                }
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

            current_req = Some(ParsedRequirement {
                name,
                description: String::new(),
                scenarios: Vec::new(),
            });
            in_scenario = false;
            collecting_then = false;
            continue;
        }

        // Check for scenario header (#### Scenario: <name>)
        if trimmed.starts_with("#### Scenario:") {
            // Save previous scenario
            if let Some(scenario) = current_scenario.take() {
                if let Some(ref mut req) = current_req {
                    req.scenarios.push(scenario);
                }
            }

            let scenario_name = trimmed
                .strip_prefix("#### Scenario:")
                .unwrap_or("")
                .trim()
                .to_string();

            current_scenario = Some(ParsedScenario {
                name: scenario_name,
                when_clause: String::new(),
                then_clauses: Vec::new(),
            });
            in_scenario = true;
            collecting_then = false;
            continue;
        }

        // Parse scenario clauses
        if in_scenario {
            if let Some(ref mut scenario) = current_scenario {
                // Check for WHEN clause
                if trimmed.contains("**WHEN**") || trimmed.contains("**When**") {
                    let clause = extract_clause_text(trimmed, &["**WHEN**", "**When**"]);
                    scenario.when_clause = clause;
                    collecting_then = false;
                }
                // Check for THEN clause
                else if trimmed.contains("**THEN**") || trimmed.contains("**Then**") {
                    let clause = extract_clause_text(trimmed, &["**THEN**", "**Then**"]);
                    scenario.then_clauses.push(clause);
                    collecting_then = true;
                }
                // Check for AND clause (continuation of THEN)
                else if trimmed.contains("**AND**") || trimmed.contains("**And**") {
                    let clause = extract_clause_text(trimmed, &["**AND**", "**And**"]);
                    if collecting_then {
                        scenario.then_clauses.push(clause);
                    }
                }
            }
        } else if current_req.is_some() && !trimmed.is_empty() {
            // Accumulate requirement description (before scenarios)
            if let Some(ref mut req) = current_req {
                if !req.description.is_empty() {
                    req.description.push(' ');
                }
                req.description.push_str(trimmed);
            }
        }
    }

    // Save final scenario if any
    if let Some(scenario) = current_scenario.take() {
        if let Some(ref mut req) = current_req {
            req.scenarios.push(scenario);
        }
    }

    // Save final requirement
    if let Some(req) = current_req.take() {
        requirements.push(req);
    }

    requirements
}

/// Extract the text content from a clause line, removing the keyword marker.
fn extract_clause_text(line: &str, keywords: &[&str]) -> String {
    let mut text = line.to_string();

    // Remove bullet point prefix if present
    if text.starts_with("- ") {
        text = text[2..].to_string();
    }

    // Remove keyword markers
    for keyword in keywords {
        text = text.replace(keyword, "");
    }

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_spec(dir: &Path, name: &str, content: &str) {
        let spec_dir = dir.join(name);
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("spec.md"), content).unwrap();
    }

    const VALID_SPEC: &str = r#"# Auth Specification

## Purpose

This spec defines authentication requirements for the system.

## Requirements

### Requirement: User Login

The system SHALL allow users to login with email and password.

#### Scenario: Successful login

- **WHEN** user provides valid email and password
- **THEN** user is authenticated
- **AND** session token is returned

#### Scenario: Failed login

- **WHEN** user provides invalid credentials
- **THEN** authentication error is returned

### Requirement: User Logout

The system SHALL allow users to logout.

#### Scenario: Logout clears session

- **WHEN** authenticated user requests logout
- **THEN** session is invalidated
"#;

    const SIMPLE_SPEC: &str = r#"# Simple Specification

## Purpose

A simple spec for testing.

## Requirements

### Requirement: Basic Feature

The system SHALL do something basic.

#### Scenario: Basic behavior

- **WHEN** user does something
- **THEN** something happens
"#;

    // ==================== parse_spec_file tests ====================

    #[test]
    fn test_parse_spec_file_valid() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);

        let spec_path = temp_dir.path().join("auth/spec.md");
        let result = parse_spec_file(&spec_path);

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let spec = result.unwrap();

        assert_eq!(spec.id, "auth");
        assert_eq!(spec.title, "Auth Specification");
        assert!(spec.purpose.contains("authentication requirements"));
    }

    #[test]
    fn test_parse_spec_file_extracts_requirements() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);

        let spec_path = temp_dir.path().join("auth/spec.md");
        let spec = parse_spec_file(&spec_path).unwrap();

        assert_eq!(spec.requirements.len(), 2);
        assert_eq!(spec.requirements[0].name, "User Login");
        assert_eq!(spec.requirements[1].name, "User Logout");
    }

    #[test]
    fn test_parse_spec_file_extracts_scenarios() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);

        let spec_path = temp_dir.path().join("auth/spec.md");
        let spec = parse_spec_file(&spec_path).unwrap();

        let login_req = &spec.requirements[0];
        assert_eq!(login_req.scenarios.len(), 2);
        assert_eq!(login_req.scenarios[0].name, "Successful login");
        assert_eq!(login_req.scenarios[1].name, "Failed login");
    }

    #[test]
    fn test_parse_spec_file_extracts_when_then_clauses() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);

        let spec_path = temp_dir.path().join("auth/spec.md");
        let spec = parse_spec_file(&spec_path).unwrap();

        let scenario = &spec.requirements[0].scenarios[0];
        assert!(scenario.when_clause.contains("valid email and password"));
        assert_eq!(scenario.then_clauses.len(), 2);
        assert!(scenario.then_clauses[0].contains("authenticated"));
        assert!(scenario.then_clauses[1].contains("session token"));
    }

    #[test]
    fn test_parse_spec_file_not_found() {
        let result = parse_spec_file(Path::new("/nonexistent/spec.md"));
        assert!(result.is_err());
    }

    // ==================== parse_all_specs tests ====================

    #[test]
    fn test_parse_all_specs_multiple() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);
        create_test_spec(temp_dir.path(), "simple", SIMPLE_SPEC);

        let result = parse_all_specs(temp_dir.path());

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let specs = result.unwrap();

        assert_eq!(specs.len(), 2);

        // Sort by id for deterministic testing
        let mut specs = specs;
        specs.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(specs[0].id, "auth");
        assert_eq!(specs[1].id, "simple");
    }

    #[test]
    fn test_parse_all_specs_empty_folder() {
        let temp_dir = TempDir::new().unwrap();

        let result = parse_all_specs(temp_dir.path());

        assert!(result.is_ok());
        let specs = result.unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn test_parse_all_specs_skips_non_spec_dirs() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);

        // Create a directory without spec.md
        fs::create_dir_all(temp_dir.path().join("not-a-spec")).unwrap();
        fs::write(temp_dir.path().join("not-a-spec/readme.md"), "# Not a spec").unwrap();

        // Create underscore directory (should be skipped)
        create_test_spec(temp_dir.path(), "_changes", SIMPLE_SPEC);

        let result = parse_all_specs(temp_dir.path());
        let specs = result.unwrap();

        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "auth");
    }

    // ==================== get_spec_by_id tests ====================

    #[test]
    fn test_get_spec_by_id_valid() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);
        create_test_spec(temp_dir.path(), "simple", SIMPLE_SPEC);

        let result = get_spec_by_id(temp_dir.path(), "auth");

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let spec = result.unwrap();
        assert_eq!(spec.id, "auth");
        assert_eq!(spec.title, "Auth Specification");
    }

    #[test]
    fn test_get_spec_by_id_not_found() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);

        let result = get_spec_by_id(temp_dir.path(), "nonexistent");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // ==================== Requirement description tests ====================

    #[test]
    fn test_parse_requirement_description() {
        let temp_dir = TempDir::new().unwrap();
        create_test_spec(temp_dir.path(), "auth", VALID_SPEC);

        let spec_path = temp_dir.path().join("auth/spec.md");
        let spec = parse_spec_file(&spec_path).unwrap();

        let login_req = &spec.requirements[0];
        assert!(login_req
            .description
            .contains("allow users to login with email and password"));
    }
}
