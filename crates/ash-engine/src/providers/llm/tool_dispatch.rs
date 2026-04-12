//! Tool dispatch helpers for LLM provider
//!
//! Provides utilities for:
//! - Extracting tool calls from ChatResponse values
//! - Formatting tool results as messages for follow-up requests
//! - Converting Ash ToolDef values to OpenAI ChatCompletionTool format

use ash_core::Value;
use ash_core::capability::CapabilityError;
use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject};
use std::collections::HashMap;

/// Structured representation of a tool call extracted from a response
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCallValue {
    /// The unique identifier for this tool call
    pub id: String,
    /// The name of the function being called
    pub name: String,
    /// The JSON-encoded arguments for the function call
    pub arguments: String,
}

/// Extract tool calls from a ChatResponse Value
///
/// Input: ChatResponse Value with tool_calls field
/// Output: Vec<ToolCallValue> or CapabilityError
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if the response is not a Record,
/// lacks the tool_calls field, or contains malformed tool call data.
pub fn extract_tool_calls(response: &Value) -> Result<Vec<ToolCallValue>, CapabilityError> {
    let Value::Record(fields) = response else {
        return Err(CapabilityError::InvalidArgument(
            "Response must be a Record".to_string(),
        ));
    };

    let tool_calls_field = fields.get("tool_calls").ok_or_else(|| {
        CapabilityError::InvalidArgument("Response missing 'tool_calls' field".to_string())
    })?;

    // Parse Option field (Some(List) or None)
    let tool_calls_opt = parse_option_field(tool_calls_field)?;

    match tool_calls_opt {
        None => Ok(Vec::new()),
        Some(tool_calls_value) => {
            let Value::List(tool_calls_list) = tool_calls_value else {
                return Err(CapabilityError::InvalidArgument(
                    "tool_calls must be a List when Some".to_string(),
                ));
            };

            tool_calls_list.iter().map(parse_tool_call_value).collect()
        }
    }
}

/// Parse a single tool call from a Value
fn parse_tool_call_value(value: &Value) -> Result<ToolCallValue, CapabilityError> {
    let Value::Record(fields) = value else {
        return Err(CapabilityError::InvalidArgument(
            "ToolCall must be a Record".to_string(),
        ));
    };

    let id = fields
        .get("id")
        .and_then(|v| v.as_string())
        .ok_or_else(|| CapabilityError::InvalidArgument("ToolCall missing 'id' field".to_string()))?
        .to_string();

    let name = fields
        .get("name")
        .and_then(|v| v.as_string())
        .ok_or_else(|| {
            CapabilityError::InvalidArgument("ToolCall missing 'name' field".to_string())
        })?
        .to_string();

    let arguments = fields
        .get("arguments")
        .and_then(|v| v.as_string())
        .ok_or_else(|| {
            CapabilityError::InvalidArgument("ToolCall missing 'arguments' field".to_string())
        })?
        .to_string();

    Ok(ToolCallValue {
        id,
        name,
        arguments,
    })
}

/// Format tool result as Message Value for follow-up
///
/// Input: call_id, content
/// Output: Message Value with role=Tool, tool_call_id set
///
/// The output shape is:
/// ```text
/// {
///   role: Tool variant,
///   content: String,
///   tool_call_id: Some(String),
///   tool_calls: None
/// }
/// ```
pub fn format_tool_result_message(call_id: &str, content: &str) -> Value {
    let mut fields = HashMap::new();

    // role: Tool variant
    fields.insert("role".to_string(), Value::unit_variant("Tool"));

    // content: String
    fields.insert("content".to_string(), Value::String(content.to_string()));

    // tool_call_id: Some(call_id)
    fields.insert(
        "tool_call_id".to_string(),
        Value::variant("Some", vec![("0", Value::String(call_id.to_string()))]),
    );

    // tool_calls: None
    fields.insert("tool_calls".to_string(), Value::unit_variant("None"));

    Value::Record(Box::new(fields))
}

/// Convert Ash ToolDef values to OpenAI ChatCompletionTool format
///
/// Input: List of ToolDef Values
/// Output: Vec<ChatCompletionTool>
///
/// Expected ToolDef shape:
/// ```text
/// {
///   name: String,
///   description: String,
///   parameters: String (JSON schema)
/// }
/// ```
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if a ToolDef is not a Record,
/// lacks required fields, or has invalid JSON in parameters.
pub fn tool_defs_to_openai_tools(
    tools: &[Value],
) -> Result<Vec<ChatCompletionTool>, CapabilityError> {
    tools.iter().map(tool_def_to_openai_tool).collect()
}

