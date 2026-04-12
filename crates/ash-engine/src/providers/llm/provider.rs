//! LLM Provider Implementation
//!
//! Provides OpenAI-compatible LLM capabilities through the `async-openai` crate.
//! Supports multi-provider routing, chat completions, streaming, embeddings, and tool use.

use crate::providers::llm::chat::{
    build_chat_request, build_chat_request_with_tools, chat_response_to_value, extract_chat_args,
    extract_chat_with_tools_args, values_to_chat_messages, values_to_chat_tools,
};
use crate::providers::llm::config::LlmConfig;
use crate::providers::llm::embeddings::{
    build_embed_request, embed_response_to_value, extract_embed_args, texts_to_strings,
};
use crate::providers::llm::error;
use crate::providers::llm::models::{extract_list_models_arg, list_models};
use crate::providers::llm::stream_adapter::extract_chat_stream_args;
use crate::providers::llm::stream_storage::{spawn_stream_forwarder, StreamStorage};
use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::{Constraint, Effect, Value};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

/// LLM Capability Provider
///
/// Provides OpenAI-compatible LLM operations through the `CapabilityProvider` trait.
/// Supports multiple provider configurations (e.g., `OpenAI`, Ollama) with routing by name.
#[derive(Debug)]
pub struct LlmProvider {
    /// Provider configurations keyed by name (e.g., "openai", "ollama", "local")
    configs: HashMap<String, LlmConfig>,
    /// Lazily-created async-openai clients keyed by provider name
    clients: Mutex<HashMap<String, Client<OpenAIConfig>>>,
    /// Storage for active streams (for chat_stream action)
    stream_storage: StreamStorage,
}

impl LlmProvider {
    /// Create a new LLM provider with the given configurations
    ///
    /// # Arguments
    /// * `configs` - Map of provider name to configuration
    ///
    /// # Errors
    /// Returns `CapabilityError` if any config fails validation
    pub fn new(configs: HashMap<String, LlmConfig>) -> Result<Self, CapabilityError> {
        // Validate all configs
        for (name, config) in &configs {
            // Use validate_for_local if api_key is empty (local provider)
            let validation = if config.api_key.is_empty() {
                config.validate_for_local()
            } else {
                config.validate()
            };

            validation.map_err(|e| {
                CapabilityError::ValidationFailed(format!(
                    "Invalid config for provider '{name}': {e}"
                ))
            })?;
        }

        Ok(Self {
            configs,
            clients: Mutex::new(HashMap::new()),
            stream_storage: StreamStorage::new(),
        })
    }

    /// Get or create an async-openai client for the named provider
    ///
    /// # Arguments
    /// * `provider_name` - Name of the provider configuration to use
    ///
    /// # Errors
    /// Returns `CapabilityError` if the provider name is unknown
    fn get_client(&self, provider_name: &str) -> Result<Client<OpenAIConfig>, CapabilityError> {
        // Fast path: check if client already exists
        {
            let clients = self.clients.lock().map_err(|_| {
                CapabilityError::ExecutionFailed("Failed to lock clients mutex".to_string())
            })?;
            if let Some(client) = clients.get(provider_name) {
                return Ok(client.clone());
            }
        }

        // Slow path: create client
        let config = self
            .configs
            .get(provider_name)
            .ok_or_else(|| CapabilityError::NotAvailable(format!("provider:{provider_name}")))?;

        let openai_config = OpenAIConfig::new()
            .with_api_base(&config.api_base)
            .with_api_key(&config.api_key);

        let client = Client::with_config(openai_config);

        // Store client for reuse
        {
            let mut clients = self.clients.lock().map_err(|_| {
                CapabilityError::ExecutionFailed("Failed to lock clients mutex".to_string())
            })?;
            clients.insert(provider_name.to_string(), client.clone());
        }

        Ok(client)
    }

    /// Get a reference to the internal stream storage.
    ///
    /// This is exposed for integration testing: tests can inject chunks directly
    /// into the stream storage to verify error propagation without requiring a
    /// real streaming connection.
    #[doc(hidden)]
    pub const fn stream_storage(&self) -> &StreamStorage {
        &self.stream_storage
    }

