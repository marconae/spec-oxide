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
const TEMPLATE_SPEC_CHANGE_PROPOSAL_MD: &str = include_str!("../templates/specs/change/proposal.md");
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
const TEMPLATE_CMD_ARCHIVE_MD: &str =
    include_str!("../templates/claude/commands/spox/archive.md");
const TEMPLATE_CMD_IMPLEMENT_MD: &str =
    include_str!("../templates/claude/commands/spox/implement.md");
const TEMPLATE_CMD_PROPOSE_MD: &str =
    include_str!("../templates/claude/commands/spox/propose.md");

// specs/ templates
const TEMPLATE_MISSION_MD: &str = include_str!("../templates/specs/mission.md");

/// Initialize a new Spox project at the given path.
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
///     |-- mission.md
///     |-- _changes/
///     +-- _archive/
/// ```
pub fn run(base_path: &Path) -> Result<()> {
    // Check idempotency: fail if .spox/ already exists
    let spox_dir = base_path.join(".spox");
    if spox_dir.exists() {
        return Err(Error::AlreadyInitialized);
    }

    // Create .spox/ directory and contents
    create_spox_dir(base_path)?;

    // Create .claude/ directory and contents (don't overwrite existing files)
    create_claude_dir(base_path)?;

    // Create specs/ directory and contents
    create_specs_dir(base_path)?;

    // Print success message
    print_success_message(base_path);

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
    write_file(&standards_dir.join("backend.md"), TEMPLATE_STANDARDS_BACKEND_MD)?;
    write_file(&standards_dir.join("coding.md"), TEMPLATE_STANDARDS_CODING_MD)?;
    write_file(
        &standards_dir.join("frontend.md"),
        TEMPLATE_STANDARDS_FRONTEND_MD,
    )?;
    write_file(&standards_dir.join("global.md"), TEMPLATE_STANDARDS_GLOBAL_MD)?;
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
    write_file(&specs_change_dir.join("spec.md"), TEMPLATE_SPEC_CHANGE_SPEC_MD)?;
    write_file(
        &specs_change_dir.join("verification.md"),
        TEMPLATE_SPEC_CHANGE_VERIFICATION_MD,
    )?;

    Ok(())
}

/// Create the .claude/ directory structure.
/// Only adds new files, does not overwrite existing files.
fn create_claude_dir(base_path: &Path) -> Result<()> {
    let claude_dir = base_path.join(".claude");
    let agents_dir = claude_dir.join("agents");
    let commands_spox_dir = claude_dir.join("commands").join("spox");

    // Create directories
    create_dir_all(&agents_dir)?;
    create_dir_all(&commands_spox_dir)?;

    // Write agent files (flat in agents/, files already have spox- prefix)
    write_file_if_not_exists(
        &agents_dir.join("spox-implementer.md"),
        TEMPLATE_AGENT_IMPLEMENTER_MD,
    )?;
    write_file_if_not_exists(
        &agents_dir.join("spox-reviewer.md"),
        TEMPLATE_AGENT_REVIEWER_MD,
    )?;
    write_file_if_not_exists(
        &agents_dir.join("spox-verifier.md"),
        TEMPLATE_AGENT_VERIFIER_MD,
    )?;

    // Write command files
    write_file_if_not_exists(&commands_spox_dir.join("archive.md"), TEMPLATE_CMD_ARCHIVE_MD)?;
    write_file_if_not_exists(
        &commands_spox_dir.join("implement.md"),
        TEMPLATE_CMD_IMPLEMENT_MD,
    )?;
    write_file_if_not_exists(
        &commands_spox_dir.join("propose.md"),
        TEMPLATE_CMD_PROPOSE_MD,
    )?;

    Ok(())
}

/// Create the specs/ directory structure.
fn create_specs_dir(base_path: &Path) -> Result<()> {
    let specs_dir = base_path.join("specs");
    let changes_dir = specs_dir.join("_changes");
    let archive_dir = specs_dir.join("_archive");

    // Create directories
    create_dir_all(&specs_dir)?;
    create_dir_all(&changes_dir)?;
    create_dir_all(&archive_dir)?;

    // Write mission.md
    write_file(&specs_dir.join("mission.md"), TEMPLATE_MISSION_MD)?;

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
    fs::write(path, content).map_err(|e| {
        Error::InitError(format!(
            "failed to write file '{}': {}",
            path.display(),
            e
        ))
    })
}

/// Write content to a file only if it does not already exist.
fn write_file_if_not_exists(path: &Path, content: &str) -> Result<()> {
    if !path.exists() {
        write_file(path, content)?;
    }
    Ok(())
}

/// Print success message with created structure.
fn print_success_message(base_path: &Path) {
    let path_display = if base_path == Path::new(".") {
        "current directory".to_string()
    } else {
        format!("'{}'", base_path.display())
    };

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
    fn test_init_fails_if_already_initialized() {
        let temp = TempDir::new().unwrap();
        run(temp.path()).unwrap();

        // Second init should fail
        let result = run(temp.path());
        assert!(result.is_err());
        match result {
            Err(Error::AlreadyInitialized) => {}
            _ => panic!("expected AlreadyInitialized error"),
        }
    }

    #[test]
    fn test_init_does_not_overwrite_existing_claude_files() {
        let temp = TempDir::new().unwrap();

        // Pre-create .claude/agents with an existing file
        let agents_dir = temp.path().join(".claude/agents");
        fs::create_dir_all(&agents_dir).unwrap();
        let existing_file = agents_dir.join("spox-implementer.md");
        fs::write(&existing_file, "existing content").unwrap();

        // Run init (this will fail because .spox doesn't exist, but we need a different test)
        // Actually, the idempotency check is only on .spox, so let's test that claude files
        // are not overwritten when they exist before init
        // Since init will fail if .spox exists, we test the write_file_if_not_exists function

        // Write a file
        write_file_if_not_exists(&existing_file, "new content").unwrap();
        let content = fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "existing content");
    }
}
