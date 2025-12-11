# Setup Guide | Spec Oxide

> Get up and running in a couple minutes-no additional API keys required.

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

# 4. Run claude code
claude
```

## Prerequisites

| Tool            | Purpose              | Install                                                              |
|-----------------|----------------------|----------------------------------------------------------------------|
| **Cargo**       | Build the `spox` CLI | [rustup.rs](https://rustup.rs/)                                      |
| **Claude Code** | AI coding assistant  | [docs.anthropic.com](https://docs.anthropic.com/en/docs/claude-code) |
| **uv**          | Serena MCP server    | [docs.astral.sh/uv](https://docs.astral.sh/uv/)                      |
| **Node.js 18+** | Context7 MCP server  | [nodejs.org](https://nodejs.org/)                                    |

**Platform Support:**

- ✅ Linux — Fully supported
- ✅ macOS — Fully supported
- ⚠️ Windows — Use WSL (Windows Subsystem for Linux)

## Step 1: Install spox

Use Cargo to install the `spox` CLI:

```bash
cargo install --git https://github.com/anthropics/spec-oxide
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
├── .spox/
│   ├── config.toml      # Configuration
│   ├── setup.sh         # MCP setup script
│   ├── custom/          # Your custom rules
│   └── templates/       # Spec templates
├── .claude/
│   ├── agents/          # AI agents (implementer, reviewer, verifier)
│   ├── commands/spox/   # Slash commands (propose, implement, archive)
│   └── CLAUDE.md        # Generated instructions for Claude
└── specs/
    ├── mission.md       # Project mission (edit this!)
    ├── _changes/        # Active proposals
    └── _archive/        # Completed changes
```

**First thing to do:** Edit `specs/mission.md` to describe your project's purpose, tech stack, and conventions.

## Step 3: Set Up MCP Servers

Run the interactive setup script:

```bash
.spox/setup.sh
```

The script will:

1. ✓ Check prerequisites (Claude Code, uv, Node.js)
2. ✓ Install Serena MCP (semantic code operations)
3. ✓ Install Context7 MCP (documentation lookups)
4. ✓ Index your project for Serena

**What are MCP servers?**

| Server       | Purpose                                                                        |
|--------------|--------------------------------------------------------------------------------|
| **Serena**   | Understands your code semantically — find symbols, references, refactor safely |
| **Context7** | Provides up-to-date library docs — no more outdated API assumptions            |

Both are required for the full Spec Oxide workflow.

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

## Troubleshooting

### "command not found: spox"

Ensure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.).

### MCP servers not working

1. Verify `.mcp.json` exists and is valid: `cat .mcp.json | jq`
2. Verify jq is installed: `jq --version`
3. Re-run setup: `.spox/setup.sh`

### Serena not finding symbols

Index your project:

```bash
uvx --from git+https://github.com/oraios/serena serena project index
```

## Next Steps

1. **Read the workflow** — Understand [Propose → Implement → Archive](user-guide.md#workflow)
2. **Edit your mission** — Customize `specs/mission.md` for your project
3. **Create your first proposal** — Run `/spox:propose` with a real task
4. **Explore the CLI** — See all commands with `spox --help`

Ready to dive deeper? See the full [User Guide](user-guide.md).

## Appendix

### Build from Source

```bash
git clone https://github.com/anthropics/spec-oxide.git
cd spec-oxide
cargo build --release
cargo install --path .
```

Verify installation:

```bash
spox --version
```
