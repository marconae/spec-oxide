//! MCP (Model Context Protocol) server for Spec Oxide.
//!
//! This module provides an MCP server that exposes spec tools over stdio transport
//! for integration with AI coding assistants like Claude Code.
//!
//! The server provides the following tools:
//! - `list_specs`: List all available specs with id, title, and purpose
//! - `get_spec_requirements`: Get requirements structure without scenario bodies
//! - `get_scenario`: Get a specific scenario's full content
//! - `search_specs`: Semantic search over specs (requires index)
//!
//! ## Usage
//!
//! The server is started via the CLI:
//! ```bash
//! spox mcp serve
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo};
use rmcp::tool;
use rmcp::ServerHandler;
use rmcp::ServiceExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::core::index::{self, SpecIndex};
use crate::core::spec;

// =============================================================================
// Request/Response Types
// =============================================================================

/// Summary of a spec for list_specs response.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SpecSummary {
    /// The spec ID (folder name).
    pub id: String,
    /// The spec title.
    pub title: String,
    /// The spec purpose.
    pub purpose: String,
}

/// Response for list_specs tool.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ListSpecsResponse {
    /// All available specs.
    pub specs: Vec<SpecSummary>,
}

/// Request parameters for get_spec_requirements.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRequirementsRequest {
    /// The spec ID to get requirements for.
    #[schemars(description = "The spec ID to get requirements for")]
    pub spec_id: String,
}

/// Summary of a requirement for get_spec_requirements response.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RequirementSummary {
    /// The requirement name.
    pub name: String,
    /// Number of scenarios in this requirement.
    pub scenario_count: usize,
}

/// Response for get_spec_requirements tool.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetRequirementsResponse {
    /// The spec ID.
    pub spec_id: String,
    /// All requirements in the spec.
    pub requirements: Vec<RequirementSummary>,
}

/// Request parameters for get_scenario.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetScenarioRequest {
    /// The spec ID.
    #[schemars(description = "The spec ID")]
    pub spec_id: String,
    /// The requirement name.
    #[schemars(description = "The requirement name")]
    pub requirement: String,
    /// The scenario name (optional, defaults to first scenario).
    #[schemars(description = "The scenario name (optional, defaults to first scenario)")]
    pub scenario: Option<String>,
}

/// Response for get_scenario tool.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetScenarioResponse {
    /// The requirement description.
    pub requirement_description: String,
    /// The scenario name.
    pub scenario_name: String,
    /// The WHEN clause.
    pub when_clause: String,
    /// All THEN/AND clauses.
    pub then_clauses: Vec<String>,
}

/// Request parameters for search_specs.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchSpecsRequest {
    /// The search query.
    #[schemars(description = "The search query")]
    pub query: String,
    /// Number of results to return (default 10).
    #[schemars(description = "Number of results to return (default 10)")]
    pub top_k: Option<usize>,
}

/// A single search result.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchResultItem {
    /// The spec ID.
    pub spec_id: String,
    /// The requirement name (if matched on a requirement).
    pub requirement: Option<String>,
    /// The relevance score (0.0 to 1.0).
    pub score: f32,
    /// A snippet of the matched text.
    pub snippet: String,
}

/// Response for search_specs tool.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchSpecsResponse {
    /// Search results ordered by relevance.
    pub results: Vec<SearchResultItem>,
}

// =============================================================================
// MCP Server
// =============================================================================

/// The Spox MCP server.
///
/// This server exposes spec tools via the MCP protocol.
#[derive(Clone)]
pub struct SpoxServer {
    /// Project root path.
    project_root: PathBuf,
    /// Spec folder path (relative to project root).
    spec_folder: String,
    /// The search index (if available).
    index: Option<Arc<SpecIndex>>,
}

impl SpoxServer {
    /// Create a new SpoxServer.
    ///
    /// # Arguments
    ///
    /// * `config` - The loaded configuration
    /// * `project_root` - The project root path
    ///
    /// # Returns
    ///
    /// A new SpoxServer instance. The search index is loaded if available.
    pub fn new(config: &Config, project_root: PathBuf) -> Self {
        let index_path = project_root.join(".spox/search_index.bin");
        let index = index::load_index(&index_path).ok().map(Arc::new);

        Self {
            project_root,
            spec_folder: config.spec_folder().to_string(),
            index,
        }
    }

    /// Get the full path to the specs folder.
    fn specs_path(&self) -> PathBuf {
        self.project_root.join(&self.spec_folder)
    }

    /// Core implementation for list_specs.
    pub fn do_list_specs(&self) -> Result<ListSpecsResponse, String> {
        let specs = spec::parse_all_specs(&self.specs_path())
            .map_err(|e| format!("Failed to parse specs: {}", e))?;

        let summaries = specs
            .into_iter()
            .map(|s| SpecSummary {
                id: s.id,
                title: s.title,
                purpose: s.purpose,
            })
            .collect();

        Ok(ListSpecsResponse { specs: summaries })
    }

