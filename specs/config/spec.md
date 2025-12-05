# config Specification

## Purpose

Define how Spec Oxide reads and displays project configuration from `.spox/config.toml`, including TOML parsing,
required field validation, and the `spox config show` command.

## Requirements

### Requirement: Paths Configuration Section

The configuration SHALL use a `[paths]` section for directory path settings.

#### Scenario: Paths section parsed

- **WHEN** `.spox/config.toml` contains a `[paths]` section
- **THEN** the configuration is successfully loaded
- **AND** `spec_folder`, `changes_folder`, and `archive_folder` are read from `[paths]`

### Requirement: Rules Configuration

The configuration SHALL support a `[rules]` section defining which standards templates and custom rule files to merge into `.claude/CLAUDE.md`.

#### Scenario: Valid rules configuration parsed

- **WHEN** `.spox/config.toml` contains a `[rules]` section with `system` array
- **THEN** the configuration is successfully loaded
- **AND** `system` array contains template names (without `.md` extension)
- **AND** templates are resolved from `.spox/standards/` directory

#### Scenario: Custom rules configuration parsed

- **WHEN** `.spox/config.toml` contains `[rules]` with optional `custom` array
- **THEN** custom file names are parsed
- **AND** files are resolved from `.spox/custom/` directory

#### Scenario: Empty system array rejected

- **WHEN** `.spox/config.toml` has `[rules]` with empty `system` array
- **THEN** the CLI reports validation error
- **AND** error message indicates system templates are required
- **AND** exits with code 1

#### Scenario: Invalid system template name rejected

- **WHEN** `.spox/config.toml` references a non-existent system template
- **THEN** the CLI reports validation error
- **AND** error message indicates which template is invalid
- **AND** exits with code 1

#### Scenario: Missing custom file handled gracefully

- **WHEN** `.spox/config.toml` references a custom file that does not exist
- **THEN** the CLI reports a warning
- **AND** continues processing other files
- **AND** the missing file is skipped in CLAUDE.md generation

### Requirement: Configuration Display

The `spox config show` command SHALL display the current project configuration with optional filtering.

#### Scenario: Display full configuration

- **WHEN** user runs `spox config show` in an initialized project
- **THEN** the command prints all configuration sections from `.spox/config.toml`
- **AND** displays `[paths]` section with `spec_folder`, `changes_folder`, and `archive_folder`
- **AND** displays `[rules]` section with `system` and `custom` arrays
- **AND** exits with code 0

#### Scenario: Display paths only

- **WHEN** user runs `spox config show --paths` in an initialized project
- **THEN** the command prints only the `[paths]` section
- **AND** displays `spec_folder`, `changes_folder`, and `archive_folder` values
- **AND** does NOT display `[rules]` section
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
