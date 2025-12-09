//! Integration tests for `spox index` command.

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
fn create_spec(specs_dir: &std::path::Path, name: &str) {
    let spec_dir = specs_dir.join(name);
    fs::create_dir_all(&spec_dir).unwrap();

    let content = format!(
        r#"# {} Specification

## Purpose

This spec defines {} functionality.

## Requirements

### Requirement: Basic Feature

The system SHALL provide basic {} features.

#### Scenario: Basic operation

- **WHEN** user requests operation
- **THEN** operation completes
"#,
        name, name, name
    );
    fs::write(spec_dir.join("spec.md"), content).unwrap();
}

// =============================================================================
// Test: spox index help
// =============================================================================

#[test]
fn test_index_help_shows_description() {
    spox_cmd()
        .arg("index")
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("semantic search index")
                .or(predicate::str::contains("search index")),
        );
}

// =============================================================================
// Test: spox index creates index file
// =============================================================================

#[test]
#[ignore] // Requires fastembed model download, run with: cargo test --test index_cmd -- --ignored
fn test_index_creates_index_file() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Create a spec to index
    create_spec(&specs_dir, "auth");

    // Run spox index
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
        "Index file should exist at .spox/search_index.bin"
    );
}

#[test]
#[ignore] // Requires fastembed model download
fn test_index_with_multiple_specs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Create multiple specs
    create_spec(&specs_dir, "auth");
    create_spec(&specs_dir, "payments");
    create_spec(&specs_dir, "notifications");

    // Run spox index
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .success()
        .stderr(predicate::str::contains("Found 3 specs"))
        .stderr(predicate::str::contains("Index built successfully"));

    // Verify index file was created
    let index_path = root.join(".spox/search_index.bin");
    assert!(index_path.exists(), "Index file should exist");
}

// =============================================================================
// Test: spox index with no specs (shows warning)
// =============================================================================

#[test]
#[ignore] // Requires fastembed model download
fn test_index_with_no_specs_shows_warning() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Don't create any specs

    // Run spox index - should succeed with warning
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .success()
        .stderr(predicate::str::contains("Warning: No spec files found"));

    // Empty index should still be created
    let index_path = root.join(".spox/search_index.bin");
    assert!(
        index_path.exists(),
        "Index file should exist even with no specs"
    );
}

// =============================================================================
// Test: spox index without initialization (should fail)
// =============================================================================

#[test]
fn test_index_without_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Don't create any config or directories

    // Run spox index - should fail
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

#[test]
fn test_index_with_missing_config() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create specs directory but no config
    let specs_dir = root.join("specs");
    fs::create_dir_all(&specs_dir).unwrap();

    // Run spox index - should fail
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

// =============================================================================
// Test: spox index command shows progress messages
// =============================================================================

#[test]
#[ignore] // Requires fastembed model download
fn test_index_shows_progress_messages() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Create a spec
    create_spec(&specs_dir, "auth");

    // Run spox index and verify progress messages
    spox_cmd()
        .current_dir(root)
        .arg("index")
        .assert()
        .success()
        .stderr(predicate::str::contains("Parsing specs"))
        .stderr(predicate::str::contains("Generating embeddings"))
        .stderr(predicate::str::contains("Saving index"));
}

// =============================================================================
// Test: main help includes index command
// =============================================================================

#[test]
fn test_main_help_shows_index_command() {
    spox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("index"));
}
