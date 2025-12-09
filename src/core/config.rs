//! Configuration management for Spec Oxide.
//!
//! Handles loading and validating configuration from `.spox/config.toml`.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::error::{Error, Result};

/// Valid system template names.
const VALID_SYSTEM_TEMPLATES: &[&str] = &[
    "mcp", "global", "coding", "testing", "backend", "frontend", "vcs",
];

/// Paths configuration section.
#[derive(Debug, Deserialize)]
pub struct PathsConfig {
    /// Path to the specs folder (e.g., "specs/").
    pub spec_folder: String,
    /// Path to the changes folder (e.g., "specs/_changes").
    pub changes_folder: String,
    /// Path to the archive folder (e.g., "specs/_archive").
    pub archive_folder: String,
}

/// Rules configuration section.
#[derive(Debug, Deserialize)]
pub struct RulesConfig {
    /// System templates to include (e.g., ["mcp", "global", "coding"]).
    pub system: Vec<String>,
    /// Custom rules to include (defaults to empty).
    #[serde(default)]
    pub custom: Vec<String>,
}

/// Configuration for Spec Oxide.
///
/// Loaded from `.spox/config.toml`.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Paths configuration.
    pub paths: PathsConfig,
    /// Rules configuration.
    pub rules: RulesConfig,
}

impl Config {
    /// Load configuration from a TOML file at the given path.
    ///
    /// # Errors
    ///
    /// Returns `Error::ConfigNotFound` if the file does not exist.
    /// Returns `Error::ConfigParse` if the file cannot be parsed as TOML.
    /// Returns `Error::ConfigMissingField` if a required field is missing.
    pub fn load(path: &Path) -> Result<Config> {
        if !path.exists() {
            return Err(Error::ConfigNotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| Error::ConfigParse(format!("failed to read file: {}", e)))?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            let msg = e.message();
            // Check for missing field errors
            if msg.contains("missing field") {
                // Extract the field name from the error message
                if let Some(field) = msg.split('`').nth(1) {
                    return Error::ConfigMissingField(field.to_string());
                }
            }
            Error::ConfigParse(msg.to_string())
        })?;

        Ok(config)
    }

    /// Validate the configuration.
    ///
    /// # Errors
    ///
    /// Returns `Error::ConfigParse` if validation fails:
    /// - `rules.system` is empty
    /// - `rules.system` contains an invalid template name
    pub fn validate(&self) -> Result<()> {
        // Check that system is not empty
        if self.rules.system.is_empty() {
            return Err(Error::ConfigParse(
                "rules.system must contain at least one template".to_string(),
            ));
        }

        // Check that all system templates are valid
        for template in &self.rules.system {
            if !VALID_SYSTEM_TEMPLATES.contains(&template.as_str()) {
                return Err(Error::ConfigParse(format!(
                    "invalid system template '{}', valid templates are: {}",
                    template,
                    VALID_SYSTEM_TEMPLATES.join(", ")
                )));
            }
        }

        Ok(())
    }

    /// Get the spec folder path.
    pub fn spec_folder(&self) -> &str {
        &self.paths.spec_folder
    }

    /// Get the changes folder path.
    pub fn changes_folder(&self) -> &str {
        &self.paths.changes_folder
    }

    /// Get the archive folder path.
    pub fn archive_folder(&self) -> &str {
        &self.paths.archive_folder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global"]
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.spec_folder(), "specs/");
        assert_eq!(config.changes_folder(), "specs/_changes");
        assert_eq!(config.archive_folder(), "specs/_archive");
    }

    #[test]
    fn test_load_config_not_found() {
        let path = Path::new("/nonexistent/config.toml");
        let result = Config::load(path);
        assert!(matches!(result, Err(Error::ConfigNotFound(_))));
    }

    #[test]
    fn test_load_config_missing_field() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[paths]
spec_folder = "specs/"
"#
        )
        .unwrap();

        let result = Config::load(file.path());
        assert!(matches!(result, Err(Error::ConfigMissingField(_))));
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "this is not valid toml {{{{").unwrap();

        let result = Config::load(file.path());
        assert!(matches!(result, Err(Error::ConfigParse(_))));
    }

    // ==================== Tests for new [paths] and [rules] section structure ====================

    #[test]
    fn test_load_config_with_paths_and_rules_sections() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global", "coding"]
custom = ["my-custom-rule"]
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.spec_folder(), "specs/");
        assert_eq!(config.changes_folder(), "specs/_changes");
        assert_eq!(config.archive_folder(), "specs/_archive");
        assert_eq!(config.paths.spec_folder, "specs/");
        assert_eq!(config.rules.system, vec!["mcp", "global", "coding"]);
        assert_eq!(config.rules.custom, vec!["my-custom-rule"]);
    }

    #[test]
    fn test_load_config_custom_defaults_to_empty() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert!(config.rules.custom.is_empty());
    }

    // ==================== Tests for validation errors ====================

    #[test]
    fn test_validate_empty_system_array() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = []
custom = []
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("system"));
    }

    #[test]
    fn test_validate_invalid_template_name() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "invalid-template", "coding"]
custom = []
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid-template"));
    }

    #[test]
    fn test_validate_all_valid_templates() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global", "coding", "testing", "backend", "frontend", "vcs"]
custom = []
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        let result = config.validate();
        assert!(result.is_ok());
    }
}
