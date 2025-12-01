use std::path::Path;

use crate::config::Config;
use crate::error::Result;

/// Run the `config show` command.
///
/// Loads and displays the current project configuration from `.spox/config.toml`.
///
/// # Errors
///
/// Returns an error if the configuration file does not exist or cannot be parsed.
pub fn run_show() -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    println!("Spec Oxide Configuration");
    println!("  spec_folder: {}", config.spec_folder);
    println!("  changes_folder: {}", config.changes_folder);
    println!("  archive_folder: {}", config.archive_folder);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_run_show_config_not_found() {
        // Create a temporary directory and change to it
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run the command - should fail since no config exists
        let result = run_show();
        assert!(result.is_err());

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_run_show_success() {
        // Create a temporary directory with .spox/config.toml
        let temp_dir = TempDir::new().unwrap();
        let spox_dir = temp_dir.path().join(".spox");
        fs::create_dir(&spox_dir).unwrap();

        let config_content = r#"
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"
"#;
        fs::write(spox_dir.join("config.toml"), config_content).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run the command - should succeed
        let result = run_show();
        assert!(result.is_ok());

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }
}
