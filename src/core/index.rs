//! Search index for Spec Oxide.
//!
//! This module provides semantic search capabilities over specs using embeddings.
//! It uses fastembed for local embedding generation and cosine similarity for search.

use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};

use super::error::{Error, Result};
use super::spec::ParsedSpec;

/// The embedding model to use.
const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

/// The search index containing all spec embeddings.
#[derive(Debug, Serialize, Deserialize)]
pub struct SpecIndex {
    /// The name of the model used to generate embeddings.
    pub model_name: String,
    /// All indexed specs.
    pub specs: Vec<IndexedSpec>,
}

/// An indexed spec with its embeddings.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedSpec {
    /// The spec ID.
    pub id: String,
    /// The spec title.
    pub title: String,
    /// The embedding for the spec purpose.
    pub purpose_embedding: Vec<f32>,
    /// All indexed requirements.
    pub requirements: Vec<IndexedRequirement>,
}

/// An indexed requirement with its embedding.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedRequirement {
    /// The requirement name.
    pub name: String,
    /// The requirement description.
    pub description: String,
    /// The embedding for the description.
    pub description_embedding: Vec<f32>,
}

/// A search result with relevance score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The spec ID.
    pub spec_id: String,
    /// The requirement name, if matched on a requirement (None if matched on spec purpose).
    pub requirement: Option<String>,
    /// The similarity score (0.0 to 1.0).
    pub score: f32,
    /// A snippet of the matched text.
    pub snippet: String,
}

/// Build a search index from parsed specs.
///
/// This function generates embeddings for all spec purposes and requirement descriptions.
///
/// # Arguments
///
/// * `specs` - A slice of parsed specs to index
///
/// # Returns
///
/// A `SpecIndex` containing all embeddings.
///
/// # Errors
///
/// Returns an error if embedding generation fails.
pub fn build_index(specs: &[ParsedSpec]) -> Result<SpecIndex> {
    // Initialize the embedding model
    let model =
        TextEmbedding::try_new(InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true))
            .map_err(|e| Error::Other(format!("Failed to initialize embedding model: {}", e)))?;

    let model_name = format!("{:?}", EMBEDDING_MODEL);

    let mut indexed_specs = Vec::with_capacity(specs.len());

    for spec in specs {
        // Generate embedding for the purpose
        let purpose_texts = vec![spec.purpose.as_str()];
        let purpose_embeddings = model
            .embed(purpose_texts, None)
            .map_err(|e| Error::Other(format!("Failed to generate purpose embedding: {}", e)))?;

        let purpose_embedding = purpose_embeddings.into_iter().next().unwrap_or_default();

        // Generate embeddings for requirements
        let mut indexed_requirements = Vec::with_capacity(spec.requirements.len());

        for req in &spec.requirements {
            let req_texts = vec![req.description.as_str()];
            let req_embeddings = model.embed(req_texts, None).map_err(|e| {
                Error::Other(format!("Failed to generate requirement embedding: {}", e))
            })?;

            let description_embedding = req_embeddings.into_iter().next().unwrap_or_default();

            indexed_requirements.push(IndexedRequirement {
                name: req.name.clone(),
                description: req.description.clone(),
                description_embedding,
            });
        }

        indexed_specs.push(IndexedSpec {
            id: spec.id.clone(),
            title: spec.title.clone(),
            purpose_embedding,
            requirements: indexed_requirements,
        });
    }

    Ok(SpecIndex {
        model_name,
        specs: indexed_specs,
    })
}

/// Save the search index to a file.
///
/// # Arguments
///
/// * `index` - The index to save
/// * `path` - The path to save the index to
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn save_index(index: &SpecIndex, path: &Path) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::Other(format!("Failed to create index directory: {}", e)))?;
    }

    let file = fs::File::create(path)
        .map_err(|e| Error::Other(format!("Failed to create index file: {}", e)))?;

    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, index)
        .map_err(|e| Error::Other(format!("Failed to serialize index: {}", e)))?;

    Ok(())
}

/// Load the search index from a file.
///
/// # Arguments
///
/// * `path` - The path to load the index from
///
/// # Returns
///
/// The loaded `SpecIndex`.
///
/// # Errors
///
/// Returns an error if the file cannot be read or deserialized.
pub fn load_index(path: &Path) -> Result<SpecIndex> {
    let file = fs::File::open(path)
        .map_err(|e| Error::Other(format!("Failed to open index file: {}", e)))?;

    let reader = BufReader::new(file);
    let index: SpecIndex = bincode::deserialize_from(reader)
        .map_err(|e| Error::Other(format!("Failed to deserialize index: {}", e)))?;

    Ok(index)
}

