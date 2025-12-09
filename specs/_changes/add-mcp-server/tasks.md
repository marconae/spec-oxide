# Tasks add-mcp-server

## 1. Project Setup

- [x] 1.1 Add dependencies to Cargo.toml
  - [x] 1.1.1 Add `rmcp` with `server` and `transport-io` features
  - [x] 1.1.2 Add `tokio` with `full` features
  - [x] 1.1.3 Add `fastembed`
  - [x] 1.1.4 Add `bincode` for index serialization
  - [x] 1.1.5 Add `serde_json` for MCP tool responses

## 2. Core/Interface Restructure

- [x] 2.1 Create `src/core/` module structure
  - [x] 2.1.1 Create `src/core/mod.rs` with public exports
  - [x] 2.1.2 Move config logic to `src/core/config.rs`
  - [x] 2.1.3 Move error types to `src/core/error.rs`
- [x] 2.2 Create `src/cli/` module structure
  - [x] 2.2.1 Create `src/cli/mod.rs` with CLI entry point
  - [x] 2.2.2 Move CLI parsing to `src/cli/commands.rs`
  - [x] 2.2.3 Move formatting/rendering to `src/cli/render.rs`
- [x] 2.3 Migrate existing commands to new structure
  - [x] 2.3.1 Migrate `spec list` command
  - [x] 2.3.2 Migrate `spec show` command
  - [x] 2.3.3 Migrate `spec validate` command
  - [x] 2.3.4 Migrate `change list` command
  - [x] 2.3.5 Migrate `change show` command
  - [x] 2.3.6 Migrate `change validate` command
  - [x] 2.3.7 Migrate `config show` command
  - [x] 2.3.8 Migrate `init` command
  - [x] 2.3.9 Migrate `show` (dashboard) command

## 3. Spec Parser (Core)

- [x] 3.1 Create public spec parsing types in `src/core/spec.rs`
  - [x] 3.1.1 Create `ParsedSpec` struct with id, title, purpose, requirements
  - [x] 3.1.2 Create `ParsedRequirement` struct with name, description, scenarios
  - [x] 3.1.3 Create `ParsedScenario` struct with name, when_clause, then_clauses
- [x] 3.2 Create spec parsing functions
  - [x] 3.2.1 Extract parsing logic from validation into reusable functions
  - [x] 3.2.2 Add `parse_spec_file(path) -> Result<ParsedSpec>`
  - [x] 3.2.3 Add `parse_all_specs(folder) -> Result<Vec<ParsedSpec>>`
  - [x] 3.2.4 Add `get_spec_by_id(folder, id) -> Result<ParsedSpec>`

## 4. Search Index (Core)

- [x] 4.1 Create index data structures in `src/core/index.rs`
  - [x] 4.1.1 Define `SpecIndex` struct with specs and model_name
  - [x] 4.1.2 Define `IndexedSpec` struct with embeddings
  - [x] 4.1.3 Define `IndexedRequirement` struct with embeddings
  - [x] 4.1.4 Define `SearchResult` struct
- [x] 4.2 Implement index building
  - [x] 4.2.1 Implement `build_index(specs) -> Result<SpecIndex>`
  - [x] 4.2.2 Generate embeddings for spec purposes
  - [x] 4.2.3 Generate embeddings for requirement descriptions
- [x] 4.3 Implement index persistence
  - [x] 4.3.1 Implement `save_index(index, path) -> Result<()>`
  - [x] 4.3.2 Implement `load_index(path) -> Result<SpecIndex>`
- [x] 4.4 Implement search
  - [x] 4.4.1 Implement `search(index, query, top_k) -> Result<Vec<SearchResult>>`
  - [x] 4.4.2 Implement cosine similarity calculation

## 5. CLI Commands

- [x] 5.1 Add `index` subcommand
  - [x] 5.1.1 Add `IndexCmd` to CLI parser
  - [x] 5.1.2 Implement index command handler in `src/cli/index_cmd.rs`
  - [x] 5.1.3 Display progress during embedding generation
