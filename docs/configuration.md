# Configuration

Spec Oxide uses a TOML configuration file located at `.spox/config.toml`.

## Configuration File Structure

```toml
[paths]
spec_folder = "specs"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp", "mindset", "standards", "errors", "backend", "ui", "git"]
custom = []
```

## Configuration Sections

### `[paths]` Section

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

### `[rules]` Section

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

## Custom Rules

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

## CLAUDE.md Generation

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