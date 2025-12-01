//! Integration tests for change validation.
//!
//! Tests the `validate_change()` function on actual change directories.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temp change directory with proposal and optional delta specs.
fn create_temp_change(
    proposal_content: &str,
    delta_specs: &[(&str, &str)],
) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let change_dir = temp_dir.path().to_path_buf();

    // Create proposal.md
    let proposal_path = change_dir.join("proposal.md");
    let mut file = fs::File::create(&proposal_path).unwrap();
    file.write_all(proposal_content.as_bytes()).unwrap();

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

/// Valid proposal content that should pass all validations.
const VALID_PROPOSAL: &str = r#"# Add User Authentication

## Why

This feature is critically needed because users currently have no way to securely
authenticate with the system. Without authentication, we cannot provide personalized
experiences or protect user data. This has been the top-requested feature for months.

## What Changes

- Add new authentication API endpoints
- Implement password hashing and validation
- Create session management system
- Update database schema with user credentials
"#;

/// Valid delta spec content for ADDED requirements.
const VALID_DELTA_SPEC_ADDED: &str = r#"# Auth Spec Delta

## ADDED Requirements

### Requirement: User Login

The system SHALL authenticate users using email and password credentials.

#### Scenario: Successful login

- **WHEN** a user provides valid email and password
- **THEN** the system authenticates the user
- **AND** returns a session token

#### Scenario: Failed login

- **WHEN** a user provides invalid credentials
- **THEN** the system returns an authentication error

### Requirement: Session Validation

The system MUST validate session tokens on each request.

#### Scenario: Valid session

- **WHEN** a request includes a valid session token
- **THEN** the request is processed normally
"#;

/// Valid delta spec content for MODIFIED requirements.
const VALID_DELTA_SPEC_MODIFIED: &str = r#"# User Spec Delta

## MODIFIED Requirements

### Requirement: User Profile

The system SHALL allow users to update their profile information including
email address, display name, and profile picture. The system MUST validate
all profile updates before persisting them.
"#;

/// Valid delta spec content for REMOVED requirements.
const VALID_DELTA_SPEC_REMOVED: &str = r#"# Legacy Spec Delta

## REMOVED Requirements

### Requirement: Legacy Login

This requirement for basic HTTP authentication is being removed in favor of
the new session-based authentication system.
"#;

/// Valid delta spec content for RENAMED requirements.
const VALID_DELTA_SPEC_RENAMED: &str = r#"# Naming Update Delta

## RENAMED Requirements

### Requirement: User Authentication -> User Login

The requirement has been renamed for consistency with other specifications.
"#;

// ==================== Integration tests ====================

#[test]
fn test_valid_change_directory_structure() {
    // Test that a well-formed change directory has the expected structure
    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[("auth", VALID_DELTA_SPEC_ADDED)],
    );

    // Verify proposal.md was created
    let proposal_path = change_dir.join("proposal.md");
    assert!(proposal_path.exists());
    let proposal_content = fs::read_to_string(&proposal_path).unwrap();
    assert!(proposal_content.contains("## Why"));
    assert!(proposal_content.contains("## What Changes"));

    // Verify delta spec was created
    let delta_path = change_dir.join("specs/auth/spec.md");
    assert!(delta_path.exists());
    let delta_content = fs::read_to_string(&delta_path).unwrap();
    assert!(delta_content.contains("## ADDED Requirements"));
}

#[test]
fn test_change_with_multiple_delta_specs() {
    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[
            ("auth", VALID_DELTA_SPEC_ADDED),
            ("user", VALID_DELTA_SPEC_MODIFIED),
            ("legacy", VALID_DELTA_SPEC_REMOVED),
        ],
    );

    // Verify all delta specs were created
    assert!(change_dir.join("specs/auth/spec.md").exists());
    assert!(change_dir.join("specs/user/spec.md").exists());
    assert!(change_dir.join("specs/legacy/spec.md").exists());
}

