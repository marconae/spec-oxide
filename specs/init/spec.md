# init Specification

## Purpose

Define the `spox init` command that bootstraps a new Spec Oxide project or updates an existing one by creating the
directory structure, copying template files for specs/changes/agents/commands, and managing CLAUDE.md configuration.

## Requirements

### Requirement: Project Initialization

The `spox init` command SHALL bootstrap a new Spec Oxide project or update an existing one by creating the required directory structure and copying template files.

#### Scenario: Initialize in current directory

- **WHEN** user runs `spox init` in an empty directory
- **THEN** the command creates `.spox/` directory with config.toml, setup.sh, and custom/
- **AND** creates `.spox/.gitignore` with rules to exclude spox-managed files
- **AND** creates `.claude/agents/` with spox-prefixed agent templates
- **AND** creates `.claude/commands/spox/` with command templates
- **AND** creates `.claude/.gitignore` with rules to exclude spox-managed files
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
- **THEN** the command updates agents, commands, and setup.sh to latest templates
- **AND** appends missing rules to `.spox/.gitignore` without duplicating existing rules
- **AND** appends missing rules to `.claude/.gitignore` without duplicating existing rules
- **AND** preserves existing `specs/mission.md`
- **AND** preserves existing `.spox/custom/` directory and contents
- **AND** does not modify any files in `specs/`, `specs/_changes/`, or `specs/_archive/`
- **AND** exits with code 0
- **AND** prints update success message with environment setup hints

### Requirement: Custom Rules Directory

The `spox init` command SHALL create and preserve the `.spox/custom/` directory for user-defined rule files.

#### Scenario: Custom directory created on fresh init

- **WHEN** `spox init` runs on a fresh project
- **THEN** `.spox/custom/` directory is created
- **AND** the directory is empty

#### Scenario: Custom directory preserved on update

- **WHEN** `spox init` runs on an existing project with `.spox/custom/`
- **THEN** the directory and its contents are preserved
- **AND** no files in `.spox/custom/` are modified or deleted

### Requirement: Configuration Migration

The `spox init` command SHALL migrate existing configurations to the new format with `[rules]` section.

#### Scenario: Missing rules section added on update

- **WHEN** `spox init` runs on existing project without `[rules]` in config.toml
- **THEN** the `[rules]` section is added to config.toml
- **AND** `system` array contains all available templates by default
- **AND** existing configuration values are preserved

### Requirement: Template File Copying

The `spox init` command SHALL copy Claude Code agent and command templates from bundled templates to the project directory, always updating tooling files. Spec and standards templates are bundled in the binary and NOT copied to the filesystem.

#### Scenario: Claude Code agents copied

- **WHEN** `spox init` is run
- **THEN** all agent templates from `templates/claude/agents/` SHALL be copied to `.claude/agents/`

#### Scenario: Claude Code commands copied

- **WHEN** `spox init` is run
- **THEN** all command templates from `templates/claude/commands/` SHALL be copied to `.claude/commands/`
- **AND** the setup command template SHALL be copied to `.claude/commands/spox/setup.md`

#### Scenario: Mission template copied only on fresh init

- **WHEN** `spox init` is run on a fresh project without `specs/mission.md`
- **THEN** the mission template SHALL be copied to `specs/mission.md`

#### Scenario: Mission template preserved on update

- **WHEN** `spox init` is run on an existing project with `specs/mission.md`
- **THEN** the existing `specs/mission.md` SHALL be preserved

#### Scenario: Spec templates copied to .spox/templates

- **WHEN** `spox init` is run
- **THEN** spec templates SHALL be copied to `.spox/templates/`

### Requirement: Environment Setup Script

The `spox init` command SHALL copy an interactive setup script to `.spox/setup.sh` that guides users through MCP server configuration. The script manages the project-level `.mcp.json` file directly instead of using `claude mcp add` commands. The script supports Linux and macOS; Windows users MUST use WSL.

#### Scenario: Setup script copied on fresh init

- **WHEN** initialization completes successfully on a fresh project
- **THEN** `.spox/setup.sh` exists with content from `templates/spox/setup.sh`
- **AND** the file has executable permissions (chmod +x)

#### Scenario: Setup configures .mcp.json on fresh project

- **WHEN** user runs `.spox/setup.sh` on a project without `.mcp.json`
- **AND** user confirms installation of serena and context7
- **THEN** `.mcp.json` is created with `mcpServers` containing `spox`, `serena`, and `context7` entries

#### Scenario: Setup always updates spox entry

- **WHEN** user runs `.spox/setup.sh` on a project with existing `.mcp.json`
- **THEN** the `spox` MCP entry is added or updated to the latest configuration
- **AND** Claude Code will discover the spox MCP server on next startup

#### Scenario: Setup preserves existing serena configuration

- **WHEN** user runs `.spox/setup.sh` on a project with existing `.mcp.json` containing `serena`
- **THEN** the existing `serena` entry is NOT modified
- **AND** user is NOT prompted to install serena

#### Scenario: Setup preserves existing context7 configuration

- **WHEN** user runs `.spox/setup.sh` on a project with existing `.mcp.json` containing `context7`
- **THEN** the existing `context7` entry is NOT modified
- **AND** user is NOT prompted to install context7

#### Scenario: Setup adds missing serena on user confirmation

