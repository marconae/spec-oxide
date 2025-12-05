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

The workflow template SHALL use the `spox` CLI with the correct resource-verb command pattern.

#### Scenario: CLI commands use resource-verb pattern

- **WHEN** a user reads `.spox/workflow.md`
- **THEN** all CLI command examples use the resource-verb pattern
- **AND** change operations use `spox change <verb>` (e.g., `spox change list`, `spox change show`)
- **AND** spec operations use `spox spec <verb>` (e.g., `spox spec list`, `spox spec show`)
- **AND** config operations use `spox config show`
- **AND** init uses `spox init`
- **AND** dashboard uses `spox show` (human use only)

#### Scenario: Directory paths are correct

- **WHEN** a user reads `.spox/workflow.md`
- **THEN** tooling paths reference `.spox/` (e.g., `.spox/config.toml`, `.spox/standards/`)
- **AND** spec paths reference `specs/` (e.g., `specs/mission.md`, `specs/_changes/`)

#### Scenario: Consistent branding

- **WHEN** a user reads `.spox/workflow.md`
- **THEN** the tool is referred to as "Spec Oxide" or "spox"

#### Scenario: Unimplemented commands are marked

- **WHEN** a user reads `.spox/workflow.md`
- **THEN** commands not yet implemented (like `spox archive`) are marked as "planned" or omitted

#### Scenario: Dashboard in Quick Reference

- **WHEN** a user reads `.spox/workflow.md`
- **THEN** the Quick Reference section includes `spox show` with note "(human dashboard)"
- **AND** the description indicates it is for human use, not AI agents

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

### Requirement: Command Template Content

The slash command templates SHALL use the `spox` CLI with the correct resource-verb command pattern.

#### Scenario: CLI commands use resource-verb pattern

- **WHEN** a command template contains CLI examples
- **THEN** all commands use the resource-verb pattern
- **AND** `spox change list` is used instead of `spox list`
- **AND** `spox spec list` is used instead of `spox list --specs`

#### Scenario: Metadata uses Spec Oxide branding

- **WHEN** a command template has frontmatter metadata
- **THEN** the name and category use "Spec Oxide"
- **AND** tags use "spox" or "spec-oxide"

#### Scenario: Path references are correct

- **WHEN** a command template references file paths
- **THEN** changes are in `specs/_changes/`
- **AND** specs are in `specs/`

#### Scenario: Token-efficient command usage for agents

- **WHEN** a command template is used by AI coding agents
- **THEN** it prefers `spox spec list` and `spox change list` for context (token-efficient)
- **AND** it uses `spox spec show <id>` only when specific spec details are needed
- **AND** it uses `spox change show <id>` only when specific change details are needed
- **AND** it does NOT use `spox show` (dashboard is for humans only)

#### Scenario: No file references needed in commands

- **WHEN** a command template is loaded
- **THEN** it SHALL NOT use `@` syntax for standards, workflow, or mission files
- **AND** this is because these files are merged into `.claude/CLAUDE.md` which is auto-loaded

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