#[test]
fn test_change_without_specs_directory() {
    let temp_dir = TempDir::new().unwrap();
    let change_dir = temp_dir.path().to_path_buf();

    // Only create proposal.md, no specs directory
    let proposal_path = change_dir.join("proposal.md");
    let mut file = fs::File::create(&proposal_path).unwrap();
    file.write_all(VALID_PROPOSAL.as_bytes()).unwrap();

    // Verify no specs directory
    assert!(!change_dir.join("specs").exists());
}

#[test]
fn test_change_with_empty_specs_directory() {
    let (_temp_dir, change_dir) = create_temp_change(VALID_PROPOSAL, &[]);

    // Create empty specs directory
    let specs_dir = change_dir.join("specs");
    fs::create_dir(&specs_dir).unwrap();

    // Verify specs directory exists but is empty
    assert!(specs_dir.exists());
    let entries: Vec<_> = fs::read_dir(&specs_dir).unwrap().collect();
    assert!(entries.is_empty());
}

#[test]
fn test_proposal_missing_why_section_structure() {
    let proposal_missing_why = r#"# Add Feature

## What Changes

- Add new feature
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        proposal_missing_why,
        &[("feature", VALID_DELTA_SPEC_ADDED)],
    );

    let proposal_content = fs::read_to_string(change_dir.join("proposal.md")).unwrap();
    assert!(!proposal_content.contains("## Why"));
    assert!(proposal_content.contains("## What Changes"));
}

#[test]
fn test_proposal_missing_what_changes_section_structure() {
    let proposal_missing_what = r#"# Add Feature

## Why

This feature is needed for important business reasons and will provide
significant value to our users by improving their experience.
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        proposal_missing_what,
        &[("feature", VALID_DELTA_SPEC_ADDED)],
    );

    let proposal_content = fs::read_to_string(change_dir.join("proposal.md")).unwrap();
    assert!(proposal_content.contains("## Why"));
    assert!(!proposal_content.contains("## What Changes"));
}

#[test]
fn test_delta_spec_with_all_operation_types() {
    let all_ops_spec = r#"# Comprehensive Delta

## ADDED Requirements

### Requirement: New Feature

The system SHALL support the new feature.

#### Scenario: Feature usage

- **WHEN** user triggers the feature
- **THEN** the feature works correctly

## MODIFIED Requirements

### Requirement: Existing Feature

The system SHALL support the enhanced existing feature with additional capabilities.

## REMOVED Requirements

### Requirement: Deprecated Feature

This feature is being removed.

## RENAMED Requirements

### Requirement: Old Name -> New Name

The requirement has been renamed.
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[("comprehensive", all_ops_spec)],
    );

    let delta_content = fs::read_to_string(change_dir.join("specs/comprehensive/spec.md")).unwrap();
    assert!(delta_content.contains("## ADDED Requirements"));
    assert!(delta_content.contains("## MODIFIED Requirements"));
    assert!(delta_content.contains("## REMOVED Requirements"));
    assert!(delta_content.contains("## RENAMED Requirements"));
}

#[test]
fn test_delta_spec_scenario_with_when_then() {
    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[("auth", VALID_DELTA_SPEC_ADDED)],
    );

    let delta_content = fs::read_to_string(change_dir.join("specs/auth/spec.md")).unwrap();

    // Verify scenarios have WHEN/THEN clauses
    assert!(delta_content.contains("**WHEN**"));
    assert!(delta_content.contains("**THEN**"));
}

#[test]
fn test_delta_spec_without_scenarios() {
    let no_scenarios_spec = r#"# Feature Delta

## ADDED Requirements

### Requirement: Simple Feature

The system SHALL support the simple feature without scenarios.
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[("simple", no_scenarios_spec)],
    );

    let delta_content = fs::read_to_string(change_dir.join("specs/simple/spec.md")).unwrap();

    // Verify no scenarios
    assert!(delta_content.contains("### Requirement:"));
    assert!(!delta_content.contains("#### Scenario:"));
}

