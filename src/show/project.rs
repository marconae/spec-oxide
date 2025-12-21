//! Project information display module for Spec Oxide.
//!
//! Provides project overview including version tracking, path information,
//! and configuration summary.

use std::path::Path;

use crate::config::Config;
use crate::core::version_lock::{
    compare_versions, current_version, VersionComparison, VersionLock, VERSION_LOCK_FILENAME,
};

use super::{dim, green, red, yellow};

/// Version status for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionStatus {
    /// Versions match or binary is newer - all good.
    Ok,
    /// Binary is older than project version - warning.
    Warning,
    /// Version lock file is missing.
    Unknown,
}

/// Project information gathered from the filesystem.
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Absolute path to the project root.
    pub project_root: String,
    /// Version from the lock file (None if missing).
    pub initialized_version: Option<String>,
    /// Current binary version.
    pub binary_version: String,
    /// Version compatibility status.
    pub version_status: VersionStatus,
    /// Path to specs folder.
    pub spec_folder: String,
    /// Number of rules (system + custom).
    pub rules_count: usize,
}

/// Gather project information from the config and filesystem.
///
/// # Arguments
/// * `project_root` - Absolute path to the project root
/// * `config` - The Spox configuration
///
/// # Returns
/// `ProjectInfo` containing all gathered information.
pub fn gather_project_info(project_root: &Path, config: &Config) -> ProjectInfo {
    let spox_dir = project_root.join(".spox");
    let lock_path = spox_dir.join(VERSION_LOCK_FILENAME);

    let binary_version = current_version().to_string();

    // Try to load version lock
    let (initialized_version, version_status) = match VersionLock::load(&lock_path) {
        Ok(lock) => {
            let lock_ver = lock.last_version().to_string();
            let status = match compare_versions(&binary_version, &lock_ver) {
                Ok(VersionComparison::Equal) => VersionStatus::Ok,
                Ok(VersionComparison::BinaryNewer { .. }) => VersionStatus::Ok,
                Ok(VersionComparison::BinaryOlder) => VersionStatus::Warning,
                Err(_) => VersionStatus::Unknown,
            };
            (Some(lock_ver), status)
        }
        Err(_) => (None, VersionStatus::Unknown),
    };

    // Count rules
    let rules_count = config.rules.system.len() + config.rules.custom.len();

    ProjectInfo {
        project_root: project_root.display().to_string(),
        initialized_version,
        binary_version,
        version_status,
        spec_folder: config.spec_folder().to_string(),
        rules_count,
    }
}

