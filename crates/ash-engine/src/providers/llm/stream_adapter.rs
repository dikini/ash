//! Streaming Chat Completion Adapter
//!
//! Adapts `async-openai`'s `Stream<CreateChatCompletionStreamResponse>` into Ash's stream model,
//! with chunk parsing for delta content, tool call deltas, and finish reason.
//!
//! # Streaming Contract (from SPEC-029 §9.5)
//!
//! - **SC1**: Each incoming SSE event maps to exactly one `ChatChunk`
//! - **SC2**: Events with no delta content and no delta tool calls are dropped (keep-alive pings)
//! - **SC3**: The final event (with `finish_reason`) produces a `ChatChunk` where `finish_reason` is set
//! - **SC4**: On stream error, the stream terminates with an error rather than silently stopping
//! - **SC5**: The stream does not buffer the entire response; chunks are yielded as they arrive
//!
//! # Value Contract
//!
//! ## `ChatChunk` Output Shape
//! ```text
//! {
//!   delta_content: Option<String>,        // Uses Some/None variants
//!   delta_tool_calls: Option<List<ToolCallDelta>>,  // Uses Some/None variants
//!   finish_reason: Option<String>         // Uses Some/None variants
//! }
//! ```
//!
//! ## `ToolCallDelta` Shape
//! ```text
//! {
//!   index: Int,
//!   id: Option<String>,
//!   name: Option<String>,
//!   arguments: Option<String>
//! }
//! ```

use ash_core::Value;
use ash_core::capability::CapabilityError;
use async_openai::types::{
    ChatCompletionMessageToolCallChunk, CreateChatCompletionStreamResponse, FinishReason,
};
use std::collections::HashMap;

/// Parse an Option field from a Value
///
/// Expected: `Some(value)` as `Value::Variant { name: "Some", fields: [("0", value)] }`
///           or `None` as `Value::Variant { name: "None", fields: [] }`
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if the value is not a Some/None variant or is malformed.
#[allow(dead_code)]
fn parse_option_field(value: &Value) -> Result<Option<&Value>, CapabilityError> {
    match value {
        Value::Variant { name, fields } => {
            match name.as_str() {
                "Some" => {
                    if fields.is_empty() {
                        return Err(CapabilityError::InvalidArgument(
                            "Some variant should have a value".to_string(),
                        ));
                    }
                    // Find the value field (typically "0" for tuple-like variants)
                    let val = fields
                        .iter()
                        .find(|(k, _)| k == "0")
                        .map(|(_, v)| v)
                        .or_else(|| fields.first().map(|(_, v)| v))
                        .ok_or_else(|| {
                            CapabilityError::InvalidArgument(
                                "Some variant missing value field".to_string(),
                            )
                        })?;
                    Ok(Some(val))
                }
                "None" => Ok(None),
                _ => Err(CapabilityError::InvalidArgument(format!(
                    "Expected Some or None variant, got {name}"
                ))),
            }
        }
        Value::Null => Ok(None),
        _ => Err(CapabilityError::InvalidArgument(
            "Option field must be Some or None variant".to_string(),
        )),
    }
}

