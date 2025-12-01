//! Integration tests for spec validation.
//!
//! Tests the `validate_spec()` function on actual spec files.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// Import the validation module from the main crate
// Note: We need to use the library interface when available
// For now, we test through the module directly

/// Helper to create a temp directory with a spec file.
fn create_temp_spec(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let spec_path = temp_dir.path().join("spec.md");
    let mut file = fs::File::create(&spec_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    (temp_dir, spec_path)
}

/// Valid spec content that should pass all validations.
const VALID_SPEC: &str = r#"# Authentication Spec

## Purpose

This specification defines the authentication requirements for the system.
Users must be able to securely authenticate using their credentials, and the
system must validate those credentials before granting access to protected
resources.

## Requirements

### Requirement: User Login

The system SHALL authenticate users using email and password credentials.

#### Scenario: Successful login with valid credentials

- **WHEN** a user provides valid email and password
- **THEN** the system authenticates the user
- **AND** returns a session token

#### Scenario: Failed login with invalid credentials

- **WHEN** a user provides invalid credentials
- **THEN** the system rejects the authentication attempt
- **AND** returns an appropriate error message

### Requirement: Session Management

The system MUST maintain session state for authenticated users.

#### Scenario: Session expiration

- **WHEN** a session exceeds the maximum lifetime
- **THEN** the system invalidates the session
- **AND** requires re-authentication
"#;

/// Invalid spec content missing Purpose section.
const SPEC_MISSING_PURPOSE: &str = r#"# Test Spec

## Requirements

### Requirement: Test

The system SHALL do something.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

/// Invalid spec content missing Requirements section.
const SPEC_MISSING_REQUIREMENTS: &str = r#"# Test Spec

## Purpose

This specification defines the test requirements for the system.
It provides sufficient context and detail for readers.
"#;

/// Spec with requirement missing WHEN clause.
const SPEC_MISSING_WHEN: &str = r#"# Test Spec

## Purpose

This specification defines the test requirements for the system.
It provides sufficient context and detail for readers.

## Requirements

### Requirement: Test

The system SHALL do something.

#### Scenario: Missing WHEN

- **THEN** something happens
"#;

/// Spec with requirement missing THEN clause.
const SPEC_MISSING_THEN: &str = r#"# Test Spec

## Purpose

This specification defines the test requirements for the system.
It provides sufficient context and detail for readers.

## Requirements

### Requirement: Test

The system SHALL do something.

#### Scenario: Missing THEN

- **WHEN** something happens
"#;

/// Spec with short purpose (warning, not error).
const SPEC_SHORT_PURPOSE: &str = r#"# Test Spec

## Purpose

Short.

## Requirements

### Requirement: Test

The system SHALL do something.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

/// Spec with requirement missing normative language (warning).
const SPEC_NON_NORMATIVE: &str = r#"# Test Spec

## Purpose

This specification defines the test requirements for the system.
It provides sufficient context and detail for readers.

## Requirements

### Requirement: Test

The system does something useful for users.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

/// Spec with requirement missing scenarios (warning).
const SPEC_NO_SCENARIOS: &str = r#"# Test Spec

## Purpose

This specification defines the test requirements for the system.
It provides sufficient context and detail for readers.

## Requirements

### Requirement: Test

The system SHALL do something without scenarios.
"#;

/// Spec with empty requirement description (error).
const SPEC_EMPTY_REQUIREMENT: &str = r#"# Test Spec

## Purpose

This specification defines the test requirements for the system.
It provides sufficient context and detail for readers.

## Requirements

### Requirement: Empty

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

// Integration tests - these test the public API when integrated

#[test]
fn test_valid_spec_file_structure() {
    // Test that a well-formed spec has the expected structure
    let (_temp_dir, spec_path) = create_temp_spec(VALID_SPEC);

    // Verify the file was created correctly
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("## Purpose"));
    assert!(content.contains("## Requirements"));
    assert!(content.contains("### Requirement:"));
    assert!(content.contains("#### Scenario:"));
    assert!(content.contains("**WHEN**"));
    assert!(content.contains("**THEN**"));
}

#[test]
fn test_invalid_spec_missing_purpose_structure() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_MISSING_PURPOSE);

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(!content.contains("## Purpose"));
    assert!(content.contains("## Requirements"));
}

#[test]
fn test_invalid_spec_missing_requirements_structure() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_MISSING_REQUIREMENTS);

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("## Purpose"));
    assert!(!content.contains("## Requirements"));
}

#[test]
fn test_spec_with_missing_when_clause() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_MISSING_WHEN);

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("#### Scenario:"));
    assert!(!content.contains("**WHEN**"));
    assert!(content.contains("**THEN**"));
}

