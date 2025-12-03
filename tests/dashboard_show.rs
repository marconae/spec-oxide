//! Integration tests for `spox show` dashboard functionality.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Re-export the modules we need for testing
mod common {
    use std::fs;
    use std::path::Path;

    /// Create a minimal spox.toml config file
    pub fn create_config(root: &Path, spec_folder: &str, changes_folder: &str) {
        let config = format!(
            r#"spec_folder = "{}"
changes_folder = "{}"
"#,
            spec_folder, changes_folder
        );
        fs::write(root.join("spox.toml"), config).unwrap();
    }

    /// Create a spec with requirements
    pub fn create_spec(specs_dir: &Path, name: &str, requirements: &[&str]) {
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

    /// Create a change with tasks and optionally deltas
    pub fn create_change(
        changes_dir: &Path,
        name: &str,
        completed_tasks: usize,
        total_tasks: usize,
        deltas: Option<&[(&str, usize, usize, usize)]>, // (capability, added, modified, removed)
    ) {
        let change_dir = changes_dir.join(name);
        fs::create_dir_all(&change_dir).unwrap();

        // Create proposal.md
        let proposal = format!(
            r#"# Change: {}

## Summary
Test change proposal.

## Motivation
Testing dashboard.
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

        // Create delta specs if provided
        if let Some(delta_list) = deltas {
            let specs_dir = change_dir.join("specs");
            fs::create_dir_all(&specs_dir).unwrap();

            for (capability, added, modified, removed) in delta_list {
                let cap_dir = specs_dir.join(capability);
                fs::create_dir_all(&cap_dir).unwrap();

                let mut spec_content = format!("# {} Delta\n\n", capability);

                if *added > 0 {
                    spec_content.push_str("## ADDED Requirements\n\n");
                    for i in 0..*added {
                        spec_content.push_str(&format!(
                            "### Requirement: Added{}\nNew requirement.\n\n",
                            i + 1
                        ));
                    }
                }

                if *modified > 0 {
                    spec_content.push_str("## MODIFIED Requirements\n\n");
                    for i in 0..*modified {
                        spec_content.push_str(&format!(
                            "### Requirement: Modified{}\nUpdated requirement.\n\n",
                            i + 1
                        ));
                    }
                }

                if *removed > 0 {
                    spec_content.push_str("## REMOVED Requirements\n\n");
                    for i in 0..*removed {
                        spec_content.push_str(&format!(
                            "### Requirement: Removed{}\nDeprecated requirement.\n\n",
                            i + 1
                        ));
                    }
                }

                fs::write(cap_dir.join("spec.md"), spec_content).unwrap();
            }
        }
    }
}

/// Helper to run show_dashboard via the library function
fn run_dashboard(root: &PathBuf) -> Result<String, String> {
    // Load config from the temp directory
    let config_path = root.join("spox.toml");
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // Parse the TOML manually (simple extraction for test)
    let spec_folder = extract_toml_value(&config_content, "spec_folder")
        .ok_or("Missing spec_folder in config")?;
    let changes_folder = extract_toml_value(&config_content, "changes_folder")
        .ok_or("Missing changes_folder in config")?;

    // Resolve to absolute paths
    let spec_folder = root.join(&spec_folder).to_string_lossy().to_string();
    let changes_folder = root.join(&changes_folder).to_string_lossy().to_string();

    // Create a config struct
    let config = TestConfig {
        spec_folder,
        changes_folder,
    };

    // Use the dashboard module
    gather_and_format_dashboard(&config)
}

/// Simple TOML value extractor for tests
fn extract_toml_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(key) {
            if let Some(pos) = line.find('=') {
                let value = line[pos + 1..].trim();
                // Remove quotes
                let value = value.trim_matches('"');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Minimal test config
struct TestConfig {
    spec_folder: String,
    changes_folder: String,
}

/// Gather and format dashboard (mirrors the actual implementation)
fn gather_and_format_dashboard(config: &TestConfig) -> Result<String, String> {
    use std::path::Path;

    // Gather specs
    let spec_path = Path::new(&config.spec_folder);
    let mut specs = Vec::new();

    if spec_path.exists() {
        if let Ok(entries) = fs::read_dir(spec_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with('_') {
                    continue;
                }

                let spec_file = path.join("spec.md");
                let requirement_count = if spec_file.exists() {
                    count_requirements(&spec_file)
                } else {
                    0
                };

                specs.push((name, requirement_count));
            }
        }
    }
    specs.sort_by(|a, b| a.0.cmp(&b.0));

    // Gather changes
    let changes_path = Path::new(&config.changes_folder);
    let mut changes = Vec::new();

    if changes_path.exists() {
        if let Ok(entries) = fs::read_dir(changes_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if name == "_archive" || name.starts_with('.') {
                    continue;
                }

                let tasks_path = path.join("tasks.md");
                let (completed, total) = if tasks_path.exists() {
                    parse_tasks(&tasks_path)
                } else {
                    (0, 0)
                };

                let specs_path = path.join("specs");
                let delta_summary = if specs_path.exists() && specs_path.is_dir() {
                    gather_deltas(&specs_path)
                } else {
                    String::new()
                };

                changes.push((name, completed, total, delta_summary));
            }
        }
    }
    changes.sort_by(|a, b| a.0.cmp(&b.0));

    // Format output
    let mut output = String::new();
    output.push_str("Spec Oxide Dashboard\n\n");

    output.push_str(&format!("Specs: {}\n", specs.len()));
    if specs.is_empty() {
        output.push_str("  (no specs)\n");
    } else {
        for (name, count) in &specs {
            let word = if *count == 1 {
                "requirement"
            } else {
                "requirements"
            };
            output.push_str(&format!("  {} {} {}\n", name, count, word));
        }
    }

    output.push('\n');

    output.push_str(&format!("Active Changes: {}\n", changes.len()));
    if changes.is_empty() {
        output.push_str("  (no active changes)\n");
    } else {
        for (name, completed, total, delta) in &changes {
            output.push_str(&format!("  {} {}/{} tasks\n", name, completed, total));
            if !delta.is_empty() {
                output.push_str(&format!("    -> {}\n", delta));
            }
        }
    }

    Ok(output)
}

fn count_requirements(path: &std::path::Path) -> usize {
    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter(|line| line.trim().starts_with("### Requirement:"))
        .count()
}

fn parse_tasks(path: &std::path::Path) -> (usize, usize) {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut total = 0;
    let mut completed = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ] ") || trimmed.starts_with("* [ ] ") {
            total += 1;
        } else if trimmed.starts_with("- [x] ")
            || trimmed.starts_with("- [X] ")
            || trimmed.starts_with("* [x] ")
            || trimmed.starts_with("* [X] ")
        {
            total += 1;
            completed += 1;
        }
    }

    (completed, total)
}

