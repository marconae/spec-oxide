//! Init command implementation for bootstrapping a new Spox project.

use crate::config::Config;
use crate::core::version_lock::{
    compare_versions, current_version, get_migration_hints, UpgradeType, VersionComparison,
    VersionLock, VERSION_LOCK_FILENAME,
};
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

// Embedded template files for single-binary distribution

// .spox/ templates
const TEMPLATE_CONFIG_TOML: &str = include_str!("../templates/spox/config.toml");
const TEMPLATE_WORKFLOW_MD: &str = include_str!("../templates/spox/workflow.md");
const TEMPLATE_STANDARDS_BACKEND_MD: &str = include_str!("../templates/spox/standards/backend.md");
const TEMPLATE_STANDARDS_CODING_MD: &str = include_str!("../templates/spox/standards/coding.md");
const TEMPLATE_STANDARDS_FRONTEND_MD: &str =
    include_str!("../templates/spox/standards/frontend.md");
const TEMPLATE_STANDARDS_GLOBAL_MD: &str = include_str!("../templates/spox/standards/global.md");
const TEMPLATE_STANDARDS_TESTING_MD: &str = include_str!("../templates/spox/standards/testing.md");
const TEMPLATE_STANDARDS_VCS_MD: &str = include_str!("../templates/spox/standards/vcs.md");
const TEMPLATE_STANDARDS_MCP_MD: &str = include_str!("../templates/spox/standards/mcp.md");
const TEMPLATE_SETUP_SH: &str = include_str!("../templates/spox/setup.sh");

// .spox/specs/ templates
const TEMPLATE_SPEC_SPEC_MD: &str = include_str!("../templates/specs/spec.md");
const TEMPLATE_SPEC_CHANGE_PROPOSAL_MD: &str =
    include_str!("../templates/specs/change/proposal.md");
const TEMPLATE_SPEC_CHANGE_TASKS_MD: &str = include_str!("../templates/specs/change/tasks.md");
const TEMPLATE_SPEC_CHANGE_DESIGN_MD: &str = include_str!("../templates/specs/change/design.md");
const TEMPLATE_SPEC_CHANGE_SPEC_MD: &str = include_str!("../templates/specs/change/spec.md");
const TEMPLATE_SPEC_CHANGE_VERIFICATION_MD: &str =
    include_str!("../templates/specs/change/verification.md");

// .claude/ templates
const TEMPLATE_AGENT_IMPLEMENTER_MD: &str =
    include_str!("../templates/claude/agents/spox-implementer.md");
const TEMPLATE_AGENT_REVIEWER_MD: &str =
    include_str!("../templates/claude/agents/spox-reviewer.md");
const TEMPLATE_AGENT_VERIFIER_MD: &str =
    include_str!("../templates/claude/agents/spox-verifier.md");
const TEMPLATE_CMD_ARCHIVE_MD: &str = include_str!("../templates/claude/commands/spox/archive.md");
const TEMPLATE_CMD_IMPLEMENT_MD: &str =
    include_str!("../templates/claude/commands/spox/implement.md");
const TEMPLATE_CMD_PROPOSE_MD: &str = include_str!("../templates/claude/commands/spox/propose.md");
const TEMPLATE_CMD_SETUP_MD: &str = include_str!("../templates/claude/commands/spox/setup.md");
const TEMPLATE_CMD_VIBE_MD: &str = include_str!("../templates/claude/commands/spox/vibe.md");

// specs/ templates
const TEMPLATE_MISSION_MD: &str = include_str!("../templates/specs/mission.md");

// CLAUDE.md template
const TEMPLATE_CLAUDE_MD: &str = include_str!("../templates/spox/CLAUDE-template.md");

// .gitignore templates for spox-managed directories
const TEMPLATE_SPOX_GITIGNORE: &str = include_str!("../templates/spox/gitignore");
const TEMPLATE_CLAUDE_GITIGNORE: &str = include_str!("../templates/claude/gitignore");

/// Initialize or update a Spox project at the given path.
///
/// On fresh projects, creates the full structure. On existing projects,
/// updates tooling files while preserving user content (mission.md, specs/).
///
/// Creates the following structure:
/// ```text
/// project/
/// |-- .spox/
/// |   |-- config.toml
/// |   |-- setup.sh
/// |   |-- custom/
/// |   +-- templates/
/// |       |-- spec.md
/// |       +-- change/
/// |           |-- proposal.md
/// |           |-- tasks.md
/// |           |-- design.md
/// |           |-- spec.md
/// |           +-- verification.md
/// |-- .claude/
/// |   |-- CLAUDE.md
/// |   |-- agents/
/// |   |   |-- spox-implementer.md
/// |   |   |-- spox-reviewer.md
/// |   |   +-- spox-verifier.md
/// |   +-- commands/spox/
/// |       |-- archive.md
/// |       |-- implement.md
/// |       |-- propose.md
/// |       |-- setup.md
/// |       +-- vibe.md
/// +-- specs/
///     |-- mission.md
///     |-- _changes/
///     +-- _archive/
/// ```
pub fn run(base_path: &Path) -> Result<()> {
    // Determine if this is an update (for messaging)
    let is_update = base_path.join(".spox").exists();

    // Create .spox/ directory and contents (always overwrites tooling files)
    create_spox_dir(base_path)?;

    // Create .claude/ directory and contents (always overwrites tooling files)
    create_claude_dir(base_path)?;

    // Create specs/ directory structure (preserves user content)
    create_specs_dir(base_path)?;

    // Create or update CLAUDE.md
    write_claude_md(base_path)?;

    // Print success message
    print_success_message(base_path, is_update);

    Ok(())
}

/// Create the .spox/ directory structure.
fn create_spox_dir(base_path: &Path) -> Result<()> {
    let spox_dir = base_path.join(".spox");
    let custom_dir = spox_dir.join("custom");
    let templates_dir = spox_dir.join("templates");
    let templates_change_dir = templates_dir.join("change");

    // Create directories
    create_dir_all(&custom_dir)?;
    create_dir_all(&templates_change_dir)?;

    // Write config file (preserve existing to keep user customizations)
    write_file_if_not_exists(&spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML)?;

    // Write .gitignore for spox-managed files
    append_gitignore_rules(&spox_dir.join(".gitignore"), TEMPLATE_SPOX_GITIGNORE)?;

    // Create or update version lock file
    create_or_update_version_lock(&spox_dir)?;

    // Write spec template files
    write_file(&templates_dir.join("spec.md"), TEMPLATE_SPEC_SPEC_MD)?;
    write_file(
        &templates_change_dir.join("proposal.md"),
        TEMPLATE_SPEC_CHANGE_PROPOSAL_MD,
    )?;
    write_file(
        &templates_change_dir.join("tasks.md"),
        TEMPLATE_SPEC_CHANGE_TASKS_MD,
    )?;
    write_file(
        &templates_change_dir.join("design.md"),
        TEMPLATE_SPEC_CHANGE_DESIGN_MD,
    )?;
    write_file(
        &templates_change_dir.join("spec.md"),
        TEMPLATE_SPEC_CHANGE_SPEC_MD,
    )?;
    write_file(
        &templates_change_dir.join("verification.md"),
        TEMPLATE_SPEC_CHANGE_VERIFICATION_MD,
    )?;

    // Write setup.sh and make it executable (always overwrite - tooling file)
    let setup_sh_path = spox_dir.join("setup.sh");
    write_file(&setup_sh_path, TEMPLATE_SETUP_SH)?;
    make_executable(&setup_sh_path)?;

    Ok(())
}

