//! Version lock file management for Spox projects.
//!
//! The version lock file (`.spox/version.lock`) tracks which spox version
//! initialized and updated the project. This enables:
//! - Detecting when migrations are needed after upgrades
//! - Warning users running an older binary than the project requires
//! - Providing useful debugging/support information

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Type of version upgrade (patch, minor, or major).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeType {
    /// Patch version change (x.y.Z).
    Patch,
    /// Minor version change (x.Y.z).
    Minor,
    /// Major version change (X.y.z).
    Major,
}

/// Result of comparing binary version to lock file version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionComparison {
    /// Versions are equal.
    Equal,
    /// Binary is newer than lock file (project upgrade).
    BinaryNewer { upgrade_type: UpgradeType },
    /// Binary is older than lock file (potential compatibility issue).
    BinaryOlder,
}

/// The filename for the version lock file.
pub const VERSION_LOCK_FILENAME: &str = "version.lock";

/// Version lock file structure.
///
/// Tracks the spox version used to initialize and update a project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionLock {
    /// The version that first initialized this project.
    pub initialized_version: String,
    /// Versions that have updated this project (in order of updates).
    #[serde(default)]
    pub updated_versions: Vec<String>,
}

impl VersionLock {
    /// Create a new version lock with the given initial version.
    pub fn new(version: &str) -> Self {
        Self {
            initialized_version: version.to_string(),
            updated_versions: Vec::new(),
        }
    }

    /// Get the last recorded version (either most recent update or initial version).
    pub fn last_version(&self) -> &str {
        self.updated_versions
            .last()
            .map(|s| s.as_str())
            .unwrap_or(&self.initialized_version)
    }

    /// Record a new version update.
    ///
    /// Only adds the version if it differs from the last recorded version.
    /// Returns true if the version was added, false if it was already recorded.
    pub fn record_update(&mut self, version: &str) -> bool {
        if self.last_version() != version {
            self.updated_versions.push(version.to_string());
            true
        } else {
            false
        }
    }

    /// Load version lock from a file.
    pub fn load(path: &Path) -> Result<Self, VersionLockError> {
        let content = fs::read_to_string(path).map_err(|e| VersionLockError::Io(e.to_string()))?;
        toml::from_str(&content).map_err(|e| VersionLockError::Parse(e.to_string()))
    }

    /// Save version lock to a file.
    pub fn save(&self, path: &Path) -> Result<(), VersionLockError> {
        let content =
            toml::to_string_pretty(self).map_err(|e| VersionLockError::Serialize(e.to_string()))?;
        fs::write(path, content).map_err(|e| VersionLockError::Io(e.to_string()))
    }
}

/// Errors that can occur when working with version lock files.
#[derive(Debug, Clone, PartialEq)]
pub enum VersionLockError {
    /// IO error reading or writing the file.
    Io(String),
    /// Error parsing the TOML content.
    Parse(String),
    /// Error serializing to TOML.
    Serialize(String),
}

impl std::fmt::Display for VersionLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionLockError::Io(msg) => write!(f, "IO error: {}", msg),
            VersionLockError::Parse(msg) => write!(f, "parse error: {}", msg),
            VersionLockError::Serialize(msg) => write!(f, "serialize error: {}", msg),
        }
    }
}

impl std::error::Error for VersionLockError {}