- **WHEN** user runs `.spox/setup.sh` on a project with existing `.mcp.json` NOT containing `serena`
- **AND** user confirms serena installation
- **THEN** `serena` entry is added to `.mcp.json`

#### Scenario: Setup adds missing context7 on user confirmation

- **WHEN** user runs `.spox/setup.sh` on a project with existing `.mcp.json` NOT containing `context7`
- **AND** user confirms context7 installation
- **THEN** `context7` entry is added to `.mcp.json`

#### Scenario: Setup requires jq for JSON manipulation

- **WHEN** user runs `.spox/setup.sh` without `jq` installed
- **THEN** script displays error with `jq` installation instructions
- **AND** script exits without modifying `.mcp.json`

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

The `spox init` command SHALL create or update the project's `.claude/CLAUDE.md` file by processing the CLAUDE-template.md and replacing section markers with merged content.

#### Scenario: Fresh project without CLAUDE.md

- **WHEN** `spox init` runs in a directory without `.claude/CLAUDE.md`
- **THEN** `.claude/CLAUDE.md` is created from CLAUDE-template.md
- **AND** `<!-- SPOX:SYSTEM-TEMPLATES -->` is replaced with concatenated system template contents
- **AND** `<!-- SPOX:USER-TEMPLATES -->` is replaced with concatenated custom rule contents, or removed if no custom rules
- **AND** `<!-- SPOX:WORKFLOW -->` is replaced with workflow.md contents

#### Scenario: Existing CLAUDE.md without SPOX markers

- **WHEN** `spox init` runs in a directory with `.claude/CLAUDE.md` that lacks SPOX markers
- **THEN** the processed SPOX block is appended to the end of the existing file
- **AND** existing content is preserved

#### Scenario: Existing CLAUDE.md with SPOX markers

- **WHEN** `spox init` runs in a directory with `.claude/CLAUDE.md` containing `<!-- SPOX:START -->` and `<!-- SPOX:END -->` markers
- **THEN** the content between markers is replaced with newly processed template
- **AND** content outside the markers is preserved
- **AND** section markers are replaced based on current config.toml rules settings

#### Scenario: System templates merged in config order

- **WHEN** `spox init` processes system templates
- **THEN** templates are concatenated in the order specified in `[rules].system` array
- **AND** each template content is included without modification

#### Scenario: User templates section removed when empty

- **WHEN** `spox init` runs and `[rules].custom` is empty or not specified
- **THEN** the `<!-- SPOX:USER-TEMPLATES -->` marker and surrounding whitespace are removed
- **AND** no empty section remains in the generated CLAUDE.md

### Requirement: Gitignore Management

The `spox init` command SHALL create or update .gitignore files in `.spox/` and `.claude/` directories to exclude spox-managed files from version control.

#### Scenario: Create .spox/.gitignore on fresh init

- **WHEN** `spox init` runs and `.spox/.gitignore` does not exist
- **THEN** `.spox/.gitignore` is created from `templates/spox/gitignore` template
- **AND** the template contains rules excluding spox-generated files:
  - `/templates/` (spec and change templates)
  - `/standards/` (if it existed from older versions)
  - `/specs/` (if it existed from older versions)
  - `workflow.md` (if it existed from older versions)
  - `setup.sh`

#### Scenario: Append to existing .spox/.gitignore

- **WHEN** `spox init` runs and `.spox/.gitignore` already exists
- **THEN** missing rules are appended to the file
- **AND** existing rules are not duplicated
- **AND** existing custom rules added by user are preserved

#### Scenario: Create .claude/.gitignore on fresh init

- **WHEN** `spox init` runs and `.claude/.gitignore` does not exist
- **THEN** `.claude/.gitignore` is created from `templates/claude/gitignore` template
- **AND** the template contains rules excluding spox-managed files:
  - `agents/spox-*.md`
  - `commands/spox/`

#### Scenario: Append to existing .claude/.gitignore

- **WHEN** `spox init` runs and `.claude/.gitignore` already exists
- **THEN** missing rules are appended to the file
- **AND** existing rules are not duplicated
- **AND** existing custom rules added by user are preserved

### Requirement: CLAUDE.md Mission Reference

The generated CLAUDE.md SHALL reference the project mission via `@specs/mission.md` instead of embedding mission content directly.

#### Scenario: Mission referenced not embedded

- **WHEN** `spox init` generates `.claude/CLAUDE.md`
- **THEN** the file contains `@specs/mission.md` reference
- **AND** the file does NOT contain embedded mission content

### Requirement: CLAUDE.md Format Validation

The generated CLAUDE.md SHALL be valid markdown with correct heading hierarchy and formatting.

#### Scenario: Valid heading hierarchy

- **WHEN** CLAUDE.md is generated
- **THEN** headings follow proper hierarchy (h1 -> h2 -> h3, no skipping levels)
- **AND** no h2 follows directly after h1 without content (unless intentional section grouping)

#### Scenario: Valid markdown tables

- **WHEN** CLAUDE.md contains tables
- **THEN** all tables have consistent column counts
- **AND** all tables have proper separator rows (---|---)
- **AND** header rows match data row column counts

#### Scenario: Valid code fences

- **WHEN** CLAUDE.md contains code blocks
- **THEN** all code fences have matching opening and closing markers
- **AND** code fence language hints are valid (no malformed hints)
