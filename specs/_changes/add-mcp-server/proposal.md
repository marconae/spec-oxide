# Change: Add MCP Server for Progressive Spec Disclosure

## Why

Spec files are the source of truth for implementation, but loading all specs into an LLM's context window is expensive. A typical spec corpus of 50 files consumes 30-50k tokens—context that could be used for reasoning.

MCP (Model Context Protocol) enables progressive disclosure: the LLM loads only what it needs, when it needs it. Instead of dumping entire specs into context, the LLM can:
1. List specs to orient itself (~50 tokens per spec)
2. Get requirement structure without scenario bodies
3. Fetch individual scenarios on demand
4. Search semantically across all specs

## What Changes

- **New `core` capability:** Extract business logic into a Core layer that both CLI and MCP can use
- **Modify `cli` capability:** Refactor CLI as a presentation layer that delegates to Core
- **New `mcp` capability:** Add MCP server as a presentation layer that delegates to Core
- Add `spox mcp serve` command (startable by Claude Code)
- Add `spox index` command to build a search index with embeddings
- Implement 4 MCP tools: `list_specs`, `get_spec_requirements`, `get_scenario`, `search_specs`
- Add dependencies: `rmcp` (Rust MCP SDK), `fastembed`, `bincode`, `tokio`, `serde_json`

## Impact

- Affected specs:
  - **core** (NEW): Business logic layer
  - **cli** (MODIFIED): Refactored as interface layer
  - **mcp** (NEW): MCP server interface layer
- Affected code:
  - New `src/core/` module (business logic)
  - Refactor existing code into `src/cli/` (presentation)
  - New `src/mcp/` module (MCP presentation)
  - Updates to `src/main.rs` for new commands
