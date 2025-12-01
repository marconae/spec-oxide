use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper function to create a Command for the spox binary.
fn spox_cmd() -> Command {
    Command::cargo_bin("spox").unwrap()
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
        .stdout(predicate::str::contains("0.1.0"));
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
fn test_init_fails_if_already_initialized() {
    let temp_dir = TempDir::new().unwrap();

    // First init succeeds
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Second init fails with appropriate error message
    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
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
fn test_init_creates_spox_workflow_file() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let workflow_path = temp_dir.path().join(".spox/workflow.md");
    assert!(workflow_path.exists());
    assert!(workflow_path.is_file());
}

#[test]
fn test_init_creates_standards_directory() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    let standards_dir = temp_dir.path().join(".spox/standards");
    assert!(standards_dir.exists());
    assert!(standards_dir.is_dir());

    // Verify all 6 standards files exist
    assert!(standards_dir.join("backend.md").exists());
    assert!(standards_dir.join("coding.md").exists());
    assert!(standards_dir.join("frontend.md").exists());
    assert!(standards_dir.join("global.md").exists());
    assert!(standards_dir.join("testing.md").exists());
    assert!(standards_dir.join("vcs.md").exists());
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
    assert!(commands_dir.join("orchestrate.md").exists());
    assert!(commands_dir.join("proposal.md").exists());
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
fn test_init_verifies_all_files_copied_correctly() {
    let temp_dir = TempDir::new().unwrap();

    spox_cmd()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // .spox/ files (2 files)
    assert!(temp_dir.path().join(".spox/config.toml").exists());
    assert!(temp_dir.path().join(".spox/workflow.md").exists());

    // .spox/standards/ files (6 files)
    assert!(temp_dir.path().join(".spox/standards/backend.md").exists());
    assert!(temp_dir.path().join(".spox/standards/coding.md").exists());
    assert!(temp_dir.path().join(".spox/standards/frontend.md").exists());
    assert!(temp_dir.path().join(".spox/standards/global.md").exists());
    assert!(temp_dir.path().join(".spox/standards/testing.md").exists());
    assert!(temp_dir.path().join(".spox/standards/vcs.md").exists());

    // .claude/agents/ files (3 files with spox- prefix)
    assert!(temp_dir.path().join(".claude/agents/spox-implementer.md").exists());
    assert!(temp_dir.path().join(".claude/agents/spox-reviewer.md").exists());
    assert!(temp_dir.path().join(".claude/agents/spox-verifier.md").exists());

    // .claude/commands/spox/ files (3 files)
    assert!(temp_dir.path().join(".claude/commands/spox/archive.md").exists());
    assert!(temp_dir.path().join(".claude/commands/spox/orchestrate.md").exists());
    assert!(temp_dir.path().join(".claude/commands/spox/proposal.md").exists());

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
        .stdout(predicate::str::contains("workflow.md"))
        .stdout(predicate::str::contains("standards/"))
        .stdout(predicate::str::contains(".claude/"))
        .stdout(predicate::str::contains("agents/"))
        .stdout(predicate::str::contains("commands/spox/"))
        .stdout(predicate::str::contains("specs/"))
        .stdout(predicate::str::contains("mission.md"))
        .stdout(predicate::str::contains("_changes/"))
        .stdout(predicate::str::contains("_archive/"));
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
