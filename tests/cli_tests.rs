use assert_cmd::Command;
use predicates::prelude::*;

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
