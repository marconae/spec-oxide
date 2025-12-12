use std::path::Path;

use crate::config::Config;
use crate::error::Result;

/// Run the `config show` command.
///
/// Loads and displays the current project configuration from `.spox/config.toml`.
/// If `paths_only` is true, only displays the `[paths]` section.
///
/// # Errors
///
/// Returns an error if the configuration file does not exist or cannot be parsed.
pub fn run_show(paths_only: bool) -> Result<()> {
    let config_path = Path::new(".spox/config.toml");
    let config = Config::load(config_path)?;

    // Validate config and print helpful error messages
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed:");
        eprintln!("  {}", e);
        eprintln!();
        eprintln!("Hint: Check your .spox/config.toml file and ensure:");
        eprintln!("  - rules.system is not empty");
        eprintln!("  - All template names are valid (mcp, global, coding, testing, backend, frontend, vcs)");
        return Err(e);
    }

    if paths_only {
        // Display paths section only
        println!("[paths]");
        println!("spec_folder = \"{}\"", config.spec_folder());
        println!("changes_folder = \"{}\"", config.changes_folder());
        println!("archive_folder = \"{}\"", config.archive_folder());
    } else {
        // Display full configuration
        println!("Spec Oxide Configuration");
        println!();
        println!("[paths]");
        println!("  spec_folder = \"{}\"", config.spec_folder());
        println!("  changes_folder = \"{}\"", config.changes_folder());
        println!("  archive_folder = \"{}\"", config.archive_folder());
        println!();
        println!("[rules]");
        println!(
            "  system = [{}]",
            config
                .rules
                .system
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        );
        if config.rules.custom.is_empty() {
            println!("  custom = []");
        } else {
            println!(
                "  custom = [{}]",
                config
                    .rules
                    .custom
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

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
        let result = run_show(false);

        // Restore original directory before assertions to avoid leaving cwd in temp dir
        let restore_result = env::set_current_dir(&original_dir);

        assert!(result.is_err());
        let _ = restore_result; // Ignore restore errors in parallel test execution
    }

    #[test]
    fn test_run_show_success() {
        // Create a temporary directory with .spox/config.toml
        let temp_dir = TempDir::new().unwrap();
        let spox_dir = temp_dir.path().join(".spox");
        fs::create_dir(&spox_dir).unwrap();

        let config_content = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#;
        fs::write(spox_dir.join("config.toml"), config_content).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run the command - should succeed
        let result = run_show(false);

        // Restore original directory before assertions to avoid leaving cwd in temp dir
        let restore_result = env::set_current_dir(&original_dir);

        assert!(result.is_ok());
        let _ = restore_result; // Ignore restore errors in parallel test execution
    }

    #[test]
    fn test_run_show_paths_only() {
        // Create a temporary directory with .spox/config.toml
        let temp_dir = TempDir::new().unwrap();
        let spox_dir = temp_dir.path().join(".spox");
        fs::create_dir(&spox_dir).unwrap();

        let config_content = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global"]
custom = ["my-rules.md"]
"#;
        fs::write(spox_dir.join("config.toml"), config_content).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run the command with paths_only=true - should succeed
        let result = run_show(true);

        // Restore original directory before assertions to avoid leaving cwd in temp dir
        let restore_result = env::set_current_dir(&original_dir);

        assert!(result.is_ok());
        let _ = restore_result; // Ignore restore errors in parallel test execution
    }

    #[test]
    fn test_run_show_full_config_with_rules() {
        // Create a temporary directory with .spox/config.toml
        let temp_dir = TempDir::new().unwrap();
        let spox_dir = temp_dir.path().join(".spox");
        fs::create_dir(&spox_dir).unwrap();

        let config_content = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global", "coding"]
custom = ["project-rules.md", "team-rules.md"]
"#;
        fs::write(spox_dir.join("config.toml"), config_content).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run the command - should succeed
        let result = run_show(false);

        // Restore original directory before assertions to avoid leaving cwd in temp dir
        let restore_result = env::set_current_dir(&original_dir);

        assert!(result.is_ok());
        let _ = restore_result; // Ignore restore errors in parallel test execution
    }

    #[test]
    fn test_run_show_validation_error_empty_system() {
        // Create a temporary directory with invalid config
        let temp_dir = TempDir::new().unwrap();
        let spox_dir = temp_dir.path().join(".spox");
        fs::create_dir(&spox_dir).unwrap();

        let config_content = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = []
"#;
        fs::write(spox_dir.join("config.toml"), config_content).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run the command - should fail due to validation
        let result = run_show(false);

        // Restore original directory before any assertions to avoid leaving cwd in temp dir
        let restore_result = env::set_current_dir(&original_dir);

        assert!(result.is_err());
        let _ = restore_result; // Ignore restore errors in parallel test execution
    }

    #[test]
    fn test_run_show_validation_error_invalid_template() {
        // Create a temporary directory with invalid config
        let temp_dir = TempDir::new().unwrap();
        let spox_dir = temp_dir.path().join(".spox");
        fs::create_dir(&spox_dir).unwrap();

        let config_content = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "invalid-template"]
"#;
        fs::write(spox_dir.join("config.toml"), config_content).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run the command - should fail due to validation
        let result = run_show(false);

        // Restore original directory before any assertions to avoid leaving cwd in temp dir
        let restore_result = env::set_current_dir(&original_dir);

        assert!(result.is_err());
        let _ = restore_result; // Ignore restore errors in parallel test execution
    }
}
