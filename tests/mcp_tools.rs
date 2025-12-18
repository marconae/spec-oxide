//! Integration tests for MCP tools via CLI.
//!
//! Note: The MCP tools are unit-tested extensively in src/mcp/mod.rs.
//! This file tests CLI-accessible aspects of the MCP functionality.
//!
//! For full MCP protocol testing, see the unit tests in src/mcp/mod.rs
//! which cover:
//! - list_specs with fixture specs
//! - get_spec_requirements with valid/invalid IDs
//! - get_scenario with valid/invalid parameters
//! - search_specs with and without index

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a Command for the spox binary.
fn spox_cmd() -> Command {
    cargo_bin_cmd!("spox")
}

/// Helper to create a minimal .spox/config.toml
fn create_config(root: &std::path::Path, spec_folder: &str, changes_folder: &str) {
    let spox_dir = root.join(".spox");
    fs::create_dir_all(&spox_dir).unwrap();

    let config = format!(
        r#"[paths]
spec_folder = "{}"
changes_folder = "{}"
archive_folder = "{}_archive"

[rules]
system = ["mcp"]
"#,
        spec_folder, changes_folder, changes_folder
    );
    fs::write(spox_dir.join("config.toml"), config).unwrap();
}

/// Helper to create a spec with requirements
fn create_spec(specs_dir: &std::path::Path, name: &str, content: &str) {
    let spec_dir = specs_dir.join(name);
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(spec_dir.join("spec.md"), content).unwrap();
}

const AUTH_SPEC: &str = r#"# Auth Specification

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

const PAYMENTS_SPEC: &str = r#"# Payments Specification

## Purpose

This spec defines payment processing requirements.

## Requirements

### Requirement: Process Payment

The system SHALL process credit card payments.

#### Scenario: Successful payment

- **WHEN** valid credit card details provided
- **THEN** payment is processed
- **AND** confirmation is returned
"#;

// =============================================================================
// MCP tools are tested via spec commands (which use the same parsing logic)
//
// The MCP server exposes these tools:
// - list_specs: Uses parse_all_specs (same as spec list)
// - get_spec_requirements: Uses get_spec_by_id (same as spec show)
// - get_scenario: Uses get_spec_by_id (same as spec show)
// - search_specs: Uses the search index
//
// These integration tests verify the underlying spec parsing works correctly
// through the CLI, which exercises the same code paths as the MCP tools.
// =============================================================================

// =============================================================================
// Test: Spec list (same parsing as list_specs MCP tool)
// =============================================================================

#[test]
fn test_spec_list_with_fixture_specs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Create fixture specs
    create_spec(&specs_dir, "auth", AUTH_SPEC);
    create_spec(&specs_dir, "payments", PAYMENTS_SPEC);

    // Run spec list (uses same parsing as list_specs MCP tool)
    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("payments"));
}

#[test]
fn test_spec_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No specs found"));
}

// =============================================================================
// Test: Spec show (same parsing as get_spec_requirements MCP tool)
// =============================================================================

#[test]
fn test_spec_show_valid_id() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");
    create_spec(&specs_dir, "auth", AUTH_SPEC);

    // Run spec show (uses same parsing as get_spec_requirements/get_scenario MCP tools)
    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("show")
        .arg("auth")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("authentication"))
        .stdout(predicate::str::contains("User Login"))
        .stdout(predicate::str::contains("User Logout"));
}

#[test]
fn test_spec_show_invalid_id() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");
    create_spec(&specs_dir, "auth", AUTH_SPEC);

    // Run spec show with invalid ID
    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("show")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("error")));
}

// =============================================================================
// Test: Spec with scenarios (tests scenario parsing used by get_scenario)
// =============================================================================

#[test]
fn test_spec_show_displays_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");
    create_spec(&specs_dir, "auth", AUTH_SPEC);

    // Verify scenarios are parsed (same parsing as get_scenario MCP tool)
    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("show")
        .arg("auth")
        .assert()
        .success()
        .stdout(predicate::str::contains("Successful login"))
        .stdout(predicate::str::contains("Failed login"))
        .stdout(predicate::str::contains("Logout clears session"));
}

// =============================================================================
// Test: Index command (prerequisite for search_specs MCP tool)
// =============================================================================

#[test]
#[ignore] // Requires fastembed model download
fn test_index_prerequisite_for_search() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");
    create_spec(&specs_dir, "auth", AUTH_SPEC);
    create_spec(&specs_dir, "payments", PAYMENTS_SPEC);

    // Build index (required before search_specs MCP tool can work)
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .success()
        .stderr(predicate::str::contains("Index built successfully"));

    // Verify index file was created
    let index_path = root.join(".spox/search_index.bin");
    assert!(
        index_path.exists(),
        "Index file should exist for search_specs to work"
    );
}

// =============================================================================
// Test: MCP server capabilities
// =============================================================================

#[test]
fn test_mcp_help_describes_tools() {
    // The mcp serve help should mention it exposes spec tools
    spox_cmd()
        .arg("mcp")
        .arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("MCP")
                .or(predicate::str::contains("stdio"))
                .or(predicate::str::contains("server")),
        );
}

// =============================================================================
// Test: Existing CLI commands work after restructure
// =============================================================================

#[test]
fn test_spec_validate_works() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");
    create_spec(&specs_dir, "auth", AUTH_SPEC);

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("validate")
        .assert()
        .success();
}

#[test]
fn test_config_show_works() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    spox_cmd()
        .current_dir(root)
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec_folder"))
        .stdout(predicate::str::contains("changes_folder"));
}

#[test]
fn test_change_list_works() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_init_works() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    // Verify basic structure was created
    assert!(root.join(".spox/config.toml").exists());
    assert!(root.join("specs").exists());
}

#[test]
fn test_show_dashboard_works() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");
    create_spec(&specs_dir, "auth", AUTH_SPEC);

    spox_cmd().current_dir(root).arg("show").assert().success();
}

// =============================================================================
// Test: rebuild_index MCP tool (via index command)
// =============================================================================

#[test]
#[ignore] // Requires fastembed model download
fn test_rebuild_index_creates_index_with_specs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");
    create_spec(&specs_dir, "auth", AUTH_SPEC);
    create_spec(&specs_dir, "payments", PAYMENTS_SPEC);

    // Build index (this exercises the same core::rebuild_index function as the MCP tool)
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .success()
        .stderr(predicate::str::contains("Index built successfully"));

    // Verify index file was created
    let index_path = root.join(".spox/search_index.bin");
    assert!(
        index_path.exists(),
        "Index file should exist after rebuild_index"
    );
}

#[test]
#[ignore] // Requires fastembed model download
fn test_rebuild_index_with_no_specs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Build index with no specs (should succeed with 0 specs indexed)
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .success()
        .stderr(predicate::str::contains("Index built"));

    // Verify index file was created (even though empty)
    let index_path = root.join(".spox/search_index.bin");
    assert!(
        index_path.exists(),
        "Index file should exist even when no specs"
    );
}