    /// Core implementation for get_spec_requirements.
    pub fn do_get_spec_requirements(
        &self,
        spec_id: &str,
    ) -> Result<GetRequirementsResponse, String> {
        let spec = spec::get_spec_by_id(&self.specs_path(), spec_id)
            .map_err(|e| format!("Failed to get spec '{}': {}", spec_id, e))?;

        let requirements = spec
            .requirements
            .iter()
            .map(|r| RequirementSummary {
                name: r.name.clone(),
                scenario_count: r.scenarios.len(),
            })
            .collect();

        Ok(GetRequirementsResponse {
            spec_id: spec.id,
            requirements,
        })
    }

    /// Core implementation for get_scenario.
    pub fn do_get_scenario(
        &self,
        spec_id: &str,
        requirement_name: &str,
        scenario_name: Option<&str>,
    ) -> Result<GetScenarioResponse, String> {
        let spec = spec::get_spec_by_id(&self.specs_path(), spec_id)
            .map_err(|e| format!("Failed to get spec '{}': {}", spec_id, e))?;

        // Find the requirement
        let requirement = spec
            .requirements
            .iter()
            .find(|r| r.name == requirement_name)
            .ok_or_else(|| {
                format!(
                    "Requirement '{}' not found in spec '{}'",
                    requirement_name, spec_id
                )
            })?;

        // Find the scenario (or use first)
        let scenario = if let Some(name) = scenario_name {
            requirement
                .scenarios
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| {
                    format!(
                        "Scenario '{}' not found in requirement '{}'",
                        name, requirement_name
                    )
                })?
        } else {
            requirement.scenarios.first().ok_or_else(|| {
                format!("No scenarios found in requirement '{}'", requirement_name)
            })?
        };

        Ok(GetScenarioResponse {
            requirement_description: requirement.description.clone(),
            scenario_name: scenario.name.clone(),
            when_clause: scenario.when_clause.clone(),
            then_clauses: scenario.then_clauses.clone(),
        })
    }

    /// Core implementation for search_specs.
    pub fn do_search_specs(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<SearchSpecsResponse, String> {
        let idx = self.index.as_ref().ok_or(
            "Search index not available. Run 'spox index' first to build the search index.",
        )?;

        let results =
            index::search(idx, query, top_k).map_err(|e| format!("Search failed: {}", e))?;

        let items = results
            .into_iter()
            .map(|r| SearchResultItem {
                spec_id: r.spec_id,
                requirement: r.requirement,
                score: r.score,
                snippet: r.snippet,
            })
            .collect();

        Ok(SearchSpecsResponse { results: items })
    }
}

// =============================================================================
// Tool Implementations
// =============================================================================

#[tool(tool_box)]
impl SpoxServer {
    /// List all specs with their ID, title, and purpose.
    #[tool(
        description = "List all specs with their ID, title, and purpose. Returns a minimal index suitable for discovering available specs."
    )]
    async fn list_specs(&self) -> String {
        match self.do_list_specs() {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// Get the requirements of a spec without scenario bodies.
    #[tool(
        description = "Get the requirements of a spec without scenario bodies. Returns requirement names and scenario counts."
    )]
    async fn get_spec_requirements(&self, #[tool(aggr)] req: GetRequirementsRequest) -> String {
        match self.do_get_spec_requirements(&req.spec_id) {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// Get a specific scenario's full content.
    #[tool(
        description = "Get a specific scenario's full content including requirement description and WHEN/THEN clauses."
    )]
    async fn get_scenario(&self, #[tool(aggr)] req: GetScenarioRequest) -> String {
        match self.do_get_scenario(&req.spec_id, &req.requirement, req.scenario.as_deref()) {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// Search specs semantically.
    #[tool(
        description = "Search specs semantically using the pre-built search index. Returns ranked results. Requires 'spox index' to be run first."
    )]
    async fn search_specs(&self, #[tool(aggr)] req: SearchSpecsRequest) -> String {
        let top_k = req.top_k.unwrap_or(10);
        match self.do_search_specs(&req.query, top_k) {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }
}

// =============================================================================
// ServerHandler Implementation
// =============================================================================

#[tool(tool_box)]
impl ServerHandler for SpoxServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "spox".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            instructions: Some(
                "Spox is a spec-driven development tool. Use list_specs to discover available specs, \
                 get_spec_requirements to see the structure of a spec, get_scenario for full scenario \
                 details, and search_specs to find relevant content across all specs."
                    .into(),
            ),
        }
    }
}

// =============================================================================
// Server Runner
// =============================================================================

