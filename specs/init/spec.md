# init Specification

## Purpose

Define the `spox init` command that bootstraps a new Spec Oxide project or updates an existing one by creating the
directory structure, copying template files for specs/changes/agents/commands, and managing CLAUDE.md configuration.

## Requirements

### Requirement: Project Initialization

The `spox init` command SHALL bootstrap a new Spec Oxide project or update an existing one by creating the required directory structure and copying template files.

#### Scenario: Initialize in current directory

- **WHEN** user runs `spox init` in an empty directory
- **THEN** the command creates `.spox/` directory with config.toml, workflow.md, standards/, and setup.sh
- **AND** creates `.claude/agents/` with spox-prefixed agent templates
- **AND** creates `.claude/commands/spox/` with command templates
- **AND** creates `specs/` directory with mission.md
- **AND** creates `specs/_changes/` directory
- **AND** creates `specs/_archive/` directory
- **AND** exits with code 0
- **AND** prints success message with environment setup hints

#### Scenario: Initialize with custom path

- **WHEN** user runs `spox init /path/to/project`
- **THEN** the command creates the project structure at the specified path
- **AND** creates all directories relative to that path

#### Scenario: Update existing project

- **WHEN** user runs `spox init` in a directory that already has `.spox/`
- **THEN** the command updates agents, commands, standards, and setup.sh to latest templates
- **AND** preserves existing `specs/mission.md`
- **AND** does not modify any files in `specs/`, `specs/_changes/`, or `specs/_archive/`
- **AND** exits with code 0
- **AND** prints update success message with environment setup hints

### Requirement: Template File Copying

The `spox init` command SHALL copy all template files from bundled templates to the project directory, always updating tooling files.

#### Scenario: Standards templates copied

- **WHEN** initialization or update completes successfully
- **THEN** all files from `templates/spox/standards/` exist in `.spox/standards/`
- **AND** file contents match the embedded templates (overwriting existing)

#### Scenario: Workflow template copied

- **WHEN** initialization or update completes successfully
- **THEN** `.spox/workflow.md` exists with content from `templates/spox/workflow.md`

#### Scenario: Claude Code agents copied

- **WHEN** initialization or update completes successfully
- **THEN** all files from `templates/claude/agents/` exist in `.claude/agents/` (flat, not in subfolder)
- **AND** agent files retain their `spox-` prefix (e.g., `spox-implementer.md`)
- **AND** existing spox agent files are overwritten with latest templates

#### Scenario: Claude Code commands copied

- **WHEN** initialization or update completes successfully
- **THEN** all files from `templates/claude/commands/spox/` exist in `.claude/commands/spox/`
- **AND** command files use correct names (`propose.md`, `implement.md`, `archive.md`)
- **AND** existing spox command files are overwritten with latest templates

#### Scenario: Mission template copied only on fresh init

- **WHEN** initialization completes successfully on a fresh project
- **THEN** `specs/mission.md` exists with content from `templates/specs/mission.md`

#### Scenario: Mission template preserved on update

- **WHEN** update runs on existing project with `specs/mission.md`
- **THEN** existing `specs/mission.md` is not modified

#### Scenario: Spec templates copied to .spox

- **WHEN** initialization or update completes successfully
- **THEN** `.spox/specs/` directory exists
- **AND** `.spox/specs/spec.md` exists with content from `templates/specs/spec.md`
- **AND** `.spox/specs/mission.md` exists with content from `templates/specs/mission.md`
- **AND** `.spox/specs/change/` directory exists with all change templates:
    - `.spox/specs/change/proposal.md`
    - `.spox/specs/change/tasks.md`
    - `.spox/specs/change/design.md`
    - `.spox/specs/change/spec.md`
    - `.spox/specs/change/verification.md`

### Requirement: Environment Setup Script

The `spox init` command SHALL copy an interactive setup script to `.spox/setup.sh` that guides users through MCP server installation. The script supports Linux and macOS; Windows users MUST use WSL.

#### Scenario: Setup script copied on fresh init

- **WHEN** initialization completes successfully on a fresh project
- **THEN** `.spox/setup.sh` exists with content from `templates/spox/setup.sh`
- **AND** the file has executable permissions (chmod +x)

#### Scenario: Setup script updated on existing project

- **WHEN** update runs on existing project
- **THEN** `.spox/setup.sh` is overwritten with latest template
- **AND** the file has executable permissions

#### Scenario: Setup script checks prerequisites

- **WHEN** user runs `.spox/setup.sh`
- **THEN** the script checks for `claude` command availability
- **AND** the script checks for `uv` command availability (for Serena)
- **AND** the script checks for `node` command availability (for Context7)
- **AND** missing prerequisites are reported with installation guidance

#### Scenario: Setup script installs Serena MCP interactively

- **WHEN** user runs `.spox/setup.sh` and Serena is not configured
- **THEN** the script asks user for confirmation before installing
- **AND** if confirmed, runs the Serena MCP installation command
- **AND** if declined, skips Serena installation with informational message

#### Scenario: Setup script installs Context7 MCP interactively

- **WHEN** user runs `.spox/setup.sh` and Context7 is not configured
- **THEN** the script asks user for confirmation before installing
- **AND** if confirmed, runs the Context7 MCP installation command
- **AND** if declined, skips Context7 installation with informational message

#### Scenario: Setup script indexes project with Serena

- **WHEN** Serena MCP installation completes successfully
- **THEN** the script asks user for confirmation to index the project
- **AND** if confirmed, runs `uvx --from git+https://github.com/oraios/serena serena project index`
- **AND** provides feedback on indexing progress

### Requirement: Claude Code Detection

The `spox init` command SHALL detect whether Claude Code is available and provide appropriate guidance.

#### Scenario: Claude Code detected

- **WHEN** initialization completes and `claude` command is found in PATH
- **THEN** the success message includes a reminder to run `.spox/setup.sh` for MCP setup

#### Scenario: Claude Code not detected

- **WHEN** initialization completes and `claude` command is not found in PATH
- **THEN** the success message includes a hint about installing Claude Code
- **AND** provides installation guidance (npm install or curl command)
- **AND** includes a reminder to run `.spox/setup.sh` after installing Claude Code

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

### Requirement: CLAUDE.md Template Handling

The `spox init` command SHALL create or update the project's CLAUDE.md file with Spec Oxide instructions.

#### Scenario: Fresh project without CLAUDE.md

- **WHEN** `spox init` runs in a directory without CLAUDE.md
- **THEN** CLAUDE.md is created from the bundled template

#### Scenario: Existing CLAUDE.md without SPOX markers

- **WHEN** `spox init` runs in a directory with CLAUDE.md that lacks SPOX markers
- **THEN** the SPOX block is appended to the end of the existing file
- **AND** existing content is preserved

#### Scenario: Existing CLAUDE.md with SPOX markers

- **WHEN** `spox init` runs in a directory with CLAUDE.md containing `<!-- SPOX:START -->` and `<!-- SPOX:END -->` markers
- **THEN** the content between markers is replaced with the bundled template block
- **AND** content outside the markers is preserved
