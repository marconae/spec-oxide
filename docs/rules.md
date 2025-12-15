# Rules and Best Practices

Spec Oxide enforces rules and best practices through templates that are merged into your project's `CLAUDE.md` file.
These standards guide AI agents to produce consistent, maintainable code.

## How Standards Work

When you run `spox init`, the standards templates are combined into `.claude/CLAUDE.md`. Claude Code reads this file
and follows the guidelines when writing code for your project.

You can customize which standards are included via `.spox/config.toml`:

```toml
[rules]
system = ["mcp", "global", "coding", "testing", "backend", "frontend", "vcs"]
custom = []
```

## Standard Templates

### Global Mindset

General principles for project organization and simplicity.

- Consistent directory structure
- Environment variables for config (never commit secrets)
- Minimal dependencies with justification
- Simplicity first: default to <100 lines of new code
- Verification: TDD with >80% coverage target

[View template](https://github.com/marconae/spec-oxide/blob/main/templates/spox/standards/global.md)

### Coding Standards

Core principles and patterns for clean code.

- **KISS**: Keep it simple
- **DRY**: Don't repeat yourself
- **YAGNI**: You aren't gonna need it
- **Single Responsibility**: One function, one purpose
- Naming conventions, function design, code organization
- Code smells to avoid

[View template](https://github.com/marconae/spec-oxide/blob/main/templates/spox/standards/coding.md)

### Testing

Test-driven development guardrails.

- **Red-Green-Refactor** cycle enforcement
- No production code without failing test first
- Unit test and integration test requirements
- Bug fix protocol: reproduce with test first

[View template](https://github.com/marconae/spec-oxide/blob/main/templates/spox/standards/testing.md)

### Backend

API and database conventions.

- RESTful endpoint patterns (GET, POST, PUT, PATCH, DELETE)
- URL rules, response codes, versioning
- Database migration best practices
- Query security (parameterized queries, no interpolation)

[View template](https://github.com/marconae/spec-oxide/blob/main/templates/spox/standards/backend.md)

### Frontend

UI and accessibility standards.

- Accessibility requirements (semantic HTML, keyboard navigation, ARIA)
- Component design principles (single responsibility, composable, reusable)
- CSS best practices
- Responsive design patterns

[View template](https://github.com/marconae/spec-oxide/blob/main/templates/spox/standards/frontend.md)

### Version Control

Git safety guardrails.

- **Read-only by default**: Agents inspect but don't write to git
- Allowed commands: `git status`, `git diff`, `git log`, `git show`, `git branch`
- Forbidden commands: `git add`, `git commit`, `git push`, and all write operations
- Commit conventions when user explicitly requests commits

[View template](https://github.com/marconae/spec-oxide/blob/main/templates/spox/standards/vcs.md)

### MCP Tools

Tool usage priorities for AI agents.

- **Priority order**: Spox → Serena → Context7 → text tools
- Spox for spec operations
- Serena for semantic code understanding
- Context7 for library documentation

[View template](https://github.com/marconae/spec-oxide/blob/main/templates/spox/standards/mcp.md)

## Custom Standards

Add project-specific standards by creating markdown files in `.spox/custom/` and listing them in config:

```toml
[rules]
system = ["mcp", "global", "coding"]
custom = ["my-project-rules.md", "team-conventions.md"]
```

Custom templates are appended after system templates in `CLAUDE.md`.

## Updating Standards

When Spec Oxide is updated with new standards, run `spox init` again to regenerate `CLAUDE.md` with the latest
templates. Your custom rules and `specs/mission.md` are preserved.
