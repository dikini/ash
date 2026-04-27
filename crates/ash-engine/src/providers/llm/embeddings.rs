//! Embedding Implementation
//!
//! Implements text embedding action converting between Ash Values and async-openai types.
//!
//! # Value Contract
//!
//! ## Input texts: `List<String>`
//! A list of strings to embed.
//!
//! ## Output Embedding Shape (per SPEC-029 §3.9)
//! ```text
//! {
//!   index: Int,
//!   embedding: List(Float)  // Embedding vector as Float values
//! }
//! ```
//!
//! ## Full Response: List(Embedding)
//! A list of Embedding records, one per input text.

use ash_core::Value;
use ash_core::capability::CapabilityError;
use async_openai::types::{CreateEmbeddingRequest, CreateEmbeddingResponse, Embedding};

/// Extract embed action arguments
///
/// Expected: [provider, model, texts]
/// - provider: String - provider name for routing
/// - model: String - model identifier
/// - texts: `List<String>` - texts to embed
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if args are missing or have wrong types.
pub fn extract_embed_args(args: &[Value]) -> Result<(&str, &str, &[Value]), CapabilityError> {
    if args.len() < 3 {
        return Err(CapabilityError::InvalidArgument(
            "embed requires provider, model, and texts args".to_string(),
        ));
    }

    let provider = args[0]
        .as_string()
        .ok_or_else(|| CapabilityError::InvalidArgument("provider must be a string".to_string()))?;

    let model = args[1]
        .as_string()
        .ok_or_else(|| CapabilityError::InvalidArgument("model must be a string".to_string()))?;

    let texts = match &args[2] {
        Value::List(t) => t.as_slice(),
        _ => {
            return Err(CapabilityError::InvalidArgument(
                "texts must be a list".to_string(),
            ));
        }
    };

    // Validate that texts is non-empty
    if texts.is_empty() {
        return Err(CapabilityError::InvalidArgument(
            "texts list cannot be empty".to_string(),
        ));
    }

    Ok((provider, model, texts))
}

/// Build `CreateEmbeddingRequest` from action arguments
///
/// # Arguments
/// * `model` - The model identifier to use for embedding
/// * `texts` - Vector of strings to embed
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if texts is empty or contains non-string values.
pub fn build_embed_request(
    model: &str,
    texts: Vec<String>,
) -> Result<CreateEmbeddingRequest, CapabilityError> {
    if texts.is_empty() {
        return Err(CapabilityError::InvalidArgument(
            "texts list cannot be empty".to_string(),
        ));
    }

    Ok(CreateEmbeddingRequest {
        model: model.to_string(),
        input: async_openai::types::EmbeddingInput::StringArray(texts),
        encoding_format: Some(async_openai::types::EncodingFormat::Float),
        dimensions: None,
        user: None,
    })
}

/// Convert a single `Embedding` to Ash Value
///
/// Output shape (per SPEC-029 §3.9):
/// ```text
/// {
///   index: Int,
///   embedding: List(Float)  // Embedding vector values as Float
/// }
/// ```
fn embedding_to_value(embedding: &Embedding) -> Value {
    // Convert embedding vector (Vec<f32>) to List(Float)
    let embedding_values: Vec<Value> = embedding
        .embedding
        .iter()
        .map(|f| Value::Float(f64::from(*f)))
        .collect();

    let mut fields = std::collections::HashMap::new();
    fields.insert("index".to_string(), Value::Int(i64::from(embedding.index)));
    fields.insert(
        "embedding".to_string(),
        Value::List(Box::new(embedding_values)),
    );

    Value::Record(Box::new(fields))
}

/// Convert `CreateEmbeddingResponse` to Ash Value
///
/// Returns: List(Embedding)
/// Each Embedding: { index: Int, embedding: List(Float) }
///
/// # Postconditions
/// - E1: result.length == texts.length (verified in tests)
/// - E2: result\[i\].index == i for all i (verified in tests)
///
/// # Errors
/// Returns `CapabilityError::ExecutionFailed` if the response contains no data.
pub fn embed_response_to_value(
    response: CreateEmbeddingResponse,
) -> Result<Value, CapabilityError> {
    if response.data.is_empty() {
        return Err(CapabilityError::ExecutionFailed(
            "No embeddings in response".to_string(),
        ));
    }

    // Sort embeddings by index to ensure correct ordering (E2)
    let mut embeddings = response.data;
    embeddings.sort_by_key(|e| e.index);

    // Verify postcondition E2: indices should be 0, 1, 2, ...
    for (i, embedding) in embeddings.iter().enumerate() {
        if embedding.index != u32::try_from(i).unwrap_or(u32::MAX) {
            return Err(CapabilityError::ExecutionFailed(format!(
                "Embedding index mismatch: expected {i}, got {}",
                embedding.index
            )));
        }
    }

    let embedding_values: Vec<Value> = embeddings.iter().map(embedding_to_value).collect();

    Ok(Value::List(Box::new(embedding_values)))
}

