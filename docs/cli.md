# CLI Guide

The `spox` CLI provides commands for listing, viewing, and validating specs and changes.

## Commands Overview

| Command                          | Description                          |
|----------------------------------|--------------------------------------|
| `spox init [path]`               | Initialize or update a project       |
| `spox show`                      | Display project dashboard            |
| `spox config show [--paths]`     | Show configuration                   |
| `spox spec list`                 | List all specs                       |
| `spox spec show <id>`            | Show spec content                    |
| `spox spec validate [id]`        | Validate specs                       |
| `spox change list`               | List active changes                  |
| `spox change show <id>`          | Show change proposal                 |
| `spox change validate [id]`      | Validate changes                     |
| `spox index`                     | Build semantic search index          |
| `spox mcp serve`                 | Start MCP server                     |

## Command Reference

### `spox init [path]`

Initialize or update a Spec Oxide project.

**Usage:**

```bash
# Initialize in current directory
spox init

# Initialize at specific path
spox init /path/to/project
```

**What it does:**

- Creates `.spox/`, `.claude/`, and `specs/` directories
- Copies agent and command templates
- Generates `.claude/CLAUDE.md` from templates
- Creates `specs/mission.md` (on fresh init)
- Updates existing projects with latest templates

**Options:**

- `path` - Optional path to initialize (defaults to current directory)

### `spox show`

Display a human-friendly project dashboard.

**Usage:**

```bash
spox show
```

**What it shows:**

- List of specs with requirement counts
- Active changes with task progress bars
- Delta summaries per change (added/modified/removed counts)

**Note:** This command produces colored output for terminal viewing. AI agents should use `spox spec list` and
`spox change list` for token-efficient output.

**Example output:**

```
Specs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
auth                                               2 requirements
config                                             1 requirement

Active Changes
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
add-feature                          [██████----] 2/4 tasks
  ├─ auth: +2 requirements
  └─ notifications: +1 requirement
```

### `spox config show`

Display project configuration.

**Usage:**

```bash
# Show full configuration
spox config show

# Show paths only
spox config show --paths
```

**What it shows:**

- `[paths]` section: `spec_folder`, `changes_folder`, `archive_folder`
- `[rules]` section: `system` and `custom` template arrays

**Example output:**

```
[paths]
spec_folder = "specs"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "mindset", "standards", "errors", "backend", "ui", "git"]
custom = []
```

**Options:**

- `--paths` - Show only the `[paths]` section

### `spox spec list`

List all specifications with requirement counts.

**Usage:**

```bash
spox spec list
```

**Example output:**

```
Specs:
- auth    2 requirements
- config  1 requirement
- cli     5 requirements
```

**Note:** Output is AI-friendly with no ANSI color codes, optimized for token efficiency.

### `spox spec show <id>`

Display a specification's content.

**Usage:**

```bash
spox spec show auth
```

**What it shows:**

- Purpose section
- Requirements with descriptions
- Scenarios for each requirement

**Example output:**

```
auth Specification

Purpose
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Define authentication requirements for user login.

Requirements
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Requirement: Password Authentication
Users SHALL authenticate using username and password.

  Scenario: Valid credentials
  - WHEN valid username and password provided
  - THEN user is logged in
  - AND session token is created
```

### `spox spec validate [<id>] [--strict]`

Validate spec structure and content.

**Usage:**

```bash
# Validate all specs
spox spec validate

# Validate specific spec
spox spec validate auth

# Strict mode (warnings cause failure)
spox spec validate --strict
```

**What it checks:**

- `## Purpose` section exists and has sufficient content
- `## Requirements` section exists
- Requirements use normative language (SHALL/MUST)
- Requirements have at least one scenario
- Scenarios have WHEN and THEN clauses

**Example output:**

```
Validating specs...
✓ auth
✗ config
  ERROR: Missing Purpose section

Summary: 1 error, 0 warnings
Validation failed.
```

**Options:**

- `<id>` - Validate specific spec only
- `--strict` - Treat warnings as errors

**Exit codes:**

- `0` - Validation passed
- `1` - Validation failed (errors found, or warnings in strict mode)

### `spox change list`

List all active changes with task progress.

**Usage:**

```bash
spox change list
```

**Example output:**

```
Changes:
- add-feature  2/4 tasks
- fix-bug      0/2 tasks
```

**Note:** Output is AI-friendly with no ANSI color codes.

### `spox change show <id> [--deltas-only]`

Display a change proposal's content.

**Usage:**

```bash
# Show full proposal
spox change show add-feature

# Show only delta requirements
spox change show add-feature --deltas-only
```

**What it shows (full mode):**

- Why section (motivation)
- What Changes section
- Task completion with progress bar
- Delta requirements grouped by capability

**What it shows (deltas-only mode):**

- Only delta requirements
- Grouped by capability
- Operation type (ADDED/MODIFIED/REMOVED/RENAMED)
- Requirement names and descriptions

**Example output:**

```
add-feature

Why
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Users need two-factor authentication for enhanced security.

Tasks
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[██████----] 2/4 tasks

Deltas
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
auth
  ADDED Requirements
    - Two-Factor Authentication
    - OTP Validation
```

**Options:**

- `--deltas-only` - Show only delta requirements (useful for debugging parsing issues)

### `spox change validate [<id>] [--strict]`

Validate change structure and content.

**Usage:**

```bash
# Validate all changes
spox change validate

# Validate specific change
spox change validate add-feature

# Strict mode
spox change validate --strict
```

**What it checks:**

- `proposal.md` exists with `## Why` and `## What Changes` sections
- `tasks.md` exists with checkbox items
- At least one delta spec exists
- Delta specs use valid headers (ADDED, MODIFIED, REMOVED, RENAMED)
- Requirements have scenarios with WHEN/THEN clauses

**Example output:**

```
Validating changes...
✓ add-feature
✗ fix-bug
  ERROR: Missing proposal.md

Summary: 1 error, 0 warnings
Validation failed.
```

**Options:**

- `<id>` - Validate specific change only
- `--strict` - Treat warnings as errors

**Exit codes:**

- `0` - Validation passed
- `1` - Validation failed

## CLI Best Practices

**Check progress, specs and changes:**

- Use `spox show` to get a visual overview of the project
- Use `spox spec list` and `spox change list` for quick checks
- Use `spox config show` to verify paths before creating proposals

**For validation:**

- Run `spox change validate` before requesting approval
- Run `spox spec validate` after archiving changes
- Use `--strict` mode in CI/CD pipelines