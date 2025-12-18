//! Index command handler for building the semantic search index.

use std::path::Path;

use crate::core::index::rebuild_index;
use crate::error::{Error, Result};

/// Run the index command to build the semantic search index.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - Specs cannot be parsed
/// - Index cannot be built or saved
pub fn run() -> Result<()> {
    eprintln!("Building search index...");

    // Use the core rebuild_index function which handles all the logic
    let project_root = Path::new(".");
    let specs_indexed = rebuild_index(project_root).map_err(|e| Error::Other(format!("{}", e)))?;

    if specs_indexed == 0 {
        eprintln!("Warning: No spec files found");
    }

    eprintln!("Index built successfully with {} specs", specs_indexed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    /// Index file path (matches core::index::INDEX_PATH)
    const INDEX_PATH: &str = ".spox/search_index.bin";

    fn create_test_environment() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create .spox directory
        let spox_dir = temp_dir.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Create config file
        let config_content = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global"]
"#;
        fs::write(spox_dir.join("config.toml"), config_content).unwrap();

        // Create specs directory
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        temp_dir
    }

    fn create_test_spec(dir: &Path, name: &str) {
        let spec_dir = dir.join("specs").join(name);
        fs::create_dir_all(&spec_dir).unwrap();

        let spec_content = format!(
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
        fs::write(spec_dir.join("spec.md"), spec_content).unwrap();
    }

    // Integration tests that require the embedding model are ignored by default
    // Run with: cargo test index_cmd -- --ignored

    #[test]
    #[ignore]
    #[serial]
    fn test_run_with_specs() {
        let temp_dir = create_test_environment();
        create_test_spec(temp_dir.path(), "auth");
        create_test_spec(temp_dir.path(), "payments");

        // Change to temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = run();

        // Restore original directory (ignore errors in parallel test execution)
        let _ = std::env::set_current_dir(original_dir);

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);

        // Verify index file was created
        let index_path = temp_dir.path().join(INDEX_PATH);
        assert!(index_path.exists(), "Index file should exist");
    }

    #[test]
    #[ignore]
    #[serial]
    fn test_run_with_no_specs() {
        let temp_dir = create_test_environment();

        // Change to temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = run();

        // Restore original directory (ignore errors in parallel test execution)
        let _ = std::env::set_current_dir(original_dir);

        // Should succeed with warning (empty index created)
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);

        // Verify index file was created
        let index_path = temp_dir.path().join(INDEX_PATH);
        assert!(
            index_path.exists(),
            "Index file should exist even with no specs"
        );
    }
}
