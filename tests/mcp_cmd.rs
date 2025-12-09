//! Integration tests for `spox mcp` command.

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

// =============================================================================
// Test: spox mcp help
// =============================================================================

#[test]
fn test_mcp_help_shows_serve() {
    spox_cmd()
        .arg("mcp")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("serve"));
}

#[test]
fn test_mcp_serve_help() {
    spox_cmd()
        .arg("mcp")
        .arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdio").or(predicate::str::contains("MCP")));
}

// =============================================================================
// Test: main help includes mcp command
// =============================================================================

#[test]
fn test_main_help_shows_mcp_command() {
    spox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("mcp"));
}

// =============================================================================
// Test: spox mcp serve without initialization (should fail)
// =============================================================================

#[test]
fn test_mcp_serve_without_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Don't create any config or directories

    // Run spox mcp serve - should fail quickly (no stdin means it won't hang)
    // We use timeout since it tries to read from stdin
    spox_cmd()
        .current_dir(root)
        .arg("mcp")
        .arg("serve")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

// =============================================================================
// Test: spox mcp serve starts with valid config
// Note: Full MCP protocol testing is complex as it requires stdin/stdout
// interaction. This test verifies the server starts and fails gracefully
// when stdin is closed.
// =============================================================================

#[test]
fn test_mcp_serve_with_config_attempts_startup() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Run spox mcp serve with timeout
    // The server will try to start but stdin will be closed, causing it to exit
    // We just want to make sure it doesn't fail due to config issues
    let result = spox_cmd()
        .current_dir(root)
        .arg("mcp")
        .arg("serve")
        .timeout(std::time::Duration::from_secs(2))
        .assert();

    // The command may timeout or fail due to closed stdin, but should NOT
    // fail due to "config not found" - that would indicate a setup issue
    let output = result.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("config not found"),
        "Should not fail due to config issues: {}",
        stderr
    );
}

// =============================================================================
// Test: MCP subcommand structure
// =============================================================================

#[test]
fn test_mcp_without_subcommand_shows_help() {
    // Running just "spox mcp" without a subcommand should show help/usage
    let result = spox_cmd().arg("mcp").assert();

    // Check that it either shows help or returns an error about missing subcommand
    let output = result.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("serve") || combined.contains("Usage") || combined.contains("subcommand"),
        "Should mention 'serve' subcommand or usage info: {}",
        combined
    );
}
