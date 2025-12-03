# config Specification

## Purpose

Define how Spec Oxide reads and displays project configuration from `.spox/config.toml`, including TOML parsing,
required field validation, and the `spox config show` command.

## Requirements

### Requirement: Configuration Display

The `spox config show` command SHALL display the current project configuration.

#### Scenario: Display configuration

- **WHEN** user runs `spox config show` in an initialized project
- **THEN** the command prints the configuration values from `.spox/config.toml`
- **AND** displays `spec_folder`, `changes_folder`, and `archive_folder` values
- **AND** exits with code 0

#### Scenario: No configuration found

- **WHEN** user runs `spox config show` in a directory without `.spox/config.toml`
- **THEN** the command exits with code 1
- **AND** prints an error message indicating no configuration found

### Requirement: Configuration Parsing

The CLI SHALL parse TOML configuration from `.spox/config.toml`.

#### Scenario: Valid configuration parsed

- **WHEN** `.spox/config.toml` contains valid TOML with required fields
- **THEN** the configuration is successfully loaded
- **AND** folder paths are available for use by other commands

#### Scenario: Invalid configuration format

- **WHEN** `.spox/config.toml` contains invalid TOML syntax
- **THEN** the CLI reports a parse error with file location
- **AND** exits with code 1

#### Scenario: Missing required fields

- **WHEN** `.spox/config.toml` is missing required fields
- **THEN** the CLI reports which fields are missing
- **AND** exits with code 1