/// Convert a single ToolDef Value to ChatCompletionTool
fn tool_def_to_openai_tool(value: &Value) -> Result<ChatCompletionTool, CapabilityError> {
    let Value::Record(fields) = value else {
        return Err(CapabilityError::InvalidArgument(
            "ToolDef must be a Record".to_string(),
        ));
    };

    let name = fields
        .get("name")
        .and_then(|v| v.as_string())
        .ok_or_else(|| {
            CapabilityError::InvalidArgument("ToolDef missing 'name' field".to_string())
        })?;

    let description = fields
        .get("description")
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    let parameters = fields
        .get("parameters")
        .and_then(|v| v.as_string())
        .ok_or_else(|| {
            CapabilityError::InvalidArgument("ToolDef missing 'parameters' field".to_string())
        })?;

    // Parse JSON schema
    let parameters: serde_json::Value = serde_json::from_str(parameters).map_err(|e| {
        CapabilityError::InvalidArgument(format!("Invalid JSON schema in tool '{name}': {e}"))
    })?;

    Ok(ChatCompletionTool {
        r#type: ChatCompletionToolType::Function,
        function: FunctionObject {
            name: name.to_string(),
            description: Some(description.to_string()),
            parameters: Some(parameters),
            strict: None,
        },
    })
}

