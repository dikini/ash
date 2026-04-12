#![allow(clippy::box_default)]

//! Chat Completion Implementation
//!
//! Implements chat completion action converting between Ash Values and async-openai types.
//!
//! # Value Contract
//!
//! ## Role (ADT Variant)
//! Roles are represented as ADT variants, not lowercase strings:
//! - `Value::Variant { name: "System", fields: [] }`
//! - `Value::Variant { name: "User", fields: [] }`
//! - `Value::Variant { name: "Assistant", fields: [] }`
//! - `Value::Variant { name: "Tool", fields: [] }`
//!
//! ## Option Fields
//! Optional fields use `Some`/`None` variants:
//! - `Some(value)` => `Value::Variant { name: "Some", fields: [("0", value)] }`
//! - `None` => `Value::Variant { name: "None", fields: [] }`
//!
//! ## Message Input Shape
//! ```text
//! {
//!   role: Role variant (System | User | Assistant | Tool),
//!   content: String,
//!   tool_calls: Option<List<ToolCall>>,
//!   tool_call_id: Option<String>
//! }
//! ```
//!
//! ## `ChatResponse` Output Shape
//! ```text
//! {
//!   content: Option<String>,
//!   tool_calls: Option<List<ToolCall>>,
//!   finish_reason: Option<String>,
//!   usage: Option<Usage>,
//!   model: String,
//!   id: String
//! }

use ash_core::Value;
use ash_core::capability::CapabilityError;
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
    ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, ChatCompletionTool,
    ChatCompletionToolType, CompletionUsage, CreateChatCompletionRequest, FunctionCall,
    FunctionObject,
};
use std::collections::HashMap;

/// Helper to get field from record value
fn get_field<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
    match value {
        Value::Record(fields) => fields.get(field),
        _ => None,
    }
}

/// Parse a Role from an ADT Variant value
///
/// Expected: `Value::Variant { name: "System" | "User" | "Assistant" | "Tool", fields: [] }`
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if the value is not a variant, has unexpected fields, or is not a recognized role.
fn parse_role(value: &Value) -> Result<&str, CapabilityError> {
    match value {
        Value::Variant { name, fields } => {
            if !fields.is_empty() {
                return Err(CapabilityError::InvalidArgument(format!(
                    "Role variant '{name}' should have no fields"
                )));
            }
            match name.as_str() {
                "System" => Ok("system"),
                "User" => Ok("user"),
                "Assistant" => Ok("assistant"),
                "Tool" => Ok("tool"),
                _ => Err(CapabilityError::InvalidArgument(format!(
                    "Unknown role variant: {name}. Expected System, User, Assistant, or Tool"
                ))),
            }
        }
        _ => Err(CapabilityError::InvalidArgument(
            "Role must be an ADT variant (System, User, Assistant, Tool), not a string".to_string(),
        )),
    }
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

/// Convert an Option<Value> to a Value using ADT variants
///
/// - `Some(x)` => `Value::Variant { name: "Some", fields: [("0", x)] }`
/// - `None` => `Value::Variant { name: "None", fields: [] }`
fn option_to_value(opt: Option<Value>) -> Value {
    opt.map_or_else(
        || Value::unit_variant("None"),
        |v| Value::variant("Some", vec![("0", v)]),
    )
}

/// Convert an Ash Value message to async-openai `ChatCompletionRequestMessage`
///
/// Expected Value shape (Message record with Role variant):
/// ```text
/// {
///   role: Role variant (System | User | Assistant | Tool),
///   content: String,
///   tool_calls: Option<List<ToolCall>>,
///   tool_call_id: Option<String>
/// }
/// ```
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if the value is not a Record, lacks required fields, or has an unknown role.
pub fn value_to_chat_message(
    value: &Value,
) -> Result<ChatCompletionRequestMessage, CapabilityError> {
    // Validate that value is a Record
    let Value::Record(_fields) = value else {
        return Err(CapabilityError::InvalidArgument(
            "Message must be a Record value".to_string(),
        ));
    };

    let role_value = get_field(value, "role").ok_or_else(|| {
        CapabilityError::InvalidArgument("Message missing 'role' field".to_string())
    })?;

    let role = parse_role(role_value)?;

    let content = get_field(value, "content")
        .and_then(|v| v.as_string())
        .unwrap_or_default()
        .to_string();

    match role {
        "system" => {
            let msg = ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(content),
                name: None,
            };
            Ok(ChatCompletionRequestMessage::System(msg))
        }
        "user" => {
            let msg = ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(content),
                name: None,
            };
            Ok(ChatCompletionRequestMessage::User(msg))
        }
        "assistant" => {
            // Check for tool_calls (Option field)
            let tool_calls = get_field(value, "tool_calls")
                .and_then(|v| parse_option_field(v).ok()?)
                .and_then(|opt_v| {
                    if let Value::List(calls) = opt_v {
                        let parsed: Result<Vec<_>, _> =
                            calls.iter().map(parse_tool_call_from_value).collect();
                        parsed.ok()
                    } else {
                        None
                    }
                });

            let msg = ChatCompletionRequestAssistantMessage {
                content: if content.is_empty() && tool_calls.is_some() {
                    None
                } else {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(content))
                },
                refusal: None,
                name: None,
                audio: None,
                tool_calls,
                ..Default::default()
            };
            Ok(ChatCompletionRequestMessage::Assistant(msg))
        }
        "tool" => {
            let tool_call_id = get_field(value, "tool_call_id")
                .and_then(|v| parse_option_field(v).ok()?)
                .and_then(|v| v.as_string())
                .ok_or_else(|| {
                    CapabilityError::InvalidArgument(
                        "Tool message missing 'tool_call_id' field".to_string(),
                    )
                })?;

            let msg = ChatCompletionRequestToolMessage {
                content: ChatCompletionRequestToolMessageContent::Text(content),
                tool_call_id: tool_call_id.to_string(),
            };
            Ok(ChatCompletionRequestMessage::Tool(msg))
        }
        _ => Err(CapabilityError::InvalidArgument(format!(
            "Unknown role: {role}"
        ))),
    }
}

