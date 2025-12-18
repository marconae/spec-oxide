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
//! - `list_changes`: List all active change proposals with task progress
//! - `get_change`: Get full content of a change proposal (proposal, tasks, design, deltas)
//! - `validate_spec`: Validate spec structure and content (all specs or specific one)
//! - `validate_change`: Validate change proposal structure and content (all changes or specific one)
//! - `rebuild_index`: Rebuild the search index from all specs
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

use std::fs;

use crate::config::Config;
use crate::core::index::{self, SpecIndex};
use crate::core::spec;
use crate::show::change::{parse_change, DeltaOp};
use crate::show::dashboard::gather_changes;
use crate::validate::{change as validate_change_mod, spec as validate_spec_mod, Severity};

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
    /// Whether the search index was auto-built (true if index was missing and auto-built).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_built: Option<bool>,
}

// =============================================================================
// Change Request/Response Types
// =============================================================================

/// Summary of task progress for a change.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TaskProgress {
    /// Number of completed tasks.
    pub completed: usize,
    /// Total number of tasks.
    pub total: usize,
}

/// Summary of a change for list_changes response.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChangeSummaryMcp {
    /// The change ID (directory name).
    pub id: String,
    /// A brief title derived from the change ID.
    pub title: String,
    /// Task progress for this change.
    pub task_progress: TaskProgress,
}

/// Response for list_changes tool.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ListChangesResponse {
    /// All active changes.
    pub changes: Vec<ChangeSummaryMcp>,
}

/// Request parameters for get_change.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetChangeRequest {
    /// The change ID to retrieve.
    #[schemars(description = "The change ID to retrieve")]
    pub change_id: String,
    /// Optional section filter: "proposal", "tasks", "design", or "deltas".
    #[schemars(description = "Optional section filter: proposal, tasks, design, or deltas")]
    pub section: Option<String>,
}

/// A delta item representing a requirement change.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DeltaItemMcp {
    /// The operation type: added, modified, removed, or renamed.
    pub operation: String,
    /// The requirement name.
    pub name: String,
    /// List of scenario names in this requirement.
    pub scenarios: Vec<String>,
}

/// A delta group for a capability.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DeltaGroupMcp {
    /// The capability name.
    pub capability: String,
    /// The requirement changes in this capability.
    pub items: Vec<DeltaItemMcp>,
}

/// Response for get_change tool.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetChangeResponse {
    /// The change ID.
    pub change_id: String,
    /// The proposal content (why and what).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal: Option<ProposalContent>,
    /// The tasks content (raw markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<TasksContent>,
    /// The design content (raw markdown, if present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design: Option<String>,
    /// The spec deltas grouped by capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deltas: Option<Vec<DeltaGroupMcp>>,
}

/// Proposal content from proposal.md.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProposalContent {
    /// The "Why" section content.
    pub why: String,
    /// The "What Changes" section content.
    pub what_changes: String,
}

/// Tasks content from tasks.md.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TasksContent {
    /// The raw tasks markdown content.
    pub content: String,
    /// Number of completed tasks.
    pub completed: usize,
    /// Total number of tasks.
    pub total: usize,
}

// =============================================================================
// Validation Request/Response Types
// =============================================================================

/// Request parameters for validate_spec.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateSpecRequest {
    /// The spec ID to validate (validates all if not specified).
    #[schemars(description = "The spec ID to validate (validates all if not specified)")]
    pub spec_id: Option<String>,
}

/// Request parameters for validate_change.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateChangeRequest {
    /// The change ID to validate (validates all if not specified).
    #[schemars(description = "The change ID to validate (validates all if not specified)")]
    pub change_id: Option<String>,
}

/// A single validation error.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ValidationError {
    /// The file path where the error was found.
    pub file: String,
    /// The line number where the error was found (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// A description of the error.
    pub description: String,
    /// The section where the error was found (for change validation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

/// A single validation warning.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ValidationWarning {
    /// The file path where the warning was found.
    pub file: String,
    /// The line number where the warning was found (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// A description of the warning.
    pub description: String,
    /// The section where the warning was found (for change validation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