/// Create the .claude/ directory structure.
/// Always overwrites tooling files to ensure they stay up-to-date.
fn create_claude_dir(base_path: &Path) -> Result<()> {
    let claude_dir = base_path.join(".claude");
    let agents_dir = claude_dir.join("agents");
    let commands_spox_dir = claude_dir.join("commands").join("spox");

    // Create directories
    create_dir_all(&agents_dir)?;
    create_dir_all(&commands_spox_dir)?;

    // Write .gitignore for spox-managed files in .claude/
    append_gitignore_rules(&claude_dir.join(".gitignore"), TEMPLATE_CLAUDE_GITIGNORE)?;

    // Write agent files (flat in agents/, files already have spox- prefix)
    write_file(
        &agents_dir.join("spox-implementer.md"),
        TEMPLATE_AGENT_IMPLEMENTER_MD,
    )?;
    write_file(
        &agents_dir.join("spox-reviewer.md"),
        TEMPLATE_AGENT_REVIEWER_MD,
    )?;
    write_file(
        &agents_dir.join("spox-verifier.md"),
        TEMPLATE_AGENT_VERIFIER_MD,
    )?;

    // Write command files
    write_file(
        &commands_spox_dir.join("archive.md"),
        TEMPLATE_CMD_ARCHIVE_MD,
    )?;
    write_file(
        &commands_spox_dir.join("implement.md"),
        TEMPLATE_CMD_IMPLEMENT_MD,
    )?;
    write_file(
        &commands_spox_dir.join("propose.md"),
        TEMPLATE_CMD_PROPOSE_MD,
    )?;
    write_file(&commands_spox_dir.join("setup.md"), TEMPLATE_CMD_SETUP_MD)?;
    write_file(&commands_spox_dir.join("vibe.md"), TEMPLATE_CMD_VIBE_MD)?;

    Ok(())
}

/// Create the specs/ directory structure.
/// Only creates directories; preserves existing user content (mission.md).
fn create_specs_dir(base_path: &Path) -> Result<()> {
    let specs_dir = base_path.join("specs");
    let changes_dir = specs_dir.join("_changes");
    let archive_dir = specs_dir.join("_archive");

    // Create directories only (never overwrites user files in specs/)
    create_dir_all(&specs_dir)?;
    create_dir_all(&changes_dir)?;
    create_dir_all(&archive_dir)?;

    // Write mission.md only if it doesn't exist (user content)
    write_file_if_not_exists(&specs_dir.join("mission.md"), TEMPLATE_MISSION_MD)?;

    Ok(())
}

/// Create a directory and all its parents.
fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| {
        Error::Init(format!(
            "failed to create directory '{}': {}",
            path.display(),
            e
        ))
    })
}

/// Write content to a file, creating parent directories if needed.
fn write_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)
        .map_err(|e| Error::Init(format!("failed to write file '{}': {}", path.display(), e)))
}

/// Write content to a file only if it does not already exist.
fn write_file_if_not_exists(path: &Path, content: &str) -> Result<()> {
    if !path.exists() {
        write_file(path, content)?;
    }
    Ok(())
}

/// Format a warning message for when the binary is older than the lock file version.
fn format_version_warning(binary_version: &str, lock_version: &str) -> String {
    format!(
        "Warning: Your spox binary ({}) is older than this project's version ({}).\n\
         Consider upgrading spox to avoid compatibility issues.",
        binary_version, lock_version
    )
}

/// Format an upgrade message, optionally including migration hints.
fn format_upgrade_message(
    from_version: &str,
    to_version: &str,
    upgrade_type: UpgradeType,
) -> String {
    let upgrade_type_str = match upgrade_type {
        UpgradeType::Patch => "patch",
        UpgradeType::Minor => "minor",
        UpgradeType::Major => "major",
    };

    let mut message = format!(
        "Upgrading project from {} to {} ({} version update).",
        from_version, to_version, upgrade_type_str
    );

    // Add migration hints for minor/major upgrades
    if let Some(hint) = get_migration_hints(from_version, to_version) {
        message.push_str("\n\nMigration note: ");
        message.push_str(hint);
    }

    message
}

/// Create or update the version lock file.
///
/// On fresh init, creates a new version.lock with the current binary version.
/// On subsequent init, appends the current version to updated_versions if it differs
/// from the last recorded version.
///
/// Displays warnings/hints based on version comparison:
/// - Warning if binary is older than lock file version
/// - Upgrade message with migration hints if binary is newer
fn create_or_update_version_lock(spox_dir: &Path) -> Result<()> {
    let lock_path = spox_dir.join(VERSION_LOCK_FILENAME);
    let version = current_version();

    if lock_path.exists() {
        // Load existing lock and update if version changed
        let mut lock = VersionLock::load(&lock_path).map_err(|e| {
            Error::Init(format!(
                "failed to read version lock '{}': {}",
                lock_path.display(),
                e
            ))
        })?;

        let last_version = lock.last_version().to_string();

        // Compare versions and display appropriate messages
        if let Ok(comparison) = compare_versions(version, &last_version) {
            match comparison {
                VersionComparison::BinaryOlder => {
                    // Display warning for older binary (non-blocking)
                    eprintln!();
                    eprintln!("{}", format_version_warning(version, &last_version));
                    eprintln!();
                }
                VersionComparison::BinaryNewer { upgrade_type } => {
                    // Display upgrade message with optional migration hints
                    if matches!(upgrade_type, UpgradeType::Minor | UpgradeType::Major) {
                        println!();
                        println!(
                            "{}",
                            format_upgrade_message(&last_version, version, upgrade_type)
                        );
                        println!();
                    }
                }
                VersionComparison::Equal => {
                    // Versions match, nothing to display
                }
            }
        }

        if lock.record_update(version) {
            lock.save(&lock_path).map_err(|e| {
                Error::Init(format!(
                    "failed to write version lock '{}': {}",
                    lock_path.display(),
                    e
                ))
            })?;
        }
    } else {
        // Create new lock file
        let lock = VersionLock::new(version);
        lock.save(&lock_path).map_err(|e| {
            Error::Init(format!(
                "failed to write version lock '{}': {}",
                lock_path.display(),
                e
            ))
        })?;
    }

    Ok(())
}

/// Append gitignore rules to a .gitignore file, avoiding duplicates.
/// Creates the file if it doesn't exist.
fn append_gitignore_rules(gitignore_path: &Path, template_content: &str) -> Result<()> {
    use std::collections::HashSet;

    // Read existing content or start with empty string
    let existing_content = if gitignore_path.exists() {
        fs::read_to_string(gitignore_path).map_err(|e| {
            Error::Init(format!(
                "failed to read file '{}': {}",
                gitignore_path.display(),
                e
            ))
        })?
    } else {
        String::new()
    };

    // Parse existing rules (non-empty, non-comment lines)
    let existing_rules: HashSet<&str> = existing_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    // Find new rules to add
    let mut new_rules = Vec::new();
    for line in template_content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if !existing_rules.contains(trimmed) {
                new_rules.push(line);
            }
        } else {
            // Keep comments and empty lines from template for new additions
            new_rules.push(line);
        }
    }

    // If file doesn't exist, write full template
    if !gitignore_path.exists() {
        fs::write(gitignore_path, template_content).map_err(|e| {
            Error::Init(format!(
                "failed to write file '{}': {}",
                gitignore_path.display(),
                e
            ))
        })?;
        return Ok(());
    }

    // If we have new rules to add, append them
    if new_rules
        .iter()
        .any(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
    {
        let mut content = existing_content;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("\n# Added by spox init\n");
        for rule in new_rules {
            let trimmed = rule.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                content.push_str(rule);
                content.push('\n');
            }
        }
        fs::write(gitignore_path, content).map_err(|e| {
            Error::Init(format!(
                "failed to write file '{}': {}",
                gitignore_path.display(),
                e
            ))
        })?;
    }

    Ok(())
}

/// Make a file executable on Unix systems (chmod +x equivalent).
#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|e| {
        Error::Init(format!(
            "failed to set executable permissions on '{}': {}",
            path.display(),
            e
        ))
    })
}

