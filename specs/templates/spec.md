# templates Specification

## Purpose

Define the structure and content requirements for Spec Oxide templates, including agent templates optimized for Claude
Code comprehension, workflow documentation using the resource-verb CLI pattern, and slash command templates.

## Requirements

### Requirement: Vibe Command Template

A `spox:vibe.md` command template SHALL be provided for vibe-coding workflows.

#### Scenario: Vibe command exists

- **WHEN** initialization or update completes successfully
- **THEN** `.claude/commands/spox/vibe.md` exists
- **AND** the command supports rapid prototyping without full spec workflow

### Requirement: Agent Template Structure

Agent templates SHALL follow a consistent structure optimized for Claude Code comprehension.

#### Scenario: Consistent frontmatter

- **WHEN** an agent template is loaded
- **THEN** it SHALL have frontmatter with: name, description, tools, color, model
- **AND** the description SHALL be concise and action-oriented

#### Scenario: Scannable format

- **WHEN** an agent contains instructions
- **THEN** steps SHALL be formatted as numbered checklists
- **AND** rules SHALL be formatted as numbered items or tables
- **AND** prose SHALL be minimal, favoring structured formats

### Requirement: Workflow Template Content

The workflow template SHALL document CLI commands for human use.

#### Scenario: CLI commands for humans

- **WHEN** a user reads workflow.md
- **THEN** CLI commands are documented for human use
- **AND** `spox show` is noted as the human dashboard

#### Scenario: No MCP section in workflow

- **WHEN** a user reads workflow.md
- **THEN** there is no MCP tools section
- **AND** MCP tools are documented in `standards/mcp.md` instead

#### Scenario: Directory paths are correct

- **WHEN** a user reads workflow.md
- **THEN** tooling paths reference `.spox/` (e.g., `.spox/config.toml`)
- **AND** spec paths reference `specs/` (e.g., `specs/mission.md`, `specs/_changes/`)

#### Scenario: Consistent branding

- **WHEN** a user reads workflow.md
- **THEN** the tool is referred to as "Spec Oxide" or "spox"

### Requirement: Spox MCP in Standards

The MCP standards template SHALL document Spox MCP tools for spec and change operations.

#### Scenario: Spox MCP section is first

- **WHEN** a user reads `standards/mcp.md`
- **THEN** the "Spox MCP" section appears before Serena and Context7 sections
- **AND** the section documents `list_specs`, `get_spec_requirements`, `get_scenario`, `list_changes`, `get_change`, `search_specs`, `validate_spec`, `validate_change`

#### Scenario: Priority rule includes Spox

- **WHEN** a user reads `standards/mcp.md`
- **THEN** the priority rule lists Spox first for spec operations

### Requirement: MCP Tools for Agent Templates

Agent and command templates SHALL use Spox MCP tools for spec and change operations.

#### Scenario: Agent reads change via MCP

- **WHEN** an agent template needs to read a change proposal
- **THEN** it uses `get_change` MCP tool
- **AND** it does NOT shell out to CLI commands

#### Scenario: Agent discovers specs via MCP

- **WHEN** an agent template needs to discover available specs
- **THEN** it uses `list_specs` MCP tool
- **AND** it uses `get_spec_requirements` for requirement structure
- **AND** it uses `get_scenario` for scenario details

#### Scenario: Agent discovers changes via MCP

- **WHEN** an agent template needs to discover active changes
- **THEN** it uses `list_changes` MCP tool

#### Scenario: Agent validates via MCP

- **WHEN** an agent template needs to validate specs or changes
- **THEN** it uses `validate_spec` or `validate_change` MCP tools
- **AND** it does NOT shell out to `spox spec validate` or `spox change validate` CLI commands

### Requirement: CLI for Human Workflows

The workflow template SHALL document CLI commands for human use only.

#### Scenario: CLI section is human-only

- **WHEN** a user reads workflow.md
- **THEN** CLI commands are documented for human use
- **AND** there is no MCP section (MCP is documented in standards)

#### Scenario: Validation commands documented

- **WHEN** a user reads workflow.md
- **THEN** `spox change validate` and `spox spec validate` are documented
- **AND** `spox show` is documented as the human dashboard

### Requirement: Agent Template Content

The agent templates SHALL NOT reference standards files directly since content is merged into `.claude/CLAUDE.md`.

#### Scenario: No project context references needed

- **WHEN** an agent template is loaded
- **THEN** it does NOT use `@specs/mission.md` reference
- **AND** mission content is available via merged CLAUDE.md

#### Scenario: No standards references needed

- **WHEN** an agent template is loaded
- **THEN** it does NOT use `@.spox/standards/` path references
- **AND** standards content is available via merged CLAUDE.md

#### Scenario: No workflow references needed

- **WHEN** an agent template is loaded
- **THEN** it does NOT use `@.spox/workflow.md` reference
- **AND** workflow content is available via merged CLAUDE.md

### Requirement: CLAUDE Template Convention

The CLAUDE-template.md SHALL use section markers for content injection during init/update.

#### Scenario: Template with section markers

- **WHEN** the CLAUDE-template.md is viewed
- **THEN** it contains `<!-- SPOX:START -->` and `<!-- SPOX:END -->` outer markers
- **AND** it contains `<!-- SPOX:SYSTEM-TEMPLATES -->` marker for system template injection
- **AND** it contains `<!-- SPOX:USER-TEMPLATES -->` marker for custom rules injection
- **AND** it contains `<!-- SPOX:WORKFLOW -->` marker for workflow injection
- **AND** it contains section headers for "Standards Compliance and Guardrails" and "Spec Oxide Workflow"

#### Scenario: No backtick-escaped references needed

- **WHEN** the CLAUDE-template.md is viewed
- **THEN** it does NOT contain `@` references (escaped or unescaped)
- **AND** content is dynamically injected at section markers based on config.toml rules
