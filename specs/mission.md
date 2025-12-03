# Project Mission

## Purpose

Spec Oxide (spox) enables spec-driven development for AI-assisted coding. AI coding assistants are powerful but unpredictable when requirements live only in chat history. Spec Oxide locks intent before implementation: you agree on *what* to build before writing any code, giving you deterministic, reviewable outputs.

**Core principle:** Specs are the source of truth. Changes are proposals that modify that truth. Keep them in sync.

## Tech Stack

- **Language:** Rust (edition 2021)
- **CLI framework:** clap with derive macros
- **Serialization:** serde, toml
- **Testing:** `cargo test` (using assert_cmd, predicates, tempfile)
- **Linting:** `cargo clippy`
- **Formatting:** `cargo fmt` (rustfmt)
- **Building:** `cargo build`

## Project Conventions

### Architecture Patterns

This is a CLI application built with clap. The `spox` binary augments a template-driven workflow system optimized for Claude Code.

### Naming

- CLI binary: `spox` (short for spec-oxide)
- Change IDs: kebab-case, verb-led, unique (e.g., `add-two-factor-auth`)

## Domain Context

- **Spec:** A capability specification describing what the system does (source of truth)
- **Change:** A proposal to modify specs, containing deltas
- **Delta:** The difference between current and proposed spec (ADDED/MODIFIED/REMOVED/RENAMED requirements)
- **Capability:** A logical grouping of related requirements (e.g., auth, notifications)
- **Archive:** Historical record of implemented changes

The workflow follows three stages: **Propose → Implement → Archive**

## Important Constraints

- Requires Claude Code for the AI-assisted workflow
- Requires Serena MCP for semantic code operations
- Requires Context7 MCP for documentation lookups

## Performance Considerations

Not a primary concern. The tool is designed for projects with tens to hundreds of specs, not thousands.

## External Dependencies

- **Current:** Self-contained CLI with no external service dependencies
- **Planned:** GitHub Action for CI validation
