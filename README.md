<p>
  <img src="assets/spox-logo-light.svg" alt="Spec Oxide" width="180" align="left">
  <h1>Spec Oxide</h1>
</p>

> Spec-driven development for humans and AI - optimized for Claude Code.

Spec Oxide is a comprehensive workflow that enables spec-driven development for AI-assisted coding. It excels not only in green-field projects but also in long-living projects by keeping specs up to date through integrating implemented change deltas into an existing repository of specs grouped by capability.

The workflow is augmented by the `spox` CLI tool.

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize a new project
spox init

# View project dashboard (human-friendly, colored output)
spox show

# List and view specs
spox spec list
spox spec show auth

# Validate specs and changes
spox spec validate
spox change validate
```

## Commands

| Command | Description |
|---------|-------------|
| `spox init [--path <path>]` | Initialize a new Spox project |
| `spox show` | Show project dashboard (human use) |
| `spox config show` | Display project configuration |
| `spox spec list` | List all specs with requirement counts |
| `spox spec show <id>` | Show a spec's purpose and requirements |
| `spox spec validate [<id>] [--strict]` | Validate spec structure |
| `spox change list` | List active changes with task progress |
| `spox change show <id> [--deltas-only]` | Show a change proposal |
| `spox change validate [<id>] [--strict]` | Validate change structure |

### Command Details

#### `spox init`

Creates the project structure:

```
.spox/
  config.toml           # Project configuration
  workflow.md           # Workflow documentation
  standards/            # Coding standards
  specs/                # Spec templates
.claude/
  agents/               # AI agent templates
  commands/spox/        # Slash commands
specs/
  mission.md            # Project mission
  _changes/             # Active change proposals
  _archive/             # Archived changes
```

#### `spox show`

Displays a human-friendly dashboard with:
- List of specs with requirement counts
- Active changes with task progress bars
- Delta summaries per change

Note: This command produces colored output for terminal viewing. AI agents should use `spox spec list` and `spox change list` instead.

#### `spox spec list` / `spox change list`

Lists specs or changes in a compact, AI-friendly format (no colors):

```
Specs:
- auth    2 requirements
- config  1 requirement
```

```
Changes:
- add-feature  2/4 tasks
- fix-bug      0/2 tasks
```

#### `spox spec show <id>`

Displays a spec with:
- Purpose section
- Requirements with descriptions
- Scenarios for each requirement

#### `spox change show <id>`

Displays a change proposal with:
- Why section (motivation)
- What Changes section
- Task progress bar
- Delta requirements grouped by capability

Use `--deltas-only` to show only the delta requirements.

#### `spox spec validate` / `spox change validate`

Validates structure and content:
- Checks required sections (Purpose, Requirements, Why, What Changes)
- Verifies scenario format (WHEN/THEN clauses)
- Reports errors and warnings
- Use `--strict` to treat warnings as errors

## Workflow

Spec Oxide follows a three-stage workflow:

### Stage 1: Propose

Use `/spox:propose` to scaffold a new change proposal in `specs/_changes/<change-id>/`:

```
specs/_changes/add-feature/
  proposal.md           # Why and what changes
  tasks.md              # Implementation checklist
  design.md             # Technical decisions (optional)
  specs/
    <capability>/
      spec.md           # Delta requirements
```

Delta format:
```markdown
## ADDED|MODIFIED|REMOVED|RENAMED Requirements

### Requirement: Name
Description using SHALL/MUST.

#### Scenario: Description
- **WHEN** condition
- **THEN** outcome
```

### Stage 2: Implement

Use `/spox:implement` to execute the approved change. Complete tasks from `tasks.md`, mark them as done, and verify the implementation.

### Stage 3: Archive

Use `/spox:archive` to finalize a deployed change. Move the change to `specs/_archive/YYYY-MM-DD-<id>/` and apply deltas to the main specs.

## Claude Code Integration

Spec Oxide includes templates for Claude Code AI assistance.

### Agents

| Agent | Description |
|-------|-------------|
| `spox-implementer` | Implements features by following tasks.md |
| `spox-reviewer` | Performs code quality review |
| `spox-verifier` | Verifies implementation against specs |

## Directory Structure

```
project/
├── .spox/
│   ├── config.toml         # Configuration (spec_folder, changes_folder, archive_folder)
│   ├── workflow.md         # Workflow reference
│   ├── standards/          # Coding standards
│   └── specs/              # Templates
├── .claude/
│   ├── agents/
│   │   ├── spox-implementer.md
│   │   ├── spox-reviewer.md
│   │   └── spox-verifier.md
│   └── commands/spox/
│       ├── propose.md
│       ├── implement.md
│       └── archive.md
└── specs/
    ├── mission.md          # Project mission statement
    ├── <capability>/
    │   └── spec.md         # Capability specification
    ├── _changes/           # Active change proposals
    │   └── <change-id>/
    │       ├── proposal.md
    │       ├── tasks.md
    │       ├── design.md   # Optional
    │       └── specs/
    │           └── <capability>/
    │               └── spec.md
    └── _archive/           # Archived changes
        └── YYYY-MM-DD-<id>/
```

## Configuration

The `.spox/config.toml` file contains:

```toml
spec_folder = "specs"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"
```

Use `spox config show` to display current configuration.

## Validation Rules

### Spec Validation

- Must have `## Purpose` section (warning if < 50 characters)
- Must have `## Requirements` section
- Requirements must use normative language (SHALL/MUST)
- Requirements must have at least one scenario
- Scenarios must have WHEN and THEN clauses

### Change Validation

- Must have `proposal.md` with `## Why` and `## What Changes` sections
- Must have `tasks.md` with checkbox items
- Must have at least one delta spec
- Delta specs must use valid headers (ADDED, MODIFIED, REMOVED, RENAMED)

## License

See LICENSE file.