/// No-op on non-Unix systems.
#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Check if Claude Code CLI is available on the system.
fn is_claude_code_available() -> bool {
    Command::new("sh")
        .args(["-c", "command -v claude"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

const SPOX_START_MARKER: &str = "<!-- SPOX:START -->";
const SPOX_END_MARKER: &str = "<!-- SPOX:END -->";

/// Markers for CLAUDE.md template sections
const MARKER_SYSTEM_TEMPLATES: &str = "<!-- SPOX:SYSTEM-TEMPLATES -->";
const MARKER_USER_TEMPLATES: &str = "<!-- SPOX:USER-TEMPLATES -->";
const MARKER_WORKFLOW: &str = "<!-- SPOX:WORKFLOW -->";

/// Process the CLAUDE template by replacing section markers with content.
///
/// # Arguments
/// * `template` - The CLAUDE template content
/// * `system_templates` - Content for system templates (concatenated)
/// * `user_templates` - Content for user templates (may be empty)
/// * `workflow` - Content for workflow section
fn process_claude_template(
    template: &str,
    system_templates: &str,
    user_templates: &str,
    workflow: &str,
) -> String {
    let result = template.replace(MARKER_SYSTEM_TEMPLATES, system_templates);

    // Remove user templates section if empty, otherwise replace marker
    let result = if user_templates.trim().is_empty() {
        result
            .replace(MARKER_USER_TEMPLATES, "")
            .replace("\n\n\n", "\n\n")
    } else {
        result.replace(MARKER_USER_TEMPLATES, user_templates)
    };

    result.replace(MARKER_WORKFLOW, workflow)
}

/// Get the content for system templates based on config.
fn get_system_templates_content(system_templates: &[String]) -> String {
    let mut content = String::new();

    for template_name in system_templates {
        let template_content = match template_name.as_str() {
            "mcp" => TEMPLATE_STANDARDS_MCP_MD,
            "global" => TEMPLATE_STANDARDS_GLOBAL_MD,
            "coding" => TEMPLATE_STANDARDS_CODING_MD,
            "testing" => TEMPLATE_STANDARDS_TESTING_MD,
            "backend" => TEMPLATE_STANDARDS_BACKEND_MD,
            "frontend" => TEMPLATE_STANDARDS_FRONTEND_MD,
            "vcs" => TEMPLATE_STANDARDS_VCS_MD,
            _ => continue, // Skip invalid templates (already validated)
        };

        if !content.is_empty() {
            content.push_str("\n\n");
        }
        content.push_str(template_content.trim());
    }

    content
}

/// Get the content for user templates based on config.
///
/// Returns the concatenated content of files listed in rules.custom.
/// Files that don't exist are skipped with a warning printed to stderr.
fn get_user_templates_content(base_path: &Path, custom_templates: &[String]) -> String {
    let custom_dir = base_path.join(".spox").join("custom");
    let mut content = String::new();

    for template_name in custom_templates {
        let template_path = custom_dir.join(template_name);

        match fs::read_to_string(&template_path) {
            Ok(template_content) => {
                if !content.is_empty() {
                    content.push_str("\n\n");
                }
                content.push_str(template_content.trim());
            }
            Err(_) => {
                eprintln!(
                    "Warning: custom template '{}' not found at {}",
                    template_name,
                    template_path.display()
                );
            }
        }
    }

    content
}

/// Load config, migrating old format if needed.
///
/// When a config file is missing required sections (like `[rules]` or `[paths]`),
/// this function merges the default values for those sections while preserving
/// any existing user customizations.
fn load_or_migrate_config(base_path: &Path) -> Result<Config> {
    let config_path = base_path.join(".spox").join("config.toml");

    match Config::load(&config_path) {
        Ok(config) => Ok(config),
        Err(Error::ConfigMissingField(field)) if field == "rules" || field == "paths" => {
            // Old config format - merge with defaults and load again
            merge_config_with_defaults(&config_path)?;
            Config::load(&config_path)
        }
        Err(e) => Err(e),
    }
}

/// Merge existing config with default values for any missing sections.
///
/// Reads the existing config, adds any missing sections from the default template,
/// and writes the merged result back to the file. Preserves all existing values.
fn merge_config_with_defaults(config_path: &Path) -> Result<()> {
    // Read existing config
    let existing_content = fs::read_to_string(config_path)
        .map_err(|e| Error::Init(format!("failed to read config: {}", e)))?;

    // Parse existing config as TOML table
    let mut existing: toml::Table = existing_content.parse().map_err(|e: toml::de::Error| {
        Error::Init(format!("failed to parse existing config: {}", e))
    })?;

    // Parse default config as TOML table
    let default: toml::Table = TEMPLATE_CONFIG_TOML.parse().map_err(|e: toml::de::Error| {
        Error::Init(format!("failed to parse default config: {}", e))
    })?;

    // Merge missing top-level sections from default into existing
    for (key, value) in default {
        if !existing.contains_key(&key) {
            existing.insert(key, value);
        }
    }

    // Serialize merged config back to TOML
    let merged_content = toml::to_string_pretty(&existing)
        .map_err(|e| Error::Init(format!("failed to serialize merged config: {}", e)))?;

    // Write merged config back to file
    write_file(config_path, &merged_content)
}

/// Write CLAUDE.md, handling three cases:
/// 1. File doesn't exist: process and write template
/// 2. File exists without markers: append processed SPOX block
/// 3. File exists with markers: replace content between markers
fn write_claude_md(base_path: &Path) -> Result<()> {
    let claude_md_path = base_path.join(".claude/CLAUDE.md");

    // Load config to get template list
    let config = load_or_migrate_config(base_path)?;

    // Build the content for each section
    let system_content = get_system_templates_content(&config.rules.system);
    let user_content = get_user_templates_content(base_path, &config.rules.custom);
    let workflow_content = TEMPLATE_WORKFLOW_MD.trim().to_string();

    // Process the template
    let processed_template = process_claude_template(
        TEMPLATE_CLAUDE_MD,
        &system_content,
        &user_content,
        &workflow_content,
    );

    if !claude_md_path.exists() {
        // Case 1: Fresh project - write processed template directly
        return write_file(&claude_md_path, &processed_template);
    }

    // File exists - read current content
    let existing_content = fs::read_to_string(&claude_md_path).map_err(|e| {
        Error::Init(format!(
            "failed to read '{}': {}",
            claude_md_path.display(),
            e
        ))
    })?;

    let new_content = if existing_content.contains(SPOX_START_MARKER)
        && existing_content.contains(SPOX_END_MARKER)
    {
        // Case 3: Replace content between markers
        replace_spox_block(&existing_content, &processed_template)
    } else {
        // Case 2: Append SPOX block to end
        let mut content = existing_content;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(&processed_template);
        content
    };

    write_file(&claude_md_path, &new_content)
}

/// Replace the SPOX block in existing content with the template block.
fn replace_spox_block(existing: &str, template: &str) -> String {
    let start_idx = existing.find(SPOX_START_MARKER);
    let end_idx = existing.find(SPOX_END_MARKER);

    match (start_idx, end_idx) {
        (Some(start), Some(end)) if start < end => {
            let before = &existing[..start];
            let after = &existing[end + SPOX_END_MARKER.len()..];
            format!("{}{}{}", before, template, after)
        }
        _ => {
            // Malformed markers - append instead
            let mut content = existing.to_string();
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push('\n');
            content.push_str(template);
            content
        }
    }
}

/// Print success message with created/updated structure.
fn print_success_message(base_path: &Path, is_update: bool) {
    let path_display = if base_path == Path::new(".") {
        "current directory".to_string()
    } else {
        format!("'{}'", base_path.display())
    };

    if is_update {
        println!("Updated Spox tooling in {}", path_display);
        println!();
        println!("Updated tooling files:");
        println!("  .spox/           (config, workflow, standards, templates, setup.sh)");
        println!("  .claude/         (agents, commands, CLAUDE.md)");
        println!();
        println!("Preserved user content:");
        println!("  specs/mission.md (not modified)");
        println!("  specs/_changes/  (not modified)");
        println!("  specs/_archive/  (not modified)");
    } else {
        println!("Initialized Spox project in {}", path_display);
        println!();
        println!("Created structure:");
        println!("  .spox/");
        println!("    config.toml");
        println!("    setup.sh");
        println!("    custom/");
        println!("    templates/");
        println!("      spec.md");
        println!("      change/");
        println!("        proposal.md");
        println!("        tasks.md");
        println!("        design.md");
        println!("        spec.md");
        println!("        verification.md");
        println!("  .claude/");
        println!("    CLAUDE.md");
        println!("    agents/");
        println!("      spox-implementer.md");
        println!("      spox-reviewer.md");
        println!("      spox-verifier.md");
        println!("    commands/spox/");
        println!("      archive.md");
        println!("      implement.md");
        println!("      propose.md");
        println!("      setup.md");
        println!("      vibe.md");
        println!("  specs/");
        println!("    mission.md");
        println!("    _changes/");
        println!("    _archive/");
    }

    // Print environment setup instructions
    print_environment_setup_instructions();
}

/// Print instructions for setting up the Claude Code environment.
fn print_environment_setup_instructions() {
    println!();
    println!("Environment Setup:");
    println!("------------------");

    if is_claude_code_available() {
        println!("Claude Code detected. Run the setup script to configure MCP:");
        println!();
        println!("  .spox/setup.sh");
    } else {
        println!("Claude Code not detected. Install it first:");
        println!();
        println!("  npm install -g @anthropic-ai/claude-code");
        println!("  # or");
        println!("  curl -fsSL https://claude.ai/install.sh | bash");
        println!();
        println!("Then run the setup script to configure MCP:");
        println!();
        println!("  .spox/setup.sh");
    }

    println!();
    println!("Note: Windows users require WSL to run the setup script.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ==========================================================================
    // Markdown Linting Functions (Task 7.1)
    // ==========================================================================

    /// Validates markdown content for common formatting issues.
    /// Returns Ok(()) if valid, Err with description if invalid.
    fn validate_markdown(content: &str) -> std::result::Result<(), String> {
        validate_heading_hierarchy(content)?;
        validate_markdown_tables(content)?;
        validate_code_fences(content)?;
        Ok(())
    }

    /// Validates heading hierarchy (h1 -> h2 -> h3, no skipping levels).
    /// First heading can be at any level; subsequent headings must not skip levels.
    fn validate_heading_hierarchy(content: &str) -> std::result::Result<(), String> {
        let mut current_level = 0;
        let mut in_code_block = false;
        let mut fence_char = ' ';
        let mut fence_count = 0;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();

            // Track code block state to skip headings inside them
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let c = trimmed.chars().next().unwrap();
                let count = trimmed.chars().take_while(|&ch| ch == c).count();

                if !in_code_block {
                    in_code_block = true;
                    fence_char = c;
                    fence_count = count;
                } else if c == fence_char && count >= fence_count {
                    in_code_block = false;
                }
                continue;
            }

            // Skip headings inside code blocks
            if in_code_block {
                continue;
            }

            // Check for heading (must start with # followed by space or end of line)
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                // Valid heading: has space after hashes or is just hashes
                let after_hashes = &trimmed[level..];
                if level > 0
                    && level <= 6
                    && (after_hashes.is_empty() || after_hashes.starts_with(' '))
                {
                    // First heading can be any level
                    if current_level > 0 && level > current_level + 1 {
                        return Err(format!(
                            "Line {}: Heading level skipped from h{} to h{}",
                            line_num + 1,
                            current_level,
                            level
                        ));
                    }
                    current_level = level;
                }
            }
        }
        Ok(())
    }

    /// Validates markdown table formatting.
    /// Checks: separator row format, consistent column count across rows.
    fn validate_markdown_tables(content: &str) -> std::result::Result<(), String> {
        let mut in_code_block = false;
        let mut fence_char = ' ';
        let mut fence_count = 0;

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let trimmed = lines[i].trim();

            // Track code block state
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let c = trimmed.chars().next().unwrap();
                let count = trimmed.chars().take_while(|&ch| ch == c).count();

                if !in_code_block {
                    in_code_block = true;
                    fence_char = c;
                    fence_count = count;
                } else if c == fence_char && count >= fence_count {
                    in_code_block = false;
                }
                i += 1;
                continue;
            }

            // Skip content inside code blocks
            if in_code_block {
                i += 1;
                continue;
            }

            // Detect potential table header (line with pipes)
            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let header_cols = count_table_columns(trimmed);

                // Check for separator row on next line
                if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    if is_table_separator(next_line) {
                        let sep_cols = count_table_columns(next_line);
                        if sep_cols != header_cols {
                            return Err(format!(
                                "Line {}: Table separator has {} columns, but header has {}",
                                i + 2,
                                sep_cols,
                                header_cols
                            ));
                        }

                        // Validate data rows
                        let mut j = i + 2;
                        while j < lines.len() {
                            let row = lines[j].trim();
                            if row.is_empty() || !row.starts_with('|') {
                                break;
                            }
                            let row_cols = count_table_columns(row);
                            if row_cols != header_cols {
                                return Err(format!(
                                    "Line {}: Table row has {} columns, but header has {}",
                                    j + 1,
                                    row_cols,
                                    header_cols
                                ));
                            }
                            j += 1;
                        }
                        i = j;
                        continue;
                    } else if next_line.starts_with('|') {
                        // Has a second row but it's not a separator
                        return Err(format!(
                            "Line {}: Table missing separator row (expected |---|...)",
                            i + 2
                        ));
                    }
                }
            }

            i += 1;
        }

        Ok(())
    }

    /// Counts the number of columns in a table row.
    /// Handles escaped pipes (\|) within cells.
    fn count_table_columns(row: &str) -> usize {
        let trimmed = row.trim();
        if trimmed.is_empty() {
            return 0;
        }
        // Count unescaped pipes: a pipe not preceded by backslash
        let mut count = 0;
        let chars: Vec<char> = trimmed.chars().collect();
        for i in 0..chars.len() {
            if chars[i] == '|' {
                // Check if this pipe is escaped (preceded by backslash)
                let is_escaped = i > 0 && chars[i - 1] == '\\';
                if !is_escaped {
                    count += 1;
                }
            }
        }
        // Subtract 1 for leading and trailing pipe
        if count >= 2 {
            count - 1
        } else {
            count
        }
    }

    /// Checks if a line is a valid table separator row.
    fn is_table_separator(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            return false;
        }
        // Check that it contains only |, -, :, and spaces
        trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
            && trimmed.contains('-')
    }

    /// Validates code fences have matching opening/closing.
    fn validate_code_fences(content: &str) -> std::result::Result<(), String> {
        let mut in_code_block = false;
        let mut fence_char = ' ';
        let mut fence_count = 0;
        let mut open_line = 0;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let c = trimmed.chars().next().unwrap();
                let count = trimmed.chars().take_while(|&ch| ch == c).count();

                if !in_code_block {
                    in_code_block = true;
                    fence_char = c;
                    fence_count = count;
                    open_line = line_num + 1;
                } else if c == fence_char && count >= fence_count {
                    in_code_block = false;
                }
            }
        }

        if in_code_block {
            return Err(format!(
                "Unclosed code fence starting at line {}",
                open_line
            ));
        }
        Ok(())
    }

    #[test]
    fn test_init_creates_spox_directory() {
        let temp = TempDir::new().unwrap();
        let result = run(temp.path());
        assert!(result.is_ok());
        assert!(temp.path().join(".spox").exists());
        assert!(temp.path().join(".spox/config.toml").exists());
    }

    #[test]
    fn test_init_creates_specs_templates_directory() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let templates = temp.path().join(".spox/templates");
        assert!(templates.exists());
        assert!(templates.join("spec.md").exists());
        let change = templates.join("change");
        assert!(change.exists());
        assert!(change.join("proposal.md").exists());
        assert!(change.join("tasks.md").exists());
        assert!(change.join("design.md").exists());
        assert!(change.join("spec.md").exists());
        assert!(change.join("verification.md").exists());
    }

    #[test]
    fn test_init_creates_claude_directory() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let claude = temp.path().join(".claude");
        assert!(claude.exists());
        assert!(claude.join("agents/spox-implementer.md").exists());
        assert!(claude.join("agents/spox-reviewer.md").exists());
        assert!(claude.join("agents/spox-verifier.md").exists());
        assert!(claude.join("commands/spox/archive.md").exists());
        assert!(claude.join("commands/spox/implement.md").exists());
        assert!(claude.join("commands/spox/propose.md").exists());
        assert!(claude.join("commands/spox/setup.md").exists());
        assert!(claude.join("commands/spox/vibe.md").exists());
    }

    #[test]
    fn test_init_creates_specs_directory() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let specs = temp.path().join("specs");
        assert!(specs.exists());
        assert!(specs.join("mission.md").exists());
        assert!(specs.join("_changes").exists());
        assert!(specs.join("_archive").exists());
    }

    #[test]
    fn test_init_succeeds_on_already_initialized_project() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        // Second init should succeed (update mode)
        let result = run(temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_updates_tooling_files_on_reinit() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        // Modify a tooling file
        let implementer_file = temp.path().join(".claude/agents/spox-implementer.md");
        fs::write(&implementer_file, "old content").unwrap();

        // Re-run init
        run(temp.path()).unwrap();

        // Tooling file should be updated (not "old content")
        let content = fs::read_to_string(&implementer_file).unwrap();
        assert_ne!(content, "old content");
        assert!(content.contains("implementer") || content.len() > 20); // Has real template content
    }

    #[test]
    fn test_init_preserves_mission_md_on_reinit() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        // Modify mission.md with user content
        let mission_file = temp.path().join("specs/mission.md");
        let user_content = "# My Custom Mission\n\nUser-specific content here.\n";
        fs::write(&mission_file, user_content).unwrap();

        // Re-run init
        run(temp.path()).unwrap();

        // mission.md should be preserved
        let content = fs::read_to_string(&mission_file).unwrap();
        assert_eq!(content, user_content);
    }

    #[test]
    fn test_write_file_if_not_exists_preserves_existing() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.md");

        // Create existing file
        fs::write(&file_path, "existing content").unwrap();

        // Try to write new content
        write_file_if_not_exists(&file_path, "new content").unwrap();

        // Original content should be preserved
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "existing content");
    }

    #[test]
    fn test_write_file_if_not_exists_creates_new() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.md");

        // File doesn't exist, should create it
        write_file_if_not_exists(&file_path, "new content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "new content");
    }

    #[test]
    fn test_init_creates_claude_md() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let claude_md = temp.path().join(".claude/CLAUDE.md");
        assert!(claude_md.exists());
        let content = fs::read_to_string(&claude_md).unwrap();
        assert!(content.contains(SPOX_START_MARKER));
        assert!(content.contains(SPOX_END_MARKER));
    }

    #[test]
    fn test_write_claude_md_appends_to_existing_without_markers() {
        let temp = TempDir::new().unwrap();
        let claude_dir = temp.path().join(".claude");
        let claude_md = claude_dir.join("../.claude/CLAUDE.md");

        // Create .spox directory with config.toml (required for write_claude_md)
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML).unwrap();

        // Create .claude directory and existing CLAUDE.md without markers
        fs::create_dir_all(&claude_dir).unwrap();
        let existing_content = "# My Project\n\nSome custom instructions.\n";
        fs::write(&claude_md, existing_content).unwrap();

        // Call write_claude_md
        write_claude_md(temp.path()).unwrap();

        let result = fs::read_to_string(&claude_md).unwrap();

        // Original content should be preserved
        assert!(result.starts_with("# My Project"));
        assert!(result.contains("Some custom instructions."));

        // SPOX block should be appended
        assert!(result.contains(SPOX_START_MARKER));
        assert!(result.contains(SPOX_END_MARKER));
    }

    #[test]
    fn test_write_claude_md_replaces_existing_spox_block() {
        let temp = TempDir::new().unwrap();
        let claude_dir = temp.path().join(".claude");
        let claude_md = claude_dir.join("../.claude/CLAUDE.md");

        // Create .spox directory with config.toml (required for write_claude_md)
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML).unwrap();

        // Create .claude directory and existing CLAUDE.md with old SPOX block
        fs::create_dir_all(&claude_dir).unwrap();
        let existing_content = "# My Project\n\n<!-- SPOX:START -->\nOld SPOX content\n<!-- SPOX:END -->\n\n## My Custom Section\n";
        fs::write(&claude_md, existing_content).unwrap();

        // Call write_claude_md
        write_claude_md(temp.path()).unwrap();

        let result = fs::read_to_string(&claude_md).unwrap();

        // Content before markers should be preserved
        assert!(result.starts_with("# My Project"));

        // Content after markers should be preserved
        assert!(result.contains("## My Custom Section"));

        // Old SPOX content should be replaced
        assert!(!result.contains("Old SPOX content"));

        // New SPOX block should be present
        assert!(result.contains(SPOX_START_MARKER));
        assert!(result.contains(SPOX_END_MARKER));
        assert!(result.contains("# Standards Compliance and Guardrails"));
    }

    #[test]
    fn test_write_claude_md_preserves_content_outside_markers() {
        let temp = TempDir::new().unwrap();
        let claude_dir = temp.path().join(".claude");
        let claude_md = claude_dir.join("../.claude/CLAUDE.md");

        // Create .spox directory with config.toml (required for write_claude_md)
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML).unwrap();

        // Create .claude directory and existing CLAUDE.md with content before and after SPOX block
        fs::create_dir_all(&claude_dir).unwrap();
        let existing_content = "# Header Before\n\nContent before.\n\n<!-- SPOX:START -->\nOld block\n<!-- SPOX:END -->\n\n# Header After\n\nContent after.\n";
        fs::write(&claude_md, existing_content).unwrap();

        // Call write_claude_md
        write_claude_md(temp.path()).unwrap();

        let result = fs::read_to_string(&claude_md).unwrap();

        // All content outside markers should be preserved
        assert!(result.contains("# Header Before"));
        assert!(result.contains("Content before."));
        assert!(result.contains("# Header After"));
        assert!(result.contains("Content after."));

        // Only one SPOX block should exist
        assert_eq!(result.matches(SPOX_START_MARKER).count(), 1);
        assert_eq!(result.matches(SPOX_END_MARKER).count(), 1);
    }

    #[test]
    fn test_init_creates_setup_sh() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let setup_sh = temp.path().join(".spox/setup.sh");
        assert!(setup_sh.exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_setup_sh_is_executable() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let setup_sh = temp.path().join(".spox/setup.sh");

        let metadata = fs::metadata(&setup_sh).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Check that executable bits are set (at least user execute)
        assert!(mode & 0o100 != 0, "setup.sh should be executable");
    }

    #[test]
    fn test_init_updates_setup_sh_on_reinit() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        // Modify setup.sh with old content
        let setup_sh = temp.path().join(".spox/setup.sh");
        fs::write(&setup_sh, "old content").unwrap();

        // Re-run init
        run(temp.path()).unwrap();

        // setup.sh should be updated (not "old content")
        let content = fs::read_to_string(&setup_sh).unwrap();
        assert_ne!(content, "old content");
    }

    #[test]
    fn test_is_claude_code_available_returns_bool() {
        // This test just verifies the function runs without panicking
        // and returns a boolean. The actual result depends on the system.
        let _result: bool = is_claude_code_available();
        // If we got here, the function ran successfully
    }

    // ==================== Tests for CLAUDE.md template processing ====================

    #[test]
    fn test_process_claude_template() {
        let template = "<!-- SPOX:START -->\n# Header\n\n<!-- SPOX:SYSTEM-TEMPLATES -->\n\n<!-- SPOX:USER-TEMPLATES -->\n\n<!-- SPOX:WORKFLOW -->\n\n<!-- SPOX:END -->";

        let result = process_claude_template(
            template,
            "system content",
            "user content",
            "workflow content",
        );

        assert!(result.contains("system content"));
        assert!(result.contains("user content"));
        assert!(result.contains("workflow content"));
    }

    #[test]
    fn test_process_claude_template_empty_user_templates() {
        let template = "<!-- SPOX:START -->\n# Header\n\n<!-- SPOX:SYSTEM-TEMPLATES -->\n\n<!-- SPOX:USER-TEMPLATES -->\n\n<!-- SPOX:WORKFLOW -->\n\n<!-- SPOX:END -->";

        let result = process_claude_template(template, "system content", "", "workflow content");

        assert!(result.contains("system content"));
        assert!(!result.contains("<!-- SPOX:USER-TEMPLATES -->"));
        assert!(result.contains("workflow content"));
    }

    #[test]
    fn test_get_system_templates_content() {
        let templates = vec!["mcp".to_string(), "global".to_string()];
        let content = get_system_templates_content(&templates);

        // Should contain content from both templates
        assert!(content.contains("MCP Tools"));
        assert!(content.contains("General mindset"));
    }

    #[test]
    fn test_get_system_templates_content_respects_order() {
        let templates = vec!["global".to_string(), "mcp".to_string()];
        let content = get_system_templates_content(&templates);

        // global content should appear before mcp content
        let global_pos = content.find("General mindset").unwrap();
        let mcp_pos = content.find("MCP Tools").unwrap();
        assert!(global_pos < mcp_pos);
    }

    #[test]
    fn test_get_system_templates_content_skips_invalid() {
        let templates = vec![
            "mcp".to_string(),
            "invalid-template".to_string(),
            "global".to_string(),
        ];
        let content = get_system_templates_content(&templates);

        // Should contain valid templates
        assert!(content.contains("MCP Tools"));
        assert!(content.contains("General mindset"));
        // Should not panic or fail on invalid template
    }

    #[test]
    fn test_get_user_templates_content_reads_files() {
        let temp = TempDir::new().unwrap();
        let custom_dir = temp.path().join(".spox").join("custom");
        fs::create_dir_all(&custom_dir).unwrap();

        // Create a custom template file
        fs::write(
            custom_dir.join("my-rules.md"),
            "# My Custom Rules\n\nRule 1",
        )
        .unwrap();

        let content = get_user_templates_content(temp.path(), &["my-rules.md".to_string()]);

        assert!(content.contains("My Custom Rules"));
        assert!(content.contains("Rule 1"));
    }

    #[test]
    fn test_get_user_templates_content_missing_file_skipped() {
        let temp = TempDir::new().unwrap();
        let custom_dir = temp.path().join(".spox").join("custom");
        fs::create_dir_all(&custom_dir).unwrap();

        // Only create one file
        fs::write(custom_dir.join("existing.md"), "Existing content").unwrap();

        let content = get_user_templates_content(
            temp.path(),
            &["missing.md".to_string(), "existing.md".to_string()],
        );

        // Should contain the existing file's content
        assert!(content.contains("Existing content"));
        // Should not fail on missing file
    }

    #[test]
    fn test_init_creates_custom_directory() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        assert!(temp.path().join(".spox/custom").exists());
    }

    #[test]
    fn test_claude_md_contains_merged_standards() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        let claude_md = temp.path().join(".claude/CLAUDE.md");
        let content = fs::read_to_string(&claude_md).unwrap();

        // Should contain merged system templates
        assert!(content.contains("MCP Tools")); // from mcp.md
        assert!(content.contains("General mindset")); // from global.md
        assert!(content.contains("# Spec Oxide Workflow")); // from workflow.md header
    }

    #[test]
    fn test_claude_md_has_no_leftover_markers() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        let claude_md = temp.path().join(".claude/CLAUDE.md");
        let content = fs::read_to_string(&claude_md).unwrap();

        // Should not contain template markers (they should be replaced)
        assert!(!content.contains("<!-- SPOX:SYSTEM-TEMPLATES -->"));
        assert!(!content.contains("<!-- SPOX:USER-TEMPLATES -->"));
        assert!(!content.contains("<!-- SPOX:WORKFLOW -->"));
    }

    #[test]
    fn test_load_or_migrate_config_loads_valid_config() {
        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML).unwrap();

        let config = load_or_migrate_config(temp.path()).unwrap();
        assert!(!config.rules.system.is_empty());
    }

    #[test]
    fn test_load_or_migrate_config_adds_missing_rules_section() {
        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Config with paths only (missing [rules] section)
        let old_config = r#"
[paths]
spec_folder = "custom/specs/"
changes_folder = "custom/_changes"
archive_folder = "custom/_archive"
"#;
        fs::write(spox_dir.join("config.toml"), old_config).unwrap();

        let config = load_or_migrate_config(temp.path()).unwrap();

        // Should have default rules
        assert!(!config.rules.system.is_empty());
        // Should preserve custom paths
        assert_eq!(config.paths.spec_folder, "custom/specs/");
        assert_eq!(config.paths.changes_folder, "custom/_changes");
        assert_eq!(config.paths.archive_folder, "custom/_archive");
    }

    #[test]
    fn test_load_or_migrate_config_adds_missing_paths_section() {
        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Config with rules only (missing [paths] section)
        let old_config = r#"
[rules]
system = ["mcp", "global"]
custom = ["my-rules.md"]
"#;
        fs::write(spox_dir.join("config.toml"), old_config).unwrap();

        let config = load_or_migrate_config(temp.path()).unwrap();

        // Should preserve custom rules
        assert_eq!(config.rules.system, vec!["mcp", "global"]);
        assert_eq!(config.rules.custom, vec!["my-rules.md"]);
        // Should have default paths
        assert_eq!(config.paths.spec_folder, "specs/");
        assert_eq!(config.paths.changes_folder, "specs/_changes");
        assert_eq!(config.paths.archive_folder, "specs/_archive");
    }

    #[test]
    fn test_load_or_migrate_config_preserves_custom_system_array() {
        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // User customized system array (removed some defaults)
        let old_config = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "coding"]
