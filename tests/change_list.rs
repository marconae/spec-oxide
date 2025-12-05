//! Integration tests for `spox change list` command.

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

/// Helper to create a change with tasks
fn create_change(
    changes_dir: &std::path::Path,
    name: &str,
    completed_tasks: usize,
    total_tasks: usize,
) {
    let change_dir = changes_dir.join(name);
    fs::create_dir_all(&change_dir).unwrap();

    // Create proposal.md
    let proposal = format!(
        r#"# Change: {}

## Summary
Test change proposal.

## Motivation
Testing change list.
"#,
        name
    );
    fs::write(change_dir.join("proposal.md"), proposal).unwrap();

    // Create tasks.md with checkbox items
    let mut tasks = "# Tasks\n\n".to_string();
    for i in 0..total_tasks {
        if i < completed_tasks {
            tasks.push_str(&format!("- [x] Task {}\n", i + 1));
        } else {
            tasks.push_str(&format!("- [ ] Task {}\n", i + 1));
        }
    }
    fs::write(change_dir.join("tasks.md"), tasks).unwrap();
}

// =============================================================================
// Test: spox change list in initialized project with changes
// =============================================================================

#[test]
fn test_change_list_with_multiple_changes() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    create_config(root, "specs", "specs/_changes");

    // Create changes
    create_change(&changes_dir, "add-2fa", 2, 5);
    create_change(&changes_dir, "fix-login", 0, 3);

    // Run change list
    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes:"))
        .stdout(predicate::str::contains("add-2fa"))
        .stdout(predicate::str::contains("2/5 tasks"))
        .stdout(predicate::str::contains("fix-login"))
        .stdout(predicate::str::contains("0/3 tasks"));
}

#[test]
fn test_change_list_alphabetical_order() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create changes in non-alphabetical order
    create_change(&changes_dir, "zebra-change", 1, 2);
    create_change(&changes_dir, "alpha-change", 2, 3);
    create_change(&changes_dir, "middle-change", 0, 1);

    let output = spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Verify alphabetical order
    let alpha_pos = output_str.find("alpha-change").unwrap();
    let middle_pos = output_str.find("middle-change").unwrap();
    let zebra_pos = output_str.find("zebra-change").unwrap();

    assert!(
        alpha_pos < middle_pos,
        "alpha-change should appear before middle-change"
    );
    assert!(
        middle_pos < zebra_pos,
        "middle-change should appear before zebra-change"
    );
}

#[test]
fn test_change_list_completed_change() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create a fully completed change
    create_change(&changes_dir, "done-feature", 5, 5);

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("done-feature"))
        .stdout(predicate::str::contains("5/5 tasks"));
}

#[test]
fn test_change_list_excludes_archive() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    let archive_dir = changes_dir.join("_archive");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create an active change
    create_change(&changes_dir, "active-change", 1, 3);

    // Create an archived change
    create_change(&archive_dir, "archived-change", 3, 3);

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("active-change"))
        .stdout(predicate::str::contains("archived-change").not())
        .stdout(predicate::str::contains("_archive").not());
}

#[test]
fn test_change_list_excludes_hidden_folders() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create an active change
    create_change(&changes_dir, "visible-change", 1, 2);

    // Create a hidden folder (starts with .)
    let hidden_dir = changes_dir.join(".hidden");
    fs::create_dir_all(&hidden_dir).unwrap();
    fs::write(hidden_dir.join("tasks.md"), "- [x] Task\n").unwrap();

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("visible-change"))
        .stdout(predicate::str::contains(".hidden").not());
}

// =============================================================================
// Test: spox change list in initialized project with no changes
// =============================================================================

#[test]
fn test_change_list_with_no_changes() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Don't create any changes

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No active changes."));
}

#[test]
fn test_change_list_empty_with_only_archive() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    let archive_dir = changes_dir.join("_archive");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Only archived changes exist
    create_change(&archive_dir, "old-change", 3, 3);

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No active changes."));
}

// =============================================================================
// Test: spox change list without initialization (should fail)
// =============================================================================

#[test]
fn test_change_list_without_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Don't create any config or directories

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

#[test]
fn test_change_list_with_missing_config() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create changes directory but no config
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&changes_dir).unwrap();

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

// =============================================================================
// Additional edge case tests
// =============================================================================

#[test]
fn test_change_list_change_without_tasks_md() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create a change directory without tasks.md file
    let empty_change = changes_dir.join("empty-change");
    fs::create_dir_all(&empty_change).unwrap();
    fs::write(empty_change.join("proposal.md"), "# Change\n").unwrap();

    // Create a normal change
    create_change(&changes_dir, "normal-change", 1, 2);

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("empty-change"))
        .stdout(predicate::str::contains("0/0 tasks"))
        .stdout(predicate::str::contains("normal-change"))
        .stdout(predicate::str::contains("1/2 tasks"));
}

#[test]
fn test_change_list_changes_folder_does_not_exist() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create config pointing to non-existent changes folder
    let spox_dir = root.join(".spox");
    fs::create_dir_all(&spox_dir).unwrap();

    let config = r#"[paths]
spec_folder = "specs"
changes_folder = "nonexistent"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#;
    fs::write(spox_dir.join("config.toml"), config).unwrap();

    // Should succeed with empty list
    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No active changes."));
}

#[test]
fn test_change_list_many_changes() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create many changes
    for i in 1..=10 {
        let name = format!("change-{:02}", i);
        create_change(&changes_dir, &name, i % 5, 5);
    }

    let output = spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Verify all 10 changes are listed
    for i in 1..=10 {
        let name = format!("change-{:02}", i);
        assert!(
            output_str.contains(&name),
            "Should contain change: {}",
            name
        );
    }
}

#[test]
fn test_change_list_uppercase_x_checkbox() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create a change with uppercase X checkboxes
    let change_dir = changes_dir.join("mixed-case");
    fs::create_dir_all(&change_dir).unwrap();

    let tasks = r#"# Tasks

- [x] Lowercase x
- [X] Uppercase X
- [ ] Incomplete
"#;
    fs::write(change_dir.join("tasks.md"), tasks).unwrap();

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("mixed-case"))
        .stdout(predicate::str::contains("2/3 tasks"));
}

#[test]
fn test_change_list_asterisk_checkbox() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let specs_dir = root.join("specs");
    let changes_dir = root.join("specs/_changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    create_config(root, "specs", "specs/_changes");

    // Create a change with asterisk checkboxes
    let change_dir = changes_dir.join("asterisk-style");
    fs::create_dir_all(&change_dir).unwrap();

    let tasks = r#"# Tasks

* [x] Completed with asterisk
* [ ] Incomplete with asterisk
"#;
    fs::write(change_dir.join("tasks.md"), tasks).unwrap();

    spox_cmd()
        .current_dir(root)
        .arg("change")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("asterisk-style"))
        .stdout(predicate::str::contains("1/2 tasks"));
}