/// Parse a `ToolCall` from a Value
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if the value is not a Record or lacks required fields (id, name, arguments).
fn parse_tool_call_from_value(
    value: &Value,
) -> Result<ChatCompletionMessageToolCall, CapabilityError> {
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

    Ok(ChatCompletionMessageToolCall {
        id,
        r#type: ChatCompletionToolType::Function,
        function: FunctionCall { name, arguments },
    })
}

/// Convert a list of Ash Value messages to async-openai messages
///
/// # Errors
/// Returns an error if any individual message conversion fails (see `value_to_chat_message`).
pub fn values_to_chat_messages(
    values: &[Value],
) -> Result<Vec<ChatCompletionRequestMessage>, CapabilityError> {
    values.iter().map(value_to_chat_message).collect()
}

/// Convert a list of Ash Value `ToolDefs` to async-openai `ChatCompletionTools`
///
/// Expected `ToolDef` shape:
/// ```text
/// {
///   name: String,
///   description: String,
///   parameters: String (JSON schema)
/// }
/// ```
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if a `ToolDef` is not a Record, lacks required fields, or has invalid JSON in parameters.
pub fn values_to_chat_tools(values: &[Value]) -> Result<Vec<ChatCompletionTool>, CapabilityError> {
    values
        .iter()
        .map(|v| {
            let Value::Record(fields) = v else {
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
                    CapabilityError::InvalidArgument(
                        "ToolDef missing 'parameters' field".to_string(),
                    )
                })?;

            // Parse JSON schema
            let parameters: serde_json::Value = serde_json::from_str(parameters).map_err(|e| {
                CapabilityError::InvalidArgument(format!(
                    "Invalid JSON schema in tool '{name}': {e}"
                ))
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
        })
        .collect()
}

/// Parse float from Value (handles both Int and String representations)
#[allow(clippy::cast_precision_loss)]
fn value_to_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Int(i) => Some(*i as f32),
        Value::String(s) => s.parse::<f32>().ok(),
        _ => None,
    }
}