/// Search the index for relevant specs and requirements.
///
/// # Arguments
///
/// * `index` - The search index to query
/// * `query` - The search query text
/// * `top_k` - The maximum number of results to return
///
/// # Returns
///
/// A vector of search results, sorted by relevance score (highest first).
///
/// # Errors
///
/// Returns an error if the search fails.
pub fn search(index: &SpecIndex, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
    // Initialize the embedding model for query
    let model = TextEmbedding::try_new(
        InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(false),
    )
    .map_err(|e| Error::Other(format!("Failed to initialize embedding model: {}", e)))?;

    // Generate embedding for the query
    let query_texts = vec![query];
    let query_embeddings = model
        .embed(query_texts, None)
        .map_err(|e| Error::Other(format!("Failed to generate query embedding: {}", e)))?;

    let query_embedding = query_embeddings
        .into_iter()
        .next()
        .ok_or_else(|| Error::Other("Failed to get query embedding".to_string()))?;

    // Collect all results
    let mut results = Vec::new();

    for spec in &index.specs {
        // Score against spec purpose
        let purpose_score = cosine_similarity(&query_embedding, &spec.purpose_embedding);
        if purpose_score > 0.0 {
            results.push(SearchResult {
                spec_id: spec.id.clone(),
                requirement: None,
                score: purpose_score,
                snippet: format!("[{}] {}", spec.title, truncate_text(&spec.title, 100)),
            });
        }

        // Score against each requirement
        for req in &spec.requirements {
            let req_score = cosine_similarity(&query_embedding, &req.description_embedding);
            if req_score > 0.0 {
                results.push(SearchResult {
                    spec_id: spec.id.clone(),
                    requirement: Some(req.name.clone()),
                    score: req_score,
                    snippet: truncate_text(&req.description, 100),
                });
            }
        }
    }

    // Sort by score (highest first)
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top_k results
    results.truncate(top_k);

    Ok(results)
}

