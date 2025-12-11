use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper function to create a Command for the spox binary.
fn spox_cmd() -> Command {
    cargo_bin_cmd!("spox")
}

// =============================================================================
// Help Output Tests
// =============================================================================

#[test]
fn test_help_flag_succeeds() {
    spox_cmd().arg("--help").assert().success();
}

#[test]
fn test_help_flag_contains_spox() {
    spox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("spox"));
}

#[test]
fn test_help_flag_contains_init_subcommand() {
    spox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"));
}

#[test]
fn test_help_flag_contains_spec_subcommand() {
    spox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"));
}

#[test]
fn test_help_flag_contains_change_subcommand() {
    spox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("change"));
}

#[test]
fn test_help_flag_contains_config_subcommand() {
    spox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("config"));
}

#[test]
fn test_help_flag_contains_all_key_subcommands() {
    spox_cmd().arg("--help").assert().success().stdout(
        predicate::str::contains("init")
            .and(predicate::str::contains("spec"))
            .and(predicate::str::contains("change"))
            .and(predicate::str::contains("config")),
    );
}

// =============================================================================
// Version Output Tests
// =============================================================================

#[test]
fn test_version_flag_succeeds() {
    spox_cmd().arg("--version").assert().success();
}

#[test]
fn test_version_flag_contains_version_number() {
    spox_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.2.0"));
}

#[test]
fn test_version_flag_contains_spox_name() {
    spox_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("spox"));
}

// =============================================================================
// Unknown Command Handling Tests
// =============================================================================

#[test]
fn test_unknown_command_fails() {
    spox_cmd().arg("unknown-command").assert().failure();
}

#[test]
fn test_unknown_command_has_nonzero_exit_code() {
    spox_cmd()
        .arg("unknown-command")
        .assert()
        .code(predicate::ne(0));
}

#[test]
fn test_unknown_command_stderr_contains_error() {
    spox_cmd()
        .arg("unknown-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_unknown_command_stderr_mentions_unrecognized() {
    spox_cmd()
        .arg("unknown-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized"));
}

// =============================================================================
// No Arguments Shows Help Tests
// =============================================================================

#[test]
fn test_no_arguments_succeeds() {
    spox_cmd().assert().success();
}

#[test]
fn test_no_arguments_shows_usage_info() {
    spox_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_no_arguments_shows_commands() {
    spox_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands"));
}

#[test]
fn test_no_arguments_shows_options() {
    spox_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Options"));
}

#[test]
fn test_no_arguments_shows_help_option() {
    spox_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("--help"));
}

// =============================================================================
// Init Command Integration Tests
// =============================================================================

#[test]
fn test_init_creates_project_structure() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Spox project"));

    // Verify .spox/ directory was created
    assert!(temp_dir.path().join(".spox").exists());
    assert!(temp_dir.path().join(".spox").is_dir());
}

#[test]
fn test_init_with_custom_path() {
    let temp_dir = TempDir::new().unwrap();
    let custom_path = temp_dir.path().join("my-project");
    std::fs::create_dir_all(&custom_path).unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(&custom_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Spox project"));

    // Verify structure was created at custom path
    assert!(custom_path.join(".spox").exists());
    assert!(custom_path.join(".spox/config.toml").exists());
    assert!(custom_path.join("specs").exists());
}

#[test]
fn test_reinit_succeeds() {
    let temp_dir = TempDir::new().unwrap();

    // First init succeeds
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Second init should also succeed (not fail)
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_reinit_preserves_mission_md() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Modify specs/mission.md with custom content
    let mission_path = temp_dir.path().join("specs/mission.md");
    let custom_content = "# My Custom Mission\n\nThis is my custom project mission.\n";
    std::fs::write(&mission_path, custom_content).unwrap();

    // Second init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify specs/mission.md still has custom content
    let content = std::fs::read_to_string(&mission_path).unwrap();
    assert_eq!(content, custom_content);
}

#[test]
fn test_reinit_overwrites_agents() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Modify .claude/agents/spox-implementer.md with custom content
    let agent_path = temp_dir.path().join(".claude/agents/spox-implementer.md");
    let original_content = std::fs::read_to_string(&agent_path).unwrap();
    let custom_content = "# Custom Agent\n\nThis has been modified.\n";
    std::fs::write(&agent_path, custom_content).unwrap();

    // Second init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify .claude/agents/spox-implementer.md has been reset to template
    let content = std::fs::read_to_string(&agent_path).unwrap();
    assert_ne!(content, custom_content);
    assert_eq!(content, original_content);
}