    /// List available models from a provider
    ///
    /// # Arguments
    /// * `provider_name` - Name of the provider to query
    ///
    /// # Errors
    /// Returns `CapabilityError` if the provider is unknown or the API call fails.
    async fn list_models_for_provider(
        &self,
        provider_name: &str,
    ) -> Result<Value, CapabilityError> {
        let client = self.get_client(provider_name)?;
        list_models(&client).await
    }
}

#[async_trait]
impl CapabilityProvider for LlmProvider {
    fn name(&self) -> &'static str {
        "llm"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        // Observe is unused for LLM action surface in Phase 77
        Err(CapabilityError::NotAvailable(
            "LLM provider does not support observe".to_string(),
        ))
    }

    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        match action_name {
            "list_models" => {
                let provider_name = extract_list_models_arg(args)?;
                self.list_models_for_provider(provider_name).await
            }
            "chat" => {
                // Extract and validate arguments
                let (provider_name, model, messages, params) = extract_chat_args(args)?;

                // Convert messages
                let chat_messages = values_to_chat_messages(messages)?;

                // Build request
                let request = build_chat_request(model, chat_messages, params)?;

                // Get client and execute
                let client = self.get_client(provider_name)?;
                let response = client
                    .chat()
                    .create(request)
                    .await
                    .map_err(|e| error::map_openai_error(e, provider_name))?;

                // Convert response to Value
                chat_response_to_value(response)
            }
            "chat_with_tools" => {
                // Extract and validate arguments: [provider, model, messages, tools, params]
                let (provider_name, model, messages, tools, params) =
                    extract_chat_with_tools_args(args)?;

                // Convert messages
                let chat_messages = values_to_chat_messages(messages)?;

                // Convert tool definitions
                let chat_tools = values_to_chat_tools(tools)?;

                // Build request with tools
                let request =
                    build_chat_request_with_tools(model, chat_messages, chat_tools, params)?;

                // Get client and execute
                let client = self.get_client(provider_name)?;
                let response = client
                    .chat()
                    .create(request)
                    .await
                    .map_err(|e| error::map_openai_error(e, provider_name))?;

                // Convert response to Value
                chat_response_to_value(response)
            }
            "chat_stream" => {
                // Extract and validate arguments: [provider, model, messages, params]
                let (provider_name, model, messages, params) = extract_chat_stream_args(args)?;

                // Convert messages
                let chat_messages = values_to_chat_messages(messages)?;

                // Build request with streaming enabled
                let mut request = build_chat_request(model, chat_messages, params)?;
                request.stream = Some(true);

                // Get client
                let client = self.get_client(provider_name)?;

                // Initiate streaming request
                let stream = client
                    .chat()
                    .create_stream(request)
                    .await
                    .map_err(|e| error::map_openai_error(e, provider_name))?;

                // Create stream ID
                let stream_id = format!(
                    "llm_stream_{}_{}",
                    provider_name,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                );

                // Create channel for stream chunks
                let (tx, rx) = mpsc::channel(100);

                // Spawn task to forward stream chunks to channel
                spawn_stream_forwarder(stream, tx);

                // Store receiver in stream storage
                self.stream_storage.store_stream(stream_id.clone(), rx);

                // Return Stream handle per SPEC-013/SPEC-029 contract
                Ok(Value::Stream(ash_core::StreamHandle::new(
                    stream_id,
                    "ChatChunk"
                )))
            }
            "pull_stream_chunk" => {
                use crate::providers::llm::stream_storage::StreamChunk;

                // Extract stream_id from args
                if args.is_empty() {
                    return Err(CapabilityError::InvalidArgument(
                        "pull_stream_chunk requires stream_id argument".to_string()
                    ));
                }
                let stream_id = args[0].as_string()
                    .ok_or_else(|| CapabilityError::InvalidArgument(
                        "stream_id must be a string".to_string()
                    ))?;

                // Pull chunk from stream storage
                match self.stream_storage.pull_chunk(stream_id) {
                    Ok(Some(StreamChunk::Data(chunk))) => {
                        // Got a valid chunk - return as Some(chunk)
                        Ok(Value::variant("Some", vec![("0", chunk)]))
                    }
                    Ok(Some(StreamChunk::End)) => {
                        // Stream ended normally - return End variant
                        Ok(Value::unit_variant("End"))
                    }
                    Ok(Some(StreamChunk::Error(msg))) => {
                        // Stream error - propagate as execution failure
                        Err(CapabilityError::ExecutionFailed(format!(
                            "Stream error: {msg}"
                        )))
                    }
                    Ok(None) => {
                        // No chunk available yet - return None variant
                        Ok(Value::unit_variant("None"))
                    }
                    Err(e) => Err(CapabilityError::ExecutionFailed(e)),
                }
            }
            "embed" => {
                // Extract and validate arguments: [provider, model, texts]
                let (provider_name, model, texts) = extract_embed_args(args)?;

                // Convert texts to strings
                let text_strings = texts_to_strings(texts)?;

                // Build request
                let request = build_embed_request(model, text_strings)?;

                // Get client and execute
                let client = self.get_client(provider_name)?;
                let response = client
                    .embeddings()
                    .create(request)
                    .await
                    .map_err(|e| error::map_openai_error(e, provider_name))?;

                // Convert response to Value
                embed_response_to_value(response)
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown action: {action_name}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_new_with_valid_configs() {
        let mut configs = HashMap::new();
        configs.insert("openai".to_string(), LlmConfig::openai("sk-test123"));
        configs.insert("ollama".to_string(), LlmConfig::ollama());

        let provider = LlmProvider::new(configs);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_local_provider_uses_validate_for_local() {
        // Local providers like Ollama have empty api_key
        // LlmProvider::new uses validate_for_local() for these configs
        let mut configs = HashMap::new();
        let ollama_config = LlmConfig::ollama();
        assert!(
            ollama_config.api_key.is_empty(),
            "Ollama config should have empty api_key"
        );

        configs.insert("ollama".to_string(), ollama_config);

        // This should succeed because LlmProvider detects empty api_key
        // and uses validate_for_local() instead of validate()
        let result = LlmProvider::new(configs);
        assert!(
            result.is_ok(),
            "Local provider with empty api_key should be accepted"
        );
    }

    #[test]
    fn test_provider_new_with_invalid_config() {
        let mut configs = HashMap::new();
        configs.insert(
            "invalid".to_string(),
            LlmConfig {
                api_base: "not-a-url".to_string(),
                api_key: "key".to_string(),
                ..LlmConfig::default()
            },
        );

        let result = LlmProvider::new(configs);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid config"));
        assert!(err.contains("invalid"));
    }

    #[test]
    fn test_capability_provider_name() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        assert_eq!(provider.name(), "llm");
    }

    #[test]
    fn test_capability_provider_effect() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        assert_eq!(provider.effect(), Effect::Operational);
    }

    #[tokio::test]
    async fn test_observe_returns_not_available() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        let result = provider.observe(&[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("does not support observe")
        );
    }

    #[tokio::test]
    async fn test_execute_unknown_action() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        let result = provider.execute("unknown_action", &[]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown action"));
    }

    #[tokio::test]
    async fn test_list_models_missing_arg() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        let result = provider.execute("list_models", &[]).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("requires provider name"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[tokio::test]
    async fn test_list_models_wrong_arg_type() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        let result = provider.execute("list_models", &[Value::Int(42)]).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("must be a string"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[tokio::test]
    async fn test_list_models_empty_provider() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        let result = provider
            .execute("list_models", &[Value::String(String::new())])
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::InvalidArgument(msg) => {
                assert!(msg.contains("cannot be empty"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[tokio::test]
    async fn test_list_models_unknown_provider() {
        let provider = LlmProvider::new(HashMap::new()).unwrap();
        let result = provider
            .execute("list_models", &[Value::String("unknown".to_string())])
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::NotAvailable(msg) => {
                assert!(msg.contains("provider:unknown"));
            }
            _ => panic!("Expected NotAvailable error"),
        }
    }
}
