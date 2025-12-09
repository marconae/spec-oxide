## ADDED Requirements

### Requirement: Core Layer Architecture

The Core layer SHALL contain all business logic, independent of any interface (CLI, MCP).

#### Scenario: Core returns structured data

- **WHEN** any core function is called
- **THEN** it returns Rust types (structs, enums, Results)
- **AND** it does not perform any formatting, output, or I/O

#### Scenario: Core has no interface dependencies

- **WHEN** the core module is compiled
- **THEN** it does not import from CLI or MCP modules
- **AND** it can be tested in isolation

### Requirement: Spec Parsing

The Core SHALL provide functions to parse spec files into structured data.

#### Scenario: Parse single spec file

- **WHEN** `parse_spec_file(path)` is called with a valid spec file
- **THEN** it returns a `ParsedSpec` containing id, title, purpose, and requirements
- **AND** each requirement contains name, description, and scenarios
- **AND** each scenario contains name, when_clause, and then_clauses

#### Scenario: Parse all specs in folder

- **WHEN** `parse_all_specs(folder)` is called
- **THEN** it returns a list of all parsed specs in the folder
- **AND** each spec is identified by its folder name as the id

#### Scenario: Get spec by ID

- **WHEN** `get_spec_by_id(folder, id)` is called with a valid id
- **THEN** it returns the parsed spec matching that id

#### Scenario: Spec not found

- **WHEN** `get_spec_by_id(folder, id)` is called with an invalid id
- **THEN** it returns an error indicating the spec was not found

### Requirement: Change Parsing

The Core SHALL provide functions to parse change proposals into structured data.

#### Scenario: Parse single change

- **WHEN** `parse_change(path)` is called with a valid change directory
- **THEN** it returns a `ParsedChange` containing id, proposal, tasks, and deltas

#### Scenario: Parse all changes

- **WHEN** `parse_all_changes(folder)` is called
- **THEN** it returns a list of all parsed changes in the changes folder

### Requirement: Validation

The Core SHALL provide functions to validate specs and changes.

#### Scenario: Validate spec

- **WHEN** `validate_spec(spec)` is called
- **THEN** it returns validation results with any errors or warnings

#### Scenario: Validate change

- **WHEN** `validate_change(change)` is called
- **THEN** it returns validation results with any errors or warnings

### Requirement: Configuration

The Core SHALL provide functions to load and manage configuration.

#### Scenario: Load configuration

- **WHEN** `load_config(project_root)` is called
- **THEN** it returns the project configuration from `.spox/config.toml`

#### Scenario: Get configured paths

- **WHEN** configuration is loaded
- **THEN** `spec_folder`, `changes_folder`, and `archive_folder` paths are available

### Requirement: Search Index

The Core SHALL provide functions to build and query a semantic search index.

#### Scenario: Build search index

- **WHEN** `build_index(specs)` is called with parsed specs
- **THEN** it generates embeddings for spec purposes and requirement descriptions
- **AND** it returns a `SpecIndex` that can be persisted

#### Scenario: Save and load index

- **WHEN** `save_index(index, path)` is called
- **THEN** the index is serialized to the specified path
- **AND** `load_index(path)` can restore it

#### Scenario: Search index

- **WHEN** `search(index, query, top_k)` is called
- **THEN** it returns ranked search results
- **AND** each result includes spec_id, requirement, score, and snippet
