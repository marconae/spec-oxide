//! Init command implementation for bootstrapping a new Spox project.

use crate::config::Config;
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
const TEMPLATE_SPEC_MISSION_MD: &str = include_str!("../templates/specs/mission.md");
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
const TEMPLATE_CMD_VIBE_MD: &str = include_str!("../templates/claude/commands/spox/vibe.md");

// specs/ templates
const TEMPLATE_MISSION_MD: &str = include_str!("../templates/specs/mission.md");

// CLAUDE.md template
const TEMPLATE_CLAUDE_MD: &str = include_str!("../templates/spox/CLAUDE-template.md");

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
/// |   |-- workflow.md
/// |   |-- setup.sh
/// |   |-- standards/
/// |   |   |-- backend.md
/// |   |   |-- coding.md
/// |   |   |-- frontend.md
/// |   |   |-- global.md
/// |   |   |-- mcp.md
/// |   |   |-- testing.md
/// |   |   +-- vcs.md
/// |   +-- specs/
/// |       |-- spec.md
/// |       |-- mission.md
/// |       +-- change/
/// |           |-- proposal.md
/// |           |-- tasks.md
/// |           |-- design.md
/// |           |-- spec.md
/// |           +-- verification.md
/// |-- .claude/
/// |   |-- agents/
/// |   |   |-- spox-implementer.md
/// |   |   |-- spox-reviewer.md
/// |   |   +-- spox-verifier.md
/// |   +-- commands/spox/
/// |       |-- archive.md
/// |       |-- implement.md
/// |       |-- propose.md
/// |       +-- vibe.md
/// +-- specs/
/// |   |-- mission.md
/// |   |-- _changes/
/// |   +-- _archive/
/// +-- CLAUDE.md
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
    let standards_dir = spox_dir.join("standards");
    let custom_dir = spox_dir.join("custom");
    let specs_dir = spox_dir.join("specs");
    let specs_change_dir = specs_dir.join("change");

    // Create directories
    create_dir_all(&standards_dir)?;
    create_dir_all(&custom_dir)?;
    create_dir_all(&specs_change_dir)?;

    // Write config files
    write_file(&spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML)?;
    write_file(&spox_dir.join("workflow.md"), TEMPLATE_WORKFLOW_MD)?;

    // Write standards files
    write_file(
        &standards_dir.join("backend.md"),
        TEMPLATE_STANDARDS_BACKEND_MD,
    )?;
    write_file(
        &standards_dir.join("coding.md"),
        TEMPLATE_STANDARDS_CODING_MD,
    )?;
    write_file(
        &standards_dir.join("frontend.md"),
        TEMPLATE_STANDARDS_FRONTEND_MD,
    )?;
    write_file(
        &standards_dir.join("global.md"),
        TEMPLATE_STANDARDS_GLOBAL_MD,
    )?;
    write_file(
        &standards_dir.join("testing.md"),
        TEMPLATE_STANDARDS_TESTING_MD,
    )?;
    write_file(&standards_dir.join("vcs.md"), TEMPLATE_STANDARDS_VCS_MD)?;
    write_file(&standards_dir.join("mcp.md"), TEMPLATE_STANDARDS_MCP_MD)?;

    // Write spec template files
    write_file(&specs_dir.join("spec.md"), TEMPLATE_SPEC_SPEC_MD)?;
    write_file(&specs_dir.join("mission.md"), TEMPLATE_SPEC_MISSION_MD)?;
    write_file(
        &specs_change_dir.join("proposal.md"),
        TEMPLATE_SPEC_CHANGE_PROPOSAL_MD,
    )?;
    write_file(
        &specs_change_dir.join("tasks.md"),
        TEMPLATE_SPEC_CHANGE_TASKS_MD,
    )?;
    write_file(
        &specs_change_dir.join("design.md"),
        TEMPLATE_SPEC_CHANGE_DESIGN_MD,
    )?;
    write_file(
        &specs_change_dir.join("spec.md"),
        TEMPLATE_SPEC_CHANGE_SPEC_MD,
    )?;
    write_file(
        &specs_change_dir.join("verification.md"),
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
fn load_or_migrate_config(base_path: &Path) -> Result<Config> {
    let config_path = base_path.join(".spox").join("config.toml");

    match Config::load(&config_path) {
        Ok(config) => Ok(config),
        Err(Error::ConfigMissingField(field)) if field == "rules" || field == "paths" => {
            // Old config format - write new config and load again
            write_file(&config_path, TEMPLATE_CONFIG_TOML)?;
            Config::load(&config_path)
        }
        Err(e) => Err(e),
    }
}

/// Write CLAUDE.md, handling three cases:
/// 1. File doesn't exist: process and write template
/// 2. File exists without markers: append processed SPOX block
/// 3. File exists with markers: replace content between markers
fn write_claude_md(base_path: &Path) -> Result<()> {
    let claude_md_path = base_path.join("CLAUDE.md");

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
        println!("  .spox/           (config, workflow, standards, spec templates, setup.sh)");
        println!("  .claude/         (agents, commands)");
        println!("  CLAUDE.md        (SPOX block)");
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
        println!("    workflow.md");
        println!("    setup.sh");
        println!("    standards/");
        println!("      backend.md");
        println!("      coding.md");
        println!("      frontend.md");
        println!("      global.md");
        println!("      mcp.md");
        println!("      testing.md");
        println!("      vcs.md");
        println!("    specs/");
        println!("      spec.md");
        println!("      mission.md");
        println!("      change/");
        println!("        proposal.md");
        println!("        tasks.md");
        println!("        design.md");
        println!("        spec.md");
        println!("        verification.md");
        println!("  .claude/");
        println!("    agents/");
        println!("      spox-implementer.md");
        println!("      spox-reviewer.md");
        println!("      spox-verifier.md");
        println!("    commands/spox/");
        println!("      archive.md");
        println!("      implement.md");
        println!("      propose.md");
        println!("      vibe.md");
        println!("  specs/");
        println!("    mission.md");
        println!("    _changes/");
        println!("    _archive/");
        println!("  CLAUDE.md");
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

    #[test]
    fn test_init_creates_spox_directory() {
        let temp = TempDir::new().unwrap();
        let result = run(temp.path());
        assert!(result.is_ok());
        assert!(temp.path().join(".spox").exists());
        assert!(temp.path().join(".spox/config.toml").exists());
        assert!(temp.path().join(".spox/workflow.md").exists());
    }

    #[test]
    fn test_init_creates_standards_directory() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let standards = temp.path().join(".spox/standards");
        assert!(standards.exists());
        assert!(standards.join("backend.md").exists());
        assert!(standards.join("coding.md").exists());
        assert!(standards.join("frontend.md").exists());
        assert!(standards.join("global.md").exists());
        assert!(standards.join("mcp.md").exists());
        assert!(standards.join("testing.md").exists());
        assert!(standards.join("vcs.md").exists());
    }

    #[test]
    fn test_init_creates_specs_templates_directory() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();
        let specs = temp.path().join(".spox/specs");
        assert!(specs.exists());
        assert!(specs.join("spec.md").exists());
        assert!(specs.join("mission.md").exists());
        let change = specs.join("change");
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
        let claude_md = temp.path().join("CLAUDE.md");
        assert!(claude_md.exists());
        let content = fs::read_to_string(&claude_md).unwrap();
        assert!(content.contains(SPOX_START_MARKER));
        assert!(content.contains(SPOX_END_MARKER));
    }

    #[test]
    fn test_write_claude_md_appends_to_existing_without_markers() {
        let temp = TempDir::new().unwrap();
        let claude_md = temp.path().join("CLAUDE.md");

        // Create .spox directory with config.toml (required for write_claude_md)
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML).unwrap();

        // Create existing CLAUDE.md without markers
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
        let claude_md = temp.path().join("CLAUDE.md");

        // Create .spox directory with config.toml (required for write_claude_md)
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML).unwrap();

        // Create existing CLAUDE.md with old SPOX block
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
        let claude_md = temp.path().join("CLAUDE.md");

        // Create .spox directory with config.toml (required for write_claude_md)
        let spox_dir = temp.path().join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(spox_dir.join("config.toml"), TEMPLATE_CONFIG_TOML).unwrap();

        // Create existing CLAUDE.md with content before and after SPOX block
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

        let claude_md = temp.path().join("CLAUDE.md");
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

        let claude_md = temp.path().join("CLAUDE.md");
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
}