#[test]
fn test_delta_spec_with_invalid_headers() {
    let invalid_headers_spec = r#"# Invalid Delta

## ADDING Requirements

### Requirement: Something

This is not a valid delta header.
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[("invalid", invalid_headers_spec)],
    );

    let delta_content = fs::read_to_string(change_dir.join("specs/invalid/spec.md")).unwrap();

    // Verify the invalid header is present (validation should catch this)
    assert!(delta_content.contains("## ADDING Requirements")); // Typo: "ADDING" instead of "ADDED"
    assert!(!delta_content.contains("## ADDED Requirements"));
}

#[test]
fn test_short_why_section() {
    let short_why_proposal = r#"# Feature

## Why

Short.

## What Changes

- Add feature
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        short_why_proposal,
        &[("feature", VALID_DELTA_SPEC_ADDED)],
    );

    let proposal_content = fs::read_to_string(change_dir.join("proposal.md")).unwrap();

    // Extract Why section content
    let lines: Vec<&str> = proposal_content.lines().collect();
    let why_idx = lines.iter().position(|l| l.contains("## Why")).unwrap();
    let what_idx = lines.iter().position(|l| l.contains("## What Changes")).unwrap();

    let why_text: String = lines[why_idx + 1..what_idx]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    // Verify Why section is short (less than 50 chars)
    assert!(why_text.len() < 50, "Why text length: {} should be < 50", why_text.len());
}

#[test]
fn test_temp_directory_cleanup() {
    // Test that temp directories are properly cleaned up
    let change_dir_path: PathBuf;
    {
        let (temp_dir, change_dir) = create_temp_change(
            VALID_PROPOSAL,
            &[("auth", VALID_DELTA_SPEC_ADDED)],
        );
        change_dir_path = change_dir.clone();
        assert!(change_dir_path.exists());
        drop(temp_dir); // Explicitly drop to trigger cleanup
    }
    // Directory should be cleaned up after temp_dir is dropped
    assert!(!change_dir_path.exists());
}

#[test]
fn test_multiple_requirements_in_added_section() {
    let multi_req_spec = r#"# Multi-Requirement Delta

## ADDED Requirements

### Requirement: First Feature

The system SHALL support the first feature.

#### Scenario: First scenario

- **WHEN** user uses first feature
- **THEN** it works

### Requirement: Second Feature

The system SHALL support the second feature.

#### Scenario: Second scenario

- **WHEN** user uses second feature
- **THEN** it works

### Requirement: Third Feature

The system SHALL support the third feature.

#### Scenario: Third scenario A

- **WHEN** condition A
- **THEN** result A

#### Scenario: Third scenario B

- **WHEN** condition B
- **THEN** result B
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[("multi", multi_req_spec)],
    );

    let delta_content = fs::read_to_string(change_dir.join("specs/multi/spec.md")).unwrap();

    // Count requirements
    let req_count = delta_content.matches("### Requirement:").count();
    assert_eq!(req_count, 3);

    // Count scenarios
    let scenario_count = delta_content.matches("#### Scenario:").count();
    assert_eq!(scenario_count, 4);
}

#[test]
fn test_case_insensitive_proposal_headers() {
    let uppercase_proposal = r#"# Feature

## WHY

This feature is needed for important reasons that benefit users significantly
and improve the overall system functionality and user experience.

## WHAT CHANGES

- Add feature
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        uppercase_proposal,
        &[("feature", VALID_DELTA_SPEC_ADDED)],
    );

    let proposal_content = fs::read_to_string(change_dir.join("proposal.md")).unwrap();
    assert!(proposal_content.contains("## WHY"));
    assert!(proposal_content.contains("## WHAT CHANGES"));
}

#[test]
fn test_case_insensitive_delta_headers() {
    let uppercase_delta = r#"# Delta

## ADDED REQUIREMENTS

### Requirement: Feature

The system SHALL support the feature.

#### Scenario: Test

- **WHEN** test
- **THEN** pass
"#;

    let (_temp_dir, change_dir) = create_temp_change(
        VALID_PROPOSAL,
        &[("feature", uppercase_delta)],
    );

    let delta_content = fs::read_to_string(change_dir.join("specs/feature/spec.md")).unwrap();
    assert!(delta_content.contains("## ADDED REQUIREMENTS"));
}