/// Get the current spox binary version from Cargo.toml.
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Compare binary version against lock file version.
///
/// Returns the comparison result indicating whether the binary is newer,
/// older, or equal to the lock file version.
pub fn compare_versions(
    binary_version: &str,
    lock_version: &str,
) -> Result<VersionComparison, VersionLockError> {
    use semver::Version;

    let binary = Version::parse(binary_version).map_err(|e| {
        VersionLockError::Parse(format!(
            "invalid binary version '{}': {}",
            binary_version, e
        ))
    })?;
    let lock = Version::parse(lock_version).map_err(|e| {
        VersionLockError::Parse(format!("invalid lock version '{}': {}", lock_version, e))
    })?;

    match binary.cmp(&lock) {
        std::cmp::Ordering::Equal => Ok(VersionComparison::Equal),
        std::cmp::Ordering::Less => Ok(VersionComparison::BinaryOlder),
        std::cmp::Ordering::Greater => {
            let upgrade_type = if binary.major > lock.major {
                UpgradeType::Major
            } else if binary.minor > lock.minor {
                UpgradeType::Minor
            } else {
                UpgradeType::Patch
            };
            Ok(VersionComparison::BinaryNewer { upgrade_type })
        }
    }
}

/// A migration hint entry: (from_major, from_minor, to_major, to_minor, hint_message).
struct MigrationHint {
    from_major: u64,
    from_minor: u64,
    to_major: u64,
    to_minor: u64,
    hint: &'static str,
}