custom = []
"#;
        fs::write(spox_dir.join("config.toml"), old_config).unwrap();

        let config = load_or_migrate_config(temp.path()).unwrap();

        // User's customized system array should be preserved
        assert_eq!(config.rules.system, vec!["mcp", "coding"]);
    }

    #[test]
    fn test_load_or_migrate_config_preserves_custom_array() {
        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // User added custom rules
        let old_config = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global", "coding", "testing", "backend", "frontend", "vcs"]
custom = ["my-team-rules.md", "security-policy.md"]
"#;
        fs::write(spox_dir.join("config.toml"), old_config).unwrap();

        let config = load_or_migrate_config(temp.path()).unwrap();

        // User's custom array should be preserved
        assert_eq!(
            config.rules.custom,
            vec!["my-team-rules.md", "security-policy.md"]
        );
    }

    #[test]
    fn test_load_or_migrate_config_merged_file_is_valid_toml() {
        let temp = TempDir::new().unwrap();
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Config with paths only (missing [rules] section)
        let old_config = r#"
[paths]
spec_folder = "custom/specs/"
changes_folder = "custom/_changes"
archive_folder = "custom/_archive"
"#;
        fs::write(spox_dir.join("config.toml"), old_config).unwrap();

        // First migration
        load_or_migrate_config(temp.path()).unwrap();

        // Reload the file to verify it's valid TOML that can be loaded
        let config = load_or_migrate_config(temp.path()).unwrap();

        // Should still have preserved paths and added rules
        assert_eq!(config.paths.spec_folder, "custom/specs/");
        assert!(!config.rules.system.is_empty());
    }

    // ==================== Tests for append_gitignore_rules ====================

    #[test]
    fn test_append_gitignore_rules_creates_new_file() {
        let temp = TempDir::new().unwrap();
        let gitignore_path = temp.path().join(".gitignore");

        append_gitignore_rules(&gitignore_path, "rule1\nrule2\n").unwrap();

        assert!(gitignore_path.exists());
        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("rule1"));
        assert!(content.contains("rule2"));
    }

    #[test]
    fn test_append_gitignore_rules_avoids_duplicates() {
        let temp = TempDir::new().unwrap();
        let gitignore_path = temp.path().join(".gitignore");

        // Create existing file with some rules
        fs::write(&gitignore_path, "existing_rule\n").unwrap();

        // Append rules including the existing one
        append_gitignore_rules(&gitignore_path, "existing_rule\nnew_rule\n").unwrap();

        let content = fs::read_to_string(&gitignore_path).unwrap();
        // existing_rule should appear only once
        assert_eq!(content.matches("existing_rule").count(), 1);
        // new_rule should be added
        assert!(content.contains("new_rule"));
    }

    #[test]
    fn test_append_gitignore_rules_handles_comments_and_empty_lines() {
        let temp = TempDir::new().unwrap();
        let gitignore_path = temp.path().join(".gitignore");

        let template = "# Comment\n\nrule1\nrule2\n";
        append_gitignore_rules(&gitignore_path, template).unwrap();

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("rule1"));
        assert!(content.contains("rule2"));
    }

    #[test]
    fn test_append_gitignore_rules_appends_to_existing() {
        let temp = TempDir::new().unwrap();
        let gitignore_path = temp.path().join(".gitignore");

        // Create existing file
        fs::write(&gitignore_path, "old_rule\n").unwrap();

        // Append new rules
        append_gitignore_rules(&gitignore_path, "new_rule\n").unwrap();

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("old_rule"));
        assert!(content.contains("new_rule"));
        assert!(content.contains("# Added by spox init"));
    }

    #[test]
    fn test_init_creates_spox_gitignore() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let gitignore = temp.path().join(".spox/.gitignore");
        assert!(gitignore.exists());
        let content = fs::read_to_string(&gitignore).unwrap();
        assert!(content.contains("/templates/"));
    }

    #[test]
    fn test_init_creates_claude_gitignore() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let gitignore = temp.path().join(".claude/.gitignore");
        assert!(gitignore.exists());
        let content = fs::read_to_string(&gitignore).unwrap();
        assert!(content.contains("agents/spox-*.md"));
    }

    // ==========================================================================
    // Integration Test: Generated CLAUDE.md Passes Linting (Task 7.2)
    // ==========================================================================

    #[test]
    fn test_generated_claude_md_passes_markdown_linting() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        let claude_md = temp.path().join(".claude/CLAUDE.md");
        assert!(claude_md.exists(), "CLAUDE.md should be created");

        let content = fs::read_to_string(&claude_md).unwrap();

        // Validate the generated CLAUDE.md passes all linting rules
        let result = validate_markdown(&content);
        assert!(
            result.is_ok(),
            "Generated CLAUDE.md should pass markdown linting: {:?}",
            result.err()
        );
    }

    // ==========================================================================
    // Markdown Linting Tests (Task 7.3)
    // ==========================================================================

    // --------------------------------------------------------------------------
    // Heading Hierarchy Tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_validate_heading_hierarchy_passes_valid() {
        let content = r#"# Heading 1