/// Calculate cosine similarity between two vectors.
///
/// # Arguments
///
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
///
/// The cosine similarity score between 0.0 and 1.0.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Truncate text to a maximum length, adding ellipsis if needed.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Cosine similarity tests ====================

    #[test]
    fn test_cosine_similarity_identical_vectors() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.0001, "Expected 1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_orthogonal_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 0.0001, "Expected 0.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_opposite_vectors() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 0.0001, "Expected -1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0, "Expected 0.0 for different length vectors");
    }

    #[test]
    fn test_cosine_similarity_empty_vectors() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0, "Expected 0.0 for empty vectors");
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0, "Expected 0.0 when one vector is zero");
    }

    // ==================== Truncate text tests ====================

    #[test]
    fn test_truncate_text_short() {
        let result = truncate_text("hello", 10);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_text_long() {
        let result = truncate_text("hello world this is a long text", 15);
        assert_eq!(result, "hello world ...");
    }

    #[test]
    fn test_truncate_text_exact() {
        let result = truncate_text("hello", 5);
        assert_eq!(result, "hello");
    }

    // ==================== Index save/load tests ====================

    #[test]
    fn test_save_and_load_index() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index.bin");

        // Create a simple index
        let index = SpecIndex {
            model_name: "test-model".to_string(),
            specs: vec![IndexedSpec {
                id: "test-spec".to_string(),
                title: "Test Spec".to_string(),
                purpose_embedding: vec![0.1, 0.2, 0.3],
                requirements: vec![IndexedRequirement {
                    name: "Test Requirement".to_string(),
                    description: "This is a test requirement.".to_string(),
                    description_embedding: vec![0.4, 0.5, 0.6],
                }],
            }],
        };

        // Save the index
        let save_result = save_index(&index, &index_path);
        assert!(
            save_result.is_ok(),
            "Failed to save index: {:?}",
            save_result
        );

        // Load the index
        let load_result = load_index(&index_path);
        assert!(
            load_result.is_ok(),
            "Failed to load index: {:?}",
            load_result
        );

        let loaded_index = load_result.unwrap();
        assert_eq!(loaded_index.model_name, "test-model");
        assert_eq!(loaded_index.specs.len(), 1);
        assert_eq!(loaded_index.specs[0].id, "test-spec");
        assert_eq!(loaded_index.specs[0].requirements.len(), 1);
    }

    #[test]
    fn test_load_index_not_found() {
        let result = load_index(Path::new("/nonexistent/path/index.bin"));
        assert!(result.is_err());
    }

    // ==================== Integration tests (require model download) ====================

    // These tests are marked with #[ignore] by default because they require
    // downloading the embedding model (~23MB). Run with:
    //   cargo test core::index -- --ignored

    #[test]
    #[ignore]
    fn test_build_index_with_specs() {
        use super::super::spec::ParsedSpec;

        let specs = vec![ParsedSpec {
            id: "auth".to_string(),
            title: "Authentication".to_string(),
            purpose: "Handle user authentication and sessions.".to_string(),
            requirements: vec![super::super::spec::ParsedRequirement {
                name: "Login".to_string(),
                description: "Users can log in with email and password.".to_string(),
                scenarios: vec![],
            }],
        }];

        let result = build_index(&specs);
        assert!(result.is_ok(), "Failed to build index: {:?}", result);

        let index = result.unwrap();
        assert_eq!(index.specs.len(), 1);
        assert_eq!(index.specs[0].id, "auth");
        assert!(!index.specs[0].purpose_embedding.is_empty());
        assert_eq!(index.specs[0].requirements.len(), 1);
        assert!(!index.specs[0].requirements[0]
            .description_embedding
            .is_empty());
    }

    #[test]
    #[ignore]
    fn test_search_with_matches() {
        use super::super::spec::ParsedSpec;

        let specs = vec![
            ParsedSpec {
                id: "auth".to_string(),
                title: "Authentication".to_string(),
                purpose: "Handle user authentication and login sessions.".to_string(),
                requirements: vec![super::super::spec::ParsedRequirement {
                    name: "Login".to_string(),
                    description: "Users can log in with email and password.".to_string(),
                    scenarios: vec![],
                }],
            },
            ParsedSpec {
                id: "payments".to_string(),
                title: "Payments".to_string(),
                purpose: "Handle payment processing and transactions.".to_string(),
                requirements: vec![super::super::spec::ParsedRequirement {
                    name: "Checkout".to_string(),
                    description: "Process credit card payments at checkout.".to_string(),
                    scenarios: vec![],
                }],
            },
        ];

        let index = build_index(&specs).expect("Failed to build index");

        // Search for authentication-related content
        let results = search(&index, "user login authentication", 5).expect("Failed to search");

        assert!(!results.is_empty(), "Expected search results");

        // The auth spec should be in the results
        let has_auth = results.iter().any(|r| r.spec_id == "auth");
        assert!(has_auth, "Expected auth spec in results");
    }

    #[test]
    #[ignore]
    fn test_search_with_no_matches() {
        use super::super::spec::ParsedSpec;

        let specs = vec![ParsedSpec {
            id: "auth".to_string(),
            title: "Authentication".to_string(),
            purpose: "Handle user authentication.".to_string(),
            requirements: vec![],
        }];

        let index = build_index(&specs).expect("Failed to build index");

        // Search for something completely unrelated - with semantic search,
        // we may still get some results with low scores
        let results = search(&index, "xyzzy quantum chromodynamics", 5).expect("Failed to search");

        // With semantic search, we might get results but with low scores
        // Just verify the search doesn't crash
        assert!(results.len() <= 5);
    }

    #[test]
    #[ignore]
    fn test_build_and_save_load_roundtrip() {
        use super::super::spec::ParsedSpec;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let index_path = temp_dir.path().join("roundtrip_index.bin");

        let specs = vec![ParsedSpec {
            id: "test".to_string(),
            title: "Test Spec".to_string(),
            purpose: "This is a test specification.".to_string(),
            requirements: vec![super::super::spec::ParsedRequirement {
                name: "Test Requirement".to_string(),
                description: "This is a test requirement description.".to_string(),
                scenarios: vec![],
            }],
        }];

        // Build the index
        let index = build_index(&specs).expect("Failed to build index");

        // Save it
        save_index(&index, &index_path).expect("Failed to save index");

        // Load it back
        let loaded_index = load_index(&index_path).expect("Failed to load index");

        // Verify the loaded index matches
        assert_eq!(loaded_index.model_name, index.model_name);
        assert_eq!(loaded_index.specs.len(), 1);
        assert_eq!(loaded_index.specs[0].id, "test");
        assert_eq!(
            loaded_index.specs[0].purpose_embedding.len(),
            index.specs[0].purpose_embedding.len()
        );
    }
}