/// Parse an Option field from a Value
///
/// Expected: `Some(value)` as `Value::Variant { name: "Some", fields: [("0", value)] }`
///           or `None` as `Value::Variant { name: "None", fields: [] }`
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if the value is not a Some/None variant or is malformed.
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

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Helper Functions =====

    fn create_chat_response_with_tool_calls(tool_calls: Vec<Value>) -> Value {
        let mut fields = HashMap::new();
        fields.insert("content".to_string(), Value::unit_variant("None"));
        fields.insert(
            "tool_calls".to_string(),
            Value::variant("Some", vec![("0", Value::List(Box::new(tool_calls)))]),
        );
        fields.insert(
            "finish_reason".to_string(),
            Value::variant("Some", vec![("0", Value::String("tool_calls".to_string()))]),
        );
        fields.insert("usage".to_string(), Value::unit_variant("None"));
        fields.insert("model".to_string(), Value::String("gpt-4o".to_string()));
        fields.insert("id".to_string(), Value::String("resp_123".to_string()));
        Value::Record(Box::new(fields))
    }

    fn create_tool_call_record(id: &str, name: &str, arguments: &str) -> Value {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::String(id.to_string()));
        fields.insert("name".to_string(), Value::String(name.to_string()));
        fields.insert(
            "arguments".to_string(),
            Value::String(arguments.to_string()),
        );
        Value::Record(Box::new(fields))
    }

    fn create_tool_def(name: &str, description: &str, parameters: &str) -> Value {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String(name.to_string()));
        fields.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
        fields.insert(
            "parameters".to_string(),
            Value::String(parameters.to_string()),
        );
        Value::Record(Box::new(fields))
    }

    // ===== extract_tool_calls Tests =====

    #[test]
    fn test_extract_tool_calls_valid() {
        let tool_calls = vec![
            create_tool_call_record("call_1", "get_weather", r#"{"city": "NYC"}"#),
            create_tool_call_record("call_2", "get_time", r#"{"timezone": "UTC"}"#),
        ];
        let response = create_chat_response_with_tool_calls(tool_calls);

        let result = extract_tool_calls(&response).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "call_1");
        assert_eq!(result[0].name, "get_weather");
        assert_eq!(result[0].arguments, r#"{"city": "NYC"}"#);
        assert_eq!(result[1].id, "call_2");
        assert_eq!(result[1].name, "get_time");
    }

    #[test]
    fn test_extract_tool_calls_empty_list() {
        let tool_calls: Vec<Value> = vec![];
        let mut fields = HashMap::new();
        fields.insert("content".to_string(), Value::unit_variant("None"));
        fields.insert(
            "tool_calls".to_string(),
            Value::variant("Some", vec![("0", Value::List(Box::new(tool_calls)))]),
        );
        fields.insert("finish_reason".to_string(), Value::unit_variant("None"));
        fields.insert("usage".to_string(), Value::unit_variant("None"));
        fields.insert("model".to_string(), Value::String("gpt-4o".to_string()));
        fields.insert("id".to_string(), Value::String("resp_123".to_string()));
        let response = Value::Record(Box::new(fields));

        let result = extract_tool_calls(&response).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_tool_calls_none() {
        let mut fields = HashMap::new();
        fields.insert(
            "content".to_string(),
            Value::variant("Some", vec![("0", Value::String("Hello".to_string()))]),
        );
        fields.insert("tool_calls".to_string(), Value::unit_variant("None"));
        fields.insert(
            "finish_reason".to_string(),
            Value::variant("Some", vec![("0", Value::String("stop".to_string()))]),
        );
        fields.insert("usage".to_string(), Value::unit_variant("None"));
        fields.insert("model".to_string(), Value::String("gpt-4o".to_string()));
        fields.insert("id".to_string(), Value::String("resp_123".to_string()));
        let response = Value::Record(Box::new(fields));

        let result = extract_tool_calls(&response).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_tool_calls_not_a_record() {
        let response = Value::String("not a record".to_string());
        let result = extract_tool_calls(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a Record"));
    }

    #[test]
    fn test_extract_tool_calls_missing_field() {
        let fields = HashMap::new();
        let response = Value::Record(Box::new(fields));
        let result = extract_tool_calls(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tool_calls"));
    }

    #[test]
    fn test_extract_tool_calls_malformed_tool_call() {
        let tool_calls = vec![
            create_tool_call_record("call_1", "get_weather", r#"{"city": "NYC"}"#),
            {
                let mut fields = HashMap::new();
                fields.insert("id".to_string(), Value::String("call_2".to_string()));
                // Missing name and arguments
                Value::Record(Box::new(fields))
            },
        ];
        let response = create_chat_response_with_tool_calls(tool_calls);

        let result = extract_tool_calls(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    // ===== format_tool_result_message Tests =====

    #[test]
    fn test_format_tool_result_message_valid() {
        let message = format_tool_result_message("call_123", "The weather is sunny");

        let Value::Record(fields) = &message else {
            panic!("Expected Record");
        };

        // Check role is Tool variant
        match fields.get("role").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Tool");
                assert!(fields.is_empty());
            }
            _ => panic!("Expected Tool variant for role"),
        }

        // Check content
        assert_eq!(
            fields.get("content").unwrap().as_string(),
            Some("The weather is sunny")
        );

        // Check tool_call_id is Some(call_id)
        match fields.get("tool_call_id").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                assert_eq!(fields[0].1.as_string(), Some("call_123"));
            }
            _ => panic!("Expected Some variant for tool_call_id"),
        }

        // Check tool_calls is None
        match fields.get("tool_calls").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "None");
                assert!(fields.is_empty());
            }
            _ => panic!("Expected None variant for tool_calls"),
        }
    }

    #[test]
    fn test_format_tool_result_message_empty_content() {
        let message = format_tool_result_message("call_456", "");

        let Value::Record(fields) = &message else {
            panic!("Expected Record");
        };

        assert_eq!(fields.get("content").unwrap().as_string(), Some(""));
    }

    // ===== tool_defs_to_openai_tools Tests =====

    #[test]
    fn test_tool_defs_to_openai_tools_valid() {
        let tools = vec![
            create_tool_def(
                "get_weather",
                "Get the current weather",
                r#"{"type": "object", "properties": {"city": {"type": "string"}}}"#,
            ),
            create_tool_def(
                "get_time",
                "Get the current time",
                r#"{"type": "object", "properties": {"timezone": {"type": "string"}}}"#,
            ),
        ];

        let result = tool_defs_to_openai_tools(&tools).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].function.name, "get_weather");
        assert_eq!(
            result[0].function.description,
            Some("Get the current weather".to_string())
        );
        assert!(result[0].function.parameters.is_some());
    }

    #[test]
    fn test_tool_defs_to_openai_tools_empty() {
        let tools: Vec<Value> = vec![];
        let result = tool_defs_to_openai_tools(&tools).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_tool_defs_to_openai_tools_invalid_json() {
        let tools = vec![create_tool_def("bad_tool", "Bad tool", "not valid json")];

        let result = tool_defs_to_openai_tools(&tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_tool_defs_to_openai_tools_missing_name() {
        let mut fields = HashMap::new();
        fields.insert(
            "description".to_string(),
            Value::String("Bad tool".to_string()),
        );
        fields.insert("parameters".to_string(), Value::String("{}".to_string()));
        let tools = vec![Value::Record(Box::new(fields))];

        let result = tool_defs_to_openai_tools(&tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_tool_defs_to_openai_tools_missing_parameters() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String("bad_tool".to_string()));
        fields.insert(
            "description".to_string(),
            Value::String("Bad tool".to_string()),
        );
        let tools = vec![Value::Record(Box::new(fields))];

        let result = tool_defs_to_openai_tools(&tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parameters"));
    }

    #[test]
    fn test_tool_defs_to_openai_tools_not_record() {
        let tools = vec![Value::String("not a record".to_string())];
        let result = tool_defs_to_openai_tools(&tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a Record"));
    }

    #[test]
    fn test_tool_defs_to_openai_tools_optional_description() {
        // Description is optional, should default to empty string
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            Value::String("no_desc_tool".to_string()),
        );
        fields.insert("parameters".to_string(), Value::String("{}".to_string()));
        let tools = vec![Value::Record(Box::new(fields))];

        let result = tool_defs_to_openai_tools(&tools).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].function.name, "no_desc_tool");
        assert_eq!(result[0].function.description, Some("".to_string()));
    }
}
