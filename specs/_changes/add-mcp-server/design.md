## Context

Spec files are the source of truth for implementation, but loading all specs into an LLM's context window is expensive.
A typical spec corpus of 50 files consumes 30-50k tokens. MCP (Model Context Protocol) enables progressive disclosure:
the LLM loads only what it needs, when it needs it.

**Stakeholders:**

- Claude Code users who want token-efficient spec access
- Developers maintaining the spox CLI

**Constraints:**

- Must work with Claude Code's MCP integration (stdio transport)
- Must support offline operation (no external API calls for search)
- Must integrate cleanly with existing spox CLI structure

## Goals / Non-Goals

**Goals:**

- Expose spec content via MCP tools for progressive disclosure
- Provide semantic search across specs without external dependencies
- Keep the implementation simple and maintainable
- Support standard MCP stdio transport for Claude Code integration
- Clean separation between core logic and interface layers

**Non-Goals:**

- Real-time spec file watching (user rebuilds index manually)
- Multi-user concurrent access (single-user CLI tool)
- SSE/HTTP transport (stdio only for initial release)
- Change proposal tools (read-only spec access for now)

## Decisions

### Decision 1: Core/Interface Separation Architecture

The codebase SHALL follow a layered architecture:

```
┌─────────────┐    ┌─────────────┐
│   CLI       │    │    MCP      │  ← Interfaces (presentation)
└──────┬──────┘    └──────┬──────┘
       │                  │
       └────────┬─────────┘
                │
       ┌────────▼────────┐
       │      Core       │  ← Business logic
       └─────────────────┘
```

**Core** (`src/core/`):

- Spec parsing and validation
- Search index building and querying
- Configuration management
- Returns structured data (Rust types)

**Interfaces** (`src/cli/`, `src/mcp/`):

- CLI: Formats core output for terminal (tables, colors, text)
- MCP: Formats core output for MCP protocol (JSON tool responses)
- Neither contains business logic

**Benefits:**

- Testable core without interface dependencies
- Consistent behavior across CLI and MCP
- Easy to add new interfaces (HTTP API, TUI, etc.)

**Alternatives considered:**

- Feature flags for optional components: Adds complexity, harder to test
- Monolithic design: Harder to maintain, test, and extend

### Decision 2: Use official Rust MCP SDK (`rmcp`)

The official `rmcp` crate from `modelcontextprotocol/rust-sdk` provides:

- Async-first design with Tokio
- `#[tool]` and `#[tool_router]` macros for declarative tool definitions
- Built-in stdio transport
- Active maintenance from Anthropic

**Alternatives considered:**

- `rust-mcp-sdk` (third-party): Less documentation, smaller community
- Manual JSON-RPC implementation: More work, no benefit

### Decision 3: Use `fastembed` for local embeddings

For semantic search, `fastembed-rs` provides:

- Pure Rust with ONNX runtime (no Python dependency)
- `all-MiniLM-L6-v2` model: ~23MB, 384-dim vectors, ~5ms per query
- Local inference without external API calls

**Alternatives considered:**

- External embedding API (OpenAI, etc.): Adds network dependency, API keys, latency
- Keyword search only: Misses semantic matches ("auth" vs "login")
- `candle` + manual model loading: More complex, same result

### Decision 4: Brute-force cosine similarity for search

For a small corpus (<100 specs, ~500 requirements), brute-force search is fast enough:

- 500 comparisons at <1ms is negligible
- No need for HNSW, Annoy, or other ANN indexes
- Simpler implementation, easier debugging

**Alternatives considered:**

- `hnswlib-rs`: Overkill for small corpus, adds dependency complexity
- SQLite FTS5: Good for keyword search, not semantic

### Decision 5: Binary index file format

Store embeddings in `.spox/search_index.bin` using `bincode`:

- Fast serialization/deserialization
- Compact storage
- No schema migration concerns (rebuild index if format changes)

**Alternatives considered:**

- JSON: Larger files, slower parsing for float arrays
- SQLite: More complex, no benefit for this use case

### Decision 6: Separate `spox mcp serve` and `spox index` commands

- `spox mcp serve`: Starts MCP server, reads index if available
- `spox index`: Builds/rebuilds search index

This separation allows:

- MCP server to work without search (graceful degradation)
- Index rebuild without restarting server
- Clear user control over when embeddings are generated

### Decision 7: No feature flags for search

Include all functionality in the default build:

- Simpler build process
- Consistent binary behavior
- Binary size increase (~10-15MB) is acceptable for a CLI tool

**Alternatives considered:**

- Optional `--features search`: Adds testing complexity, user confusion

## Risks / Trade-offs

### Risk: Model download on first use

`fastembed` downloads the ONNX model (~23MB) on first use.

**Mitigation:**

- Document this behavior clearly
- Show download progress
- Cache in standard location (`~/.cache/fastembed`)

### Risk: Index becomes stale

If specs change, index may return outdated results.

**Mitigation:**

- `spox index` command for manual rebuild
- Index stores spec file hashes; warn if mismatch detected
- Future: file watcher for auto-rebuild

### Risk: Binary size increase

`fastembed` and ONNX runtime add to binary size.

**Mitigation:**

- Accept the trade-off; ~15MB is reasonable for a dev tool
- Document size expectations

### Trade-off: Stdio-only transport

SSE/HTTP would allow remote access but adds complexity.

**Decision:** Start with stdio (Claude Code's primary transport). Add HTTP later if needed.

## Migration Plan

No migration needed—this is additive functionality:

1. Restructure existing code into `src/core/` and `src/cli/`
2. Add new `src/mcp/` module
3. Add new dependencies
4. Add new commands (`mcp serve`, `index`)
5. Add new spec (`mcp`)
6. Existing CLI behavior unchanged

**Rollback:** Revert commits; no data migration needed.

## Open Questions

1. **Index location:** `.spox/search_index.bin` or user-configurable?
    - **Proposed:** Fixed location for simplicity; add config later if requested

2. **Model selection:** Allow user to choose embedding model?
    - **Proposed:** Hardcode `all-MiniLM-L6-V2` initially; good enough for this use case

3. **Change proposal tools:** Should MCP also expose change operations?
    - **Proposed:** Out of scope for initial release; focus on read-only spec access