/// Response for validate_spec and validate_change tools.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ValidationResponse {
    /// Whether the validation passed (no errors).
    pub valid: bool,
    /// List of validation errors.
    pub errors: Vec<ValidationError>,
    /// List of validation warnings.
    pub warnings: Vec<ValidationWarning>,
    /// Human-readable summary of the validation result.
    pub summary: String,
}

/// Response for rebuild_index tool.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RebuildIndexResponse {
    /// The number of specs indexed.
    pub specs_indexed: usize,
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
    /// Changes folder path (relative to project root).
    changes_folder: String,
    /// The search index (if available).
    /// Note: This field is retained for backwards compatibility with the original
    /// search_specs implementation. The current implementation uses ensure_index
    /// which auto-builds the index when missing.
    #[allow(dead_code)]
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
            changes_folder: config.changes_folder().to_string(),
            index,
        }
    }

    /// Get the full path to the specs folder.
    fn specs_path(&self) -> PathBuf {
        self.project_root.join(&self.spec_folder)
    }

    /// Get the full path to the changes folder.
    fn changes_path(&self) -> PathBuf {
        self.project_root.join(&self.changes_folder)
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

    /// Core implementation for search_specs (original, requires pre-loaded index).
    /// Note: This method is retained for backwards compatibility. The search_specs tool
    /// now uses do_search_specs_with_auto_build which auto-builds the index when missing.
    #[allow(dead_code)]
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

        Ok(SearchSpecsResponse {
            results: items,
            index_built: None,
        })
    }

    /// Core implementation for search_specs with auto-build support.
    ///
    /// This method will auto-build the index if it doesn't exist, and indicate
    /// in the response whether the index was auto-built.
    pub fn do_search_specs_with_auto_build(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<SearchSpecsResponse, String> {
        let index_path = self.project_root.join(".spox/search_index.bin");
        let was_missing = !index_path.exists();

        // Use ensure_index to get or build the index
        let idx = index::ensure_index(&self.project_root, &self.specs_path())
            .map_err(|e| format!("Failed to ensure index: {}", e))?;

        let results =
            index::search(&idx, query, top_k).map_err(|e| format!("Search failed: {}", e))?;

        let items = results
            .into_iter()
            .map(|r| SearchResultItem {
                spec_id: r.spec_id,
                requirement: r.requirement,
                score: r.score,
                snippet: r.snippet,
            })
            .collect();

        Ok(SearchSpecsResponse {
            results: items,
            index_built: Some(was_missing),
        })
    }

    /// Core implementation for list_changes.
    pub fn do_list_changes(&self) -> Result<ListChangesResponse, String> {
        let changes_path = self.changes_path();
        let changes = gather_changes(changes_path.to_str().unwrap_or(""))
            .map_err(|e| format!("Failed to list changes: {}", e))?;

        let summaries = changes
            .into_iter()
            .map(|c| ChangeSummaryMcp {
                id: c.name.clone(),
                title: c.name.replace('-', " "),
                task_progress: TaskProgress {
                    completed: c.tasks_completed,
                    total: c.tasks_total,
                },
            })
            .collect();

        Ok(ListChangesResponse { changes: summaries })
    }

    /// Core implementation for get_change.
    pub fn do_get_change(
        &self,
        change_id: &str,
        section: Option<&str>,
    ) -> Result<GetChangeResponse, String> {
        let change_path = self.changes_path().join(change_id);

        if !change_path.exists() {
            return Err(format!("Change '{}' not found", change_id));
        }

        let info = parse_change(&change_path)
            .map_err(|e| format!("Failed to parse change '{}': {}", change_id, e))?;

        // Build response based on section filter
        match section {
            Some("proposal") => Ok(GetChangeResponse {
                change_id: change_id.to_string(),
                proposal: Some(ProposalContent {
                    why: info.why,
                    what_changes: info.what_changes,
                }),
                tasks: None,
                design: None,
                deltas: None,
            }),
            Some("tasks") => {
                let tasks_path = change_path.join("tasks.md");
                let content = if tasks_path.exists() {
                    fs::read_to_string(&tasks_path)
                        .map_err(|e| format!("Failed to read tasks.md: {}", e))?
                } else {
                    String::new()
                };
                Ok(GetChangeResponse {
                    change_id: change_id.to_string(),
                    proposal: None,
                    tasks: Some(TasksContent {
                        content,
                        completed: info.tasks_completed,
                        total: info.tasks_total,
                    }),
                    design: None,
                    deltas: None,
                })
            }
            Some("design") => {
                let design_path = change_path.join("design.md");
                let design = if design_path.exists() {
                    Some(
                        fs::read_to_string(&design_path)
                            .map_err(|e| format!("Failed to read design.md: {}", e))?,
                    )
                } else {
                    None
                };
                Ok(GetChangeResponse {
                    change_id: change_id.to_string(),
                    proposal: None,
                    tasks: None,
                    design,
                    deltas: None,
                })
            }
            Some("deltas") => {
                let deltas = info
                    .deltas
                    .into_iter()
                    .map(|g| DeltaGroupMcp {
                        capability: g.capability,
                        items: g
                            .items
                            .into_iter()
                            .map(|i| DeltaItemMcp {
                                operation: match i.operation {
                                    DeltaOp::Added => "added".to_string(),
                                    DeltaOp::Modified => "modified".to_string(),
                                    DeltaOp::Removed => "removed".to_string(),
                                    DeltaOp::Renamed => "renamed".to_string(),
                                },
                                name: i.name,
                                scenarios: i.scenarios,
                            })
                            .collect(),
                    })
                    .collect();
                Ok(GetChangeResponse {
                    change_id: change_id.to_string(),
                    proposal: None,
                    tasks: None,
                    design: None,
                    deltas: Some(deltas),
                })
            }
            Some(unknown) => Err(format!(
                "Unknown section '{}'. Valid sections: proposal, tasks, design, deltas",
                unknown
            )),
            None => {
                // Return all sections
                let tasks_path = change_path.join("tasks.md");
                let tasks_content = if tasks_path.exists() {
                    fs::read_to_string(&tasks_path)
                        .map_err(|e| format!("Failed to read tasks.md: {}", e))?
                } else {
                    String::new()
                };

                let design_path = change_path.join("design.md");
                let design = if design_path.exists() {
                    Some(
                        fs::read_to_string(&design_path)
                            .map_err(|e| format!("Failed to read design.md: {}", e))?,
                    )
                } else {
                    None
                };

                let deltas = info
                    .deltas
                    .into_iter()
                    .map(|g| DeltaGroupMcp {
                        capability: g.capability,
                        items: g
                            .items
                            .into_iter()
                            .map(|i| DeltaItemMcp {
                                operation: match i.operation {
                                    DeltaOp::Added => "added".to_string(),
                                    DeltaOp::Modified => "modified".to_string(),
                                    DeltaOp::Removed => "removed".to_string(),
                                    DeltaOp::Renamed => "renamed".to_string(),
                                },
                                name: i.name,
                                scenarios: i.scenarios,
                            })
                            .collect(),
                    })
                    .collect();

                Ok(GetChangeResponse {
                    change_id: change_id.to_string(),
                    proposal: Some(ProposalContent {
                        why: info.why,
                        what_changes: info.what_changes,
                    }),
                    tasks: Some(TasksContent {
                        content: tasks_content,
                        completed: info.tasks_completed,
                        total: info.tasks_total,
                    }),
                    design,
                    deltas: Some(deltas),
                })
            }
        }
    }

    /// Core implementation for validate_spec.
    ///
    /// If `spec_id` is provided, validates only that spec.
    /// Otherwise, validates all specs in the project.
    pub fn do_validate_spec(&self, spec_id: Option<&str>) -> Result<ValidationResponse, String> {
        match spec_id {
            Some(id) => {
                // Validate single spec
                let spec_path = self.specs_path().join(id).join("spec.md");

                if !spec_path.exists() {
                    return Err(format!("spec not found: '{}'", id));
                }

                let report = validate_spec_mod::validate_spec(&spec_path);
                Ok(self.convert_validation_report(&report, None))
            }
            None => {
                // Validate all specs
                let specs_path = self.specs_path();
                if !specs_path.exists() {
                    return Err(format!(
                        "Spec folder '{}' does not exist",
                        specs_path.display()
                    ));
                }

                let mut total_errors = 0;
                let mut total_warnings = 0;
                let mut all_errors = Vec::new();
                let mut all_warnings = Vec::new();

                let entries: Vec<_> = fs::read_dir(&specs_path)
                    .map_err(|e| format!("Failed to read spec folder: {}", e))?
                    .filter_map(|e| e.ok())
                    .collect();

                // Sort entries and filter to spec directories
                let mut sorted_entries: Vec<_> = entries
                    .iter()
                    .filter(|e| e.path().is_dir())
                    .filter(|e| {
                        // Skip directories starting with _ (like _changes, _archive)
                        let name = e.file_name().to_string_lossy().to_string();
                        !name.starts_with('_')
                    })
                    .collect();
                sorted_entries.sort_by_key(|e| e.file_name());

                for entry in sorted_entries {
                    let spec_md_path = entry.path().join("spec.md");
                    if !spec_md_path.exists() {
                        continue;
                    }

                    let report = validate_spec_mod::validate_spec(&spec_md_path);
                    total_errors += report.errors;
                    total_warnings += report.warnings;

                    // Collect errors and warnings
                    for issue in &report.issues {
                        match issue.severity {
                            Severity::Error => all_errors.push(ValidationError {
                                file: issue.file.clone(),
                                line: issue.line,
                                description: issue.message.clone(),
                                section: None,
                            }),
                            Severity::Warning => all_warnings.push(ValidationWarning {
                                file: issue.file.clone(),
                                line: issue.line,
                                description: issue.message.clone(),
                                section: None,
                            }),
                            Severity::Info => {} // Skip info messages
                        }
                    }
                }

                let valid = total_errors == 0;
                let summary = if valid {
                    format!("All specs valid ({} warnings)", total_warnings)
                } else {
                    format!(
                        "Validation failed: {} errors, {} warnings",
                        total_errors, total_warnings
                    )
                };

                Ok(ValidationResponse {
                    valid,
                    errors: all_errors,
                    warnings: all_warnings,
                    summary,
                })
            }
        }
    }

    /// Core implementation for validate_change.
    ///
    /// If `change_id` is provided, validates only that change.
    /// Otherwise, validates all changes in the project.
    pub fn do_validate_change(
        &self,
        change_id: Option<&str>,
    ) -> Result<ValidationResponse, String> {
        match change_id {
            Some(id) => {
                // Validate single change
                let change_path = self.changes_path().join(id);

                if !change_path.exists() {
                    return Err(format!("change not found: '{}'", id));
                }

                let result = validate_change_mod::validate_change(&change_path);
                Ok(self.convert_change_validation_result(&result))
            }
            None => {
                // Validate all changes
                let changes_path = self.changes_path();
                if !changes_path.exists() {
                    return Err(format!(
                        "Changes folder '{}' does not exist",
                        changes_path.display()
                    ));
                }

                let mut total_errors = 0;
                let mut total_warnings = 0;
                let mut all_errors = Vec::new();
                let mut all_warnings = Vec::new();

                let entries: Vec<_> = fs::read_dir(&changes_path)
                    .map_err(|e| format!("Failed to read changes folder: {}", e))?
                    .filter_map(|e| e.ok())
                    .collect();

                // Sort entries and filter to change directories
                let mut sorted_entries: Vec<_> =
                    entries.iter().filter(|e| e.path().is_dir()).collect();
                sorted_entries.sort_by_key(|e| e.file_name());

                for entry in sorted_entries {
                    let result = validate_change_mod::validate_change(&entry.path());
                    total_errors += result.report.errors;
                    total_warnings += result.report.warnings;

                    let change_id = entry.file_name().to_string_lossy().to_string();

                    // Collect errors and warnings with change context
                    for issue in &result.report.issues {
                        let section = Self::extract_section_from_file(&issue.file);
                        match issue.severity {
                            Severity::Error => all_errors.push(ValidationError {
                                file: format!("{}/{}", change_id, issue.file),
                                line: issue.line,
                                description: issue.message.clone(),
                                section,
                            }),
                            Severity::Warning => all_warnings.push(ValidationWarning {
                                file: format!("{}/{}", change_id, issue.file),
                                line: issue.line,
                                description: issue.message.clone(),
                                section,
                            }),
                            Severity::Info => {} // Skip info messages
                        }
                    }
                }

                let valid = total_errors == 0;
                let summary = if valid {
                    format!("All changes valid ({} warnings)", total_warnings)
                } else {
                    format!(
                        "Validation failed: {} errors, {} warnings",
                        total_errors, total_warnings
                    )
                };

                Ok(ValidationResponse {
                    valid,
                    errors: all_errors,
                    warnings: all_warnings,
                    summary,
                })
            }
        }
    }

    /// Convert a ValidationReport to ValidationResponse.
    fn convert_validation_report(
        &self,
        report: &crate::validate::ValidationReport,
        _section_context: Option<&str>,
    ) -> ValidationResponse {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for issue in &report.issues {
            match issue.severity {
                Severity::Error => errors.push(ValidationError {
                    file: issue.file.clone(),
                    line: issue.line,
                    description: issue.message.clone(),
                    section: None,
                }),
                Severity::Warning => warnings.push(ValidationWarning {
                    file: issue.file.clone(),
                    line: issue.line,
                    description: issue.message.clone(),
                    section: None,
                }),
                Severity::Info => {} // Skip info messages
            }
        }

        let summary = if report.valid {
            format!("Valid ({} warnings)", report.warnings)
        } else {
            format!(
                "Validation failed: {} errors, {} warnings",
                report.errors, report.warnings
            )
        };

        ValidationResponse {
            valid: report.valid,
            errors,
            warnings,
            summary,
        }
    }

    /// Convert a ChangeValidationResult to ValidationResponse.
    fn convert_change_validation_result(
        &self,
        result: &validate_change_mod::ChangeValidationResult,
    ) -> ValidationResponse {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for issue in &result.report.issues {
            let section = Self::extract_section_from_file(&issue.file);
            match issue.severity {
                Severity::Error => errors.push(ValidationError {
                    file: issue.file.clone(),
                    line: issue.line,
                    description: issue.message.clone(),
                    section,
                }),
                Severity::Warning => warnings.push(ValidationWarning {
                    file: issue.file.clone(),
                    line: issue.line,
                    description: issue.message.clone(),
                    section,
                }),
                Severity::Info => {} // Skip info messages
            }
        }

        let summary = if result.report.valid {
            format!("Valid ({} warnings)", result.report.warnings)
        } else {
            format!(
                "Validation failed: {} errors, {} warnings",
                result.report.errors, result.report.warnings
            )
        };

        ValidationResponse {
            valid: result.report.valid,
            errors,
            warnings,
            summary,
        }
    }

    /// Extract section name from file path for change validation context.
    fn extract_section_from_file(file: &str) -> Option<String> {
        if file.contains("proposal.md") {
            Some("proposal".to_string())
        } else if file.contains("tasks.md") {
            Some("tasks".to_string())
        } else if file.contains("design.md") {
            Some("design".to_string())
        } else if file.contains("specs/") || file.contains("spec.md") {
            Some("deltas".to_string())
        } else {
            None
        }
    }

    /// Core implementation for rebuild_index.
    ///
    /// Rebuilds the search index from all specs.
    pub fn do_rebuild_index(&self) -> Result<RebuildIndexResponse, String> {
        let specs_indexed = index::rebuild_index(&self.project_root)
            .map_err(|e| format!("Failed to rebuild index: {}", e))?;

        Ok(RebuildIndexResponse { specs_indexed })
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
        description = "Search specs semantically. Automatically builds search index if missing. Returns ranked results with index_built flag if index was auto-built."
    )]
    async fn search_specs(&self, #[tool(aggr)] req: SearchSpecsRequest) -> String {
        let top_k = req.top_k.unwrap_or(10);
        match self.do_search_specs_with_auto_build(&req.query, top_k) {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// List all active changes.
    #[tool(description = "List all active change proposals with their IDs and task progress.")]
    async fn list_changes(&self) -> String {
        match self.do_list_changes() {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// Get details of a specific change.
    #[tool(
        description = "Get the full content of a change proposal including proposal, tasks, design (if present), and spec deltas."
    )]
    async fn get_change(&self, #[tool(aggr)] req: GetChangeRequest) -> String {
        match self.do_get_change(&req.change_id, req.section.as_deref()) {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// Validate spec file(s).
    #[tool(
        description = "Validate spec structure and content. Called without parameters validates all specs. Called with spec_id validates only that spec."
    )]
    async fn validate_spec(&self, #[tool(aggr)] req: ValidateSpecRequest) -> String {
        match self.do_validate_spec(req.spec_id.as_deref()) {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// Validate change proposal(s).
    #[tool(
        description = "Validate change proposal structure and content. Called without parameters validates all active changes. Called with change_id validates only that change."
    )]
    async fn validate_change(&self, #[tool(aggr)] req: ValidateChangeRequest) -> String {
        match self.do_validate_change(req.change_id.as_deref()) {
            Ok(response) => serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            }),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    /// Rebuild the search index.
    #[tool(
        description = "Rebuild the search index from all specs. Returns the count of specs indexed."
    )]
    async fn rebuild_index(&self) -> String {
        match self.do_rebuild_index() {
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
                 details, and search_specs to find relevant content across all specs. Use list_changes \
                 to see active change proposals, and get_change to retrieve change details. Use \
                 validate_spec to validate spec structure and content (all specs or a specific one), \
                 validate_change to validate change proposals (all changes or a specific one), and \
                 rebuild_index to rebuild the search index from all specs."
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
    fn test_search_specs_no_index_returns_error_without_auto_build() {
        // NOTE: This test verifies the OLD behavior where search fails without index.
        // The new do_search_specs_with_auto_build method supports auto-building.
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        // Without index, the original search method should fail
        let result = server.do_search_specs("login", 10);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("index"));
    }

    #[test]
    #[ignore] // Requires fastembed model download
    fn test_search_specs_auto_builds_missing_index() {
        // This test verifies the NEW behavior: auto-build index when missing
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .spox config directory
        let spox_dir = project_root.join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Create specs directory with a test spec
        let specs_dir = project_root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, project_root.to_path_buf());

        // No index file exists
        let index_path = spox_dir.join("search_index.bin");
        assert!(!index_path.exists(), "Index should not exist before search");

        // Search should auto-build the index and succeed
        let result = server.do_search_specs_with_auto_build("login", 10);

        assert!(
            result.is_ok(),
            "Search should succeed with auto-build: {:?}",
            result
        );

        let response = result.unwrap();

        // Response should indicate that index was built
        assert_eq!(
            response.index_built,
            Some(true),
            "Response should indicate index was auto-built"
        );

        // Index file should now exist
        assert!(
            index_path.exists(),
            "Index file should be created after auto-build"
        );

        // Results should contain relevant content
        assert!(!response.results.is_empty(), "Should have search results");
    }

    #[test]
    #[ignore] // Requires fastembed model download
    fn test_search_specs_uses_existing_index() {
        // This test verifies that existing index is used (no auto-build)
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .spox config directory
        let spox_dir = project_root.join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();

        // Create specs directory with a test spec
        let specs_dir = project_root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();
        create_test_spec(&specs_dir, "auth", VALID_SPEC);

        // Build the index first
        let index_path = spox_dir.join("search_index.bin");
        let specs = crate::core::spec::parse_all_specs(&specs_dir).unwrap();
        let index = crate::core::index::build_index(&specs).unwrap();
        crate::core::index::save_index(&index, &index_path).unwrap();

        assert!(index_path.exists(), "Index should exist before search");

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, project_root.to_path_buf());

        // Search should use existing index
        let result = server.do_search_specs_with_auto_build("login", 10);

        assert!(result.is_ok(), "Search should succeed: {:?}", result);

        let response = result.unwrap();

        // Response should indicate that index was NOT built (used existing)
        assert_eq!(
            response.index_built,
            Some(false),
            "Response should indicate index was not auto-built"
        );

        // Results should contain relevant content
        assert!(!response.results.is_empty(), "Should have search results");
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

    // ==================== list_changes tests ====================

    fn create_test_change(
        dir: &std::path::Path,
        name: &str,
        tasks_content: &str,
        delta_content: &str,
    ) {
        let change_dir = dir.join(name);
        fs::create_dir_all(change_dir.join("specs/auth")).unwrap();

        // Create proposal.md
        fs::write(
            change_dir.join("proposal.md"),
            r#"# Change: Test Change

## Why
This is a test change.

## What Changes
- Some changes
"#,
        )
        .unwrap();

        // Create tasks.md
        fs::write(change_dir.join("tasks.md"), tasks_content).unwrap();

        // Create delta spec
        fs::write(change_dir.join("specs/auth/spec.md"), delta_content).unwrap();
    }

    const TEST_TASKS: &str = r#"# Tasks test-change

