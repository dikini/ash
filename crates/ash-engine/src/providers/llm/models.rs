//! Model Listing Implementation
//!
//! Handles listing available models from LLM providers.

use ash_core::Value;
use ash_core::capability::CapabilityError;
use async_openai::Client;
use async_openai::config::OpenAIConfig;

/// Extract and validate the provider argument for `list_models`
///
/// # Arguments
/// * `args` - Action arguments, expects [`provider_name`: String]
///
/// # Errors
/// Returns `CapabilityError::InvalidArgument` if:
/// - args is empty
/// - args[0] is not a String variant
/// - provider string is empty
pub fn extract_list_models_arg(args: &[Value]) -> Result<&str, CapabilityError> {
    if args.is_empty() {
        return Err(CapabilityError::InvalidArgument(
            "list_models requires provider name".to_string(),
        ));
    }

    // Check that args[0] IS a String, not just convertible
    let provider = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(CapabilityError::InvalidArgument(
                "provider must be a string".to_string(),
            ));
        }
    };

    if provider.is_empty() {
        return Err(CapabilityError::InvalidArgument(
            "provider name cannot be empty".to_string(),
        ));
    }

    Ok(provider)
}

/// List available models from a provider
///
/// # Arguments
/// * `client` - async-openai client for the provider
///
/// # Returns
/// `Value::List` of model ID strings
///
/// # Errors
/// Returns `CapabilityError` on API or network failures.
pub async fn list_models(client: &Client<OpenAIConfig>) -> Result<Value, CapabilityError> {
    let response = client
        .models()
        .list()
        .await
        .map_err(|e| super::error::map_openai_error(e, "models"))?;

    let model_ids: Vec<Value> = response
        .data
        .into_iter()
        .map(|m| Value::String(m.id))
        .collect();

    Ok(Value::List(Box::new(model_ids)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_list_models_arg_valid() {
        let args = vec![Value::String("openai".to_string())];
        let result = extract_list_models_arg(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "openai");
    }

    #[test]
    fn test_extract_list_models_missing_arg() {
        let args: Vec<Value> = vec![];
        let result = extract_list_models_arg(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("requires provider name"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_list_models_wrong_arg_type() {
        // Test with Int - should fail with InvalidArgument
        let args = vec![Value::Int(42)];
        let result = extract_list_models_arg(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("must be a string"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_extract_list_models_empty_provider() {
        let args = vec![Value::String(String::new())];
        let result = extract_list_models_arg(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("cannot be empty"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }
}