/// Parse integer from Value
fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Int(i) => Some(*i),
        Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    }
}

/// Validate chat completion parameters
///
/// Returns an error if any parameter is out of valid range.
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if any parameter (temperature, `top_p`, `max_tokens`, stop, seed) is out of valid range or malformed.
fn validate_params(params: &HashMap<String, Value>) -> Result<(), CapabilityError> {
    for (key, value) in params {
        match key.as_str() {
            "temperature" => {
                if let Some(t) = value_to_f32(value) {
                    if !(0.0..=2.0).contains(&t) {
                        return Err(CapabilityError::InvalidArgument(format!(
                            "temperature must be between 0.0 and 2.0, got {t}"
                        )));
                    }
                }
            }
            "top_p" => {
                if let Some(t) = value_to_f32(value) {
                    if t <= 0.0 || t > 1.0 {
                        return Err(CapabilityError::InvalidArgument(format!(
                            "top_p must be in range (0.0, 1.0], got {t}"
                        )));
                    }
                }
            }
            "max_tokens" => {
                if let Some(m) = value_to_i64(value) {
                    if m <= 0 {
                        return Err(CapabilityError::InvalidArgument(format!(
                            "max_tokens must be positive, got {m}"
                        )));
                    }
                }
            }
            "stop" => {
                // Validate stop sequences
                match value {
                    Value::String(_) => {} // Single string is valid
                    Value::List(items) => {
                        if items.len() > 4 {
                            return Err(CapabilityError::InvalidArgument(format!(
                                "stop can have at most 4 sequences, got {}",
                                items.len()
                            )));
                        }
                        // Ensure all items are strings
                        for (i, item) in items.iter().enumerate() {
                            if item.as_string().is_none() {
                                return Err(CapabilityError::InvalidArgument(format!(
                                    "stop sequence at index {i} must be a string"
                                )));
                            }
                        }
                    }
                    _ => {
                        return Err(CapabilityError::InvalidArgument(
                            "stop must be a string or list of strings".to_string(),
                        ));
                    }
                }
            }
            "seed" => {
                if value.as_int().is_none() && value.as_string().is_none() {
                    return Err(CapabilityError::InvalidArgument(
                        "seed must be an integer".to_string(),
                    ));
                }
            }
            _ => {} // Unknown params are ignored
        }
    }
    Ok(())
}

/// Build `CreateChatCompletionRequest` from action arguments
///
/// Expected args for chat:
/// - args[0]: provider (String) - provider name for routing
/// - args[1]: model (String) - model identifier  
/// - args[2]: messages (List<Value>) - conversation messages
/// - args[3]: params (Option<Value>) - completion parameters (temperature, `top_p`, etc.)
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if params fail validation or are malformed.
pub fn build_chat_request(
    model: &str,
    messages: Vec<ChatCompletionRequestMessage>,
    params: Option<&Value>,
) -> Result<CreateChatCompletionRequest, CapabilityError> {
    let mut request = CreateChatCompletionRequest {
        model: model.to_string(),
        messages,
        ..Default::default()
    };

    // Apply params if provided
    if let Some(p) = params {
        let params_opt = parse_option_field(p)?;
        if let Some(param_value) = params_opt {
            if let Value::Record(fields) = param_value {
                // Validate parameters first
                validate_params(fields)?;

                for (key, value) in fields.iter() {
                    match key.as_str() {
                        "temperature" => {
                            if let Some(t) = value_to_f32(value) {
                                request.temperature = Some(t);
                            }
                        }
                        "top_p" => {
                            if let Some(t) = value_to_f32(value) {
                                request.top_p = Some(t);
                            }
                        }
                        "max_tokens" => {
                            if let Some(m) = value.as_int() {
                                let Ok(m) = u32::try_from(m) else {
                                    return Err(CapabilityError::InvalidArgument(format!(
                                        "max_tokens must be in range 1..={}",
                                        u32::MAX
                                    )));
                                };
                                request.max_completion_tokens = Some(m);
                            }
                        }
                        "stop" => {
                            if let Some(s) = value.as_string() {
                                request.stop =
                                    Some(async_openai::types::Stop::String(s.to_string()));
                            } else if let Value::List(stops) = value {
                                let stop_strings: Vec<String> = stops
                                    .iter()
                                    .filter_map(|v| {
                                        v.as_string().map(std::string::ToString::to_string)
                                    })
                                    .collect();
                                if !stop_strings.is_empty() {
                                    request.stop =
                                        Some(async_openai::types::Stop::StringArray(stop_strings));
                                }
                            }
                        }
                        "seed" => {
                            if let Some(s) = value.as_int() {
                                request.seed = Some(s);
                            }
                        }
                        _ => {} // Ignore unknown params
                    }
                }
            }
        }
    }

    Ok(request)
}