/// Convert an async-openai stream chunk to Ash Value
///
/// Returns `None` for empty chunks (keep-alive pings per SC2).
/// Returns `Some(Value::Record(...))` for valid chunks containing delta content,
/// tool call deltas, or finish reason.
///
/// # Arguments
/// * `chunk` - The stream response chunk from async-openai
///
/// # Returns
/// * `Some(Value)` - A `ChatChunk` record with `delta_content`, `delta_tool_calls`, and `finish_reason`
/// * `None` - If the chunk is empty (no delta content and no tool calls)
#[must_use]
pub fn stream_chunk_to_value(chunk: &CreateChatCompletionStreamResponse) -> Option<Value> {
    // Get the first choice (there should typically be one)
    let choice = chunk.choices.first()?;
    let delta = &choice.delta;

    // Extract delta content
    let delta_content = delta.content.clone();

    // Extract tool call deltas
    let delta_tool_calls = delta.tool_calls.clone();

    // Check if this is an empty chunk (SC2: drop keep-alive pings)
    let has_content = delta_content.is_some();
    // Fix: Check if tool_calls is Some AND non-empty
    let has_tool_calls = delta_tool_calls.as_ref().is_some_and(|v| !v.is_empty());

    // Extract finish_reason if present (SC3)
    let finish_reason = choice.finish_reason.as_ref().map(|fr| match fr {
        FinishReason::Stop => "stop",
        FinishReason::Length => "length",
        FinishReason::ToolCalls => "tool_calls",
        FinishReason::ContentFilter => "content_filter",
        FinishReason::FunctionCall => "function_call",
    });

    // If no content, no tool calls, and no finish_reason, drop this chunk (SC2)
    if !has_content && !has_tool_calls && finish_reason.is_none() {
        return None;
    }

    // Build finish_reason field using Some/None variants
    let finish_reason_value = finish_reason.map_or_else(
        || Value::unit_variant("None"),
        |r| Value::variant("Some", vec![("0", Value::String(r.to_string()))]),
    );

    // Build delta_content and delta_tool_calls fields
    // Per SPEC-029: final chunk has finish_reason=Some(_) and delta_content/delta_tool_calls=None
    let (delta_content_value, delta_tool_calls_value) = if finish_reason.is_some() {
        // Final chunk: finish_reason is set, deltas are None
        (Value::unit_variant("None"), Value::unit_variant("None"))
    } else {
        // Non-final chunk: include deltas
        let content = delta_content.map_or_else(
            || Value::unit_variant("None"),
            |c| Value::variant("Some", vec![("0", Value::String(c))]),
        );
        let tool_calls = delta_tool_calls.map_or_else(
            || Value::unit_variant("None"),
            |calls| {
                let tool_call_values: Vec<Value> =
                    calls.into_iter().map(tool_call_chunk_to_value).collect();
                Value::variant("Some", vec![("0", Value::list_from_vec(tool_call_values))])
            },
        );
        (content, tool_calls)
    };

    // Build ChatChunk record
    let mut fields = HashMap::new();
    fields.insert("delta_content".to_string(), delta_content_value);
    fields.insert("delta_tool_calls".to_string(), delta_tool_calls_value);
    fields.insert("finish_reason".to_string(), finish_reason_value);

    Some(Value::Record(Box::new(fields)))
}

/// Convert a `ChatCompletionMessageToolCallChunk` to a `ToolCallDelta` Value
///
/// # Arguments
/// * `chunk` - The tool call chunk from the stream
///
/// # Returns
/// A `Value::Record` representing the `ToolCallDelta`
fn tool_call_chunk_to_value(chunk: ChatCompletionMessageToolCallChunk) -> Value {
    let mut fields = HashMap::new();

    // index is always present
    fields.insert("index".to_string(), Value::Int(i64::from(chunk.index)));

    // id is optional
    let id_value = chunk.id.map_or_else(
        || Value::unit_variant("None"),
        |id| Value::variant("Some", vec![("0", Value::String(id))]),
    );
    fields.insert("id".to_string(), id_value);

    // name and arguments come from the function field
    let (name, arguments) = chunk
        .function
        .map_or((None, None), |f| (f.name, f.arguments));

    let name_value = name.map_or_else(
        || Value::unit_variant("None"),
        |n| Value::variant("Some", vec![("0", Value::String(n))]),
    );
    fields.insert("name".to_string(), name_value);

    let arguments_value = arguments.map_or_else(
        || Value::unit_variant("None"),
        |a| Value::variant("Some", vec![("0", Value::String(a))]),
    );
    fields.insert("arguments".to_string(), arguments_value);

    Value::Record(Box::new(fields))
}

/// Stream adapter that implements Ash stream protocol
///
/// Wraps the async-openai stream and provides conversion to Ash Values.
/// Implements SC1-SC5 streaming contracts.
pub struct ChatStream {
    /// Unique identifier for this stream instance
    pub id: String,
    /// Provider name for error context
    pub provider: String,
    /// Model name for error context
    pub model: String,
}

impl ChatStream {
    /// Create a new `ChatStream`
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this stream
    /// * `provider` - Provider name (for error context)
    /// * `model` - Model name (for error context)
    pub fn new(
        id: impl Into<String>,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            provider: provider.into(),
            model: model.into(),
        }
    }
}