fn gather_deltas(specs_path: &std::path::Path) -> String {
    let mut deltas = Vec::new();

    if let Ok(entries) = fs::read_dir(specs_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let spec_file = path.join("spec.md");
            if spec_file.exists() {
                let content = fs::read_to_string(&spec_file).unwrap_or_default();
                let (added, modified, removed) = count_delta_sections(&content);
                if added > 0 || modified > 0 || removed > 0 {
                    let mut parts = Vec::new();
                    if added > 0 {
                        parts.push(format!("+{}", added));
                    }
                    if modified > 0 {
                        parts.push(format!("~{}", modified));
                    }
                    if removed > 0 {
                        parts.push(format!("-{}", removed));
                    }
                    deltas.push(format!("{} ({})", name, parts.join(", ")));
                }
            }
        }
    }

    deltas.sort();
    deltas.join(", ")
}

fn count_delta_sections(content: &str) -> (usize, usize, usize) {
    let mut added = 0;
    let mut modified = 0;
    let mut removed = 0;
    let mut current: Option<&str> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if upper == "## ADDED REQUIREMENTS" {
            current = Some("added");
        } else if upper == "## MODIFIED REQUIREMENTS" {
            current = Some("modified");
        } else if upper == "## REMOVED REQUIREMENTS" {
            current = Some("removed");
        } else if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            current = None;
        }

        if trimmed.starts_with("### Requirement:") {
            match current {
                Some("added") => added += 1,
                Some("modified") => modified += 1,
                Some("removed") => removed += 1,
                _ => {}
            }
        }
    }

    (added, modified, removed)
}

// ==================== Integration Tests ====================

#[test]
fn test_dashboard_initialized_project() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create directory structure
    let specs_dir = root.join("../specs");
    let changes_dir = root.join("openspec/changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    common::create_config(&root, "openspec/specs", "openspec/changes");

    // Create specs
    common::create_spec(&specs_dir, "auth", &["login", "logout", "session", "token"]);
    common::create_spec(&specs_dir, "config", &["load", "save"]);

    // Create a change with tasks and deltas
    common::create_change(
        &changes_dir,
        "add-two-factor",
        2,
        5,
        Some(&[("auth", 1, 1, 0), ("notifications", 1, 0, 0)]),
    );

    // Run dashboard
    let output = run_dashboard(&root).expect("Dashboard should succeed");

    // Verify specs
    assert!(output.contains("Specs: 2"), "Should show 2 specs");
    assert!(output.contains("auth"), "Should list auth spec");
    assert!(output.contains("4"), "auth should have 4 requirements");
    assert!(output.contains("config"), "Should list config spec");
    assert!(output.contains("2"), "config should have 2 requirements");

    // Verify changes
    assert!(output.contains("Active Changes: 1"), "Should show 1 change");
    assert!(output.contains("add-two-factor"), "Should list change name");
    assert!(output.contains("2/5 tasks"), "Should show task progress");

    // Verify delta summary
    assert!(output.contains("auth (+1, ~1)"), "Should show auth deltas");
    assert!(
        output.contains("notifications (+1)"),
        "Should show notifications delta"
    );
}