/// Build `CreateChatCompletionRequest` with tools from action arguments
///
/// Expected args for `chat_with_tools`:
/// - args[0]: provider (String) - provider name for routing
/// - args[1]: model (String) - model identifier  
/// - args[2]: messages (List<Value>) - conversation messages
/// - args[3]: tools (List<ToolDef>) - tool definitions
/// - args[4]: params (Option<Value>) - completion parameters
///
/// # Errors
/// Returns an error if the underlying `build_chat_request` call fails.
pub fn build_chat_request_with_tools(
    model: &str,
    messages: Vec<ChatCompletionRequestMessage>,
    tools: Vec<ChatCompletionTool>,
    params: Option<&Value>,
) -> Result<CreateChatCompletionRequest, CapabilityError> {
    let mut request = build_chat_request(model, messages, params)?;
    request.tools = Some(tools);
    Ok(request)
}

/// Convert `ChatCompletionResponse` to Ash Value
///
/// Output shape:
/// ```text
/// {
///   content: Option<String>,
///   tool_calls: Option<List<ToolCall>>,
///   finish_reason: Option<String>,
///   usage: Option<Usage>,
///   model: String,
///   id: String
/// }
/// ```
///
/// # Errors
/// Returns `CapabilityError::ExecutionFailed` if the response contains no choices.
pub fn chat_response_to_value(
    response: async_openai::types::CreateChatCompletionResponse,
) -> Result<Value, CapabilityError> {
    let choice = response.choices.into_iter().next().ok_or_else(|| {
        CapabilityError::ExecutionFailed("No choices in chat response".to_string())
    })?;

    let message = choice.message;

    // Extract content - use None if empty and there are tool calls
    let content = message.content.filter(|c| !c.is_empty());
    let content_value = option_to_value(content.map(Value::String));

    // Extract tool_calls if present
    let tool_calls_value = option_to_value(message.tool_calls.map(|calls| {
        let tc_values: Vec<Value> = calls
            .into_iter()
            .map(|tc| {
                let mut fields = HashMap::new();
                fields.insert("id".to_string(), Value::String(tc.id));
                fields.insert("name".to_string(), Value::String(tc.function.name));
                fields.insert(
                    "arguments".to_string(),
                    Value::String(tc.function.arguments),
                );
                Value::Record(Box::new(fields))
            })
            .collect();
        Value::List(Box::new(tc_values))
    }));

    // Extract finish_reason
    let finish_reason_value = option_to_value(choice.finish_reason.map(|fr| {
        let reason_str = match fr {
            async_openai::types::FinishReason::Stop => "stop",
            async_openai::types::FinishReason::Length => "length",
            async_openai::types::FinishReason::ToolCalls => "tool_calls",
            async_openai::types::FinishReason::ContentFilter => "content_filter",
            async_openai::types::FinishReason::FunctionCall => "function_call",
        };
        Value::String(reason_str.to_string())
    }));

    // Build response record
    let mut fields = HashMap::new();
    fields.insert("content".to_string(), content_value);
    fields.insert("tool_calls".to_string(), tool_calls_value);
    fields.insert("finish_reason".to_string(), finish_reason_value);
    fields.insert("usage".to_string(), usage_to_value(response.usage.as_ref()));
    fields.insert("model".to_string(), Value::String(response.model));
    fields.insert("id".to_string(), Value::String(response.id));

    Ok(Value::Record(Box::new(fields)))
}