/// Format project information for display.
///
/// # Arguments
/// * `info` - The project information to format
///
/// # Returns
/// A formatted string ready for terminal display.
pub fn format_project_info(info: &ProjectInfo) -> String {
    let mut output = String::new();

    // Project section header
    output.push_str(&format!("{}\n", yellow("Project:")));

    // Project root
    output.push_str(&format!("  {:<16} {}\n", "Root:", info.project_root));

    // Initialized version
    let init_version_display = match &info.initialized_version {
        Some(v) => v.clone(),
        None => dim("unknown").to_string(),
    };
    output.push_str(&format!(
        "  {:<16} {}\n",
        "Initialized:", init_version_display
    ));

    // Binary version
    output.push_str(&format!("  {:<16} {}\n", "Binary:", info.binary_version));

    // Status
    let status_display = match info.version_status {
        VersionStatus::Ok => green("OK"),
        VersionStatus::Warning => {
            format!(
                "{} {}",
                red("WARNING"),
                dim("(binary older than project, consider upgrading)")
            )
        }
        VersionStatus::Unknown => {
            format!(
                "{} {}",
                yellow("unknown"),
                dim("(run 'spox init' to create version tracking)")
            )
        }
    };
    output.push_str(&format!("  {:<16} {}\n", "Status:", status_display));

    output.push('\n');

    // Config section header
    output.push_str(&format!("{}\n", yellow("Config:")));

    // Specs folder
    output.push_str(&format!("  {:<16} {}\n", "Specs folder:", info.spec_folder));

    // Rules count
    output.push_str(&format!("  {:<16} {}\n", "Rules:", info.rules_count));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Unit Tests for ProjectInfo ====================

    #[test]
    fn test_version_status_equality() {
        assert_eq!(VersionStatus::Ok, VersionStatus::Ok);
        assert_eq!(VersionStatus::Warning, VersionStatus::Warning);
        assert_eq!(VersionStatus::Unknown, VersionStatus::Unknown);
        assert_ne!(VersionStatus::Ok, VersionStatus::Warning);
        assert_ne!(VersionStatus::Ok, VersionStatus::Unknown);
    }

    #[test]
    fn test_format_project_info_ok_status() {
        let info = ProjectInfo {
            project_root: "/path/to/project".to_string(),
            initialized_version: Some("0.4.0".to_string()),
            binary_version: "0.4.0".to_string(),
            version_status: VersionStatus::Ok,
            spec_folder: "specs/".to_string(),
            rules_count: 5,
        };

        let output = format_project_info(&info);

        assert!(output.contains("Project:"));
        assert!(output.contains("Root:"));
        assert!(output.contains("/path/to/project"));
        assert!(output.contains("Initialized:"));
        assert!(output.contains("0.4.0"));
        assert!(output.contains("Binary:"));
        assert!(output.contains("Status:"));
        assert!(output.contains("OK"));
        assert!(output.contains("Config:"));
        assert!(output.contains("Specs folder:"));
        assert!(output.contains("specs/"));
        assert!(output.contains("Rules:"));
        assert!(output.contains("5"));
    }

    #[test]
    fn test_format_project_info_warning_status() {
        let info = ProjectInfo {
            project_root: "/path/to/project".to_string(),
            initialized_version: Some("0.5.0".to_string()),
            binary_version: "0.4.0".to_string(),
            version_status: VersionStatus::Warning,
            spec_folder: "specs/".to_string(),
            rules_count: 3,
        };

        let output = format_project_info(&info);

        assert!(output.contains("WARNING"));
        assert!(output.contains("upgrading"));
    }

    #[test]
    fn test_format_project_info_unknown_status() {
        let info = ProjectInfo {
            project_root: "/path/to/project".to_string(),
            initialized_version: None,
            binary_version: "0.4.0".to_string(),
            version_status: VersionStatus::Unknown,
            spec_folder: "specs/".to_string(),
            rules_count: 2,
        };

        let output = format_project_info(&info);

        assert!(output.contains("unknown"));
        assert!(output.contains("spox init"));
    }

    #[test]
    fn test_format_project_info_no_initialized_version() {
        let info = ProjectInfo {
            project_root: "/path/to/project".to_string(),
            initialized_version: None,
            binary_version: "0.4.0".to_string(),
            version_status: VersionStatus::Unknown,
            spec_folder: "specs/".to_string(),
            rules_count: 0,
        };

        let output = format_project_info(&info);

        // Should show "unknown" for initialized version when None
        assert!(output.contains("Initialized:"));
        // The dim() color function wraps "unknown"
        assert!(output.contains("unknown"));
    }

    #[test]
    fn test_format_project_info_multiple_rules() {
        let info = ProjectInfo {
            project_root: "/my/project".to_string(),
            initialized_version: Some("1.0.0".to_string()),
            binary_version: "1.0.0".to_string(),
            version_status: VersionStatus::Ok,
            spec_folder: "my-specs/".to_string(),
            rules_count: 12,
        };

        let output = format_project_info(&info);

        assert!(output.contains("12"));
        assert!(output.contains("my-specs/"));
    }

    #[test]
    fn test_project_info_clone() {
        let info = ProjectInfo {
            project_root: "/test".to_string(),
            initialized_version: Some("1.0.0".to_string()),
            binary_version: "1.0.0".to_string(),
            version_status: VersionStatus::Ok,
            spec_folder: "specs/".to_string(),
            rules_count: 3,
        };

        let cloned = info.clone();
        assert_eq!(info.project_root, cloned.project_root);
        assert_eq!(info.initialized_version, cloned.initialized_version);
        assert_eq!(info.binary_version, cloned.binary_version);
        assert_eq!(info.version_status, cloned.version_status);
        assert_eq!(info.spec_folder, cloned.spec_folder);
        assert_eq!(info.rules_count, cloned.rules_count);
    }

    // ==================== Integration Tests with filesystem ====================

    #[test]
    fn test_gather_project_info_with_lock_file() {
        use std::fs;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Create config file
        let config_path = spox_dir.join("config.toml");
        fs::write(
            &config_path,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global", "coding"]
custom = ["my-rule"]
"#,
        )
        .unwrap();

        // Create version lock file
        let lock_path = spox_dir.join(VERSION_LOCK_FILENAME);
        let lock = VersionLock::new(current_version());
        lock.save(&lock_path).unwrap();

        // Load config
        let config = Config::load(&config_path).unwrap();

        // Gather project info
        let info = gather_project_info(temp.path(), &config);

        assert_eq!(info.project_root, temp.path().display().to_string());
        assert_eq!(
            info.initialized_version,
            Some(current_version().to_string())
        );
        assert_eq!(info.binary_version, current_version());
        assert_eq!(info.version_status, VersionStatus::Ok);
        assert_eq!(info.spec_folder, "specs/");
        assert_eq!(info.rules_count, 4); // 3 system + 1 custom
    }

    #[test]
    fn test_gather_project_info_without_lock_file() {
        use std::fs;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Create config file only (no lock file)
        let config_path = spox_dir.join("config.toml");
        fs::write(
            &config_path,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#,
        )
        .unwrap();

        // Load config
        let config = Config::load(&config_path).unwrap();

        // Gather project info
        let info = gather_project_info(temp.path(), &config);

        assert!(info.initialized_version.is_none());
        assert_eq!(info.version_status, VersionStatus::Unknown);
        assert_eq!(info.rules_count, 1);
    }

    #[test]
    fn test_gather_project_info_binary_older_than_lock() {
        use std::fs;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Create config file
        let config_path = spox_dir.join("config.toml");
        fs::write(
            &config_path,
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#,
        )
        .unwrap();

        // Create version lock file with a future version
        let lock_path = spox_dir.join(VERSION_LOCK_FILENAME);
        let lock = VersionLock::new("99.99.99"); // Future version
        lock.save(&lock_path).unwrap();

        // Load config
        let config = Config::load(&config_path).unwrap();

        // Gather project info
        let info = gather_project_info(temp.path(), &config);

        assert_eq!(info.initialized_version, Some("99.99.99".to_string()));
        assert_eq!(info.version_status, VersionStatus::Warning);
    }
}