/// Chat stream extraction result: (provider, model, messages, params)
type ChatStreamArgsResult<'a> =
    Result<(&'a str, &'a str, Vec<Value>, Option<&'a Value>), CapabilityError>;

/// Extract `chat_stream` action arguments
///
/// Expected args: [provider, model, messages, params]
/// Returns: (provider, model, messages, params)
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if args are missing or have wrong types.
///
/// # Examples
///
/// ```
/// use ash_core::Value;
/// use ash_engine::providers::llm::stream_adapter::extract_chat_stream_args;
///
/// let args = vec![
///     Value::String("openai".to_string()),
///     Value::String("gpt-4o".to_string()),
///     Value::list_nil(),
/// ];
/// let result = extract_chat_stream_args(&args);
/// assert!(result.is_ok());
/// ```
pub fn extract_chat_stream_args(args: &[Value]) -> ChatStreamArgsResult<'_> {
    if args.len() < 3 {
        return Err(CapabilityError::InvalidArgument(
            "chat_stream requires provider, model, and messages args".to_string(),
        ));
    }

    let provider = args[0].as_string().ok_or_else(|| {
        CapabilityError::InvalidArgument("chat_stream provider must be a string".to_string())
    })?;

    let model = args[1].as_string().ok_or_else(|| {
        CapabilityError::InvalidArgument("chat_stream model must be a string".to_string())
    })?;

    let messages = args[2].list_to_vec().ok_or_else(|| {
        CapabilityError::InvalidArgument("chat_stream messages must be a list".to_string())
    })?;

    let params = args.get(3);

    Ok((provider, model, messages, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_openai::types::{
        ChatChoiceStream, ChatCompletionMessageToolCallChunk, ChatCompletionStreamResponseDelta,
        FunctionCallStream,
    };

    // ===== Stream Chunk Conversion Tests =====

    #[test]
    fn test_stream_chunk_with_delta_content() {
        let chunk = CreateChatCompletionStreamResponse {
            id: "chunk_123".to_string(),
            choices: vec![ChatChoiceStream {
                index: 0,
                delta: ChatCompletionStreamResponseDelta {
                    content: Some("Hello".to_string()),
                    #[allow(deprecated)]
                    function_call: None,
                    tool_calls: None,
                    role: None,
                    refusal: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            service_tier: None,
            system_fingerprint: None,
            object: "chat.completion.chunk".to_string(),
            usage: None,
        };

        let result = stream_chunk_to_value(&chunk);
        assert!(result.is_some());

        let Value::Record(fields) = result.unwrap() else {
            panic!("Expected Record");
        };

        // Check delta_content is Some("Hello")
        match fields.get("delta_content").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                assert_eq!(fields[0].1.as_string(), Some("Hello"));
            }
            _ => panic!("Expected Some variant for delta_content"),
        }

        // Check delta_tool_calls is None
        match fields.get("delta_tool_calls").unwrap() {
            Value::Variant { name, fields: _ } => {
                assert_eq!(name, "None");
            }
            _ => panic!("Expected None variant for delta_tool_calls"),
        }

        // Check finish_reason is None
        match fields.get("finish_reason").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "None");
                assert!(fields.is_empty());
            }
            _ => panic!("Expected None variant for finish_reason"),
        }
    }

    #[test]
    fn test_stream_chunk_with_tool_call_delta() {
        let chunk = CreateChatCompletionStreamResponse {
            id: "chunk_123".to_string(),
            choices: vec![ChatChoiceStream {
                index: 0,
                delta: ChatCompletionStreamResponseDelta {
                    content: None,
                    #[allow(deprecated)]
                    function_call: None,
                    tool_calls: Some(vec![ChatCompletionMessageToolCallChunk {
                        index: 0,
                        id: Some("call_123".to_string()),
                        r#type: None,
                        function: Some(FunctionCallStream {
                            name: Some("get_weather".to_string()),
                            arguments: Some(r#"{"city": "NYC"}"#.to_string()),
                        }),
                    }]),
                    role: None,
                    refusal: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            service_tier: None,
            system_fingerprint: None,
            object: "chat.completion.chunk".to_string(),
            usage: None,
        };

        let result = stream_chunk_to_value(&chunk);
        assert!(result.is_some());

        let Value::Record(fields) = result.unwrap() else {
            panic!("Expected Record");
        };

        // Check delta_content is None
        match fields.get("delta_content").unwrap() {
            Value::Variant { name, fields: _ } => {
                assert_eq!(name, "None");
            }
            _ => panic!("Expected None variant for delta_content"),
        }

        // Check delta_tool_calls is Some with one tool call
        match fields.get("delta_tool_calls").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                let tool_calls = fields[0].1.list_to_vec().expect("Expected List");
                assert_eq!(tool_calls.len(), 1);

                // Check the tool call delta structure
                let Value::Record(tc_fields) = &tool_calls[0] else {
                    panic!("Expected Record for tool call");
                };

                // Check index
                assert_eq!(tc_fields.get("index").unwrap().as_int(), Some(0));

                // Check id is Some("call_123")
                match tc_fields.get("id").unwrap() {
                    Value::Variant { name, fields } => {
                        assert_eq!(name, "Some");
                        assert_eq!(fields[0].1.as_string(), Some("call_123"));
                    }
                    _ => panic!("Expected Some variant for id"),
                }

                // Check name is Some("get_weather")
                match tc_fields.get("name").unwrap() {
                    Value::Variant { name, fields } => {
                        assert_eq!(name, "Some");
                        assert_eq!(fields[0].1.as_string(), Some("get_weather"));
                    }
                    _ => panic!("Expected Some variant for name"),
                }

                // Check arguments is Some(...)
                match tc_fields.get("arguments").unwrap() {
                    Value::Variant { name, fields } => {
                        assert_eq!(name, "Some");
                        assert!(fields[0].1.as_string().is_some());
                    }
                    _ => panic!("Expected Some variant for arguments"),
                }
            }
            _ => panic!("Expected Some variant for delta_tool_calls"),
        }
    }

    #[test]
    fn test_stream_chunk_final_with_finish_reason() {
        let chunk = CreateChatCompletionStreamResponse {
            id: "chunk_123".to_string(),
            choices: vec![ChatChoiceStream {
                index: 0,
                delta: ChatCompletionStreamResponseDelta {
                    content: Some("World".to_string()),
                    #[allow(deprecated)]
                    function_call: None,
                    tool_calls: None,
                    role: None,
                    refusal: None,
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            service_tier: None,
            system_fingerprint: None,
            object: "chat.completion.chunk".to_string(),
            usage: None,
        };

        let result = stream_chunk_to_value(&chunk);
        assert!(result.is_some());

        let Value::Record(fields) = result.unwrap() else {
            panic!("Expected Record");
        };

        // Check finish_reason is Some("stop")
        match fields.get("finish_reason").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                assert_eq!(fields[0].1.as_string(), Some("stop"));
            }
            _ => panic!("Expected Some variant for finish_reason"),
        }
    }

    #[test]
    fn test_stream_chunk_empty_dropped() {
        // SC2: Empty chunks (no content, no tool calls, no finish_reason) should be dropped
        let chunk = CreateChatCompletionStreamResponse {
            id: "chunk_123".to_string(),
            choices: vec![ChatChoiceStream {
                index: 0,
                delta: ChatCompletionStreamResponseDelta {
                    content: None,
                    #[allow(deprecated)]
                    function_call: None,
                    tool_calls: None,
                    role: Some(async_openai::types::Role::Assistant),
                    refusal: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            service_tier: None,
            system_fingerprint: None,
            object: "chat.completion.chunk".to_string(),
            usage: None,
        };

        let result = stream_chunk_to_value(&chunk);
        assert!(result.is_none(), "Empty chunk should be dropped (SC2)");
    }

    #[test]
    fn test_stream_chunk_to_value_encoding() {
        // Test that the output uses proper Some/None variants
        let chunk = CreateChatCompletionStreamResponse {
            id: "chunk_123".to_string(),
            choices: vec![ChatChoiceStream {
                index: 0,
                delta: ChatCompletionStreamResponseDelta {
                    content: Some("test".to_string()),
                    #[allow(deprecated)]
                    function_call: None,
                    tool_calls: None,
                    role: None,
                    refusal: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            service_tier: None,
            system_fingerprint: None,
            object: "chat.completion.chunk".to_string(),
            usage: None,
        };

        let result = stream_chunk_to_value(&chunk).unwrap();
        let Value::Record(fields) = result else {
            panic!("Expected Record");
        };

        // Verify all optional fields use Variant encoding
        for key in ["delta_content", "delta_tool_calls", "finish_reason"] {
            match fields.get(key).unwrap() {
                Value::Variant { name, .. } => {
                    assert!(
                        name == "Some" || name == "None",
                        "{key} should be Some or None variant"
                    );
                }
                _ => panic!("{key} should be a Variant"),
            }
        }
    }

    #[test]
    fn test_stream_chunk_with_finish_reason_only() {
        // A chunk with only finish_reason should not be dropped
        let chunk = CreateChatCompletionStreamResponse {
            id: "chunk_123".to_string(),
            choices: vec![ChatChoiceStream {
                index: 0,
                delta: ChatCompletionStreamResponseDelta {
                    content: None,
                    #[allow(deprecated)]
                    function_call: None,
                    tool_calls: None,
                    role: None,
                    refusal: None,
                },
                finish_reason: Some(FinishReason::Length),
                logprobs: None,
            }],
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            service_tier: None,
            system_fingerprint: None,
            object: "chat.completion.chunk".to_string(),
            usage: None,
        };

        let result = stream_chunk_to_value(&chunk);
        assert!(
            result.is_some(),
            "Chunk with finish_reason should not be dropped"
        );

        let Value::Record(fields) = result.unwrap() else {
            panic!("Expected Record");
        };

        match fields.get("finish_reason").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                assert_eq!(fields[0].1.as_string(), Some("length"));
            }
            _ => panic!("Expected Some variant for finish_reason"),
        }
    }

    #[test]
    fn test_stream_chunk_all_finish_reasons() {
        let test_cases = vec![
            (FinishReason::Stop, "stop"),
            (FinishReason::Length, "length"),
            (FinishReason::ToolCalls, "tool_calls"),
            (FinishReason::ContentFilter, "content_filter"),
            (FinishReason::FunctionCall, "function_call"),
        ];

        for (finish_reason, expected_str) in test_cases {
            let chunk = CreateChatCompletionStreamResponse {
                id: "chunk_123".to_string(),
                choices: vec![ChatChoiceStream {
                    index: 0,
                    delta: ChatCompletionStreamResponseDelta {
                        content: Some("test".to_string()),
                        #[allow(deprecated)]
                        function_call: None,
                        tool_calls: None,
                        role: None,
                        refusal: None,
                    },
                    finish_reason: Some(finish_reason),
                    logprobs: None,
                }],
                created: 1_234_567_890,
                model: "gpt-4o".to_string(),
                service_tier: None,
                system_fingerprint: None,
                object: "chat.completion.chunk".to_string(),
                usage: None,
            };

            let result = stream_chunk_to_value(&chunk).unwrap();
            let Value::Record(fields) = result else {
                panic!("Expected Record");
            };

            match fields.get("finish_reason").unwrap() {
                Value::Variant { name, fields } => {
                    assert_eq!(name, "Some");
                    assert_eq!(fields[0].1.as_string(), Some(expected_str));
                }
                _ => panic!("Expected Some variant for finish_reason"),
            }
        }
    }

    // ===== ChatStream Struct Tests =====

    #[test]
    fn test_chat_stream_new() {
        let stream = ChatStream::new("stream_1", "openai", "gpt-4o");
        assert_eq!(stream.id, "stream_1");
        assert_eq!(stream.provider, "openai");
        assert_eq!(stream.model, "gpt-4o");
    }

    // ===== Argument Extraction Tests =====

    #[test]
    fn test_extract_chat_stream_args_valid() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::list_nil(),
        ];
        let result = extract_chat_stream_args(&args);
        assert!(result.is_ok());

        let (provider, model, messages, params) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4o");
        assert!(messages.is_empty());
        assert!(params.is_none());
    }

    #[test]
    fn test_extract_chat_stream_args_with_params() {
        let params = Value::variant("Some", vec![("0", Value::Record(Box::default()))]);
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::list_nil(),
            params,
        ];
        let result = extract_chat_stream_args(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().3.is_some());
    }

    #[test]
    fn test_extract_chat_stream_args_too_few() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
        ];
        let result = extract_chat_stream_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("chat_stream requires"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_chat_stream_args_missing_provider() {
        let args: Vec<Value> = vec![];
        let result = extract_chat_stream_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("requires provider"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_chat_stream_args_wrong_provider_type() {
        let args = vec![
            Value::Int(42),
            Value::String("gpt-4o".to_string()),
            Value::list_nil(),
        ];
        let result = extract_chat_stream_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("provider must be a string"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_chat_stream_args_wrong_model_type() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::Int(42),
            Value::list_nil(),
        ];
        let result = extract_chat_stream_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("model must be a string"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_chat_stream_args_wrong_messages_type() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::Int(42),
        ];
        let result = extract_chat_stream_args(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("messages must be a list"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_chat_stream_args_empty_provider() {
        let args = vec![
            Value::String(String::new()),
            Value::String("gpt-4o".to_string()),
            Value::list_nil(),
        ];
        // Empty string is still a valid string, so this should succeed
        let result = extract_chat_stream_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "");
    }

    #[test]
    fn test_extract_chat_stream_args_empty_model() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String(String::new()),
            Value::list_nil(),
        ];
        // Empty string is still a valid string, so this should succeed
        let result = extract_chat_stream_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, "");
    }
}
