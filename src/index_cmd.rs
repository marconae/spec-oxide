//! Index command handler for building the semantic search index.

use std::path::Path;

use crate::config::Config;
use crate::core::index::{build_index, save_index};
use crate::core::spec::parse_all_specs;
use crate::error::{Error, Result};

/// Default index file path relative to project root.
const INDEX_PATH: &str = ".spox/search_index.bin";

/// Run the index command to build the semantic search index.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - Specs cannot be parsed
/// - Index cannot be built or saved
pub fn run() -> Result<()> {
    // Load configuration
    let config = Config::load(Path::new(".spox/config.toml"))?;

    // Get spec folder from config
    let spec_folder = Path::new(config.spec_folder());

    // Parse all specs
    eprintln!("Parsing specs from {}...", spec_folder.display());
    let specs = parse_all_specs(spec_folder).map_err(|e| Error::Other(e.to_string()))?;

    if specs.is_empty() {
        eprintln!("Warning: No spec files found");
    } else {
        eprintln!("Found {} specs", specs.len());
    }

    // Build the index
    eprintln!("Generating embeddings...");
    let index = build_index(&specs).map_err(|e| Error::Other(e.to_string()))?;

    // Save the index
    let index_path = Path::new(INDEX_PATH);
    eprintln!("Saving index to {}...", index_path.display());
    save_index(&index, index_path).map_err(|e| Error::Other(e.to_string()))?;

    eprintln!("Index built successfully with {} specs", index.specs.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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

    #[test]
    fn test_index_path_constant() {
        assert_eq!(INDEX_PATH, ".spox/search_index.bin");
    }

    // Integration tests that require the embedding model are ignored by default
    // Run with: cargo test index_cmd -- --ignored

    #[test]
    #[ignore]
    fn test_run_with_specs() {
        let temp_dir = create_test_environment();
        create_test_spec(temp_dir.path(), "auth");
        create_test_spec(temp_dir.path(), "payments");

        // Change to temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = run();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);

        // Verify index file was created
        let index_path = temp_dir.path().join(INDEX_PATH);
        assert!(index_path.exists(), "Index file should exist");
    }

    #[test]
    #[ignore]
    fn test_run_with_no_specs() {
        let temp_dir = create_test_environment();

        // Change to temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = run();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

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