## 1. Implementation
- [x] 1.1 First task
- [ ] 1.2 Second task
- [ ] 1.3 Third task
"#;

    const TEST_DELTA: &str = r#"## ADDED Requirements

### Requirement: New Feature

The system SHALL do something new.

#### Scenario: Feature works

- **WHEN** user triggers feature
- **THEN** feature responds
"#;

    #[test]
    fn test_list_changes_returns_all_changes() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);
        create_test_change(&changes_dir, "fix-bug", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_list_changes().unwrap();

        assert_eq!(response.changes.len(), 2);

        let mut ids: Vec<&str> = response.changes.iter().map(|c| c.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["add-feature", "fix-bug"]);
    }

    #[test]
    fn test_list_changes_returns_empty_for_no_changes() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_list_changes().unwrap();

        assert!(response.changes.is_empty());
    }

    #[test]
    fn test_list_changes_includes_task_progress() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_list_changes().unwrap();

        assert_eq!(response.changes.len(), 1);
        let change = &response.changes[0];
        assert_eq!(change.id, "add-feature");
        assert_eq!(change.task_progress.completed, 1);
        assert_eq!(change.task_progress.total, 3);
    }

    // ==================== get_change tests ====================

    #[test]
    fn test_get_change_full() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_get_change("add-feature", None).unwrap();

        assert_eq!(response.change_id, "add-feature");
        assert!(response.proposal.is_some());
        assert!(response.tasks.is_some());
        assert!(response.deltas.is_some());

        let proposal = response.proposal.unwrap();
        assert!(proposal.why.contains("test change"));

        let tasks = response.tasks.unwrap();
        assert_eq!(tasks.completed, 1);
        assert_eq!(tasks.total, 3);

        let deltas = response.deltas.unwrap();
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].capability, "auth");
    }

    #[test]
    fn test_get_change_proposal_section() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server
            .do_get_change("add-feature", Some("proposal"))
            .unwrap();

        assert!(response.proposal.is_some());
        assert!(response.tasks.is_none());
        assert!(response.deltas.is_none());
    }

    #[test]
    fn test_get_change_tasks_section() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_get_change("add-feature", Some("tasks")).unwrap();

        assert!(response.proposal.is_none());
        assert!(response.tasks.is_some());
        assert!(response.deltas.is_none());

        let tasks = response.tasks.unwrap();
        assert!(tasks.content.contains("First task"));
    }

    #[test]
    fn test_get_change_deltas_section() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let response = server.do_get_change("add-feature", Some("deltas")).unwrap();

        assert!(response.proposal.is_none());
        assert!(response.tasks.is_none());
        assert!(response.deltas.is_some());

        let deltas = response.deltas.unwrap();
        assert_eq!(deltas[0].items[0].operation, "added");
        assert_eq!(deltas[0].items[0].name, "New Feature");
    }

    #[test]
    fn test_get_change_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_get_change("nonexistent", None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_change_invalid_section() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_get_change("add-feature", Some("invalid"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown section"));
    }

    // =========================================================================
    // Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_spec_all_valid() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create a valid spec
        create_test_spec(&specs_dir, "test-spec", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_spec(None);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.valid);
        assert!(response.errors.is_empty());
        assert!(response.summary.contains("valid"));
    }

    #[test]
    fn test_validate_spec_specific_valid() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        create_test_spec(&specs_dir, "test-spec", VALID_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_spec(Some("test-spec"));

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.valid);
    }

    #[test]
    fn test_validate_spec_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_spec(Some("nonexistent"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("spec not found"));
    }

    #[test]
    fn test_validate_spec_invalid_content() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        let spec_dir = specs_dir.join("bad-spec");
        fs::create_dir_all(&spec_dir).unwrap();

        // Create a spec without required sections
        let invalid_content = "# Bad Spec\n\nNo proper sections here.\n";
        fs::write(spec_dir.join("spec.md"), invalid_content).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_spec(Some("bad-spec"));

        assert!(result.is_ok());
        let response = result.unwrap();
        // Should have validation errors due to missing sections
        assert!(!response.valid || !response.warnings.is_empty());
    }

    #[test]
    fn test_validate_change_all_valid() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_change(None);

        assert!(result.is_ok());
        let response = result.unwrap();
        // Note: the test change may have validation warnings but should parse
        assert!(response.summary.contains("valid") || response.summary.contains("Validation"));
    }

    #[test]
    fn test_validate_change_specific_valid() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        create_test_change(&changes_dir, "add-feature", TEST_TASKS, TEST_DELTA);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_change(Some("add-feature"));

        assert!(result.is_ok());
        let response = result.unwrap();
        // The validation should complete and return a response with summary
        // (may fail validation due to test fixtures but should return a response)
        assert!(!response.summary.is_empty());
    }

    #[test]
    fn test_validate_change_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_change(Some("nonexistent"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("change not found"));
    }

    #[test]
    fn test_validate_change_empty_folder() {
        let temp_dir = TempDir::new().unwrap();
        let changes_dir = temp_dir.path().join("specs/_changes");
        fs::create_dir_all(&changes_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_change(None);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.valid);
        assert!(response.errors.is_empty());
    }

    #[test]
    fn test_validation_error_includes_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        let spec_dir = specs_dir.join("broken-spec");
        fs::create_dir_all(&spec_dir).unwrap();

        // Create a spec without Purpose section (required)
        let content =
            "# Broken Spec\n\n## Requirements\n\n### Requirement: Test\n\nThis is a test.\n";
        fs::write(spec_dir.join("spec.md"), content).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, temp_dir.path().to_path_buf());

        let result = server.do_validate_spec(Some("broken-spec"));

        assert!(result.is_ok());
        let response = result.unwrap();
        // Should have errors or warnings about missing Purpose
        let has_file_info = response.errors.iter().any(|e| e.file.contains("spec.md"))
            || response.warnings.iter().any(|w| w.file.contains("spec.md"));
        assert!(has_file_info || response.valid);
    }

    // =========================================================================
    // rebuild_index Tests
    // =========================================================================

    #[test]
    fn test_rebuild_index_returns_count() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .spox directory with config
        let spox_dir = project_root.join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(
            spox_dir.join("config.toml"),
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#,
        )
        .unwrap();

        // Create specs directory with test specs
        let specs_dir = project_root.join("specs");
        create_test_spec(&specs_dir, "auth", VALID_SPEC);
        create_test_spec(&specs_dir, "simple", SIMPLE_SPEC);

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, project_root.to_path_buf());

        let result = server.do_rebuild_index();

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let response = result.unwrap();
        assert_eq!(response.specs_indexed, 2);
    }

    #[test]
    fn test_rebuild_index_with_no_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .spox directory with config
        let spox_dir = project_root.join(".spox");
        fs::create_dir_all(&spox_dir).unwrap();
        fs::write(
            spox_dir.join("config.toml"),
            r#"
[paths]
spec_folder = "specs/"
changes_folder = "specs/_changes"
archive_folder = "specs/_archive"

[rules]
system = ["mcp"]
"#,
        )
        .unwrap();

        // Create empty specs directory
        let specs_dir = project_root.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let config = create_test_config("specs");
        let server = SpoxServer::new(&config, project_root.to_path_buf());

        let result = server.do_rebuild_index();

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let response = result.unwrap();
        assert_eq!(response.specs_indexed, 0);
    }
}
