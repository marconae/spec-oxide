# Setup Guide

> Get up and running in a couple minutes—no additional API keys required.

This guide walks you through installation, first-time configuration, and updating your project.

## Quick Start

```bash
# 1. Install spox
cargo install --git https://github.com/marconae/spec-oxide

# 2. Initialize your project
cd /path/to/your/project
spox init

# 3. Set up MCP servers
.spox/setup.sh

# 4. Enable project MCP servers in Claude Code settings
# Add to .claude/settings.json:
# "enableAllProjectMcpServers": true

# 5. Run Claude Code
claude
```

## Prerequisites

| Tool            | Purpose              | Install                                                              |
|-----------------|----------------------|----------------------------------------------------------------------|
| **Cargo**       | Build the `spox` CLI | [rustup.rs](https://rustup.rs/)                                      |
| **Claude Code** | AI coding assistant  | [docs.anthropic.com](https://docs.anthropic.com/en/docs/claude-code) |
| **jq**          | JSON manipulation    | `brew install jq` / `apt install jq`                                 |
| **uv**          | Serena MCP server    | [docs.astral.sh/uv](https://docs.astral.sh/uv/)                      |

**Platform Support:**

- ✅ Linux — Fully supported
- ✅ macOS — Fully supported
- ⚠️ Windows — Use WSL (Windows Subsystem for Linux)

## Step 1: Install spox

Use Cargo to install the `spox` CLI:

```bash
cargo install --git https://github.com/marconae/spec-oxide
```

Verify installation:

```bash
spox --version
```

## Step 2: Initialize Your Project

Navigate to your project and run:

```bash
cd /path/to/your/project
spox init
```

This creates:

```
your-project/
├── .mcp.json             # MCP server configuration
├── .spox/
│   ├── config.toml       # Spox configuration
│   ├── setup.sh          # MCP setup script
│   ├── custom/           # Your custom rules
│   └── templates/        # Spec templates
├── .claude/
│   ├── agents/           # AI agents (implementer, reviewer, verifier)
│   ├── commands/spox/    # Slash commands (propose, implement, archive, vibe)
│   └── CLAUDE.md         # Generated instructions for Claude
└── specs/
    ├── mission.md        # Project mission (edit this!)
    ├── _changes/         # Active proposals
    └── _archive/         # Completed changes
```

**First thing to do:** Edit `specs/mission.md` to describe your project's purpose, tech stack, and conventions.

## Step 3: Set Up MCP Servers

Run the interactive setup script:

```bash
.spox/setup.sh
```

The script will:

1. ✓ Check prerequisites (jq, uv)
2. ✓ Configure Spox MCP (built-in, always enabled)
3. ✓ Configure Serena MCP (semantic code operations)
4. ✓ Configure Context7 MCP (documentation lookups)
5. ✓ Index your project for Serena (optional)

All configuration is stored in `.mcp.json` at your project root.

**What are MCP servers?**

| Server       | Purpose                                                                        |
|--------------|--------------------------------------------------------------------------------|
| **Spox**     | Lists, searches, and validates specs and changes                               |
| **Serena**   | Understands your code semantically — find symbols, references, refactor safely |
| **Context7** | Provides up-to-date library docs — no more outdated API assumptions            |

See the [MCP Servers Guide](mcp.md) for detailed tool references and usage.

## Step 4: Enable Project MCP Servers

Claude Code must be configured to load project-level MCP servers. Add this to your Claude Code settings:

**Option A: Project settings** (`.claude/settings.json`):

```json
{
  "enableAllProjectMcpServers": true
}
```

**Option B: Global settings** (`~/.claude/settings.json`):

```json
{
  "enableAllProjectMcpServers": true
}
```

Without this setting, the MCP servers defined in `.mcp.json` will not be loaded.

## Updating an Existing Project

If your project already has Spec Oxide, run init again to update:

```bash
spox init
```

This will:

- Update agents and commands to latest versions
- Preserve your `specs/mission.md`
- Preserve your custom rules in `.spox/custom/`
- Regenerate `.claude/CLAUDE.md` with current templates

After updating, re-run the setup script to ensure MCP configuration is current:

```bash
.spox/setup.sh
```

## Troubleshooting

### "command not found: spox"

Ensure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.).

### MCP servers not loading

1. Verify `enableAllProjectMcpServers` is set to `true` in Claude Code settings
2. Verify `.mcp.json` exists and is valid: `cat .mcp.json | jq`
3. Check MCP server status in Claude Code: `/mcp`
4. Re-run setup: `.spox/setup.sh`

### Serena not finding symbols

Index your project:

```bash
uvx --from git+https://github.com/oraios/serena serena project index
```

### jq not found

Install jq (required for the setup script):

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq

# Fedora
sudo dnf install jq
```

## Build from Source

```bash
git clone https://github.com/marconae/spec-oxide.git
cd spec-oxide
cargo build --release
cargo install --path .
```

Verify installation:

```bash
spox --version
```