/// Convert `CompletionUsage` to Ash Value (Option<Usage>)
fn usage_to_value(usage: Option<&CompletionUsage>) -> Value {
    option_to_value(usage.map(|u| {
        let mut fields = HashMap::new();
        fields.insert(
            "prompt_tokens".to_string(),
            Value::Int(i64::from(u.prompt_tokens)),
        );
        fields.insert(
            "completion_tokens".to_string(),
            Value::Int(i64::from(u.completion_tokens)),
        );
        fields.insert(
            "total_tokens".to_string(),
            Value::Int(i64::from(u.total_tokens)),
        );
        Value::Record(Box::new(fields))
    }))
}

/// Chat action extraction result: (provider, model, messages, params)
type ChatArgsResult<'a> =
    Result<(&'a str, &'a str, &'a [Value], Option<&'a Value>), CapabilityError>;

/// Chat-with-tools extraction result: (provider, model, messages, tools, params)
type ChatWithToolsArgsResult<'a> = Result<
    (
        &'a str,
        &'a str,
        &'a [Value],
        &'a [Value],
        Option<&'a Value>,
    ),
    CapabilityError,
>;

/// Extract chat action arguments
///
/// Returns: (provider, model, messages, params)
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if args are missing or have wrong types.
pub fn extract_chat_args(args: &[Value]) -> ChatArgsResult<'_> {
    if args.len() < 3 {
        return Err(CapabilityError::InvalidArgument(
            "chat requires provider, model, and messages args".to_string(),
        ));
    }

    let provider = args[0]
        .as_string()
        .ok_or_else(|| CapabilityError::InvalidArgument("provider must be a string".to_string()))?;

    let model = args[1]
        .as_string()
        .ok_or_else(|| CapabilityError::InvalidArgument("model must be a string".to_string()))?;

    let messages = match &args[2] {
        Value::List(m) => m.as_slice(),
        _ => {
            return Err(CapabilityError::InvalidArgument(
                "messages must be a list".to_string(),
            ));
        }
    };

    let params = args.get(3);

    Ok((provider, model, messages, params))
}