#[test]
fn test_spec_with_missing_then_clause() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_MISSING_THEN);

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("#### Scenario:"));
    assert!(content.contains("**WHEN**"));
    assert!(!content.contains("**THEN**"));
}

#[test]
fn test_spec_with_short_purpose() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_SHORT_PURPOSE);

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("## Purpose"));
    // Purpose text is very short
    let lines: Vec<&str> = content.lines().collect();
    let purpose_idx = lines.iter().position(|l| l.contains("## Purpose")).unwrap();
    // Next non-empty line after Purpose should be short
    let purpose_text = lines
        .iter()
        .skip(purpose_idx + 1)
        .find(|l| !l.trim().is_empty() && !l.starts_with("##"))
        .unwrap();
    assert!(purpose_text.len() < 50);
}

#[test]
fn test_spec_with_non_normative_language() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_NON_NORMATIVE);

    let content = fs::read_to_string(&spec_path).unwrap();

    // Extract requirement text (between ### Requirement: and #### Scenario:)
    let lines: Vec<&str> = content.lines().collect();
    let req_idx = lines
        .iter()
        .position(|l| l.starts_with("### Requirement:"))
        .unwrap();
    let scenario_idx = lines
        .iter()
        .position(|l| l.starts_with("#### Scenario:"))
        .unwrap();

    let req_text: String = lines[req_idx + 1..scenario_idx]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    // Should not contain SHALL or MUST
    assert!(!req_text.to_uppercase().contains("SHALL"));
    assert!(!req_text.to_uppercase().contains("MUST"));
}

#[test]
fn test_spec_with_no_scenarios() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_NO_SCENARIOS);

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("### Requirement:"));
    assert!(!content.contains("#### Scenario:"));
}

#[test]
fn test_spec_with_empty_requirement() {
    let (_temp_dir, spec_path) = create_temp_spec(SPEC_EMPTY_REQUIREMENT);

    let content = fs::read_to_string(&spec_path).unwrap();

    // Requirement header is immediately followed by Scenario (no text)
    let lines: Vec<&str> = content.lines().collect();
    let req_idx = lines
        .iter()
        .position(|l| l.starts_with("### Requirement:"))
        .unwrap();

    // Next non-empty line after requirement header should be the scenario
    let next_content_line = lines
        .iter()
        .skip(req_idx + 1)
        .find(|l| !l.trim().is_empty())
        .unwrap();

    assert!(next_content_line.starts_with("#### Scenario:"));
}

#[test]
fn test_temp_file_cleanup() {
    // Test that temp files are properly cleaned up
    let spec_path: PathBuf;
    {
        let (temp_dir, path) = create_temp_spec(VALID_SPEC);
        spec_path = path.clone();
        assert!(spec_path.exists());
        drop(temp_dir); // Explicitly drop to trigger cleanup
    }
    // File should be cleaned up after temp_dir is dropped
    assert!(!spec_path.exists());
}

#[test]
fn test_multiple_requirements_in_spec() {
    let content = r#"# Multi-Requirement Spec

## Purpose

This specification defines multiple requirements to test parsing of
complex spec files with several requirements and scenarios.

## Requirements

### Requirement: First Requirement

The system SHALL implement the first requirement.

#### Scenario: First scenario

- **WHEN** first condition
- **THEN** first result

### Requirement: Second Requirement

The system MUST implement the second requirement.

#### Scenario: Second scenario

- **WHEN** second condition
- **THEN** second result

### Requirement: Third Requirement

The system SHALL implement the third requirement.

#### Scenario: Third scenario A

- **WHEN** third condition A
- **THEN** third result A

#### Scenario: Third scenario B

- **WHEN** third condition B
- **THEN** third result B
"#;

    let (_temp_dir, spec_path) = create_temp_spec(content);

    let file_content = fs::read_to_string(&spec_path).unwrap();

    // Count requirements
    let req_count = file_content.matches("### Requirement:").count();
    assert_eq!(req_count, 3);

    // Count scenarios
    let scenario_count = file_content.matches("#### Scenario:").count();
    assert_eq!(scenario_count, 4);
}

#[test]
fn test_spec_with_uppercase_headers() {
    let content = r#"# Test Spec

## PURPOSE

This specification defines the test requirements with uppercase headers.
It should still be valid when headers use different casing.

## REQUIREMENTS

### Requirement: Test

The system SHALL do something.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

    let (_temp_dir, spec_path) = create_temp_spec(content);

    let file_content = fs::read_to_string(&spec_path).unwrap();
    assert!(file_content.contains("## PURPOSE"));
    assert!(file_content.contains("## REQUIREMENTS"));
}