- [x] 5.2 Add `mcp` subcommand with `serve` action
  - [x] 5.2.1 Add `McpCmd` with `Serve` variant to CLI parser
  - [x] 5.2.2 Add async runtime initialization in main

## 6. MCP Server Interface

- [x] 6.1 Create MCP server module (`src/mcp/mod.rs`)
  - [x] 6.1.1 Define `SpoxServer` struct holding config and optional index
  - [x] 6.1.2 Implement `ServerHandler` trait for `SpoxServer`
  - [x] 6.1.3 Configure server info and capabilities
- [x] 6.2 Implement `list_specs` tool
  - [x] 6.2.1 Call core `parse_all_specs`
  - [x] 6.2.2 Format as JSON array of {id, title, purpose}
- [x] 6.3 Implement `get_spec_requirements` tool
  - [x] 6.3.1 Accept `spec_id` parameter
  - [x] 6.3.2 Call core `get_spec_by_id`
  - [x] 6.3.3 Format requirements with scenario counts
- [x] 6.4 Implement `get_scenario` tool
  - [x] 6.4.1 Accept `spec_id`, `requirement`, optional `scenario` parameters
  - [x] 6.4.2 Return requirement description and scenario content
- [x] 6.5 Implement `search_specs` tool
  - [x] 6.5.1 Check if index is loaded
  - [x] 6.5.2 Call core `search` function
  - [x] 6.5.3 Format as ranked results

## 7. Testing

- [x] 7.1 Unit tests for core spec parsing
  - [x] 7.1.1 Test parsing valid spec file
  - [x] 7.1.2 Test parsing spec with multiple requirements
  - [x] 7.1.3 Test parsing spec with multiple scenarios
- [x] 7.2 Unit tests for core search index
  - [x] 7.2.1 Test index build and save
  - [x] 7.2.2 Test index load
  - [x] 7.2.3 Test search with matches
  - [x] 7.2.4 Test search with no matches
- [x] 7.3 Integration tests for MCP tools
  - [x] 7.3.1 Test `list_specs` with fixture specs
  - [x] 7.3.2 Test `get_spec_requirements` with valid/invalid IDs
  - [x] 7.3.3 Test `get_scenario` with valid/invalid parameters
  - [x] 7.3.4 Test `search_specs` with and without index
- [x] 7.4 Integration tests for CLI commands
  - [x] 7.4.1 Test `spox index` creates index file
  - [x] 7.4.2 Test `spox mcp serve` starts and responds to initialize
  - [x] 7.4.3 Test existing CLI commands still work after restructure

## 8. Documentation

- [x] 8.1 Update README with MCP server usage
- [x] 8.2 Add example Claude Code MCP configuration snippet

# Parallelization

Tasks can be parallelized as follows:
- **Phase 1 (sequential):** 1.1 (dependencies) must complete first
- **Phase 2 (sequential):** 2.x (restructure) must complete before new features
- **Phase 3 (parallel):** 3.x (spec parser) and 4.x (search index) can run in parallel
- **Phase 4 (parallel):** 5.x (CLI commands) and 6.x (MCP server) can run in parallel after Phase 3
- **Phase 5:** 7.x tests should follow TDD (write alongside implementation)
- **Phase 6:** 8.x documentation after implementation complete

# Verification

1. `cargo build` compiles without warnings
2. `cargo test` passes all unit and integration tests
3. Existing CLI commands work unchanged after restructure
4. `spox index` creates `.spox/search_index.bin`
5. `spox mcp serve` responds to MCP initialize request
6. Claude Code can connect to spox MCP and use all four tools
7. Manual test: search for "validation" returns relevant results

# Notes

- **Restructure first:** Complete Phase 2 before adding new features. This ensures clean separation from the start.
- **TDD:** Write failing tests before implementing each function.
- **Core purity:** Core functions MUST NOT import anything from `cli` or `mcp` modules.
- **The `fastembed` crate downloads the ONNX model (~23MB) on first use.** This happens during `spox index`, not during `spox mcp serve`.
- **Claude Code configuration example:**
  ```json
  {
    "mcpServers": {
      "spox": {
        "command": "spox",
        "args": ["mcp", "serve"]
      }
    }
  }
  ```
