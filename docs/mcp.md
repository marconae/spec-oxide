# MCP Servers | Spec Oxide

Spec Oxide integrates with three MCP (Model Context Protocol) servers to provide AI agents with semantic understanding
of specs, code, and external documentation.

## Overview

| Server   | Purpose                         | Source                                                      |
|----------|---------------------------------|-------------------------------------------------------------|
| Spox     | Spec and change management      | Built-in (`spox mcp serve`)                                 |
| Serena   | Semantic code understanding     | [oraios/serena](https://github.com/oraios/serena)           |
| Context7 | Library docs and API references | [context7.com](https://context7.com)                        |

**Priority rule:** Spox (specs) → Serena (code) → Context7 (docs) → text tools (fallback only)

## Configuration

### Enable Project MCP Servers

Claude Code must be configured to allow project-level MCP servers. Add this to your Claude Code settings
(`.claude/settings.json` or global settings):

```json
{
  "enableAllProjectMcpServers": true
}
```

Without this setting, the MCP servers defined in `.mcp.json` will not be loaded.

### Server Configuration

MCP servers are configured in `.mcp.json` at your project root:

```json
{
  "mcpServers": {
    "spox": {
      "command": "spox",
      "args": ["mcp", "serve"]
    },
    "serena": {
      "command": "uvx",
      "args": [
        "--from",
        "git+https://github.com/oraios/serena",
        "serena",
        "start-mcp-server",
        "--context",
        "claude-code",
        "--project",
        "."
      ]
    },
    "context7": {
      "type": "http",
      "url": "https://mcp.context7.com/mcp"
    }
  }
}
```

This file is created automatically when you run `spox init`.

## Spox MCP

The built-in Spox MCP server provides tools for spec and change management. **Always use for spec operations**—never
use CLI commands or direct file manipulation when working through an AI agent.

### Tool Reference

| Tool                   | Description                                    |
|------------------------|------------------------------------------------|
| `list_specs`           | List all capability specs in the project       |
| `get_spec_requirements`| Retrieve requirements from a specific spec     |
| `get_scenario`         | Get details of a specific scenario             |
| `list_changes`         | List all active change proposals               |
| `get_change`           | Retrieve full details of a change proposal     |
| `search_specs`         | Full-text search across all specs and changes  |
| `validate_spec`        | Validate a spec file for correctness           |
| `validate_change`      | Validate a change proposal before approval     |

### Workflow

```
Explore → list_specs, list_changes, search_specs
Understand → get_spec_requirements, get_scenario, get_change
Validate → validate_spec, validate_change
```

### Example Usage

```
# List all specs
mcp__spox__list_specs

# Search for authentication-related specs
mcp__spox__search_specs query="authentication"

# Get requirements for a specific spec
mcp__spox__get_spec_requirements spec_id="auth"

# Validate a change proposal
mcp__spox__validate_change change_id="add-two-factor-auth"
```

## Serena MCP

Serena provides semantic code understanding and editing. **Always prefer over** `rg`, `grep`, `find`, `ag`, `ast-grep`,
or read/edit/write cycles.

### Prerequisites

Serena requires Python and `uvx` (from [uv](https://docs.astral.sh/uv/)):

```bash
# Install uv if not already installed
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### Tool Reference

| Task                   | Use                        | Avoid                  |
|------------------------|----------------------------|------------------------|
| List directory         | `list_dir`                 | `ls`, `find`           |
| Find files             | `find_file`                | `find`, `rg --files`   |
| File symbols           | `get_symbols_overview`     | `rg "class\|function"` |
| Symbol definition      | `find_symbol`              | `rg "function foo"`    |
| Symbol references      | `find_referencing_symbols` | `rg "foo("`            |
| Update function body   | `replace_symbol_body`      | read → edit → write    |
| Add code after symbol  | `insert_after_symbol`      | read → edit → write    |
| Add code before symbol | `insert_before_symbol`     | read → edit → write    |
| Rename across codebase | `rename_symbol`            | `rg` + manual edits    |

### Reflection Tools

Serena includes reflection tools to help agents maintain focus:

- `think_about_collected_information` — Use after exploration to synthesize findings
- `think_about_task_adherence` — Use during implementation to verify alignment
- `think_about_whether_you_are_done` — Use before completion to confirm task is finished

### Workflow

```
Explore → find_symbol, get_symbols_overview
Understand → find_referencing_symbols
Reflect → think_about_collected_information
Edit → replace_symbol_body, insert_*_symbol
Verify → find_referencing_symbols
Check → think_about_whether_you_are_done
```

## Context7 MCP

Context7 provides current library documentation and API references. **Prefer over** training data, assumptions, or web
search for documentation.

### Use Cases

- API signatures and parameters
- Library usage examples
- Framework patterns and best practices
- Verifying current library behavior

### Tool Reference

| Tool                 | Description                                      |
|----------------------|--------------------------------------------------|
| `resolve-library-id` | Find the Context7 ID for a library name          |
| `get-library-docs`   | Fetch documentation for a specific library       |

### Example Usage

```
# Find library ID
mcp__context7__resolve-library-id libraryName="react"

# Get documentation
mcp__context7__get-library-docs context7CompatibleLibraryID="/facebook/react" topic="hooks"
```

## Best Practices

### 1. Use the Right Tool for the Job

| Task                    | Use          |
|-------------------------|--------------|
| Spec operations         | Spox MCP     |
| Code navigation/editing | Serena MCP   |
| Library documentation   | Context7 MCP |
| Everything else         | Text tools   |

### 2. Prefer Symbolic Operations

Use Serena's semantic tools instead of pattern matching:

```
# Good: Semantic understanding
mcp__serena__find_symbol name_path_pattern="UserAuth" include_body=true

# Avoid: Pattern matching
grep -r "class UserAuth" .
```

### 3. Verify Library Behavior

Never assume library behavior. Use Context7 to verify:

```
# Before using a library API
mcp__context7__get-library-docs context7CompatibleLibraryID="/expressjs/express" topic="middleware"
```

### 4. Validate Before Proceeding

Always validate specs and changes before implementation:

```
# Validate all specs
mcp__spox__validate_spec

# Validate a specific change
mcp__spox__validate_change change_id="add-feature"
```

## Troubleshooting

### Spox MCP Not Starting

Ensure `spox` is in your PATH:

```bash
which spox
# Should return path to spox binary
```

If not found, follow the [Setup Guide](setup.md) to install Spec Oxide.

### Serena MCP Not Starting

Ensure `uvx` is available:

```bash
which uvx
# Should return path to uvx
```

If not found, install uv:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### Context7 Connection Issues

Context7 uses an HTTP connection. Verify network access:

```bash
curl -I https://mcp.context7.com/mcp
```

### Checking MCP Server Status

In Claude Code, use `/mcp` to view connected servers and their status.