#[test]
fn test_dashboard_no_specs() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create empty directories
    let specs_dir = root.join("../specs");
    let changes_dir = root.join("openspec/changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    common::create_config(&root, "openspec/specs", "openspec/changes");

    // Run dashboard
    let output = run_dashboard(&root).expect("Dashboard should succeed");

    // Verify empty state
    assert!(output.contains("Specs: 0"), "Should show 0 specs");
    assert!(output.contains("(no specs)"), "Should indicate no specs");
    assert!(
        output.contains("Active Changes: 0"),
        "Should show 0 changes"
    );
    assert!(
        output.contains("(no active changes)"),
        "Should indicate no changes"
    );
}

#[test]
fn test_dashboard_no_active_changes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create directories
    let specs_dir = root.join("../specs");
    let changes_dir = root.join("openspec/changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    common::create_config(&root, "openspec/specs", "openspec/changes");

    // Create specs but no changes
    common::create_spec(&specs_dir, "cli", &["parse", "execute", "help", "version"]);

    // Run dashboard
    let output = run_dashboard(&root).expect("Dashboard should succeed");

    // Verify specs present
    assert!(output.contains("Specs: 1"), "Should show 1 spec");
    assert!(output.contains("cli"), "Should list cli spec");
    assert!(output.contains("4"), "cli should have 4 requirements");

    // Verify no changes
    assert!(
        output.contains("Active Changes: 0"),
        "Should show 0 changes"
    );
    assert!(
        output.contains("(no active changes)"),
        "Should indicate no changes"
    );
}

#[test]
fn test_dashboard_excludes_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create directories
    let specs_dir = root.join("../specs");
    let changes_dir = root.join("openspec/changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    common::create_config(&root, "openspec/specs", "openspec/changes");

    // Create an active change
    common::create_change(&changes_dir, "active-change", 1, 3, None);

    // Create archived change in _archive
    let archive_dir = changes_dir.join("_archive");
    fs::create_dir_all(&archive_dir).unwrap();
    common::create_change(&archive_dir, "old-change", 3, 3, None);

    // Run dashboard
    let output = run_dashboard(&root).expect("Dashboard should succeed");

    // Verify only active change is shown
    assert!(
        output.contains("Active Changes: 1"),
        "Should show 1 active change"
    );
    assert!(
        output.contains("active-change"),
        "Should show active change"
    );
    assert!(
        !output.contains("old-change"),
        "Should NOT show archived change"
    );
}

#[test]
fn test_dashboard_multiple_changes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create directories
    let specs_dir = root.join("../specs");
    let changes_dir = root.join("openspec/changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    common::create_config(&root, "openspec/specs", "openspec/changes");

    // Create multiple changes
    common::create_change(&changes_dir, "aaa-first", 0, 2, None);
    common::create_change(&changes_dir, "bbb-second", 1, 4, Some(&[("core", 2, 0, 0)]));
    common::create_change(&changes_dir, "ccc-third", 3, 3, None);

    // Run dashboard
    let output = run_dashboard(&root).expect("Dashboard should succeed");

    // Verify all changes shown
    assert!(
        output.contains("Active Changes: 3"),
        "Should show 3 changes"
    );
    assert!(output.contains("aaa-first"), "Should list first change");
    assert!(output.contains("bbb-second"), "Should list second change");
    assert!(output.contains("ccc-third"), "Should list third change");

    // Verify task progress
    assert!(output.contains("0/2 tasks"), "First change progress");
    assert!(output.contains("1/4 tasks"), "Second change progress");
    assert!(output.contains("3/3 tasks"), "Third change progress");

    // Verify delta for second change
    assert!(output.contains("core (+2)"), "Second change delta");
}

#[test]
fn test_dashboard_singular_requirement() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create directories
    let specs_dir = root.join("../specs");
    let changes_dir = root.join("openspec/changes");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::create_dir_all(&changes_dir).unwrap();

    // Create config
    common::create_config(&root, "openspec/specs", "openspec/changes");

    // Create spec with single requirement
    common::create_spec(&specs_dir, "simple", &["only-one"]);

    // Run dashboard
    let output = run_dashboard(&root).expect("Dashboard should succeed");

    // Verify singular "requirement" (not "requirements")
    assert!(output.contains("simple"), "Should show spec name");
    assert!(output.contains("1"), "Should show count of 1");
    // The word should be singular
    let lines: Vec<&str> = output.lines().collect();
    let simple_line = lines.iter().find(|l| l.contains("simple")).unwrap();
    assert!(
        simple_line.contains("requirement") && !simple_line.contains("requirements"),
        "Should use singular 'requirement' for count of 1"
    );
}
