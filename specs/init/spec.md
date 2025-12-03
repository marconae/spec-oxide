# init Specification

## Purpose

Define the `spox init` command that bootstraps a new Spec Oxide project by creating the directory structure, copying
template files for specs/changes/agents/commands, and preserving existing Claude Code configuration.

## Requirements

### Requirement: Project Initialization

The `spox init` command SHALL bootstrap a new Spec Oxide project by creating the required directory structure and
copying template files.

#### Scenario: Initialize in current directory

- **WHEN** user runs `spox init` in an empty directory
- **THEN** the command creates `.spox/` directory with config.toml, workflow.md, and standards/
- **AND** creates `.claude/agents/` with spox-prefixed agent templates
- **AND** creates `.claude/commands/spox/` with command templates
- **AND** creates `specs/` directory with mission.md
- **AND** creates `specs/_changes/` directory
- **AND** creates `specs/_archive/` directory
- **AND** exits with code 0
- **AND** prints success message

#### Scenario: Initialize with custom path

- **WHEN** user runs `spox init /path/to/project`
- **THEN** the command creates the project structure at the specified path
- **AND** creates all directories relative to that path

#### Scenario: Initialize already-initialized project

- **WHEN** user runs `spox init` in a directory that already has `.spox/`
- **THEN** the command exits with code 1
- **AND** prints an error message indicating project is already initialized

### Requirement: Template File Copying

The `spox init` command SHALL copy all template files from bundled templates to the project directory.

#### Scenario: Standards templates copied

- **WHEN** initialization completes successfully
- **THEN** all files from `templates/spox/standards/` exist in `.spox/standards/`
- **AND** file contents match the embedded templates

#### Scenario: Workflow template copied

- **WHEN** initialization completes successfully
- **THEN** `.spox/workflow.md` exists with content from `templates/spox/workflow.md`

#### Scenario: Claude Code agents copied

- **WHEN** initialization completes successfully
- **THEN** all files from `templates/claude/agents/` exist in `.claude/agents/` (flat, not in subfolder)
- **AND** agent files retain their `spox-` prefix (e.g., `spox-implementer.md`)

#### Scenario: Claude Code commands copied

- **WHEN** initialization completes successfully
- **THEN** all files from `templates/claude/commands/spox/` exist in `.claude/commands/spox/`
- **AND** command files use correct names (`propose.md`, `implement.md`, `archive.md`)

#### Scenario: Mission template copied

- **WHEN** initialization completes successfully
- **THEN** `specs/mission.md` exists with content from `templates/specs/mission.md`

#### Scenario: Spec templates copied to .spox

- **WHEN** initialization completes successfully
- **THEN** `.spox/specs/` directory exists
- **AND** `.spox/specs/spec.md` exists with content from `templates/specs/spec.md`
- **AND** `.spox/specs/mission.md` exists with content from `templates/specs/mission.md`
- **AND** `.spox/specs/change/` directory exists with all change templates:
    - `.spox/specs/change/proposal.md`
    - `.spox/specs/change/tasks.md`
    - `.spox/specs/change/design.md`
    - `.spox/specs/change/spec.md`
    - `.spox/specs/change/verification.md`

### Requirement: Configuration-Based Directory Creation

The `spox init` command SHALL create spec directories based on the default configuration values.

#### Scenario: Default spec folder created

- **WHEN** initialization completes successfully
- **THEN** the directory specified by `spec_folder` config (default: `specs/`) exists

#### Scenario: Default changes folder created

- **WHEN** initialization completes successfully
- **THEN** the directory specified by `changes_folder` config (default: `specs/_changes`) exists

#### Scenario: Default archive folder created

- **WHEN** initialization completes successfully
- **THEN** the directory specified by `archive_folder` config (default: `specs/_archive`) exists

### Requirement: Claude Integration Preservation

The `spox init` command SHALL NOT overwrite existing Claude Code configuration files.

#### Scenario: Existing Claude agents preserved

- **WHEN** user runs `spox init` and `.claude/agents/` already contains files
- **THEN** existing agent files are not overwritten
- **AND** new spox-prefixed agents are added to `.claude/agents/`

#### Scenario: Existing Claude commands preserved

- **WHEN** user runs `spox init` and `.claude/commands/` already contains files
- **THEN** existing command files are not overwritten
- **AND** new spox commands are added to `.claude/commands/spox/`

