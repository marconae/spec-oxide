use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

/// Configuration for Spec Oxide.
///
/// Loaded from `.spox/config.toml`.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Path to the specs folder (e.g., "specs/").
    pub spec_folder: String,
    /// Path to the changes folder (e.g., "specs/_changes").
    pub changes_folder: String,
    /// Path to the archive folder (e.g., "specs/_archive").
    pub archive_folder: String,
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
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.spec_folder, "specs/");
        assert_eq!(config.changes_folder, "specs/_changes");
        assert_eq!(config.archive_folder, "specs/_archive");
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
}