#[test]
fn test_reinit_overwrites_commands() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Modify .claude/commands/spox/implement.md with custom content
    let command_path = temp_dir.path().join(".claude/commands/spox/implement.md");
    let original_content = std::fs::read_to_string(&command_path).unwrap();
    let custom_content = "# Custom Command\n\nThis has been modified.\n";
    std::fs::write(&command_path, custom_content).unwrap();

    // Second init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify .claude/commands/spox/implement.md has been reset to template
    let content = std::fs::read_to_string(&command_path).unwrap();
    assert_ne!(content, custom_content);
    assert_eq!(content, original_content);
}

#[test]
fn test_reinit_does_not_touch_specs_directory() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Create custom file in specs/_changes/my-change/proposal.md
    let change_dir = temp_dir.path().join("specs/_changes/my-change");
    std::fs::create_dir_all(&change_dir).unwrap();
    let proposal_path = change_dir.join("proposal.md");
    let custom_content = "# My Change Proposal\n\nThis is a custom proposal.\n";
    std::fs::write(&proposal_path, custom_content).unwrap();

    // Second init
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify custom file still exists with original content
    assert!(proposal_path.exists());
    let content = std::fs::read_to_string(&proposal_path).unwrap();
    assert_eq!(content, custom_content);
}

#[test]
fn test_init_creates_spox_config_file() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let config_path = temp_dir.path().join(".spox/config.toml");
    assert!(config_path.exists());
    assert!(config_path.is_file());

    // Verify config file has content
    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("spec_folder"));
    assert!(content.contains("changes_folder"));
    assert!(content.contains("archive_folder"));
}

#[test]
fn test_init_creates_claude_agents_directory() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let agents_dir = temp_dir.path().join(".claude/agents");
    assert!(agents_dir.exists());
    assert!(agents_dir.is_dir());

    // Verify all 3 agent files exist with spox- prefix
    assert!(agents_dir.join("spox-implementer.md").exists());
    assert!(agents_dir.join("spox-reviewer.md").exists());
    assert!(agents_dir.join("spox-verifier.md").exists());
}

#[test]
fn test_init_creates_claude_commands_directory() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let commands_dir = temp_dir.path().join(".claude/commands/spox");
    assert!(commands_dir.exists());
    assert!(commands_dir.is_dir());

    // Verify all 3 command files exist
    assert!(commands_dir.join("archive.md").exists());
    assert!(commands_dir.join("implement.md").exists());
    assert!(commands_dir.join("propose.md").exists());
}

#[test]
fn test_init_creates_specs_directory() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let specs_dir = temp_dir.path().join("specs");
    assert!(specs_dir.exists());
    assert!(specs_dir.is_dir());

    // Verify mission.md exists
    assert!(specs_dir.join("mission.md").exists());

    // Verify _changes and _archive directories exist
    assert!(specs_dir.join("_changes").exists());
    assert!(specs_dir.join("_changes").is_dir());
    assert!(specs_dir.join("_archive").exists());
    assert!(specs_dir.join("_archive").is_dir());
}

#[test]
fn test_init_creates_spox_templates_directory() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let templates_dir = temp_dir.path().join(".spox/templates");
    assert!(templates_dir.exists());
    assert!(templates_dir.is_dir());

    // Verify spec template file exists
    assert!(templates_dir.join("spec.md").exists());

    // Verify change subdirectory and its files
    let change_dir = templates_dir.join("change");
    assert!(change_dir.exists());
    assert!(change_dir.is_dir());
    assert!(change_dir.join("proposal.md").exists());
    assert!(change_dir.join("tasks.md").exists());
    assert!(change_dir.join("design.md").exists());
    assert!(change_dir.join("spec.md").exists());
    assert!(change_dir.join("verification.md").exists());
}

#[test]
fn test_init_verifies_all_files_copied_correctly() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // .spox/ files
    assert!(temp_dir.path().join(".spox/config.toml").exists());
    assert!(temp_dir.path().join(".spox/setup.sh").exists());
    assert!(temp_dir.path().join(".spox/custom").is_dir());

    // .spox/templates/ files (1 file + change subdirectory with 5 files)
    assert!(temp_dir.path().join(".spox/templates/spec.md").exists());
    assert!(temp_dir
        .path()
        .join(".spox/templates/change/proposal.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".spox/templates/change/tasks.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".spox/templates/change/design.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".spox/templates/change/spec.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".spox/templates/change/verification.md")
        .exists());

    // .claude/agents/ files (3 files with spox- prefix)
    assert!(temp_dir
        .path()
        .join(".claude/agents/spox-implementer.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".claude/agents/spox-reviewer.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".claude/agents/spox-verifier.md")
        .exists());

    // .claude/commands/spox/ files (5 files)
    assert!(temp_dir
        .path()
        .join(".claude/commands/spox/archive.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".claude/commands/spox/implement.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".claude/commands/spox/propose.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".claude/commands/spox/setup.md")
        .exists());
    assert!(temp_dir
        .path()
        .join(".claude/commands/spox/vibe.md")
        .exists());

    // specs/ structure
    assert!(temp_dir.path().join("specs/mission.md").exists());
    assert!(temp_dir.path().join("specs/_changes").is_dir());
    assert!(temp_dir.path().join("specs/_archive").is_dir());
}