/// Convert texts from Ash Values to String vector
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if any text is not a string.
pub fn texts_to_strings(texts: &[Value]) -> Result<Vec<String>, CapabilityError> {
    texts
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_string().map(String::from).ok_or_else(|| {
                CapabilityError::InvalidArgument(format!("text at index {i} must be a string"))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_openai::types::Embedding;

    // =================================================================
    // extract_embed_args tests
    // =================================================================

    #[test]
    fn test_extract_embed_args_valid() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("text-embedding-3-small".to_string()),
            Value::List(Box::new(vec![
                Value::String("Hello".to_string()),
                Value::String("World".to_string()),
            ])),
        ];

        let result = extract_embed_args(&args);
        assert!(result.is_ok());

        let (provider, model, texts) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-3-small");
        assert_eq!(texts.len(), 2);
    }

    #[test]
    fn test_extract_embed_args_too_few_args() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("model".to_string()),
        ];

        let result = extract_embed_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("requires provider, model, and texts"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_embed_args_wrong_provider_type() {
        let args = vec![
            Value::Int(42),
            Value::String("model".to_string()),
            Value::List(Box::new(vec![Value::String("text".to_string())])),
        ];

        let result = extract_embed_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("provider must be a string"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_embed_args_wrong_model_type() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::Int(42),
            Value::List(Box::new(vec![Value::String("text".to_string())])),
        ];

        let result = extract_embed_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("model must be a string"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_embed_args_wrong_texts_type() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("model".to_string()),
            Value::String("not-a-list".to_string()),
        ];

        let result = extract_embed_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("texts must be a list"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_embed_args_empty_texts() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("model".to_string()),
            Value::List(Box::default()),
        ];

        let result = extract_embed_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("cannot be empty"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    // =================================================================
    // build_embed_request tests
    // =================================================================

    #[test]
    fn test_build_embed_request_valid() {
        let texts = vec!["Hello".to_string(), "World".to_string()];
        let result = build_embed_request("text-embedding-3-small", texts);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.model, "text-embedding-3-small");

        // Verify input is string array
        match request.input {
            async_openai::types::EmbeddingInput::StringArray(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], "Hello");
                assert_eq!(arr[1], "World");
            }
            _ => panic!("Expected StringArray input"),
        }
    }

    #[test]
    fn test_build_embed_request_empty_texts() {
        let texts: Vec<String> = vec![];
        let result = build_embed_request("model", texts);

        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("cannot be empty"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    // =================================================================
    // embed_response_to_value tests
    // =================================================================

    fn create_test_embedding(index: u32, values: Vec<f32>) -> Embedding {
        Embedding {
            index,
            embedding: values,
            object: "embedding".to_string(),
        }
    }

    #[test]
    fn test_embed_response_to_value_valid() {
        let response = CreateEmbeddingResponse {
            data: vec![
                create_test_embedding(0, vec![0.1, 0.2, 0.3]),
                create_test_embedding(1, vec![0.4, 0.5, 0.6]),
            ],
            model: "text-embedding-3-small".to_string(),
            object: "list".to_string(),
            usage: async_openai::types::EmbeddingUsage {
                prompt_tokens: 10,
                total_tokens: 10,
            },
        };

        let result = embed_response_to_value(response);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            Value::List(embeddings) => {
                // E1: result.length == texts.length
                assert_eq!(embeddings.len(), 2);

                // Check first embedding
                match &embeddings[0] {
                    Value::Record(fields) => {
                        // E2: result[i].index == i
                        assert_eq!(fields.get("index"), Some(&Value::Int(0)));

                        // Check embedding values - now Float per SPEC-029
                        match fields.get("embedding") {
                            Some(Value::List(values)) => {
                                assert_eq!(values.len(), 3);
                                // Use approximate comparison for float values (f32 -> f64 conversion)
                                match (&values[0], &values[1], &values[2]) {
                                    (Value::Float(a), Value::Float(b), Value::Float(c)) => {
                                        assert!((a - 0.1).abs() < 1e-6, "Expected ~0.1, got {a}");
                                        assert!((b - 0.2).abs() < 1e-6, "Expected ~0.2, got {b}");
                                        assert!((c - 0.3).abs() < 1e-6, "Expected ~0.3, got {c}");
                                    }
                                    _ => panic!("Expected Float values in embedding list"),
                                }
                            }
                            _ => panic!("Expected embedding to be a List"),
                        }
                    }
                    _ => panic!("Expected embedding to be a Record"),
                }

                // Check second embedding
                match &embeddings[1] {
                    Value::Record(fields) => {
                        assert_eq!(fields.get("index"), Some(&Value::Int(1)));
                    }
                    _ => panic!("Expected embedding to be a Record"),
                }
            }
            _ => panic!("Expected result to be a List"),
        }
    }

    #[test]
    fn test_embed_response_to_value_empty() {
        let response = CreateEmbeddingResponse {
            data: vec![],
            model: "model".to_string(),
            object: "list".to_string(),
            usage: async_openai::types::EmbeddingUsage {
                prompt_tokens: 0,
                total_tokens: 0,
            },
        };

        let result = embed_response_to_value(response);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::ExecutionFailed(msg) => {
                assert!(msg.contains("No embeddings"));
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[test]
    fn test_embed_response_to_value_unordered_indices() {
        // Test that indices are verified to be in order (E2)
        let response = CreateEmbeddingResponse {
            data: vec![
                create_test_embedding(1, vec![0.4, 0.5, 0.6]), // Wrong index!
                create_test_embedding(0, vec![0.1, 0.2, 0.3]),
            ],
            model: "model".to_string(),
            object: "list".to_string(),
            usage: async_openai::types::EmbeddingUsage {
                prompt_tokens: 10,
                total_tokens: 10,
            },
        };

        // The function should detect index mismatch after sorting
        // Actually, it sorts and then checks, so it should pass if indices are 0,1 after sorting
        let result = embed_response_to_value(response);
        assert!(result.is_ok()); // Sorting fixes the order
    }

    #[test]
    fn test_embed_response_to_value_missing_index() {
        // Missing index 1 (has 0 and 2)
        let response = CreateEmbeddingResponse {
            data: vec![
                create_test_embedding(0, vec![0.1]),
                create_test_embedding(2, vec![0.3]), // Gap at index 1
            ],
            model: "model".to_string(),
            object: "list".to_string(),
            usage: async_openai::types::EmbeddingUsage {
                prompt_tokens: 10,
                total_tokens: 10,
            },
        };

        let result = embed_response_to_value(response);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::ExecutionFailed(msg) => {
                assert!(msg.contains("index mismatch"));
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    // =================================================================
    // texts_to_strings tests
    // =================================================================

    #[test]
    fn test_texts_to_strings_valid() {
        let texts = vec![
            Value::String("Hello".to_string()),
            Value::String("World".to_string()),
        ];

        let result = texts_to_strings(&texts);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Hello", "World"]);
    }

    #[test]
    fn test_texts_to_strings_invalid_type() {
        let texts = vec![
            Value::String("Hello".to_string()),
            Value::Int(42),
            Value::String("World".to_string()),
        ];

        let result = texts_to_strings(&texts);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("index 1 must be a string"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    // =================================================================
    // Postcondition verification tests (E1, E2)
    // =================================================================

    #[test]
    fn test_postcondition_e1_output_length_matches_input() {
        // Create response with 3 embeddings
        let response = CreateEmbeddingResponse {
            data: vec![
                create_test_embedding(0, vec![0.1]),
                create_test_embedding(1, vec![0.2]),
                create_test_embedding(2, vec![0.3]),
            ],
            model: "model".to_string(),
            object: "list".to_string(),
            usage: async_openai::types::EmbeddingUsage {
                prompt_tokens: 30,
                total_tokens: 30,
            },
        };

        let result = embed_response_to_value(response).unwrap();
        match result {
            Value::List(embeddings) => {
                // E1: result.length == texts.length
                assert_eq!(
                    embeddings.len(),
                    3,
                    "E1 violated: output length doesn't match input"
                );
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_postcondition_e2_indices_are_correct() {
        // Create response with embeddings in correct order
        let response = CreateEmbeddingResponse {
            data: vec![
                create_test_embedding(0, vec![0.1]),
                create_test_embedding(1, vec![0.2]),
                create_test_embedding(2, vec![0.3]),
            ],
            model: "model".to_string(),
            object: "list".to_string(),
            usage: async_openai::types::EmbeddingUsage {
                prompt_tokens: 30,
                total_tokens: 30,
            },
        };

        let result = embed_response_to_value(response).unwrap();
        match result {
            Value::List(embeddings) => {
                // E2: result[i].index == i for all i
                for (i, emb) in embeddings.iter().enumerate() {
                    match emb {
                        Value::Record(fields) => {
                            let index_val = fields.get("index").expect("Missing index field");
                            match index_val {
                                Value::Int(idx) => {
                                    assert_eq!(
                                        *idx,
                                        i64::try_from(i).expect("usize fits in i64"),
                                        "E2 violated: index {idx} doesn't match position {i}"
                                    );
                                }
                                _ => panic!("index field should be Int"),
                            }
                        }
                        _ => panic!("Expected Record"),
                    }
                }
            }
            _ => panic!("Expected List"),
        }
    }
}
