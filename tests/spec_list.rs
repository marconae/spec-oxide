//! Integration tests for `spox spec list` command.

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
        r#"spec_folder = "{}"
changes_folder = "{}"
archive_folder = "{}_archive"
"#,
        spec_folder, changes_folder, changes_folder
    );
    fs::write(spox_dir.join("config.toml"), config).unwrap();
}

/// Helper to create a spec with requirements
fn create_spec(specs_dir: &std::path::Path, name: &str, requirements: &[&str]) {
    let spec_dir = specs_dir.join(name);
    fs::create_dir_all(&spec_dir).unwrap();

    let mut content = format!("# Spec: {}\n\n## Purpose\nTest spec.\n\n", name);

    for req in requirements {
        content.push_str(&format!(
            "### Requirement: {}\nThe system SHALL {}.\n\n",
            req, req
        ));
    }

    fs::write(spec_dir.join("spec.md"), content).unwrap();
}

// =============================================================================
// Test: spox spec list in initialized project with specs
// =============================================================================

#[test]
fn test_spec_list_with_multiple_specs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Create specs
    create_spec(&specs_dir, "auth", &["login", "logout"]);
    create_spec(&specs_dir, "config", &["load", "save", "validate"]);

    // Run spec list
    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Specs:"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("2 requirements"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("3 requirements"));
}

#[test]
fn test_spec_list_alphabetical_order() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create specs in non-alphabetical order
    create_spec(&specs_dir, "zebra", &["stripe"]);
    create_spec(&specs_dir, "alpha", &["first"]);
    create_spec(&specs_dir, "middle", &["center"]);

    let output = spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Verify alphabetical order: alpha should appear before middle, middle before zebra
    let alpha_pos = output_str.find("alpha").unwrap();
    let middle_pos = output_str.find("middle").unwrap();
    let zebra_pos = output_str.find("zebra").unwrap();

    assert!(alpha_pos < middle_pos, "alpha should appear before middle");
    assert!(middle_pos < zebra_pos, "middle should appear before zebra");
}

#[test]
fn test_spec_list_singular_requirement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create spec with single requirement
    create_spec(&specs_dir, "single", &["only-one"]);

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 requirement"))
        .stdout(predicate::str::contains("single"));

    // Make sure it says "requirement" (singular) not "requirements"
    let output = spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let single_line = output_str.lines().find(|l| l.contains("single")).unwrap();
    assert!(
        single_line.contains("1 requirement") && !single_line.contains("1 requirements"),
        "Should use singular 'requirement' for count of 1"
    );
}

#[test]
fn test_spec_list_excludes_underscore_folders() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    let archive_dir = root.join("specs/_archive");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create a real spec
    create_spec(&specs_dir, "real-spec", &["requirement"]);

    // Create a spec-like folder with underscore prefix (should be excluded)
    create_spec(&specs_dir, "_hidden", &["hidden-req"]);

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("real-spec"))
        .stdout(predicate::str::contains("_hidden").not())
        .stdout(predicate::str::contains("_changes").not())
        .stdout(predicate::str::contains("_archive").not());
}

// =============================================================================
// Test: spox spec list in initialized project with no specs
// =============================================================================

#[test]
fn test_spec_list_with_no_specs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Don't create any specs

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No specs found."));
}

#[test]
fn test_spec_list_empty_with_only_underscore_folders() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    let archive_dir = root.join("specs/_archive");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Only underscore folders exist, no real specs

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No specs found."));
}

// =============================================================================
// Test: spox spec list without initialization (should fail)
// =============================================================================

#[test]
fn test_spec_list_without_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Don't create any config or directories

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

#[test]
fn test_spec_list_with_missing_config() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create specs directory but no config
    let specs_dir = root.join("specs");
    fs::create_dir_all(&specs_dir).unwrap();

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

// =============================================================================
// Additional edge case tests
// =============================================================================

#[test]
fn test_spec_list_spec_without_spec_md() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create a spec directory without spec.md file
    let empty_spec = specs_dir.join("empty-spec");
    fs::create_dir_all(&empty_spec).unwrap();

    // Create a normal spec
    create_spec(&specs_dir, "normal-spec", &["req1"]);

    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("empty-spec"))
        .stdout(predicate::str::contains("0 requirements"))
        .stdout(predicate::str::contains("normal-spec"))
        .stdout(predicate::str::contains("1 requirement"));
}

#[test]
fn test_spec_list_spec_folder_does_not_exist() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create config pointing to non-existent spec folder
    let spox_dir = root.join(".spox");
    fs::create_dir_all(&spox_dir).unwrap();

    let config = r#"spec_folder = "nonexistent"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"
"#;
    fs::write(spox_dir.join("config.toml"), config).unwrap();

    // Should succeed with empty list
    spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No specs found."));
}

#[test]
fn test_spec_list_many_specs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create many specs
    for i in 1..=10 {
        let name = format!("spec-{:02}", i);
        let reqs: Vec<&str> = (0..i).map(|_| "req").collect();
        create_spec(&specs_dir, &name, &reqs);
    }

    let output = spox_cmd()
        .current_dir(root)
        .arg("spec")
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Verify all 10 specs are listed
    for i in 1..=10 {
        let name = format!("spec-{:02}", i);
        assert!(output_str.contains(&name), "Should contain spec: {}", name);
    }
}