## Heading 2
### Heading 3
## Another H2
### Another H3
#### Heading 4
"#;
        assert!(validate_heading_hierarchy(content).is_ok());
    }

    #[test]
    fn test_validate_heading_hierarchy_passes_starting_at_h2() {
        // It's valid to start at any level
        let content = r#"## Starting at H2
### Then H3
"#;
        assert!(validate_heading_hierarchy(content).is_ok());
    }

    #[test]
    fn test_validate_heading_hierarchy_fails_skipped_level() {
        let content = r#"# Heading 1
### Skipped H2
"#;
        let result = validate_heading_hierarchy(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("h1 to h3"),
            "Error should mention skip: {}",
            err
        );
    }

    #[test]
    fn test_validate_heading_hierarchy_fails_multiple_level_skip() {
        let content = r#"# Heading 1
## Heading 2
##### Skipped to H5
"#;
        let result = validate_heading_hierarchy(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("h2 to h5"),
            "Error should mention skip: {}",
            err
        );
    }

    #[test]
    fn test_validate_heading_hierarchy_ignores_headings_in_code_blocks() {
        let content = r#"# Real H1
## Real H2

```markdown
# This is in a code block
##### This skip should be ignored
```

### Real H3
"#;
        assert!(validate_heading_hierarchy(content).is_ok());
    }

    #[test]
    fn test_validate_heading_hierarchy_handles_tilde_code_blocks() {
        let content = r#"# H1
## H2

~~~
# In tilde fence
##### Skip ignored
~~~

### H3
"#;
        assert!(validate_heading_hierarchy(content).is_ok());
    }

    // --------------------------------------------------------------------------
    // Markdown Table Tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_validate_markdown_tables_passes_valid() {
        let content = r#"
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
"#;
        assert!(validate_markdown_tables(content).is_ok());
    }

    #[test]
    fn test_validate_markdown_tables_passes_with_alignment() {
        let content = r#"
| Left | Center | Right |
|:-----|:------:|------:|
| L    |   C    |     R |
"#;
        assert!(validate_markdown_tables(content).is_ok());
    }

    #[test]
    fn test_validate_markdown_tables_fails_inconsistent_columns() {
        let content = r#"
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   |
"#;
        let result = validate_markdown_tables(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("column"),
            "Error should mention columns: {}",
            err
        );
    }

    #[test]
    fn test_validate_markdown_tables_fails_missing_separator() {
        let content = r#"
| Header 1 | Header 2 |
| Cell 1   | Cell 2   |
"#;
        let result = validate_markdown_tables(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("separator"),
            "Error should mention separator: {}",
            err
        );
    }

    #[test]
    fn test_validate_markdown_tables_ignores_tables_in_code_blocks() {
        let content = r#"
```markdown
| Bad | Table |
| no separator here
```

Regular text here.
"#;
        assert!(validate_markdown_tables(content).is_ok());
    }

    #[test]
    fn test_validate_markdown_tables_passes_multiple_valid_tables() {
        let content = r#"
| A | B |
|---|---|
| 1 | 2 |

Some text between tables.

| X | Y | Z |
|---|---|---|
| a | b | c |
"#;
        assert!(validate_markdown_tables(content).is_ok());
    }

    #[test]
    fn test_validate_markdown_tables_handles_escaped_pipes() {
        // A table cell containing \| should not be counted as a column separator
        let content = r#"
| Task | Use | Not |
|------|-----|-----|
| Find | foo | `rg "class\|function"` |
"#;
        assert!(validate_markdown_tables(content).is_ok());
    }

    // --------------------------------------------------------------------------
    // Code Fence Tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_validate_code_fences_passes_valid() {
        let content = r#"
