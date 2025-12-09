## ADDED Requirements

### Requirement: MCP Interface Layer

The MCP server SHALL act as a presentation layer that formats Core output for MCP protocol.

#### Scenario: MCP delegates to Core

- **WHEN** any MCP tool is called
- **THEN** it calls the appropriate Core function
- **AND** it formats the Core response as JSON for MCP protocol
- **AND** it does not contain business logic

#### Scenario: MCP handles errors from Core

- **WHEN** a Core function returns an error
- **THEN** the MCP server formats the error as an MCP tool error
- **AND** includes a descriptive message

### Requirement: MCP Server Protocol

The MCP server SHALL implement the Model Context Protocol over stdio transport.

#### Scenario: Server responds to initialize

- **WHEN** an MCP client sends an `initialize` request
- **THEN** the server responds with its capabilities
- **AND** the capabilities include `tools`

#### Scenario: Server info

- **WHEN** the server capabilities are requested
- **THEN** the server name is "spox"
- **AND** the server includes instructions for usage

### Requirement: List Specs Tool

The MCP server SHALL provide a `list_specs` tool that returns a minimal index of all specs.

#### Scenario: List all specs

- **WHEN** the `list_specs` tool is called with no arguments
- **THEN** it calls Core `parse_all_specs`
- **AND** the response contains an array of spec summaries
- **AND** each summary includes `id`, `title`, and `purpose`
- **AND** the response does not include full requirement details

#### Scenario: No specs found

- **WHEN** the `list_specs` tool is called
- **AND** no spec files exist in the spec folder
- **THEN** the response contains an empty array

### Requirement: Get Spec Requirements Tool

The MCP server SHALL provide a `get_spec_requirements` tool that returns the requirement structure of a spec without scenario bodies.

#### Scenario: Get requirements for existing spec

- **WHEN** the `get_spec_requirements` tool is called with a valid `spec_id`
- **THEN** it calls Core `get_spec_by_id`
- **AND** the response contains the spec's requirements
- **AND** each requirement includes `name` and `scenario_count`
- **AND** the response does not include scenario bodies

#### Scenario: Spec not found

- **WHEN** the `get_spec_requirements` tool is called with an invalid `spec_id`
- **THEN** the tool returns an error indicating the spec was not found

### Requirement: Get Scenario Tool

The MCP server SHALL provide a `get_scenario` tool that returns a single scenario's full content.

#### Scenario: Get specific scenario

- **WHEN** the `get_scenario` tool is called with valid `spec_id`, `requirement`, and `scenario` arguments
- **THEN** the response contains the requirement description
- **AND** the response contains the scenario's `name`, `when` clause, and `then` clauses

#### Scenario: Get first scenario by default

- **WHEN** the `get_scenario` tool is called with `spec_id` and `requirement` but no `scenario`
- **THEN** the response contains the first scenario of that requirement

#### Scenario: Scenario not found

- **WHEN** the `get_scenario` tool is called with an invalid scenario name
- **THEN** the tool returns an error indicating the scenario was not found

### Requirement: Search Specs Tool

The MCP server SHALL provide a `search_specs` tool that performs semantic search across all specs.

#### Scenario: Search with query

- **WHEN** the `search_specs` tool is called with a `query` string
- **THEN** it calls Core `search` function
- **AND** the response contains ranked search results
- **AND** each result includes `spec_id`, `requirement`, `score`, and `snippet`
- **AND** results are ordered by descending relevance score

#### Scenario: Search with no index

- **WHEN** the `search_specs` tool is called
- **AND** no search index exists
- **THEN** the tool returns an error indicating the index must be built first

#### Scenario: Search with empty results

- **WHEN** the `search_specs` tool is called with a query
- **AND** no specs match the query above the relevance threshold
- **THEN** the response contains an empty results array

### Requirement: Graceful Degradation

The MCP server SHALL operate without the search index, with reduced functionality.

#### Scenario: Server without index

- **WHEN** the MCP server starts
- **AND** no search index exists
- **THEN** the `list_specs`, `get_spec_requirements`, and `get_scenario` tools work normally
- **AND** the `search_specs` tool returns an error indicating the index is unavailable
