# cli Specification

## Purpose

Define the command-line interface for Spec Oxide (`spox`), including help and version output, subcommand structure using
a resource-verb pattern, and commands for listing, showing, and validating specs and changes.

## Requirements

### Requirement: Help output

The CLI SHALL display usage information when invoked with --help or no arguments.

#### Scenario: User requests help

- WHEN `spox --help` is executed
- THEN print usage information to stdout
- AND include the command pattern `spox <command> [options]`
- AND list available subcommands: init, spec, change, config
- AND exit with code 0

#### Scenario: User invokes without arguments

- WHEN `spox` is executed without arguments
- THEN print usage information to stdout
- AND exit with code 0

### Requirement: Version output

The CLI SHALL display version information when invoked with --version.

#### Scenario: User requests version

- WHEN `spox --version` is executed
- THEN print the version from Cargo.toml to stdout
- AND exit with code 0

### Requirement: Unknown command handling

The CLI SHALL reject unknown commands with helpful feedback.

#### Scenario: User enters unknown command

- WHEN `spox unknown-command` is executed
- THEN print an error message to stderr
- AND include a usage hint
- AND exit with non-zero code

### Requirement: Subcommand structure

The CLI SHALL accept the resource-verb pattern for operations.

#### Scenario: Spec subcommand exists

- WHEN `spox spec --help` is executed
- THEN print spec subcommand usage
- AND list actions: list, show, validate
- AND exit with code 0

#### Scenario: Change subcommand exists

- WHEN `spox change --help` is executed
- THEN print change subcommand usage
- AND list actions: init, list, show, validate, approve
- AND exit with code 0

#### Scenario: Config subcommand exists

- WHEN `spox config --help` is executed
- THEN print config subcommand usage
- AND list actions: show
- AND exit with code 0

#### Scenario: Spec validate accepts optional ID

- **WHEN** `spox spec validate` is executed without arguments
- **THEN** validate all specs in the spec folder
- **AND** print validation results for each spec
- **AND** exit with code 0 if all pass, code 1 if any fail

#### Scenario: Spec validate accepts specific ID

- **WHEN** `spox spec validate <id>` is executed with a spec ID
- **THEN** validate only the specified spec
- **AND** print validation results
- **AND** exit with code 0 if valid, code 1 if invalid

#### Scenario: Change validate accepts optional ID

- **WHEN** `spox change validate` is executed without arguments
- **THEN** validate all changes in the changes folder
- **AND** print validation results for each change
- **AND** exit with code 0 if all pass, code 1 if any fail

#### Scenario: Change validate accepts specific ID

- **WHEN** `spox change validate <id>` is executed with a change ID
- **THEN** validate only the specified change
- **AND** print validation results
- **AND** exit with code 0 if valid, code 1 if invalid

#### Scenario: Top-level show command exists

- **WHEN** `spox show` is executed
- **THEN** display the project dashboard
- **AND** exit with code 0

#### Scenario: Change show accepts deltas-only flag

- **WHEN** `spox change show <id> --deltas-only` is executed
- **THEN** display only delta requirements
- **AND** exit with code 0

### Requirement: Spec List Command

The CLI SHALL list all specs with their requirement counts when `spox spec list` is executed.

#### Scenario: List specs in initialized project

- **WHEN** `spox spec list` is executed in an initialized project
- **THEN** print "Specs:" as the first line
- **AND** print each spec on a separate line prefixed with "- "
- **AND** each line shows the spec name and requirement count
- **AND** format is `- {name}  {count} requirements` with aligned columns
- **AND** output has no ANSI color codes (AI-friendly)
- **AND** exit with code 0

#### Scenario: List specs with no specs

- **WHEN** `spox spec list` is executed and no specs exist
- **THEN** print "No specs found."
- **AND** exit with code 0

#### Scenario: List specs without initialization

- **WHEN** `spox spec list` is executed without `.spox/config.toml`
- **THEN** print an error message to stderr
- **AND** exit with code 1

### Requirement: Change List Command

The CLI SHALL list all active changes with their task progress when `spox change list` is executed.

#### Scenario: List changes in initialized project

- **WHEN** `spox change list` is executed in an initialized project
- **THEN** print "Changes:" as the first line
- **AND** print each active change on a separate line prefixed with "- "
- **AND** each line shows the change name and task progress
- **AND** format is `- {name}  {completed}/{total} tasks` with aligned columns
- **AND** output has no ANSI color codes (AI-friendly)
- **AND** exit with code 0

#### Scenario: List changes with no active changes

- **WHEN** `spox change list` is executed and no active changes exist
- **THEN** print "No active changes."
- **AND** exit with code 0

#### Scenario: List changes without initialization

- **WHEN** `spox change list` is executed without `.spox/config.toml`
- **THEN** print an error message to stderr
- **AND** exit with code 1

### Requirement: Dashboard Command

The CLI SHALL provide a top-level `spox show` command that displays a project overview dashboard.

#### Scenario: Display dashboard

- **WHEN** `spox show` is executed
- **THEN** display a dashboard with colored output
- **AND** show all specs with their requirement counts
- **AND** show all active changes with task progress bars
- **AND** show delta summaries for each change (added/modified/removed counts)
- **AND** exit with code 0

#### Scenario: Dashboard without active changes

- **WHEN** `spox show` is executed with no active changes
- **THEN** display specs section
- **AND** show "No active changes" message
- **AND** exit with code 0

### Requirement: Spec Show Command

The CLI SHALL provide `spox spec show <id>` to display a spec's content.

#### Scenario: Show specific spec

- **WHEN** `spox spec show <id>` is executed with a valid spec ID
- **THEN** display the spec's purpose section
- **AND** display each requirement with its description
- **AND** list scenarios under each requirement
- **AND** use colored output for readability
- **AND** exit with code 0

#### Scenario: Spec not found

- **WHEN** `spox spec show <id>` is executed with an invalid spec ID
- **THEN** print an error message indicating spec not found
- **AND** exit with code 1

### Requirement: Change Show Command

The CLI SHALL provide `spox change show <id>` to display a change proposal.

#### Scenario: Show specific change

- **WHEN** `spox change show <id>` is executed with a valid change ID
- **THEN** display the proposal's Why section
- **AND** display the proposal's What Changes section
- **AND** display task completion with progress bar
- **AND** display deltas grouped by capability
- **AND** use colored output for readability
- **AND** exit with code 0

#### Scenario: Show change deltas only

- **WHEN** `spox change show <id> --deltas-only` is executed
- **THEN** display only the delta requirements
- **AND** group by capability
- **AND** show operation type (ADDED/MODIFIED/REMOVED/RENAMED)
- **AND** show requirement names and descriptions
- **AND** exit with code 0

#### Scenario: Change not found

- **WHEN** `spox change show <id>` is executed with an invalid change ID
- **THEN** print an error message indicating change not found
- **AND** exit with code 1