```rust
fn main() {}
```
"#;
        assert!(validate_code_fences(content).is_ok());
    }

    #[test]
    fn test_validate_code_fences_passes_multiple_blocks() {
        let content = r#"
```rust
fn foo() {}
```

Some text.

```python
def bar(): pass
```
"#;
        assert!(validate_code_fences(content).is_ok());
    }

    #[test]
    fn test_validate_code_fences_passes_tilde_fences() {
        let content = r#"
~~~bash
echo "hello"
~~~
"#;
        assert!(validate_code_fences(content).is_ok());
    }

    #[test]
    fn test_validate_code_fences_fails_unclosed() {
        let content = r#"
```rust
fn main() {}
"#;
        let result = validate_code_fences(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("nclosed") || err.contains("code fence"),
            "Error should mention unclosed: {}",
            err
        );
    }

    #[test]
    fn test_validate_code_fences_fails_unclosed_tilde() {
        let content = r#"
~~~bash
echo "hello"
"#;
        let result = validate_code_fences(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_code_fences_handles_nested_backticks() {
        // A code block with 3 backticks can contain inline code
        let content = r#"
```markdown
Here's some `inline code` with backticks
```
"#;
        assert!(validate_code_fences(content).is_ok());
    }

    #[test]
    fn test_validate_code_fences_longer_fence_closes() {
        // A fence opened with ``` can be closed with ```` or more
        let content = r#"
```rust
code
````
"#;
        assert!(validate_code_fences(content).is_ok());
    }

    // --------------------------------------------------------------------------
    // Combined validate_markdown Tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_validate_markdown_passes_valid_document() {
        let content = r#"# Title

## Section 1

Some paragraph text.

| Name | Value |
|------|-------|
| foo  | bar   |

```rust
fn main() {}
```

## Section 2

### Subsection

More text.
"#;
        assert!(validate_markdown(content).is_ok());
    }

    #[test]
    fn test_validate_markdown_fails_on_heading_error() {
        let content = r#"# Title
#### Skipped levels
"#;
        let result = validate_markdown(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_markdown_fails_on_table_error() {
        let content = r#"# Title

| A | B | C |
|---|---|---|
| 1 | 2 |
"#;
        let result = validate_markdown(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_markdown_fails_on_code_fence_error() {
        let content = r#"# Title

```rust
unclosed
"#;
        let result = validate_markdown(content);
        assert!(result.is_err());
    }

    // ==========================================================================
    // Integration Tests for Config Preservation (Tasks 3.1-3.5)
    // ==========================================================================

    #[test]
    fn test_init_preserves_existing_config_toml_on_reinit() {
        // Task 3.1: Verify that existing config.toml is not overwritten on re-init
        let temp = TempDir::new().unwrap();

        // First init creates default config
        run(temp.path()).unwrap();

        // Modify config.toml with custom content
        let config_path = temp.path().join(".spox/config.toml");
        let custom_config = r#"
[paths]
spec_folder = "my-specs/"
changes_folder = "my-specs/_changes"
archive_folder = "my-specs/_archive"

[rules]
system = ["mcp"]
custom = ["team-rules.md"]
"#;
        fs::write(&config_path, custom_config).unwrap();

        // Re-run init (should not overwrite config.toml)
        run(temp.path()).unwrap();

        // Verify custom config is preserved
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(
            content.contains("my-specs/"),
            "Custom spec_folder should be preserved"
        );
        assert!(
            content.contains("team-rules.md"),
            "Custom rules should be preserved"
        );
    }

    #[test]
    fn test_init_preserves_custom_system_array_on_reinit() {
        // Task 3.2: Verify custom `system` array is preserved through full init flow
        let temp = TempDir::new().unwrap();

        // First init
        run(temp.path()).unwrap();

        // Modify system array (user removed some defaults)
        let config_path = temp.path().join(".spox/config.toml");
        let custom_config = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "testing"]
