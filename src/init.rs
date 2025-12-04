//! Init command implementation for bootstrapping a new Spox project.

use crate::error::{Error, Result};
use std::fs;
use std::path::Path;

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
/// |   |-- standards/
/// |   |   |-- backend.md
/// |   |   |-- coding.md
/// |   |   |-- frontend.md
/// |   |   |-- global.md
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
/// |       +-- propose.md
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
    let specs_dir = spox_dir.join("specs");
    let specs_change_dir = specs_dir.join("change");

    // Create directories
    create_dir_all(&standards_dir)?;
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
        Error::InitError(format!(
            "failed to create directory '{}': {}",
            path.display(),
            e
        ))
    })
}

/// Write content to a file, creating parent directories if needed.
fn write_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)
        .map_err(|e| Error::InitError(format!("failed to write file '{}': {}", path.display(), e)))
}

/// Write content to a file only if it does not already exist.
fn write_file_if_not_exists(path: &Path, content: &str) -> Result<()> {
    if !path.exists() {
        write_file(path, content)?;
    }
    Ok(())
}

const SPOX_START_MARKER: &str = "<!-- SPOX:START -->";
const SPOX_END_MARKER: &str = "<!-- SPOX:END -->";

/// Write CLAUDE.md, handling three cases:
/// 1. File doesn't exist: copy template directly
/// 2. File exists without markers: append SPOX block
/// 3. File exists with markers: replace content between markers
fn write_claude_md(base_path: &Path) -> Result<()> {
    let claude_md_path = base_path.join("CLAUDE.md");

    if !claude_md_path.exists() {
        // Case 1: Fresh project - copy template directly
        return write_file(&claude_md_path, TEMPLATE_CLAUDE_MD);
    }

    // File exists - read current content
    let existing_content = fs::read_to_string(&claude_md_path).map_err(|e| {
        Error::InitError(format!(
            "failed to read '{}': {}",
            claude_md_path.display(),
            e
        ))
    })?;

    let new_content = if existing_content.contains(SPOX_START_MARKER)
        && existing_content.contains(SPOX_END_MARKER)
    {
        // Case 3: Replace content between markers
        replace_spox_block(&existing_content, TEMPLATE_CLAUDE_MD)
    } else {
        // Case 2: Append SPOX block to end
        let mut content = existing_content;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(TEMPLATE_CLAUDE_MD);
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
        println!("  .spox/           (config, workflow, standards, spec templates)");
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
        println!("    standards/");
        println!("      backend.md");
        println!("      coding.md");
        println!("      frontend.md");
        println!("      global.md");
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
        println!("  specs/");
        println!("    mission.md");
        println!("    _changes/");
        println!("    _archive/");
        println!("  CLAUDE.md");
    }
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
        assert!(result.contains("## Spec Oxide Instructions"));
    }

    #[test]
    fn test_write_claude_md_preserves_content_outside_markers() {
        let temp = TempDir::new().unwrap();
        let claude_md = temp.path().join("CLAUDE.md");

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
}