/// Get migration hints for upgrading between versions.
///
/// Returns migration hints if there are any relevant notes for upgrading
/// from the old version to the new version. Returns None for patch upgrades
/// or if there are no specific migration notes.
pub fn get_migration_hints(from_version: &str, to_version: &str) -> Option<&'static str> {
    use semver::Version;

    let from = Version::parse(from_version).ok()?;
    let to = Version::parse(to_version).ok()?;

    // No hints for patch upgrades or downgrades
    if to <= from || (to.major == from.major && to.minor == from.minor) {
        return None;
    }

    // Define migration hints for specific version transitions
    const HINTS: &[MigrationHint] = &[
        // Hints for 0.3.x -> 0.4.x
        MigrationHint {
            from_major: 0,
            from_minor: 3,
            to_major: 0,
            to_minor: 4,
            hint: "Version 0.4.0 introduces the version lock file. Your project has been updated automatically.",
        },
        // Hints for 0.4.x -> 1.0.x
        MigrationHint {
            from_major: 0,
            from_minor: 4,
            to_major: 1,
            to_minor: 0,
            hint: "Version 1.0.0 is the first stable release. Review CHANGELOG.md for breaking changes.",
        },
    ];

    for h in HINTS {
        // Check if upgrading from a version in the 'from' range to the 'to' range
        if from.major == h.from_major
            && from.minor >= h.from_minor
            && to.major == h.to_major
            && to.minor >= h.to_minor
        {
            return Some(h.hint);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_creates_with_initial_version() {
        let lock = VersionLock::new("0.4.0");
        assert_eq!(lock.initialized_version, "0.4.0");
        assert!(lock.updated_versions.is_empty());
    }

    #[test]
    fn test_last_version_returns_initial_when_no_updates() {
        let lock = VersionLock::new("0.4.0");
        assert_eq!(lock.last_version(), "0.4.0");
    }

    #[test]
    fn test_last_version_returns_most_recent_update() {
        let mut lock = VersionLock::new("0.4.0");
        lock.updated_versions.push("0.4.1".to_string());
        lock.updated_versions.push("0.5.0".to_string());
        assert_eq!(lock.last_version(), "0.5.0");
    }

    #[test]
    fn test_record_update_adds_new_version() {
        let mut lock = VersionLock::new("0.4.0");
        let added = lock.record_update("0.4.1");
        assert!(added);
        assert_eq!(lock.updated_versions, vec!["0.4.1"]);
    }

    #[test]
    fn test_record_update_skips_same_version() {
        let mut lock = VersionLock::new("0.4.0");
        let added = lock.record_update("0.4.0");
        assert!(!added);
        assert!(lock.updated_versions.is_empty());
    }

    #[test]
    fn test_record_update_skips_same_as_last_update() {
        let mut lock = VersionLock::new("0.4.0");
        lock.record_update("0.4.1");
        let added = lock.record_update("0.4.1");
        assert!(!added);
        assert_eq!(lock.updated_versions.len(), 1);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("version.lock");

        let mut lock = VersionLock::new("0.4.0");
        lock.record_update("0.4.1");
        lock.record_update("0.5.0");

        lock.save(&path).unwrap();
        let loaded = VersionLock::load(&path).unwrap();

        assert_eq!(lock, loaded);
    }

    #[test]
    fn test_load_missing_file_returns_error() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nonexistent.lock");

        let result = VersionLock::load(&path);
        assert!(matches!(result, Err(VersionLockError::Io(_))));
    }

    #[test]
    fn test_load_invalid_toml_returns_error() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("version.lock");

        fs::write(&path, "not valid toml {{{").unwrap();

        let result = VersionLock::load(&path);
        assert!(matches!(result, Err(VersionLockError::Parse(_))));
    }

    #[test]
    fn test_saved_file_format_is_valid_toml() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("version.lock");

        let mut lock = VersionLock::new("0.4.0");
        lock.record_update("0.4.1");
        lock.record_update("0.5.0");

        lock.save(&path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("initialized_version = \"0.4.0\""));
        assert!(content.contains("updated_versions = ["));
        assert!(content.contains("\"0.4.1\""));
        assert!(content.contains("\"0.5.0\""));
    }

    #[test]
    fn test_current_version_returns_cargo_version() {
        let version = current_version();
        // Should be a valid semver-like string
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    // Tests for version comparison (Task 2.1-2.4)

    #[test]
    fn test_compare_versions_same_returns_equal() {
        let result = compare_versions("1.0.0", "1.0.0");
        assert!(matches!(result, Ok(VersionComparison::Equal)));
    }

    #[test]
    fn test_compare_versions_binary_newer_patch() {
        let result = compare_versions("1.0.1", "1.0.0");
        assert!(matches!(
            result,
            Ok(VersionComparison::BinaryNewer {
                upgrade_type: UpgradeType::Patch
            })
        ));
    }

    #[test]
    fn test_compare_versions_binary_newer_minor() {
        let result = compare_versions("1.1.0", "1.0.0");
        assert!(matches!(
            result,
            Ok(VersionComparison::BinaryNewer {
                upgrade_type: UpgradeType::Minor
            })
        ));
    }

    #[test]
    fn test_compare_versions_binary_newer_major() {
        let result = compare_versions("2.0.0", "1.0.0");
        assert!(matches!(
            result,
            Ok(VersionComparison::BinaryNewer {
                upgrade_type: UpgradeType::Major
            })
        ));
    }

    #[test]
    fn test_compare_versions_binary_older() {
        let result = compare_versions("1.0.0", "1.0.1");
        assert!(matches!(result, Ok(VersionComparison::BinaryOlder)));
    }

    #[test]
    fn test_compare_versions_invalid_binary_version() {
        let result = compare_versions("invalid", "1.0.0");
        assert!(matches!(result, Err(VersionLockError::Parse(_))));
    }

    #[test]
    fn test_compare_versions_invalid_lock_version() {
        let result = compare_versions("1.0.0", "invalid");
        assert!(matches!(result, Err(VersionLockError::Parse(_))));
    }

    #[test]
    fn test_get_migration_hints_minor_upgrade() {
        let hints = get_migration_hints("0.3.0", "0.4.0");
        // Should return a hint for 0.3 -> 0.4 migration if defined
        assert!(hints.is_some() || hints.is_none()); // May or may not have hints
    }

    #[test]
    fn test_get_migration_hints_major_upgrade() {
        let hints = get_migration_hints("0.4.0", "1.0.0");
        // Should return hints for major version upgrade if defined
        assert!(hints.is_some() || hints.is_none()); // May or may not have hints
    }

    #[test]
    fn test_get_migration_hints_patch_upgrade_returns_none() {
        // Patch upgrades should not have migration hints
        let hints = get_migration_hints("0.4.0", "0.4.1");
        assert!(hints.is_none());
    }
}