custom = []
"#;
        fs::write(&config_path, custom_config).unwrap();

        // Re-run init
        run(temp.path()).unwrap();

        // Load and verify config through the normal flow
        let config = load_or_migrate_config(temp.path()).unwrap();
        assert_eq!(
            config.rules.system,
            vec!["mcp", "testing"],
            "Custom system array should be preserved"
        );
    }

    #[test]
    fn test_init_preserves_custom_array_on_reinit() {
        // Task 3.3: Verify custom `custom` array is preserved through full init flow
        let temp = TempDir::new().unwrap();

        // First init
        run(temp.path()).unwrap();

        // Add custom rules
        let config_path = temp.path().join(".spox/config.toml");
        let custom_config = r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "global", "coding", "testing", "backend", "frontend", "vcs"]
custom = ["company-standards.md", "security-policy.md", "api-guidelines.md"]
"#;
        fs::write(&config_path, custom_config).unwrap();

        // Re-run init
        run(temp.path()).unwrap();

        // Verify custom array is preserved
        let config = load_or_migrate_config(temp.path()).unwrap();
        assert_eq!(
            config.rules.custom,
            vec![
                "company-standards.md",
                "security-policy.md",
                "api-guidelines.md"
            ],
            "Custom rules array should be preserved"
        );
    }

    #[test]
    fn test_init_migration_adds_rules_preserves_paths() {
        // Task 3.4: Verify migration adds missing [rules] section without overwriting [paths]
        let temp = TempDir::new().unwrap();

        // Create .spox directory manually with old config format (missing [rules])
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        let old_config = r#"
[paths]
spec_folder = "documentation/specs/"
changes_folder = "documentation/_changes"
archive_folder = "documentation/_archive"
"#;
        fs::write(spox_dir.join("config.toml"), old_config).unwrap();

        // Run init (should trigger migration)
        run(temp.path()).unwrap();

        // Verify paths are preserved and rules are added
        let config = load_or_migrate_config(temp.path()).unwrap();
        assert_eq!(
            config.paths.spec_folder, "documentation/specs/",
            "Custom paths should be preserved"
        );
        assert_eq!(
            config.paths.changes_folder, "documentation/_changes",
            "Custom changes_folder should be preserved"
        );
        assert_eq!(
            config.paths.archive_folder, "documentation/_archive",
            "Custom archive_folder should be preserved"
        );
        assert!(
            !config.rules.system.is_empty(),
            "Default rules should be added"
        );
    }

    #[test]
    fn test_init_new_project_gets_full_default_config() {
        // Task 3.5: Verify new project gets full default config
        let temp = TempDir::new().unwrap();

        // Run init on fresh directory
        run(temp.path()).unwrap();

        // Load config and verify it has all expected sections with defaults
        let config = load_or_migrate_config(temp.path()).unwrap();

        // Verify paths section has defaults
        assert_eq!(config.paths.spec_folder, "specs/");
        assert_eq!(config.paths.changes_folder, "specs/_changes");
        assert_eq!(config.paths.archive_folder, "specs/_archive");

        // Verify rules section has defaults
        assert!(
            config.rules.system.contains(&"mcp".to_string()),
            "Default system should include mcp"
        );
        assert!(
            config.rules.system.contains(&"global".to_string()),
            "Default system should include global"
        );
        assert!(
            config.rules.system.contains(&"coding".to_string()),
            "Default system should include coding"
        );
        assert!(
            config.rules.custom.is_empty(),
            "Default custom should be empty"
        );
    }

    // ==================== Tests for Version Lock ====================

    #[test]
    fn test_init_creates_version_lock_on_fresh_project() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        let lock_path = temp.path().join(".spox/version.lock");
        assert!(lock_path.exists(), "version.lock should be created");

        let lock = crate::core::version_lock::VersionLock::load(&lock_path).unwrap();
        assert_eq!(
            lock.initialized_version,
            crate::core::version_lock::current_version(),
            "initialized_version should match current binary version"
        );
        assert!(
            lock.updated_versions.is_empty(),
            "updated_versions should be empty on fresh init"
        );
    }

    #[test]
    fn test_init_updates_version_lock_on_reinit_with_different_version() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        // Manually modify the version.lock to simulate an older version
        let lock_path = temp.path().join(".spox/version.lock");
        let lock = crate::core::version_lock::VersionLock::new("0.1.0");
        lock.save(&lock_path).unwrap();

        // Re-run init (should add current version to updated_versions)
        run(temp.path()).unwrap();

        let updated_lock = crate::core::version_lock::VersionLock::load(&lock_path).unwrap();
        assert_eq!(
            updated_lock.initialized_version, "0.1.0",
            "initialized_version should be preserved"
        );
        assert!(
            updated_lock
                .updated_versions
                .contains(&crate::core::version_lock::current_version().to_string()),
            "updated_versions should contain current version"
        );
    }

    #[test]
    fn test_init_preserves_version_lock_on_reinit_with_same_version() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        let lock_path = temp.path().join(".spox/version.lock");
        let lock_before = crate::core::version_lock::VersionLock::load(&lock_path).unwrap();
        let content_before = fs::read_to_string(&lock_path).unwrap();

        // Re-run init (should not modify version.lock)
        run(temp.path()).unwrap();

        let content_after = fs::read_to_string(&lock_path).unwrap();
        let lock_after = crate::core::version_lock::VersionLock::load(&lock_path).unwrap();

        assert_eq!(
            content_before, content_after,
            "version.lock content should be unchanged"
        );
        assert_eq!(
            lock_before.initialized_version, lock_after.initialized_version,
            "initialized_version should be unchanged"
        );
        assert_eq!(
            lock_before.updated_versions, lock_after.updated_versions,
            "updated_versions should be unchanged"
        );
    }

    #[test]
    fn test_version_lock_preserves_update_history() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        // Simulate multiple version upgrades
        let lock_path = temp.path().join(".spox/version.lock");
        let mut lock = crate::core::version_lock::VersionLock::new("0.1.0");
        lock.updated_versions = vec!["0.2.0".to_string(), "0.3.0".to_string()];
        lock.save(&lock_path).unwrap();

        // Re-run init with current version
        run(temp.path()).unwrap();

        let updated_lock = crate::core::version_lock::VersionLock::load(&lock_path).unwrap();
        assert_eq!(updated_lock.initialized_version, "0.1.0");
        // Previous versions should be preserved
        assert!(updated_lock.updated_versions.contains(&"0.2.0".to_string()));
        assert!(updated_lock.updated_versions.contains(&"0.3.0".to_string()));
        // Current version should be appended
        assert!(updated_lock
            .updated_versions
            .contains(&crate::core::version_lock::current_version().to_string()));
    }

    #[test]
    fn test_version_lock_not_ignored_by_gitignore() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        let gitignore = temp.path().join(".spox/.gitignore");
        let content = fs::read_to_string(&gitignore).unwrap();

        // The gitignore should explicitly NOT ignore version.lock
        assert!(
            content.contains("!version.lock"),
            ".gitignore should contain !version.lock to ensure it's committed"
        );
    }

    // ==================== Tests for Version Compatibility Check (Task 2.2-2.4) ====================

    #[test]
    fn test_check_version_compatibility_returns_equal_for_same_version() {
        use crate::core::version_lock::{compare_versions, current_version, VersionComparison};

        let result = compare_versions(current_version(), current_version());
        assert!(matches!(result, Ok(VersionComparison::Equal)));
    }

    #[test]
    fn test_check_version_compatibility_returns_binary_older_when_outdated() {
        use crate::core::version_lock::{compare_versions, VersionComparison};

        // Binary 0.3.0, lock 0.4.0 -> BinaryOlder
        let result = compare_versions("0.3.0", "0.4.0");
        assert!(matches!(result, Ok(VersionComparison::BinaryOlder)));
    }

    #[test]
    fn test_check_version_compatibility_returns_binary_newer_for_upgrade() {
        use crate::core::version_lock::{compare_versions, UpgradeType, VersionComparison};

        // Binary 0.5.0, lock 0.4.0 -> BinaryNewer (minor upgrade)
        let result = compare_versions("0.5.0", "0.4.0");
        assert!(matches!(
            result,
            Ok(VersionComparison::BinaryNewer {
                upgrade_type: UpgradeType::Minor
            })
        ));
    }

    #[test]
    fn test_format_version_warning_produces_output() {
        let warning = format_version_warning("0.3.0", "0.4.0");
        assert!(warning.contains("0.3.0"));
        assert!(warning.contains("0.4.0"));
        assert!(warning.contains("older") || warning.contains("outdated"));
    }

    #[test]
    fn test_format_upgrade_message_includes_version() {
        use crate::core::version_lock::UpgradeType;

        let message = format_upgrade_message("0.3.0", "0.4.0", UpgradeType::Minor);
        assert!(message.contains("0.4.0"));
    }

    #[test]
    fn test_format_upgrade_message_includes_migration_hint_when_available() {
        use crate::core::version_lock::{get_migration_hints, UpgradeType};

        // When there's a migration hint for 0.3 -> 0.4
        if get_migration_hints("0.3.0", "0.4.0").is_some() {
            let message = format_upgrade_message("0.3.0", "0.4.0", UpgradeType::Minor);
            assert!(message.contains("version lock") || message.contains("Migration"));
        }
    }
}