/// Extract `chat_with_tools` action arguments
///
/// Expected: [provider, model, messages, tools, params]
/// Returns: (provider, model, messages, tools, params)
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if args are missing or have wrong types.
pub fn extract_chat_with_tools_args(args: &[Value]) -> ChatWithToolsArgsResult<'_> {
    if args.len() < 4 {
        return Err(CapabilityError::InvalidArgument(
            "chat_with_tools requires provider, model, messages, and tools args".to_string(),
        ));
    }

    let provider = args[0]
        .as_string()
        .ok_or_else(|| CapabilityError::InvalidArgument("provider must be a string".to_string()))?;

    let model = args[1]
        .as_string()
        .ok_or_else(|| CapabilityError::InvalidArgument("model must be a string".to_string()))?;

    let messages = match &args[2] {
        Value::List(m) => m.as_slice(),
        _ => {
            return Err(CapabilityError::InvalidArgument(
                "messages must be a list".to_string(),
            ));
        }
    };

    let tools = match &args[3] {
        Value::List(t) => t.as_slice(),
        _ => {
            return Err(CapabilityError::InvalidArgument(
                "tools must be a list".to_string(),
            ));
        }
    };

    let params = args.get(4);

    Ok((provider, model, messages, tools, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create Role variants
    fn role_variant(name: &str) -> Value {
        Value::unit_variant(name)
    }

    fn create_test_message_with_role_variant(role: &str, content: &str) -> Value {
        let mut fields = HashMap::new();
        fields.insert("role".to_string(), role_variant(role));
        fields.insert("content".to_string(), Value::String(content.to_string()));
        Value::Record(Box::new(fields))
    }

    fn create_test_message(role: &str, content: &str) -> Value {
        create_test_message_with_role_variant(role, content)
    }

    // ===== Role Variant Parsing Tests =====

    #[test]
    fn test_parse_role_system() {
        let role = role_variant("System");
        assert_eq!(parse_role(&role).unwrap(), "system");
    }

    #[test]
    fn test_parse_role_user() {
        let role = role_variant("User");
        assert_eq!(parse_role(&role).unwrap(), "user");
    }

    #[test]
    fn test_parse_role_assistant() {
        let role = role_variant("Assistant");
        assert_eq!(parse_role(&role).unwrap(), "assistant");
    }

    #[test]
    fn test_parse_role_tool() {
        let role = role_variant("Tool");
        assert_eq!(parse_role(&role).unwrap(), "tool");
    }

    #[test]
    fn test_parse_role_rejects_lowercase_string() {
        let role = Value::String("system".to_string());
        let result = parse_role(&role);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be an ADT variant")
        );
    }

    #[test]
    fn test_parse_role_rejects_unknown_variant() {
        let role = role_variant("Unknown");
        let result = parse_role(&role);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown role variant")
        );
    }

    // ===== Option Field Parsing Tests =====

    #[test]
    fn test_parse_option_field_some() {
        let some_val = Value::variant("Some", vec![("0", Value::String("hello".to_string()))]);
        let result = parse_option_field(&some_val).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_string(), Some("hello"));
    }

    #[test]
    fn test_parse_option_field_none() {
        let none_val = Value::unit_variant("None");
        let result = parse_option_field(&none_val).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_option_field_null() {
        let result = parse_option_field(&Value::Null).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_option_field_rejects_invalid() {
        let invalid = Value::String("not an option".to_string());
        let result = parse_option_field(&invalid);
        assert!(result.is_err());
    }

    // ===== Option to Value Conversion Tests =====

    #[test]
    fn test_option_to_value_some() {
        let val = Value::String("test".to_string());
        let result = option_to_value(Some(val));
        match result {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                assert!(!fields.is_empty());
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_option_to_value_none() {
        let result = option_to_value(None);
        match result {
            Value::Variant { name, fields } => {
                assert_eq!(name, "None");
                assert!(fields.is_empty());
            }
            _ => panic!("Expected None variant"),
        }
    }

    // ===== Message Conversion Tests =====

    #[test]
    fn test_value_to_chat_message_system() {
        let msg = create_test_message("System", "You are a helpful assistant.");
        let result = value_to_chat_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_to_chat_message_user() {
        let msg = create_test_message("User", "Hello!");
        let result = value_to_chat_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_to_chat_message_assistant() {
        let msg = create_test_message("Assistant", "Hi there!");
        let result = value_to_chat_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_to_chat_message_tool() {
        let mut fields = HashMap::new();
        fields.insert("role".to_string(), role_variant("Tool"));
        fields.insert(
            "content".to_string(),
            Value::String("Tool result".to_string()),
        );
        fields.insert(
            "tool_call_id".to_string(),
            Value::variant("Some", vec![("0", Value::String("call_123".to_string()))]),
        );
        let msg = Value::Record(Box::new(fields));

        let result = value_to_chat_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_to_chat_message_rejects_string_role() {
        let mut fields = HashMap::new();
        fields.insert("role".to_string(), Value::String("system".to_string()));
        fields.insert("content".to_string(), Value::String("test".to_string()));
        let msg = Value::Record(Box::new(fields));

        let result = value_to_chat_message(&msg);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be an ADT variant")
        );
    }

    #[test]
    fn test_value_to_chat_message_missing_role() {
        let fields = HashMap::new();
        let msg = Value::Record(Box::new(fields));

        let result = value_to_chat_message(&msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("role"));
    }

    #[test]
    fn test_value_to_chat_message_non_record_rejected() {
        let msg = Value::String("not a record".to_string());
        let result = value_to_chat_message(&msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a Record"));
    }

    // ===== Tool Call Parsing Tests =====

    #[test]
    fn test_parse_tool_call_valid() {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::String("call_123".to_string()));
        fields.insert("name".to_string(), Value::String("get_weather".to_string()));
        fields.insert(
            "arguments".to_string(),
            Value::String("{\"city\": \"NYC\"}".to_string()),
        );
        let tool_call = Value::Record(Box::new(fields));

        let result = parse_tool_call_from_value(&tool_call);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "call_123");
        assert_eq!(parsed.function.name, "get_weather");
    }

    #[test]
    fn test_parse_tool_call_missing_field() {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::String("call_123".to_string()));
        // Missing name and arguments
        let tool_call = Value::Record(Box::new(fields));

        let result = parse_tool_call_from_value(&tool_call);
        assert!(result.is_err());
    }

    // ===== Multiple Messages Tests =====

    #[test]
    fn test_values_to_chat_messages() {
        let msgs = vec![
            create_test_message("System", "Be helpful."),
            create_test_message("User", "Hello!"),
        ];
        let result = values_to_chat_messages(&msgs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    // ===== ToolDef Conversion Tests =====

    #[test]
    fn test_values_to_chat_tools_valid() {
        let mut tool_def = HashMap::new();
        tool_def.insert("name".to_string(), Value::String("get_weather".to_string()));
        tool_def.insert(
            "description".to_string(),
            Value::String("Get the weather".to_string()),
        );
        tool_def.insert(
            "parameters".to_string(),
            Value::String(r#"{"type": "object", "properties": {}}"#.to_string()),
        );

        let tools = vec![Value::Record(Box::new(tool_def))];
        let result = values_to_chat_tools(&tools);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_values_to_chat_tools_invalid_json() {
        let mut tool_def = HashMap::new();
        tool_def.insert("name".to_string(), Value::String("bad_tool".to_string()));
        tool_def.insert(
            "description".to_string(),
            Value::String("Bad tool".to_string()),
        );
        tool_def.insert(
            "parameters".to_string(),
            Value::String("not valid json".to_string()),
        );

        let tools = vec![Value::Record(Box::new(tool_def))];
        let result = values_to_chat_tools(&tools);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid JSON schema")
        );
    }

    #[test]
    fn test_values_to_chat_tools_missing_name() {
        let mut tool_def = HashMap::new();
        tool_def.insert(
            "description".to_string(),
            Value::String("Bad tool".to_string()),
        );
        tool_def.insert("parameters".to_string(), Value::String("{}".to_string()));

        let tools = vec![Value::Record(Box::new(tool_def))];
        let result = values_to_chat_tools(&tools);
        assert!(result.is_err());
    }

    #[test]
    fn test_values_to_chat_tools_non_record_rejected() {
        let tools = vec![Value::String("not a record".to_string())];
        let result = values_to_chat_tools(&tools);
        assert!(result.is_err());
    }

    // ===== Parameter Validation Tests =====

    #[test]
    fn test_validate_params_temperature_valid() {
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), Value::String("0.7".to_string()));
        assert!(validate_params(&params).is_ok());
    }

    #[test]
    fn test_validate_params_temperature_too_high() {
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), Value::String("3.0".to_string()));
        let result = validate_params(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("temperature"));
    }

    #[test]
    fn test_validate_params_temperature_negative() {
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), Value::String("-0.1".to_string()));
        let result = validate_params(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_params_top_p_invalid() {
        let mut params = HashMap::new();
        params.insert("top_p".to_string(), Value::String("1.5".to_string()));
        let result = validate_params(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("top_p"));
    }

    #[test]
    fn test_validate_params_max_tokens_zero() {
        let mut params = HashMap::new();
        params.insert("max_tokens".to_string(), Value::Int(0));
        let result = validate_params(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_tokens"));
    }

    #[test]
    fn test_validate_params_stop_too_many() {
        let stops: Vec<Value> = (0..5).map(|i| Value::String(format!("stop{i}"))).collect();
        let mut params = HashMap::new();
        params.insert("stop".to_string(), Value::List(Box::new(stops)));
        let result = validate_params(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at most 4"));
    }

    #[test]
    fn test_validate_params_stop_valid() {
        let stops: Vec<Value> = vec![
            Value::String("stop1".to_string()),
            Value::String("stop2".to_string()),
        ];
        let mut params = HashMap::new();
        params.insert("stop".to_string(), Value::List(Box::new(stops)));
        assert!(validate_params(&params).is_ok());
    }

    // ===== Chat Args Extraction Tests =====

    #[test]
    fn test_extract_chat_args_valid() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::List(Box::new(vec![create_test_message("User", "Hi")])),
        ];
        let result = extract_chat_args(&args);
        assert!(result.is_ok());
        let (provider, model, messages, params) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4o");
        assert_eq!(messages.len(), 1);
        assert!(params.is_none());
    }

    #[test]
    fn test_extract_chat_args_with_params() {
        let params = Value::variant("Some", vec![("0", Value::Record(Box::new(HashMap::new())))]);
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::List(Box::new(vec![create_test_message("User", "Hi")])),
            params,
        ];
        let result = extract_chat_args(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().3.is_some());
    }

    #[test]
    fn test_extract_chat_args_too_few() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
        ];
        let result = extract_chat_args(&args);
        assert!(result.is_err());
    }

    // ===== Chat With Tools Args Extraction Tests =====

    #[test]
    fn test_extract_chat_with_tools_args_valid() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::List(Box::new(vec![create_test_message("User", "Hi")])),
            Value::List(Box::new(vec![])),
        ];
        let result = extract_chat_with_tools_args(&args);
        assert!(result.is_ok());
        let (provider, model, messages, tools, params) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4o");
        assert_eq!(messages.len(), 1);
        assert!(tools.is_empty());
        assert!(params.is_none());
    }

    #[test]
    fn test_extract_chat_with_tools_args_too_few() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::List(Box::new(vec![])),
        ];
        let result = extract_chat_with_tools_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tools"));
    }

    #[test]
    fn test_extract_chat_with_tools_args_invalid_tools_type() {
        let args = vec![
            Value::String("openai".to_string()),
            Value::String("gpt-4o".to_string()),
            Value::List(Box::new(vec![])),
            Value::String("not a list".to_string()),
        ];
        let result = extract_chat_with_tools_args(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("tools must be a list")
        );
    }

    #[test]
    fn test_chat_response_to_value_normal_text() {
        use async_openai::types::{ChatChoice, CreateChatCompletionResponse, FinishReason};

        let response = CreateChatCompletionResponse {
            id: "resp_123".to_string(),
            object: "chat.completion".to_string(),
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: {
                    #[allow(deprecated)]
                    async_openai::types::ChatCompletionResponseMessage {
                        role: async_openai::types::Role::Assistant,
                        content: Some("Hello!".to_string()),
                        refusal: None,
                        audio: None,
                        tool_calls: None,
                        function_call: None,
                    }
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: Some(CompletionUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
            service_tier: None,
        };

        let result = chat_response_to_value(response);
        assert!(result.is_ok());

        let value = result.unwrap();
        let Value::Record(fields) = &value else {
            panic!("Expected Record");
        };

        // Check content is Some("Hello!")
        match fields.get("content").unwrap() {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                assert_eq!(fields[0].1.as_string(), Some("Hello!"));
            }
            _ => panic!("Expected Some variant for content"),
        }

        // Check model and id
        assert_eq!(fields.get("model").unwrap().as_string(), Some("gpt-4o"));
        assert_eq!(fields.get("id").unwrap().as_string(), Some("resp_123"));
    }

    #[test]
    fn test_chat_response_to_value_no_choices() {
        use async_openai::types::CreateChatCompletionResponse;

        let response = CreateChatCompletionResponse {
            id: "resp_123".to_string(),
            object: "chat.completion".to_string(),
            created: 1_234_567_890,
            model: "gpt-4o".to_string(),
            choices: vec![],
            usage: None,
            system_fingerprint: None,
            service_tier: None,
        };

        let result = chat_response_to_value(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No choices"));
    }
}