#[test]
fn test_init_output_shows_created_structure() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(".spox/"))
        .stdout(predicate::str::contains("config.toml"))
        .stdout(predicate::str::contains("setup.sh"))
        .stdout(predicate::str::contains("custom/"))
        .stdout(predicate::str::contains("templates/"))
        .stdout(predicate::str::contains(".claude/"))
        .stdout(predicate::str::contains("agents/"))
        .stdout(predicate::str::contains("commands/spox/"))
        .stdout(predicate::str::contains("specs/"))
        .stdout(predicate::str::contains("mission.md"))
        .stdout(predicate::str::contains("_changes/"))
        .stdout(predicate::str::contains("_archive/"));
}

#[test]
fn test_init_creates_claude_md() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify CLAUDE.md exists in .claude directory
    let claude_md_path = temp_dir.path().join(".claude/CLAUDE.md");
    assert!(claude_md_path.exists());
    assert!(claude_md_path.is_file());

    // Verify content contains SPOX markers
    let content = std::fs::read_to_string(&claude_md_path).unwrap();
    assert!(content.contains("<!-- SPOX:START -->"));
    assert!(content.contains("<!-- SPOX:END -->"));
}

#[test]
fn test_init_appends_spox_block_to_existing_claude_md() {
    let temp_dir = TempDir::new().unwrap();

    // Create .claude directory and existing CLAUDE.md without SPOX markers
    let claude_dir = temp_dir.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    let claude_md_path = claude_dir.join("../.claude/CLAUDE.md");
    let original_content = "# My Project\n\nCustom instructions.\n";
    std::fs::write(&claude_md_path, original_content).unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify original content is preserved at the start
    let content = std::fs::read_to_string(&claude_md_path).unwrap();
    assert!(content.starts_with("# My Project"));
    assert!(content.contains("Custom instructions."));

    // Verify SPOX markers were added after original content
    assert!(content.contains("<!-- SPOX:START -->"));
    assert!(content.contains("<!-- SPOX:END -->"));

    // Verify SPOX block appears after original content
    let original_end = content.find("Custom instructions.").unwrap();
    let spox_start = content.find("<!-- SPOX:START -->").unwrap();
    assert!(spox_start > original_end);
}

#[test]
fn test_init_replaces_spox_block_in_existing_claude_md() {
    let temp_dir = TempDir::new().unwrap();

    // Create .claude directory and existing CLAUDE.md with old SPOX block
    let claude_dir = temp_dir.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    let claude_md_path = claude_dir.join("../.claude/CLAUDE.md");
    let existing_content =
        "# Header\n\n<!-- SPOX:START -->\nOld content\n<!-- SPOX:END -->\n\n# Footer\n";
    std::fs::write(&claude_md_path, existing_content).unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(&claude_md_path).unwrap();

    // Verify header is preserved
    assert!(content.contains("# Header"));

    // Verify footer is preserved
    assert!(content.contains("# Footer"));

    // Verify old content between markers was replaced
    assert!(!content.contains("Old content"));

    // Verify SPOX:START appears exactly once (block was replaced, not appended)
    let start_marker_count = content.matches("<!-- SPOX:START -->").count();
    assert_eq!(
        start_marker_count, 1,
        "Expected exactly 1 SPOX:START marker, found {}",
        start_marker_count
    );
}

// =============================================================================
// Config Show Command Integration Tests
// =============================================================================

#[test]
fn test_config_show_displays_configuration() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize first
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Then config show (needs to run from that directory)
    spox_cmd()
        .current_dir(temp_dir.path())
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec_folder"));
}

#[test]
fn test_config_show_displays_all_config_fields() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize first
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify all config fields are displayed
    spox_cmd()
        .current_dir(temp_dir.path())
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec_folder"))
        .stdout(predicate::str::contains("changes_folder"))
        .stdout(predicate::str::contains("archive_folder"));
}

#[test]
fn test_config_show_fails_without_initialization() {
    let temp_dir = TempDir::new().unwrap();

    // Running config show without init should fail
    spox_cmd()
        .current_dir(temp_dir.path())
        .arg("config")
        .arg("show")
        .assert()
        .failure()
        .stderr(predicate::str::contains("config not found"));
}

#[test]
fn test_config_show_displays_header() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize first
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify header is displayed
    spox_cmd()
        .current_dir(temp_dir.path())
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration"));
}
