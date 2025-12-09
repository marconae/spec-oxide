# User Guide | Spec Oxide

> Spec-driven development for humans and AI - optimized for Claude Code.

Spec Oxide is a comprehensive workflow and toolset that enables spec-driven development for AI-assisted coding. You
agree on *what* to build before any code is written.

**Core principle:** Specs are the source of truth. Changes are proposals that modify that truth. Keep them in sync.

## Table of Contents

1. [Getting Started](setup-guide.md) ← Start here
2. [Workflow](#workflow)
3. [CLI Tool (spox)](#cli-tool-spox)
4. [Configuration](#configuration)
5. [MCP Integration](#mcp-integration)
6. [Spec Standards and Validation](#spec-standards-and-validation)
7. [System Standards Reference](#system-standards-reference)

## Workflow

Spec Oxide follows a three-stage workflow: **Propose → Implement → Archive**.

### Philosophy

AI coding assistants are powerful but unpredictable when requirements live only in chat history. Spec Oxide solves this
by:

1. **Separating intent from implementation** - Define what to build before writing code
2. **Creating an audit trail** - Every change is documented and reviewable
3. **Enabling deterministic outputs** - AI agents work from specs, not vague prompts
4. **Maintaining up-to-date documentation** - Specs evolve as the project grows

### The Three Stages

#### Stage 1: Propose

**Goal:** Agree on what to build before writing any code.

Use the `/spox:propose` slash command in Claude Code to create a change proposal:

```
/spox:propose Add two-factor authentication to the login flow
```

This creates a structured proposal in `specs/_changes/<change-id>/`:

```
specs/_changes/add-two-factor-auth/
  proposal.md           # Why this change? What's the impact?
  tasks.md              # Ordered implementation checklist
  design.md             # Technical decisions (optional)
  specs/
    auth/
      spec.md           # Delta requirements
```

The command is designed to discuss any open questions or ambiguities with you. Read and discuss the `proposal.md`,
`design.md`, `tasks.md`, and drafted change delta `specs/<capability>` folders.

Approve the proposal by instructing the agent to implement the task list with `/spox:implement`.

#### Stage 2: Implement

Use the `/spox:implement` slash command to execute the approved change:

```
/spox:implement add-two-factor-auth
```

This orchestrates implementation by:

1. Reading the proposal, design, and tasks
2. Spawning subagents to complete tasks
3. Tracking progress in `tasks.md`
4. Verifying implementation against specs

**Key activities:**

- Work through tasks systematically
- Mark completed tasks in `tasks.md`
- Run verification before marking complete
- Ensure implementation matches the spec exactly

**Do not archive.** Wait for deployment before moving to Stage 3.

#### Stage 3: Archive

**Goal:** Merge approved changes into the source of truth.

After deployment, use the `/spox:archive` slash command:

```
/spox:archive add-two-factor-auth
```

This finalizes the change by:

1. Moving `specs/_changes/<id>/` → `specs/_archive/YYYY-MM-DD-<id>/`
2. Applying deltas to `specs/<capability>/spec.md` files
3. Validating specs: `spox spec validate`

**Key activities:**

- Move change to archive with timestamp
- Apply ADDED/MODIFIED/REMOVED/RENAMED deltas to specs
- Verify specs are consistent
- Confirm the source of truth is updated

### Slash Commands Overview

Spec Oxide integrates with Claude Code via slash commands:

| Command           | Purpose                              | Stage     |
|-------------------|--------------------------------------|-----------|
| `/spox:propose`   | Create change proposals              | Propose   |
| `/spox:implement` | Implement approved changes           | Implement |
| `/spox:archive`   | Archive completed changes            | Archive   |
| `/spox:vibe`      | Vibe coding mode (rapid prototyping) | N/A       |

**When to use each command:**

- **`/spox:propose`** - When you have a new feature, breaking change, or architectural update
- **`/spox:implement`** - When a proposal has been reviewed and approved
- **`/spox:archive`** - After implementation is deployed to production
- **`/spox:vibe`** - For quick experiments or throwaway prototypes outside the spec workflow

### Example Workflow Walkthrough

Let's walk through adding two-factor authentication:

**1. Propose**

```bash
# Check current state
spox spec list
spox change list

# Create proposal
/spox:propose Add two-factor authentication to login flow
```

The AI agent creates:

- `specs/_changes/add-two-factor-auth/proposal.md` - Why and what changes
- `specs/_changes/add-two-factor-auth/tasks.md` - Implementation checklist
- `specs/_changes/add-two-factor-auth/specs/auth/spec.md` - Delta requirements

```bash
# Validate proposal
spox change validate add-two-factor-auth
```

Review the proposal, approve it, and move to implementation.

**2. Implement**

```bash
# Implement the change
/spox:implement add-two-factor-auth
```

The AI agent:

- Reads the proposal and specs
- Works through tasks in `tasks.md`
- Marks completed tasks with `[x]`
- Runs verification

```bash
# Check progress
spox change show add-two-factor-auth
```

Deploy to production after verification passes.

**3. Archive**

```bash
# Archive the change
/spox:archive add-two-factor-auth
```

The AI agent:

- Moves change to `specs/_archive/2025-12-06-add-two-factor-auth/`
- Applies deltas to `specs/auth/spec.md`
- Validates specs

```bash
# Verify specs
spox spec validate
spox spec list
```

Done! The two-factor authentication capability is now part of the permanent specification.

---

## CLI Tool (spox)

The `spox` CLI provides commands for listing, viewing, and validating specs and changes.

### Command Reference

#### `spox init [path]`

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

#### `spox show`

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

#### `spox config show`

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

#### `spox spec list`

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

#### `spox spec show <id>`

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

#### `spox spec validate [<id>] [--strict]`

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

#### `spox change list`

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

#### `spox change show <id> [--deltas-only]`

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

#### `spox change validate [<id>] [--strict]`

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

### CLI Best Practices

**For human users:**

- Use `spox show` to get a visual overview of the project
- Use `spox spec list` and `spox change list` for quick checks
- Use `spox config show` to verify paths before creating proposals

**For AI agents:**

- Use `spox spec list` and `spox change list` (token-efficient)
- Use `spox spec show <id>` only when specific details are needed
- Use `spox change show <id>` only when reviewing a specific proposal
- Avoid `spox show` (dashboard is for humans only)

**For validation:**

- Run `spox change validate` before requesting approval
- Run `spox spec validate` after archiving changes
- Use `--strict` mode in CI/CD pipelines

---

## Configuration

Spec Oxide uses a TOML configuration file located at `.spox/config.toml`.

### Configuration File Structure

```toml
[paths]
spec_folder = "specs"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "mindset", "standards", "errors", "backend", "ui", "git"]
custom = []
```

### Configuration Sections

#### `[paths]` Section

Defines directory paths for specs, changes, and archives.

| Setting          | Description               | Default            |
|------------------|---------------------------|--------------------|
| `spec_folder`    | Where current specs live  | `"specs"`          |
| `changes_folder` | Where active changes live | `"specs/_changes"` |
| `archive_folder` | Where archived changes go | `"specs/_archive"` |

**Example:**

```toml
[paths]
spec_folder = "specifications"
changes_folder = "specifications/active"
archive_folder = "specifications/history"
```

**Note:** After changing paths, update existing directory structure to match.

#### `[rules]` Section

Defines which templates to merge into `.claude/CLAUDE.md`.

| Setting  | Description                                    | Required |
|----------|------------------------------------------------|----------|
| `system` | Array of system template names (without `.md`) | Yes      |
| `custom` | Array of custom rule filenames (without `.md`) | No       |

**Example:**

```toml
[rules]
system = ["mcp", "mindset", "standards"]
custom = ["team-conventions", "security-guidelines"]
```

**How it works:**

1. System templates are bundled in the `spox` binary
2. Custom rules live in `.spox/custom/` directory
3. On `spox init`, templates are merged into `.claude/CLAUDE.md`
4. Content is injected between `<!-- SPOX:START -->` and `<!-- SPOX:END -->` markers

### Custom Rules

Custom rules allow you to add project-specific conventions to Claude Code instructions.

**Location:** `.spox/custom/`

**Format:** Markdown files (`.md`)

**Usage:**

1. Create a markdown file in `.spox/custom/` (e.g., `team-conventions.md`)
2. Add the filename (without `.md`) to `[rules].custom` in `config.toml`
3. Run `spox init` to regenerate `.claude/CLAUDE.md`

**Example custom rule file (`.spox/custom/team-conventions.md`):**

```markdown
## Team Conventions

### Code Review Process

All changes must:

- Pass CI/CD pipeline
- Have at least one approval
- Include tests with >80% coverage

### Naming Conventions

- Use PascalCase for React components
- Use camelCase for functions and variables
- Use UPPER_SNAKE_CASE for constants
```

After adding to `config.toml`:

```toml
[rules]
system = ["mcp", "mindset", "standards"]
custom = ["team-conventions"]
```

Run `spox init` to merge into `.claude/CLAUDE.md`.

### CLAUDE.md Generation

The `.claude/CLAUDE.md` file is auto-generated from templates and should not be edited directly.

**How it works:**

1. `spox init` reads `CLAUDE-template.md` (bundled in binary)
2. Replaces section markers with merged content:
    - `<!-- SPOX:SYSTEM-TEMPLATES -->` → system templates from `[rules].system`
    - `<!-- SPOX:USER-TEMPLATES -->` → custom rules from `[rules].custom`
    - `<!-- SPOX:WORKFLOW -->` → workflow documentation
3. Writes result to `.claude/CLAUDE.md` between `<!-- SPOX:START -->` and `<!-- SPOX:END -->` markers

**Markers:**

```markdown
<!-- SPOX:START -->

# Standards Compliance and Guardrails

<!-- SPOX:SYSTEM-TEMPLATES -->

<!-- SPOX:USER-TEMPLATES -->

# Spec Oxide Workflow

<!-- SPOX:WORKFLOW -->
<!-- SPOX:END -->
```

**Updating CLAUDE.md:**

To update generated content, modify `config.toml` or custom rule files, then run:

```bash
spox init
```

**Preserving custom content:**

Content outside `<!-- SPOX:START -->` and `<!-- SPOX:END -->` markers is preserved during updates.

---

## MCP Integration

Spec Oxide requires two MCP (Model Context Protocol) servers for full functionality.

### Why MCP Servers?

MCP servers provide semantic understanding and up-to-date documentation:

- **Serena** - Semantic code operations (symbol lookup, refactoring)
- **Context7** - Library documentation and API references

Without MCP servers, AI agents fall back to text-based search (`rg`, `grep`), which only matches patterns and lacks
semantic understanding.

### Setup Process

Run the setup script:

```bash
.spox/setup.sh
```

The script is interactive and will:

1. Check prerequisites (Claude Code, uv, Node.js)
2. Prompt to install Serena MCP
3. Prompt to install Context7 MCP
4. Index your project with Serena

**Prerequisites check:**

- `claude` command (Claude Code CLI)
- `uv` command (for Serena)
- `node` command (for Context7)

**Installation prompts:**

```
Serena MCP not configured. Install? (y/n)
```

If you confirm, the script runs:

```bash
claude mcp add serena --project uvx --from "git+https://github.com/oraios/serena"
```

```
Context7 MCP not configured. Install? (y/n)
```

If you confirm, the script runs:

```bash
claude mcp add context7 --project npx -y @context7/mcp-server
```

**Project indexing:**

After Serena is installed, the script prompts:

```
Index project with Serena? (y/n)
```

If you confirm, the script runs:

```bash
uvx --from git+https://github.com/oraios/serena serena project index
```

This creates a semantic index of your codebase for fast symbol lookup.

### How Serena is Used

Serena provides semantic code operations through MCP tools:

#### Code Exploration

| Tool                       | Purpose                                     |
|----------------------------|---------------------------------------------|
| `find_symbol`              | Find function, class, or method definitions |
| `get_symbols_overview`     | Get high-level view of file structure       |
| `find_referencing_symbols` | Find all references to a symbol             |
| `search_for_pattern`       | Flexible pattern search in codebase         |

**Example: Finding a symbol**

Instead of `rg "function foo"` (text match), Serena understands:

- `foo()` in file A calls the `foo` defined in file B
- `foo` is a method of class `Bar`
- `foo` is imported from module `baz`

#### Code Editing

| Tool                   | Purpose                      |
|------------------------|------------------------------|
| `replace_symbol_body`  | Replace function/method body |
| `insert_after_symbol`  | Insert code after a symbol   |
| `insert_before_symbol` | Insert code before a symbol  |

**Example: Refactoring**

Serena can replace a method body while preserving:

- Method signature
- Docstrings and comments
- Surrounding code structure

#### Refactoring

| Tool            | Purpose                                  |
|-----------------|------------------------------------------|
| `rename_symbol` | Rename symbol throughout entire codebase |

**Example: Renaming**

Serena renames a symbol and updates:

- All references in the codebase
- Imports and exports
- Comments (if they reference the symbol)

### How Context7 is Used

Context7 provides up-to-date library documentation through MCP tools:

| Tool                 | Purpose                           |
|----------------------|-----------------------------------|
| `resolve-library-id` | Find library by name              |
| `get-library-docs`   | Fetch documentation for a library |

**Example: API lookup**

Instead of relying on potentially outdated training data, Context7 fetches current documentation:

- Latest API signatures
- Framework patterns and best practices
- Code examples from official docs

**Modes:**

- `mode='code'` (default) - API references and code examples
- `mode='info'` - Conceptual guides and architectural information

**Example usage:**

```
User: How do I use React hooks?

AI agent:
1. Call resolve-library-id with "react"
2. Call get-library-docs with library ID and topic "hooks"
3. Provide accurate, current documentation
```

### Decision Flow

When working on a task, the AI agent follows this decision flow:

```
Need to understand code?
├─ Symbol, reference, or structure → Serena
├─ Library or API docs → Context7
└─ Neither available → Fall back to text tools
```

**Rules:**

1. **Serena first** for any code navigation or understanding task
2. **Context7 first** for any library or documentation lookup
3. **Text tools** (`rg`, `grep`) only when MCP tools are unavailable
4. **Never assume** library behavior - verify with Context7
5. **Never pattern-match** code - understand it with Serena

### Troubleshooting MCP Setup

**Serena installation fails:**

- Ensure `uv` is installed: `curl -LsSf https://astral.sh/uv/install.sh | sh`
- Check Python version: Serena requires Python 3.8+

**Context7 installation fails:**

- Ensure Node.js 18+ is installed: `node --version`
- Check npm version: `npm --version`

**Project indexing fails:**

- Ensure you're in the project root directory
- Check disk space (indexing creates cache files)
- Try manually: `uvx --from git+https://github.com/oraios/serena serena project index`

**Claude Code not found:**

- Install Claude Code: `npm install -g @anthropic-ai/claude-code`
- Or use curl: `curl -fsSL https://claude.ai/install.sh | sh`

**Re-running setup:**

You can re-run `.spox/setup.sh` at any time to:

- Install missing MCP servers
- Re-index the project with Serena
- Verify prerequisites

---

## Spec Standards and Validation

Spec Oxide enforces structure and format standards for specs and changes.

### Spec File Format

Specs follow a standardized structure:

```markdown
# [capability] Specification

## Purpose

[Description of what this capability does and why it exists]

## Requirements

### Requirement: [Name]

[Normative description using SHALL or MUST]

#### Scenario: [Description]

- **WHEN** [condition]
- **THEN** [outcome]
- **AND** [additional outcome]

#### Scenario: [Another scenario]

- **WHEN** [condition]
- **THEN** [outcome]
```

**Key sections:**

| Section                        | Required | Description                            |
|--------------------------------|----------|----------------------------------------|
| `# [capability] Specification` | Yes      | Top-level heading with capability name |
| `## Purpose`                   | Yes      | Why this capability exists             |
| `## Requirements`              | Yes      | Container for all requirements         |

**Template location:** `.spox/templates/spec.md`

### Requirement Format

Requirements describe what the system SHALL or MUST do:

```markdown
### Requirement: [Name]

[Normative description using SHALL or MUST]
```

**Rules:**

- Use `### Requirement:` (h3 header)
- Include descriptive name after colon
- Use normative language: SHALL or MUST
- Provide clear description of the requirement

**Good examples:**

```markdown
### Requirement: User Authentication

The system SHALL authenticate users via username and password.

### Requirement: Password Validation

Passwords MUST be at least 8 characters long and contain at least one number.
```

**Bad examples:**

```markdown
### Requirement: Auth

Authentication stuff.

### User Login

The system authenticates users.
```

### Scenario Format

Scenarios describe concrete examples of requirement behavior:

```markdown
#### Scenario: [Description]

- **WHEN** [condition]
- **THEN** [outcome]
- **AND** [additional outcome]
```

**Rules:**

- Use `#### Scenario:` (h4 header, not bullets)
- Include descriptive name after colon
- Use WHEN for preconditions
- Use THEN for outcomes
- Use AND for additional outcomes (optional)

**Format matters:**

```markdown
#### Scenario: Valid credentials ✓ (h4 header)

- **Scenario: Valid credentials**   ✗ (bullet point)
  **Scenario**: Valid credentials ✗ (bold text)

### Scenario: Valid credentials ✗ (h3 header)
```

**Good example:**

```markdown
#### Scenario: Valid login credentials

- **WHEN** user provides valid username and password
- **THEN** system creates session token
- **AND** user is redirected to dashboard
```

**Bad examples:**

```markdown
- **Scenario: Valid login**
    - User logs in successfully

### Scenario: Login

Valid username and password → logged in
```

### Delta Format for Changes

Changes use delta specs to describe modifications:

```markdown
## ADDED Requirements

### Requirement: [Name]

[Description]

#### Scenario: [Description]

- **WHEN** [condition]
- **THEN** [outcome]

## MODIFIED Requirements

### Requirement: [Name]

[Complete updated requirement with all scenarios]

## REMOVED Requirements

### Requirement: [Name]

[Name only, no description needed]

## RENAMED Requirements

### Requirement: [Old Name] → [New Name]
```

**Operations:**

| Header                     | Use When                           |
|----------------------------|------------------------------------|
| `## ADDED Requirements`    | Adding new standalone capability   |
| `## MODIFIED Requirements` | Changing existing behavior         |
| `## REMOVED Requirements`  | Deprecating functionality          |
| `## RENAMED Requirements`  | Renaming without changing behavior |

**Critical for MODIFIED:**

When modifying a requirement, paste the **complete** existing requirement text, then edit. Partial text results in lost
content at archive time.

**Good MODIFIED example:**

```markdown
## MODIFIED Requirements

### Requirement: Password Authentication

Users SHALL authenticate using username and password OR email and password.

#### Scenario: Valid credentials with username

- **WHEN** valid username and password provided
- **THEN** user is logged in
- **AND** session token is created

#### Scenario: Valid credentials with email

- **WHEN** valid email and password provided
- **THEN** user is logged in
- **AND** session token is created
```

**Bad MODIFIED example:**

```markdown
## MODIFIED Requirements

### Requirement: Password Authentication

Users SHALL authenticate using username and password OR email.

[Missing original scenarios - content will be lost!]
```

### Scenario Syntax (WHEN/THEN/AND)

Scenarios use a structured format:

**WHEN clause (precondition):**

- Describes the triggering condition or action
- Always starts a scenario
- Required for every scenario

**THEN clause (outcome):**

- Describes the expected result
- Follows WHEN clause
- Required for every scenario

**AND clause (additional outcomes):**

- Describes additional expected results
- Follows THEN or another AND clause
- Optional but recommended

**Example:**

```markdown
#### Scenario: Successful file upload

- **WHEN** user selects file under 10MB
- **AND** file format is supported
- **THEN** file is uploaded to server
- **AND** upload progress is displayed
- **AND** success message is shown
```

### Validation Rules

Spec Oxide validates specs and changes for structural correctness.

#### Spec Validation Rules

**Structure:**

- ✓ Must have `## Purpose` section
- ✓ Must have `## Requirements` section
- ⚠ Purpose section should be ≥50 characters

**Requirements:**

- ✓ Must use `### Requirement:` format
- ⚠ Should include descriptive text
- ⚠ Should use normative language (SHALL/MUST)
- ⚠ Should have at least one scenario

**Scenarios:**

- ✓ Must have WHEN clause
- ✓ Must have THEN clause
- ✓ Must use `#### Scenario:` format (h4 header)

#### Change Validation Rules

**Proposal:**

- ✓ Must have `proposal.md` file
- ✓ Must have `## Why` section
- ✓ Must have `## What Changes` section
- ⚠ Why section should be ≥50 characters

**Tasks:**

- ✓ Must have `tasks.md` file
- ✓ Must contain at least one checkbox item (`- [ ]` or `- [x]`)
- ⚠ Task items should use numbered prefixes (e.g., `1.1`, `2.3.1`)

**Deltas:**

- ✓ Must have at least one delta spec
- ✓ Must use valid headers (ADDED, MODIFIED, REMOVED, RENAMED)
- ✓ Requirements must have at least one scenario
- ✓ Scenarios must have WHEN and THEN clauses

### Common Validation Errors

| Error                             | Cause                        | Fix                                                 |
|-----------------------------------|------------------------------|-----------------------------------------------------|
| "Missing Purpose section"         | No `## Purpose` in spec      | Add `## Purpose` section with description           |
| "Missing Requirements section"    | No `## Requirements` in spec | Add `## Requirements` section                       |
| "Missing proposal.md"             | Change lacks proposal        | Create `proposal.md` with Why/What Changes sections |
| "Missing tasks.md"                | Change lacks tasks           | Create `tasks.md` with checkbox items               |
| "Must have at least one delta"    | No delta specs in change     | Add `specs/<capability>/spec.md` with delta headers |
| "Must have at least one scenario" | Requirement has no scenarios | Add `#### Scenario:` with WHEN/THEN clauses         |
| "Scenario missing WHEN clause"    | Scenario lacks precondition  | Add `- **WHEN** [condition]` line                   |
| "Scenario missing THEN clause"    | Scenario lacks outcome       | Add `- **THEN** [outcome]` line                     |

### Validation Best Practices

**Before proposing:**

- Run `spox change validate` to catch issues early
- Ensure every requirement has at least one scenario
- Use normative language (SHALL/MUST) in requirements

**Before requesting approval:**

- Run `spox change validate` again
- Verify deltas use correct headers (ADDED/MODIFIED/REMOVED/RENAMED)
- Check scenario format (h4 headers, not bullets)

**After archiving:**

- Run `spox spec validate` to verify merged specs
- Confirm no validation errors or warnings
- Check that deltas were applied correctly

**In CI/CD:**

- Use `spox spec validate --strict` to fail on warnings
- Use `spox change validate --strict` to enforce high standards
- Run validation on every commit

### Troubleshooting Validation Issues

**"Silent parse failure" (validation passes but content missing):**

Run `spox change show <id> --deltas-only` to inspect parsed output. This shows exactly what the parser extracted.

**Common causes:**

- Wrong scenario format (bullets instead of h4 headers)
- Malformed delta headers
- Partial MODIFIED requirement text

**Validation passes but scenarios not detected:**

Check scenario format:

```markdown
#### Scenario: Name ✓

- **Scenario: Name**   ✗
```

**Validation fails with "Missing delta":**

Ensure `specs/_changes/<id>/specs/` directory contains `.md` files with valid delta headers:

```markdown
## ADDED Requirements

## MODIFIED Requirements

## REMOVED Requirements

## RENAMED Requirements
```

---

## Appendix

### File Templates

Spec Oxide includes templates for scaffolding specs and changes:

| Template                             | Purpose                        |
|--------------------------------------|--------------------------------|
| `.spox/templates/spec.md`            | Capability specification       |
| `.spox/templates/change/proposal.md` | Change proposal (Why/What)     |
| `.spox/templates/change/tasks.md`    | Implementation checklist       |
| `.spox/templates/change/design.md`   | Technical decisions (optional) |
| `.spox/templates/change/spec.md`     | Delta specification            |

**Usage:**

Copy templates when creating new specs or changes. The `/spox:propose` slash command uses these templates automatically.

### Directory Structure Reference

```
project/
├── .spox/
│   ├── config.toml                  # Configuration
│   ├── setup.sh                     # MCP server setup script
│   ├── custom/                      # Custom rule files
│   │   └── team-conventions.md
│   └── templates/                   # Templates (bundled in binary, copied on init)
│       ├── spec.md
│       └── change/
│           ├── proposal.md
│           ├── tasks.md
│           ├── design.md
│           └── spec.md
├── .claude/
│   ├── CLAUDE.md                    # Generated instructions (DO NOT EDIT)
│   ├── .gitignore                   # Excludes spox-managed files
│   ├── agents/                      # AI agent templates
│   │   ├── spox-implementer.md
│   │   ├── spox-reviewer.md
│   │   └── spox-verifier.md
│   └── commands/spox/               # Slash commands
│       ├── propose.md
│       ├── implement.md
│       ├── archive.md
│       └── vibe.md
└── specs/
    ├── mission.md                   # Project mission statement
    ├── <capability>/                # Capability specs
    │   └── spec.md
    ├── _changes/                    # Active change proposals
    │   └── <change-id>/
    │       ├── proposal.md
    │       ├── tasks.md
    │       ├── design.md            # Optional
    │       └── specs/
    │           └── <capability>/
    │               └── spec.md
    └── _archive/                    # Archived changes
        └── YYYY-MM-DD-<change-id>/
            ├── proposal.md
            ├── tasks.md
            └── specs/
                └── <capability>/
                    └── spec.md
```

### Change ID Naming Conventions

Change IDs follow specific rules:

**Format:** `kebab-case`, `verb-led`, `unique`

**Good examples:**

- `add-two-factor-auth`
- `update-payment-flow`
- `remove-legacy-api`
- `rename-user-service`
- `fix-login-timeout`

**Bad examples:**

- `auth-changes` (not verb-led)
- `fix` (too vague)
- `stuff` (meaningless)
- `AddFeature` (wrong case)
- `new_feature` (snake_case instead of kebab-case)

**Verbs to use:**

- `add` - New functionality
- `update` - Enhance existing functionality
- `fix` - Bug fix
- `remove` - Delete functionality
- `rename` - Name change
- `refactor` - Code restructuring
- `migrate` - Data or system migration

### AI Agent Reference

Spec Oxide includes AI agents optimized for different tasks:

| Agent              | Purpose                                | When to Use                    |
|--------------------|----------------------------------------|--------------------------------|
| `spox-implementer` | Implements features following tasks.md | During Stage 2 (Implement)     |
| `spox-reviewer`    | Performs code quality review           | Before merging code            |
| `spox-verifier`    | Verifies implementation against specs  | Before marking change complete |

**Spawning agents in Claude Code:**

```
@spox-implementer Implement task 1.1: Add OTP generation service
```

**Agent workflow:**

1. **Implementer** reads tasks, writes code
2. **Reviewer** checks code quality, suggests improvements
3. **Verifier** confirms implementation matches spec

### Full-Text Search

For pattern-based search across specs, use `rg` (ripgrep):

```bash
# Search for requirement by name
rg -n "Requirement: Password" specs

# Search for scenarios
rg -n "Scenario:" specs

# Search for WHEN/THEN patterns
rg -n "WHEN.*login" specs

# Search in changes only
rg -n "ADDED Requirements" specs/_changes
```

**Note:** Prefer MCP tools (Serena) for semantic code search. Use `rg` only for text-based searches in spec files.

### Getting Help

**Command help:**

```bash
spox --help
spox spec --help
spox change --help
```

**Validation help:**

```bash
# See detailed validation errors
spox spec validate <id>
spox change validate <id>

# Debug parsing issues
spox change show <id> --deltas-only
```

**Community:**

- GitHub Issues: Report bugs or request features
- Discussions: Ask questions or share ideas
- Documentation: Check README and specs for details

---

## System Standards Reference

Spec Oxide bundles system standards that are merged into `.claude/CLAUDE.md` during initialization. These standards
provide guardrails and best practices for AI-assisted development.

### Available Standards

| File                                                     | Description                                                                                                        |
|----------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------|
| [`global.md`](../templates/spox/standards/global.md)     | General development mindset: simplicity first, minimal dependencies, complexity triggers, error handling patterns  |
| [`coding.md`](../templates/spox/standards/coding.md)     | Core coding principles (KISS, DRY, YAGNI), naming conventions, function design, code organization, and code smells |
| [`testing.md`](../templates/spox/standards/testing.md)   | TDD guardrails, red-green-refactor cycle, test quality guidelines, and verification checklists                     |
| [`backend.md`](../templates/spox/standards/backend.md)   | REST API conventions, database migrations, model design, and query best practices                                  |
| [`frontend.md`](../templates/spox/standards/frontend.md) | UI accessibility, component design, CSS best practices, and responsive design patterns                             |
| [`vcs.md`](../templates/spox/standards/vcs.md)           | Git guardrails (read-only for AI), conventional commit format, and version control discipline                      |
| [`mcp.md`](../templates/spox/standards/mcp.md)           | MCP tool preferences: Serena for code operations, Context7 for documentation, decision flows                       |

### Configuring Standards

Enable or disable standards in `.spox/config.toml`:

```toml
[rules]
# Include all standards (default)
system = ["global", "coding", "testing", "backend", "frontend", "vcs", "mcp"]

# Or select only what you need
system = ["global", "coding", "mcp"]
```

### Standards Details

#### global.md

Establishes the foundational mindset for development:

- Keep directory structures consistent and discoverable
- Use environment variables for configuration
- Minimize dependencies and justify additions
- Default to simple, single-file implementations
- Add complexity only with concrete evidence

#### coding.md

Defines code quality principles:

- **KISS** - Simpler is always better
- **DRY** - Eliminate duplication
- **YAGNI** - Build only what is required now
- **Single Responsibility** - One function, one purpose
- Naming, function design, and structural guidelines

#### testing.md

Enforces Test-Driven Development:

- **No production code without a failing test first**
- Red → Green → Refactor cycle
- Verification checkpoints at each stage
- Guidelines for good tests and common pitfalls

#### backend.md

API and database standards:

- RESTful endpoint conventions (`GET /resources`, `POST /resources`)
- Response codes and headers
- Migration best practices (one change per migration, rollback support)
- Query security (parameterized queries, no interpolation)

#### frontend.md

UI development guidelines:

- Semantic HTML and accessibility requirements
- Component design principles (single responsibility, composable)
- CSS methodology consistency
- Responsive design (mobile-first, fluid layouts)

#### vcs.md

Version control discipline:

- **AI agents must NEVER write to git** (read-only)
- Allowed commands: `git status`, `git diff`, `git log`, `git show`
- Conventional commit format with types: `feat:`, `fix:`, `refactor:`, etc.

#### mcp.md

MCP tool usage guidelines:

- **Serena first** for code navigation, understanding, and editing
- **Context7 first** for library documentation lookups
- Symbolic editing workflow (find → understand → edit → verify)
- Decision flow for choosing the right tool

---

## Summary

Spec Oxide enables deterministic, spec-driven development for AI-assisted coding:

1. **Install** - Build `spox` CLI and run `.spox/setup.sh`
2. **Propose** - Create change proposals with `/spox:propose`
3. **Implement** - Execute approved changes with `/spox:implement`
4. **Archive** - Merge to specs with `/spox:archive`
5. **Validate** - Ensure correctness with `spox validate` commands

**Core principle:** Specs are the source of truth. Changes are proposals that modify that truth. Keep them in sync.

**Key benefits:**

- Lock intent before implementation
- Create reviewable, deterministic outputs
- Maintain up-to-date documentation
- Enable AI agents to work from specs, not vague prompts

**Next steps:**

1. Initialize your project: `spox init`
2. Set up MCP servers: `.spox/setup.sh`
3. Read your project mission: `specs/mission.md`
4. Create your first proposal: `/spox:propose`

Happy spec-driven development!