/// Run the MCP server over stdio transport.
///
/// This function starts an MCP server that listens on stdin and responds on stdout.
/// It exposes spec tools for AI coding assistants.
///
/// # Arguments
///
/// * `config` - The loaded configuration containing spec paths
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a fatal error.
pub async fn run_server(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = std::env::current_dir()?;
    let server = SpoxServer::new(config, project_root);

    // Create stdio transport
    let transport = (tokio::io::stdin(), tokio::io::stdout());

    // Start the server
    let running = server.serve(transport).await?;

    // Wait for shutdown
    let _quit_reason = running.waiting().await?;

    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(spec_folder: &str) -> Config {
        // Create a minimal config for testing
        let config_content = format!(
            r#"
[paths]
spec_folder = "{}"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#,
            spec_folder
        );

        // Parse the config
        toml::from_str(&config_content).unwrap()
    }

    fn create_test_spec(dir: &std::path::Path, name: &str, content: &str) {
        let spec_dir = dir.join(name);
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("spec.md"), content).unwrap();
    }

    const VALID_SPEC: &str = r#"# Auth Specification

## Purpose

This spec defines authentication requirements for the system.

## Requirements

### Requirement: User Login

The system SHALL allow users to login with email and password.

#### Scenario: Successful login

- **WHEN** user provides valid email and password
- **THEN** user is authenticated
- **AND** session token is returned

#### Scenario: Failed login

- **WHEN** user provides invalid credentials
- **THEN** authentication error is returned

### Requirement: User Logout

The system SHALL allow users to logout.

#### Scenario: Logout clears session

- **WHEN** authenticated user requests logout
- **THEN** session is invalidated
"#;

    const SIMPLE_SPEC: &str = r#"# Simple Specification

## Purpose

A simple spec for testing.

## Requirements

### Requirement: Basic Feature

The system SHALL do something basic.

#### Scenario: Basic behavior

- **WHEN** user does something
- **THEN** something happens
"#;

    // ==================== list_specs tests ====================

    #[test]
    fn test_list_specs_returns_all_specs() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);
        create_test_spec(&specs_dir, "simple", SIMPLE_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_list_specs().unwrap();

        assert_eq!(response.specs.len(), 2);

        let mut ids: Vec<&str> = response.specs.iter().map(|s| s.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["auth", "simple"]);
    }

    #[test]
    fn test_list_specs_returns_empty_for_no_specs() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_list_specs().unwrap();

        assert!(response.specs.is_empty());
    }

    #[test]
    fn test_list_specs_includes_id_title_purpose() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_list_specs().unwrap();

        let spec = &response.specs[0];
        assert_eq!(spec.id, "auth");
        assert_eq!(spec.title, "Auth Specification");
        assert!(spec.purpose.contains("authentication requirements"));
    }

    // ==================== get_spec_requirements tests ====================

    #[test]
    fn test_get_spec_requirements_valid_spec() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_get_spec_requirements("auth").unwrap();

        assert_eq!(response.spec_id, "auth");
        assert_eq!(response.requirements.len(), 2);

        // Check first requirement
        let login = &response.requirements[0];
        assert_eq!(login.name, "User Login");
        assert_eq!(login.scenario_count, 2);

        // Check second requirement
        let logout = &response.requirements[1];
        assert_eq!(logout.name, "User Logout");
        assert_eq!(logout.scenario_count, 1);
    }

    #[test]
    fn test_get_spec_requirements_invalid_spec() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_get_spec_requirements("nonexistent");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ==================== get_scenario tests ====================

    #[test]
    fn test_get_scenario_specific() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server
            .do_get_scenario("auth", "User Login", Some("Successful login"))
            .unwrap();

        assert_eq!(response.scenario_name, "Successful login");
        assert!(response
            .requirement_description
            .contains("login with email and password"));
        assert!(response.when_clause.contains("valid email and password"));
        assert_eq!(response.then_clauses.len(), 2);
    }

    #[test]
    fn test_get_scenario_first_by_default() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_get_scenario("auth", "User Login", None).unwrap();

        // Should return first scenario
        assert_eq!(response.scenario_name, "Successful login");
    }

    #[test]
    fn test_get_scenario_invalid_spec() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_get_scenario("nonexistent", "Req", None);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_scenario_invalid_requirement() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_get_scenario("auth", "Nonexistent", None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_scenario_invalid_scenario_name() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_get_scenario("auth", "User Login", Some("Nonexistent scenario"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ==================== search_specs tests ====================

    #[test]
    fn test_search_specs_no_index() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        // Without index, search should fail
        let result = server.do_search_specs("login", 10);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("index"));
    }

    // ==================== ServerHandler tests ====================

    #[test]
    fn test_server_info() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let info = server.get_info();

        assert_eq!(info.server_info.name, "spox");
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("list_specs"));
    }
}
